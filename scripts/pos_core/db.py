#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Database Core Module (WAL Mode & Schema Management)
"""

import os
import sqlite3
import datetime

DB_PATH = "data/pos_store.db"

def get_db_connection(db_path: str = DB_PATH) -> sqlite3.Connection:
    """Establishes SQLite connection with WAL mode and performance tuning."""
    conn = sqlite3.connect(db_path, timeout=10.0)
    try:
        conn.execute("PRAGMA journal_mode=WAL;")
    except sqlite3.OperationalError:
        conn.execute("PRAGMA journal_mode=DELETE;")
    conn.execute("PRAGMA busy_timeout=5000;")
    conn.execute("PRAGMA synchronous=NORMAL;")
    conn.execute("PRAGMA cache_size=-64000;")
    return conn

def cleanup_expired_pending_invoices(conn: sqlite3.Connection = None, db_path: str = DB_PATH):
    """Automatically marks pending invoices older than 24 hours as expired."""
    close_conn = False
    if conn is None:
        conn = get_db_connection(db_path)
        close_conn = True
    try:
        cursor = conn.cursor()
        cursor.execute("UPDATE invoices SET status = 'expired', updated_at = CURRENT_TIMESTAMP WHERE status = 'pending' AND created_at < datetime('now', '-24 hours')")
        conn.commit()
    finally:
        if close_conn:
            conn.close()

def check_and_register_telegram_update(conn: sqlite3.Connection = None, update_id: int = None, db_path: str = DB_PATH) -> bool:
    """Deduplicates Telegram webhook update IDs with 24h TTL cleanup."""
    close_conn = False
    if conn is None:
        conn = get_db_connection(db_path)
        close_conn = True
    try:
        cursor = conn.cursor()
        cursor.execute("DELETE FROM processed_updates WHERE processed_at < datetime('now', '-1 day')")
        try:
            cursor.execute("INSERT INTO processed_updates (update_id) VALUES (?)", (update_id,))
            conn.commit()
            return True
        except sqlite3.IntegrityError:
            return False
    finally:
        if close_conn:
            conn.close()

def init_db(db_path: str = DB_PATH):
    """Initializes SQLite tables and default nonce pool / sample data."""
    db_dir = os.path.dirname(db_path)
    if db_dir:
        os.makedirs(db_dir, exist_ok=True)

    conn = sqlite3.connect(db_path, timeout=10.0)
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute("PRAGMA busy_timeout=5000;")
    cursor = conn.cursor()

    cursor.execute("CREATE TABLE IF NOT EXISTS invoices (id TEXT PRIMARY KEY, reference_pubkey TEXT UNIQUE NOT NULL, fiat_currency TEXT NOT NULL, fiat_amount REAL NOT NULL, usdc_amount REAL NOT NULL, status TEXT NOT NULL DEFAULT 'pending', tx_signature TEXT, customer_address TEXT, pix_id TEXT, pix_payload TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)")
    cursor.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_invoices_tx_sig ON invoices(tx_signature) WHERE tx_signature IS NOT NULL;")
    cursor.execute("CREATE TABLE IF NOT EXISTS squads_proposals (proposal_index INTEGER PRIMARY KEY, invoice_id TEXT NOT NULL, recipient_pubkey TEXT NOT NULL, amount_usdc REAL NOT NULL, status TEXT NOT NULL DEFAULT 'created', tx_base64 TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, FOREIGN KEY (invoice_id) REFERENCES invoices(id))")
    cursor.execute("CREATE TABLE IF NOT EXISTS processed_updates (update_id INTEGER PRIMARY KEY, processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)")
    cursor.execute("CREATE TABLE IF NOT EXISTS nonce_accounts (pubkey TEXT PRIMARY KEY, status TEXT NOT NULL DEFAULT 'free', locked_at TIMESTAMP)")
    cursor.execute("CREATE TABLE IF NOT EXISTS sop_checkpoints (id TEXT PRIMARY KEY, sop_id TEXT NOT NULL, step_id TEXT NOT NULL, state_data TEXT, status TEXT NOT NULL DEFAULT 'pending', created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)")

    cursor.execute("SELECT COUNT(*) FROM nonce_accounts")
    if cursor.fetchone()[0] == 0:
        cursor.executemany("INSERT INTO nonce_accounts (pubkey, status) VALUES (?, 'free')", [
            ("Nonce111111111111111111111111111111111111111",),
            ("Nonce222222222222222222222222222222222222222",),
            ("Nonce333333333333333333333333333333333333333",)
        ])

    cursor.execute("SELECT COUNT(*) FROM invoices")
    if cursor.fetchone()[0] == 0:
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        sample_data = [
            ("INV-101", "7xRefKey11111111111111111111111111111111111", "UAH", 200.0, 4.82, "paid", "5k9X...Signature1", "9xK2...Customer1", None, None, now, now),
            ("INV-102", "8xRefKey22222222222222222222222222222222222", "UAH", 150.0, 3.61, "paid", "5k9X...Signature2", "9xK2...Customer2", None, None, now, now),
            ("INV-103", "9xRefKey33333333333333333333333333333333333", "USD", 10.0, 10.00, "pending", None, None, None, None, now, now),
        ]
        cursor.executemany("INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, tx_signature, customer_address, pix_id, pix_payload, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", sample_data)

    conn.commit()
    cleanup_expired_pending_invoices(conn)
    conn.close()
