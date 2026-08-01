#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Local SQLite Database & REST API Backend Entrypoint
WAL-enabled persistence, micro-router request dispatching, and atomic transitions.
"""

import os
import sys
import json
import sqlite3
import datetime
import socket
from http.server import HTTPServer, BaseHTTPRequestHandler

# Ensure scripts directory is in sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

from sanitizer import redact_api_key, escape_telegram_markdown_v2
from pos_core import (
    DB_PATH,
    get_db_connection,
    init_db,
    cleanup_expired_pending_invoices,
    check_and_register_telegram_update,
    allocate_free_nonce_account,
    release_nonce_account,
    mark_nonce_account_stale,
    refresh_stale_nonce_account,
    token_to_atomic_units,
    usdc_to_atomic_units,
    is_payment_amount_valid,
    generate_secure_reference_key,
    initiate_refund_request,
    handle_telegram_429_retry,
    load_wasm_binary_ram_cache,
    get_required_commitment_level,
    generate_atomic_refund_instructions,
    validate_squads_multisig_account,
    verify_solana_transaction_payload,
    calculate_pix_crc16,
    generate_pix_emv_payload,
    get_multitier_fiat_rate,
    route_get,
    route_post,
    dispatch_request,
    send_json_response
)

# Set global socket timeout to prevent hung RPC HTTP connection sockets
socket.setdefaulttimeout(10.0)

# Register REST API GET routes
@route_get('/api/v1/sales/summary')
def handle_sales_summary(handler, query_params):
    conn = get_db_connection()
    conn.row_factory = sqlite3.Row
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) as total_invoices, SUM(usdc_amount) as total_usdc FROM invoices WHERE status = 'paid'")
        row = cursor.fetchone()
        cursor.execute("SELECT COUNT(*) as pending_count FROM invoices WHERE status = 'pending'")
        pending_row = cursor.fetchone()

        summary = {
            "business_name": "ZeroClaw Coffee POS",
            "currency": "USDC",
            "total_paid_invoices": row["total_invoices"] or 0,
            "total_sales_usdc": round(row["total_usdc"] or 0.0, 2),
            "total_pending_invoices": pending_row["pending_count"] or 0,
            "journal_mode": "WAL",
            "timestamp": datetime.datetime.now(datetime.timezone.utc).isoformat()
        }
        return 200, summary
    finally:
        conn.close()

@route_get('/api/v1/invoices')
def handle_get_invoices(handler, query_params):
    conn = get_db_connection()
    conn.row_factory = sqlite3.Row
    try:
        cleanup_expired_pending_invoices(conn)
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM invoices ORDER BY created_at DESC")
        rows = [dict(r) for r in cursor.fetchall()]
        return 200, rows
    finally:
        conn.close()

# Register REST API POST routes
@route_post('/api/v1/invoices/create')
def handle_create_invoice(handler, data, query_params):
    conn = get_db_connection()
    try:
        cursor = conn.cursor()
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        cursor.execute("""
            INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, 'pending', ?, ?)
        """, (data['id'], data['reference_pubkey'], data.get('fiat_currency', 'USD'), data.get('fiat_amount', data['usdc_amount']), data['usdc_amount'], now, now))
        conn.commit()
        return 201, {"success": True, "invoice_id": data['id']}
    except Exception as e:
        return 500, {"error": str(e)}
    finally:
        conn.close()

@route_post('/api/v1/invoices/update_status')
def handle_update_invoice_status(handler, data, query_params):
    conn = get_db_connection()
    try:
        cursor = conn.cursor()
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        cursor.execute("""
            UPDATE invoices 
            SET status = ?, tx_signature = ?, updated_at = ? 
            WHERE id = ? AND (status = 'pending' OR status = 'partially_paid' OR status = ?)
        """, (data['status'], data.get('tx_signature'), now, data['invoice_id'], data['status']))
        conn.commit()
        updated_count = cursor.rowcount
        if updated_count == 0:
            return 409, {"success": False, "error": "Conflict: Invoice state already finalized or invalid transition", "updated": 0}
        return 200, {"success": True, "updated": updated_count}
    except Exception as e:
        return 500, {"error": redact_api_key(str(e))}
    finally:
        conn.close()

@route_post('/api/v1/nonce/allocate')
def handle_nonce_allocate(handler, data, query_params):
    conn = get_db_connection()
    try:
        allocated = allocate_free_nonce_account(conn)
        if allocated:
            return 200, {"success": True, "nonce_pubkey": allocated}
        return 503, {"success": False, "error": "No free durable nonce account available in pool"}
    except Exception as e:
        return 500, {"error": redact_api_key(str(e))}
    finally:
        conn.close()

@route_post('/api/v1/nonce/release')
def handle_nonce_release(handler, data, query_params):
    conn = get_db_connection()
    try:
        pubkey = data.get('nonce_pubkey') if data else None
        if pubkey:
            release_nonce_account(conn, pubkey)
            return 200, {"success": True, "released": pubkey}
        return 400, {"error": "Missing nonce_pubkey"}
    except Exception as e:
        return 500, {"error": redact_api_key(str(e))}
    finally:
        conn.close()

class POSApiHandler(BaseHTTPRequestHandler):
    def _set_headers(self, status=200):
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()

    def do_GET(self):
        # x402 Machine Commerce Handshake Support
        if self.headers.get('X-ACCEPT-PAYMENT') == 'x402' and '/api/v1/sales/premium_analytics' in self.path:
            extra_headers = {
                'X-PAYMENT-REQUIRED-AMOUNT': '1.00 USDC',
                'X-PAYMENT-RECIPIENT': '8xAZmQ1111111111111111111111111111111111111'
            }
            body = {
                "error": "Payment Required",
                "x402_spec": "solana-pay",
                "amount_usdc": 1.00,
                "pay_url": "solana:8xAZmQ11111111111111111111111111111111111?amount=1.00&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            }
            send_json_response(self, 402, body, extra_headers)
            return

        dispatch_request(self, 'GET')

    def do_POST(self):
        content_length = int(self.headers.get('Content-Length', 0))
        post_data_raw = self.rfile.read(content_length)
        
        try:
            data = json.loads(post_data_raw.decode('utf-8'))
        except Exception:
            send_json_response(self, 400, {"error": "Invalid JSON"})
            return

        dispatch_request(self, 'POST', post_data=data)

def run_server(port=8080):
    init_db()
    server_address = ('127.0.0.1', port)
    httpd = HTTPServer(server_address, POSApiHandler)
    print(f"🚀 POS REST Backend API (WAL Mode & Micro-Router) listening on http://127.0.0.1:{port}")
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nStopping POS REST API server.")

if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        init_db()
        print("Database WAL mode test initialization passed.")
    else:
        port = int(sys.argv[1]) if len(sys.argv) > 1 and sys.argv[1].isdigit() else 8080
        run_server(port)
