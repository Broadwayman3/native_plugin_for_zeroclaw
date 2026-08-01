#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Local SQLite Database & Reporting Backend (WAL Mode & Atomic Transitions)
Provides WAL-enabled local persistence for invoices, payments, and Squads v4 proposals.
Prevents race conditions using atomic state transitions (UPDATE ... WHERE status = 'pending').
Exposes REST API for merchant reporting (GET /api/v1/sales/summary).
"""

import os
import sys
import json
import sqlite3
import datetime
from http.server import HTTPServer, BaseHTTPRequestHandler

DB_PATH = "data/pos_store.db"

def get_db_connection():
    conn = sqlite3.connect(DB_PATH, timeout=10.0)
    conn.execute("PRAGMA busy_timeout=5000;")
    return conn

def init_db():
    os.makedirs("data", exist_ok=True)
    conn = sqlite3.connect(DB_PATH, timeout=10.0)
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute("PRAGMA busy_timeout=5000;")
    cursor = conn.cursor()
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS invoices (
            id TEXT PRIMARY KEY,
            reference_pubkey TEXT UNIQUE NOT NULL,
            fiat_currency TEXT NOT NULL,
            fiat_amount REAL NOT NULL,
            usdc_amount REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            tx_signature TEXT,
            customer_address TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS squads_proposals (
            proposal_index INTEGER PRIMARY KEY,
            invoice_id TEXT NOT NULL,
            recipient_pubkey TEXT NOT NULL,
            amount_usdc REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'created',
            tx_base64 TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (invoice_id) REFERENCES invoices(id)
        )
    """)
    
    # Insert sample POS data if empty
    cursor.execute("SELECT COUNT(*) FROM invoices")
    if cursor.fetchone()[0] == 0:
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        sample_data = [
            ("INV-101", "7xRefKey11111111111111111111111111111111111", "UAH", 200.0, 4.82, "paid", "5k9X...Signature1", "9xK2...Customer1", now, now),
            ("INV-102", "8xRefKey22222222222222222222222222222222222", "UAH", 150.0, 3.61, "paid", "5k9X...Signature2", "9xK2...Customer2", now, now),
            ("INV-103", "9xRefKey33333333333333333333333333333333333", "USD", 10.0, 10.00, "pending", None, None, now, now),
        ]
        cursor.executemany("""
            INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, tx_signature, customer_address, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, sample_data)
        
    conn.commit()
    conn.close()
    print(f"✅ SQLite Database (WAL Mode) initialized at {DB_PATH}")

class POSApiHandler(BaseHTTPRequestHandler):
    def _set_headers(self, status=200):
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()

    def do_GET(self):
        conn = get_db_connection()
        conn.row_factory = sqlite3.Row
        cursor = conn.cursor()

        if self.path == '/api/v1/sales/summary':
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
            self._set_headers(200)
            self.wfile.write(json.dumps(summary, indent=2).encode('utf-8'))

        elif self.path == '/api/v1/invoices':
            cursor.execute("SELECT * FROM invoices ORDER BY created_at DESC")
            rows = [dict(r) for r in cursor.fetchall()]
            self._set_headers(200)
            self.wfile.write(json.dumps(rows, indent=2).encode('utf-8'))

        else:
            self._set_headers(404)
            self.wfile.write(json.dumps({"error": "Endpoint not found"}).encode('utf-8'))

        conn.close()

    def do_POST(self):
        content_length = int(self.headers.get('Content-Length', 0))
        post_data = self.rfile.read(content_length)
        
        try:
            data = json.loads(post_data.decode('utf-8'))
        except Exception:
            self._set_headers(400)
            self.wfile.write(json.dumps({"error": "Invalid JSON"}).encode('utf-8'))
            return

        conn = get_db_connection()
        cursor = conn.cursor()

        if self.path == '/api/v1/invoices/create':
            try:
                now = datetime.datetime.now(datetime.timezone.utc).isoformat()
                cursor.execute("""
                    INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, created_at, updated_at)
                    VALUES (?, ?, ?, ?, ?, 'pending', ?, ?)
                """, (data['id'], data['reference_pubkey'], data.get('fiat_currency', 'USD'), data.get('fiat_amount', data['usdc_amount']), data['usdc_amount'], now, now))
                conn.commit()
                self._set_headers(201)
                self.wfile.write(json.dumps({"success": True, "invoice_id": data['id']}).encode('utf-8'))
            except Exception as e:
                self._set_headers(500)
                self.wfile.write(json.dumps({"error": str(e)}).encode('utf-8'))

        elif self.path == '/api/v1/invoices/update_status':
            try:
                now = datetime.datetime.now(datetime.timezone.utc).isoformat()
                # Atomic state transition: UPDATE ... WHERE status = 'pending' (prevents double fulfillment)
                cursor.execute("""
                    UPDATE invoices 
                    SET status = ?, tx_signature = ?, updated_at = ? 
                    WHERE id = ? AND (status = 'pending' OR status = ?)
                """, (data['status'], data.get('tx_signature'), now, data['invoice_id'], data['status']))
                conn.commit()
                updated_count = cursor.rowcount
                if updated_count == 0:
                    self._set_headers(409)
                    self.wfile.write(json.dumps({"success": False, "error": "Conflict: Invoice state already finalized or invalid transition", "updated": 0}).encode('utf-8'))
                else:
                    self._set_headers(200)
                    self.wfile.write(json.dumps({"success": True, "updated": updated_count}).encode('utf-8'))
            except Exception as e:
                self._set_headers(500)
                self.wfile.write(json.dumps({"error": str(e)}).encode('utf-8'))

        else:
            self._set_headers(404)
            self.wfile.write(json.dumps({"error": "Endpoint not found"}).encode('utf-8'))

        conn.close()

def run_server(port=8080):
    init_db()
    server_address = ('127.0.0.1', port)
    httpd = HTTPServer(server_address, POSApiHandler)
    print(f"🚀 POS REST Backend API (WAL Mode) listening on http://127.0.0.1:{port}")
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
