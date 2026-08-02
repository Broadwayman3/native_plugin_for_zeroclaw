#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Database Core Module (WAL Mode, DAO Layer & Schema Management)
"""

import os
import sqlite3
import datetime
from contextlib import contextmanager
from typing import Optional, Generator, Dict, Any, List, Tuple
from pos_core.constants import DEFAULT_SOCKET_TIMEOUT

DB_PATH: str = "data/pos_store.db"

def get_db_connection(db_path: str = DB_PATH) -> sqlite3.Connection:
    """Establishes SQLite connection with WAL mode, auto-creating parent directories."""
    db_dir = os.path.dirname(db_path)
    if db_dir:
        os.makedirs(db_dir, exist_ok=True)
    conn = sqlite3.connect(db_path, timeout=DEFAULT_SOCKET_TIMEOUT)
    try:
        conn.execute("PRAGMA journal_mode=WAL;")
    except sqlite3.OperationalError:
        conn.execute("PRAGMA journal_mode=DELETE;")
    conn.execute("PRAGMA busy_timeout=5000;")
    conn.execute("PRAGMA synchronous=NORMAL;")
    conn.execute("PRAGMA cache_size=-64000;")
    return conn

@contextmanager
def get_db_cursor(conn: Optional[sqlite3.Connection] = None, db_path: str = DB_PATH, commit: bool = True) -> Generator[sqlite3.Cursor, None, None]:
    """
    Context manager for database cursor and transaction lifecycle management.
    - If conn is provided: reuses existing connection without closing it upon exit.
    - If conn is None: opens a new connection, executes transaction, and guarantees closing it upon exit.
    """
    should_close = False
    if conn is None:
        conn = get_db_connection(db_path)
        should_close = True
    try:
        cursor = conn.cursor()
        yield cursor
        if commit:
            conn.commit()
    except Exception:
        if should_close:
            conn.rollback()
        raise
    finally:
        if should_close:
            conn.close()

def cleanup_db_files(db_path: str) -> None:
    """Completely removes database file and associated WAL/SHM sidecar files."""
    for ext in ["", "-wal", "-shm"]:
        target = db_path + ext
        if os.path.exists(target):
            try:
                os.remove(target)
            except OSError:
                pass

def cleanup_expired_pending_invoices(conn: Optional[sqlite3.Connection] = None, db_path: str = DB_PATH) -> None:
    """Automatically marks pending invoices older than 24 hours as expired."""
    with get_db_cursor(conn, db_path) as cursor:
        cursor.execute("UPDATE invoices SET status = 'expired', updated_at = CURRENT_TIMESTAMP WHERE status = 'pending' AND created_at < datetime('now', '-24 hours')")

def check_and_register_telegram_update(conn: Optional[sqlite3.Connection] = None, update_id: Optional[int] = None, db_path: str = DB_PATH) -> bool:
    """Deduplicates Telegram webhook update IDs with 24h TTL cleanup."""
    if update_id is None:
        return False
    try:
        clean_update_id = int(update_id)
    except (ValueError, TypeError):
        return False

    with get_db_cursor(conn, db_path) as cursor:
        cursor.execute("DELETE FROM processed_updates WHERE processed_at < datetime('now', '-1 day')")
        try:
            cursor.execute("INSERT INTO processed_updates (update_id) VALUES (?)", (clean_update_id,))
            return True
        except (sqlite3.IntegrityError, sqlite3.InterfaceError):
            return False

def seed_sample_data(conn: Optional[sqlite3.Connection] = None, db_path: str = DB_PATH) -> None:
    """Populates SQLite database with default sample invoices if table is empty."""
    with get_db_cursor(conn, db_path) as cursor:
        cursor.execute("SELECT COUNT(*) FROM invoices")
        if cursor.fetchone()[0] == 0:
            now = datetime.datetime.now(datetime.timezone.utc).isoformat()
            sample_data = [
                ("INV-101", "7xRefKey11111111111111111111111111111111111", "UAH", 200.0, 4.82, "paid", "5k9X...Signature1", "9xK2...Customer1", None, None, now, now),
                ("INV-102", "8xRefKey22222222222222222222222222222222222", "UAH", 150.0, 3.61, "paid", "5k9X...Signature2", "9xK2...Customer2", None, None, now, now),
                ("INV-103", "9xRefKey33333333333333333333333333333333333", "USD", 10.0, 10.00, "pending", None, None, None, None, now, now),
            ]
            cursor.executemany("INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, tx_signature, customer_address, pix_id, pix_payload, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)", sample_data)

def init_db(db_path: str = DB_PATH, seed_sample_data: bool = True) -> None:
    """Initializes SQLite tables, default nonce pool, and optional sample data."""
    conn = get_db_connection(db_path)
    try:
        cursor = conn.cursor()

        cursor.execute("CREATE TABLE IF NOT EXISTS invoices (id TEXT PRIMARY KEY, reference_pubkey TEXT UNIQUE NOT NULL, fiat_currency TEXT NOT NULL, fiat_amount REAL NOT NULL, usdc_amount REAL NOT NULL, status TEXT NOT NULL DEFAULT 'pending', tx_signature TEXT, customer_address TEXT, pix_id TEXT, pix_payload TEXT, created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP, updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP)")
        cursor.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_invoices_tx_sig ON invoices(tx_signature) WHERE tx_signature IS NOT NULL;")
        
        # Backward-compatible schema migration for POS tax & receipt breakdown metadata
        cursor.execute("PRAGMA table_info(invoices)")
        columns = [col[1] for col in cursor.fetchall()]
        if "tax_rate_pct" not in columns:
            cursor.execute("ALTER TABLE invoices ADD COLUMN tax_rate_pct REAL DEFAULT 0.0")
        if "items_breakdown" not in columns:
            cursor.execute("ALTER TABLE invoices ADD COLUMN items_breakdown TEXT")

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

        conn.commit()

        if seed_sample_data:
            globals()["seed_sample_data"](conn=conn)

        cleanup_expired_pending_invoices(conn=conn)
    finally:
        conn.close()


# =====================================================================
# Data Access Object (DAO / Repository) Layer
# =====================================================================

def get_sales_summary_stats(db_path: str = DB_PATH) -> Dict[str, Any]:
    """Retrieves aggregated sales metrics, pending invoice counts, and breakdown by currency."""
    conn = get_db_connection(db_path)
    conn.row_factory = sqlite3.Row
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT COUNT(*) as total_invoices, SUM(usdc_amount) as total_usdc FROM invoices WHERE status = 'paid'")
        row = cursor.fetchone()
        cursor.execute("SELECT COUNT(*) as pending_count FROM invoices WHERE status = 'pending'")
        pending_row = cursor.fetchone()

        cursor.execute("SELECT fiat_currency, COUNT(*) as count, SUM(fiat_amount) as total_fiat, SUM(usdc_amount) as total_usdc FROM invoices WHERE status = 'paid' GROUP BY fiat_currency")
        by_curr = {r["fiat_currency"]: {"count": r["count"], "total_fiat": round(r["total_fiat"] or 0.0, 2), "total_usdc": round(r["total_usdc"] or 0.0, 2)} for r in cursor.fetchall()}

        return {
            "business_name": "ZeroClaw Coffee POS",
            "currency": "USDC",
            "total_paid_invoices": (row["total_invoices"] if row else 0) or 0,
            "total_sales_usdc": round((row["total_usdc"] if row else 0.0) or 0.0, 2),
            "total_pending_invoices": (pending_row["pending_count"] if pending_row else 0) or 0,
            "sales_by_currency": by_curr,
            "journal_mode": "WAL",
            "timestamp": datetime.datetime.now(datetime.timezone.utc).isoformat()
        }
    finally:
        conn.close()

def get_invoices_list(invoice_id: Optional[str] = None, db_path: str = DB_PATH) -> List[Dict[str, Any]]:
    """Fetches list of invoices or single invoice by ID after cleaning expired pending records."""
    conn = get_db_connection(db_path)
    conn.row_factory = sqlite3.Row
    try:
        cleanup_expired_pending_invoices(conn)
        cursor = conn.cursor()
        if invoice_id:
            cursor.execute("SELECT * FROM invoices WHERE id = ? ORDER BY created_at DESC", (invoice_id,))
        else:
            cursor.execute("SELECT * FROM invoices ORDER BY created_at DESC")
        return [dict(r) for r in cursor.fetchall()]
    finally:
        conn.close()

def create_invoice_record(data: Dict[str, Any], db_path: str = DB_PATH) -> Tuple[bool, str]:
    """Creates a new pending invoice record in the database."""
    conn = get_db_connection(db_path)
    try:
        cursor = conn.cursor()
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        cursor.execute("""
            INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, 'pending', ?, ?)
        """, (data['id'], data['reference_pubkey'], data.get('fiat_currency', 'USD'), data.get('fiat_amount', data['usdc_amount']), data['usdc_amount'], now, now))
        conn.commit()
        return True, data['id']
    finally:
        conn.close()

def update_invoice_status_record(invoice_id: str, status: str, tx_signature: Optional[str] = None, db_path: str = DB_PATH) -> int:
    """Atomically updates invoice status if transition is valid from pending or partially_paid."""
    conn = get_db_connection(db_path)
    try:
        cursor = conn.cursor()
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        cursor.execute("""
            UPDATE invoices 
            SET status = ?, tx_signature = ?, updated_at = ? 
            WHERE id = ? AND (status = 'pending' OR status = 'partially_paid' OR status = ?)
        """, (status, tx_signature, now, invoice_id, status))
        conn.commit()
        return cursor.rowcount
    finally:
        conn.close()

def cancel_invoice_record(invoice_id: str, db_path: str = DB_PATH) -> int:
    """Atomically cancels/voids a pending invoice."""
    conn = get_db_connection(db_path)
    try:
        cursor = conn.cursor()
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        cursor.execute("""
            UPDATE invoices 
            SET status = 'cancelled', updated_at = ? 
            WHERE id = ? AND status = 'pending'
        """, (now, invoice_id))
        conn.commit()
        return cursor.rowcount
    finally:
        conn.close()
