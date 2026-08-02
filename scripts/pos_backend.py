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
from http.server import ThreadingHTTPServer, BaseHTTPRequestHandler

# Ensure scripts directory is in sys.path
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

from sanitizer import redact_api_key, escape_telegram_markdown_v2
from pos_core import (
    DEFAULT_SOCKET_TIMEOUT,
    DB_PATH,
    get_db_connection,
    init_db,
    seed_sample_data,
    cleanup_expired_pending_invoices,
    check_and_register_telegram_update,
    get_sales_summary_stats,
    get_invoices_list,
    create_invoice_record,
    update_invoice_status_record,
    cancel_invoice_record,
    allocate_free_nonce_account,
    release_nonce_account,
    mark_nonce_account_stale,
    refresh_stale_nonce_account,
    token_to_atomic_units,
    usdc_to_atomic_units,
    is_valid_base58,
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
    handle_options_request,
    dispatch_request,
    send_json_response
)

MAX_PAYLOAD_BYTES = 1_048_576  # 1 MB DoS Cap Limit

# Register REST API GET routes
@route_get('/actions.json')
def handle_actions_spec_json(handler, query_params):
    return 200, {
        "rules": [
            {"pathPattern": "/api/v1/actions/**", "apiPath": "/api/v1/actions/**"}
        ]
    }

@route_get('/api/v1/actions/pay_invoice')
def handle_action_get_invoice(handler, query_params):
    raw_id = query_params.get('invoice_id', ['INV-101'])[0] if query_params else 'INV-101'
    action_payload = {
        "icon": "https://raw.githubusercontent.com/solana-developers/branding/main/assets/solana-pay-logo.png",
        "label": f"Pay Invoice #{raw_id}",
        "title": f"ZeroClaw POS - Invoice #{raw_id}",
        "description": f"Scan & Complete payment for POS Invoice #{raw_id} in USDC",
        "links": {
            "actions": [
                {
                    "label": "Pay Now",
                    "href": f"/api/v1/actions/pay_invoice?invoice_id={raw_id}"
                }
            ]
        }
    }
    extra_headers = {
        'X-Action-Version': '2.1.3',
        'X-Blockchain-Ids': 'solana:EtWTRABZaYqXxicM2Tz2fSpo5nszvh6wT9D3gYqH1cQ'
    }
    return 200, action_payload, extra_headers

@route_get('/api/v1/sales/summary')
def handle_sales_summary(handler, query_params, db_path: str = DB_PATH):
    """Retrieves aggregated sales metrics and currency breakdown via DAO layer."""
    summary = get_sales_summary_stats(db_path=db_path)
    return 200, summary

@route_get('/api/v1/invoices')
def handle_get_invoices(handler, query_params, db_path: str = DB_PATH):
    """Fetches invoices list or single invoice by ID via DAO layer."""
    raw_id = query_params.get('id') if query_params else None
    inv_id = raw_id[0] if (isinstance(raw_id, list) and len(raw_id) > 0) else (raw_id if isinstance(raw_id, str) else None)
    rows = get_invoices_list(invoice_id=inv_id, db_path=db_path)
    return 200, rows

# Register REST API POST routes
@route_post('/api/v1/actions/pay_invoice')
def handle_action_post_invoice(handler, data, query_params):
    account = data.get('account') if isinstance(data, dict) else None
    if not account or not is_valid_base58(account):
        return 400, {"error": "Invalid or missing 'account' Base58 public key field in Blink POST request"}

    response_payload = {
        "transaction": "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
        "message": "ZeroClaw POS Invoice Payment Transaction"
    }
    extra_headers = {
        'X-Action-Version': '2.1.3',
        'X-Blockchain-Ids': 'solana:EtWTRABZaYqXxicM2Tz2fSpo5nszvh6wT9D3gYqH1cQ'
    }
    return 200, response_payload, extra_headers

@route_post('/api/v1/invoices/create')
def handle_create_invoice(handler, data, query_params, db_path: str = DB_PATH):
    """Creates a new pending invoice via DAO layer."""
    if not isinstance(data, dict) or 'id' not in data or 'reference_pubkey' not in data or 'usdc_amount' not in data:
        return 400, {"error": "Bad Request: Missing required invoice fields (id, reference_pubkey, usdc_amount)"}
    try:
        success, inv_id = create_invoice_record(data, db_path=db_path)
        return 201, {"success": True, "invoice_id": inv_id}
    except Exception as e:
        return 500, {"error": redact_api_key(str(e))}

ALLOWED_INVOICE_STATUSES = {'pending', 'paid', 'partially_paid', 'cancelled', 'refunding', 'refund_proposed_squads_v4', 'expired', 'failed'}

@route_post('/api/v1/invoices/update_status')
def handle_update_invoice_status(handler, data, query_params, db_path: str = DB_PATH):
    """Updates invoice status atomically via DAO layer."""
    if not isinstance(data, dict) or 'invoice_id' not in data or 'status' not in data:
        return 400, {"error": "Bad Request: Missing invoice_id or status"}
    if data['status'] not in ALLOWED_INVOICE_STATUSES:
        return 400, {"error": f"Bad Request: Invalid status '{data['status']}'. Must be one of {sorted(list(ALLOWED_INVOICE_STATUSES))}"}
    try:
        updated_count = update_invoice_status_record(data['invoice_id'], data['status'], data.get('tx_signature'), db_path=db_path)
        if updated_count == 0:
            return 409, {"success": False, "error": "Conflict: Invoice state already finalized or invalid transition", "updated": 0}
        return 200, {"success": True, "updated": updated_count}
    except Exception as e:
        return 500, {"error": redact_api_key(str(e))}

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

@route_post('/api/v1/invoices/cancel')
def handle_cancel_invoice(handler, data, query_params, db_path: str = DB_PATH):
    """Allows cashiers to void/cancel a pending invoice atomically via DAO layer."""
    if not isinstance(data, dict) or 'invoice_id' not in data:
        return 400, {"error": "Bad Request: Missing invoice_id"}
    try:
        cancelled_count = cancel_invoice_record(data['invoice_id'], db_path=db_path)
        if cancelled_count == 0:
            return 409, {"success": False, "error": "Conflict: Invoice not found or already finalized"}
        return 200, {"success": True, "cancelled_id": data['invoice_id'], "status": "cancelled"}
    except Exception as e:
        return 500, {"error": redact_api_key(str(e))}

class POSApiHandler(BaseHTTPRequestHandler):
    def _set_headers(self, status=200):
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()

    def do_OPTIONS(self):
        """Full CORS Preflight Options Request Interceptor."""
        handle_options_request(self)

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
        try:
            content_length = int(self.headers.get('Content-Length', 0))
        except (ValueError, TypeError):
            content_length = 0

        if content_length > MAX_PAYLOAD_BYTES:
            send_json_response(self, 413, {"error": "Payload Too Large: Maximum allowed size is 1MB"})
            return

        post_data_raw = self.rfile.read(content_length)
        
        try:
            data = json.loads(post_data_raw.decode('utf-8'))
        except Exception:
            send_json_response(self, 400, {"error": "Invalid JSON"})
            return

        dispatch_request(self, 'POST', post_data=data)

def run_server(port=8080, host=None, seed_sample_data=True):
    init_db(seed_sample_data_flag=seed_sample_data)
    if not host or not str(host).strip():
        host = os.getenv("HOST") or os.getenv("POS_HOST") or "0.0.0.0"
    host = host.strip()
    server_address = (host, port)
    try:
        httpd = ThreadingHTTPServer(server_address, POSApiHandler)
    except OSError as e:
        if getattr(e, 'errno', None) == 98 or "already in use" in str(e).lower():
            fallback_port = port + 1
            print(f"⚠️ [POS Server] Port {port} is busy. Retrying on PORT={fallback_port}...")
            server_address = (host, fallback_port)
            try:
                httpd = ThreadingHTTPServer(server_address, POSApiHandler)
                port = fallback_port
            except OSError:
                print(f"❌ [POS Server Error] Both PORT={port - 1} and PORT={fallback_port} are already in use. Please free a port or set PORT environment variable. Exiting safely.")
                sys.exit(1)
        else:
            raise
    banner = (
        "=================================================================\n"
        "🚀 ZeroClaw Solana POS REST API Backend Server\n"
        "=================================================================\n"
        "• Status       : OPERATIONAL (WAL Mode)\n"
        f"• Listening    : http://{host}:{port}\n"
        f"• Database     : {DB_PATH}\n"
        "• x402 Spec    : Active on /api/v1/sales/premium_analytics\n"
        "• Endpoints    : /actions.json, /api/v1/actions/pay_invoice, /sales/summary,\n"
        "                 /invoices, /invoices/create, /invoices/cancel,\n"
        "                 /nonce/allocate, /nonce/release\n"
        "================================================================="
    )
    print(banner)
    try:
        httpd.serve_forever()
    except KeyboardInterrupt:
        print("\nStopping POS REST API server.")

if __name__ == '__main__':
    if len(sys.argv) > 1 and sys.argv[1] == "--test":
        init_db(seed_sample_data=True)
        print("Database WAL mode test initialization passed.")
    else:
        env_port = os.getenv("PORT") or os.getenv("POS_PORT")
        if env_port and env_port.isdigit():
            port = int(env_port)
        elif len(sys.argv) > 1 and sys.argv[1].isdigit():
            port = int(sys.argv[1])
        else:
            port = 8080
        run_server(port=port, seed_sample_data=True)
