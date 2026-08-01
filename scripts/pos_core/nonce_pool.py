#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Durable Nonce Account Pool Management Core Module
"""

import sqlite3
from typing import Optional
from pos_core.db import DB_PATH, get_db_cursor
from pos_core.constants import NONCE_TTL_MINUTES

def allocate_free_nonce_account(conn: Optional[sqlite3.Connection] = None, db_path: str = DB_PATH) -> Optional[str]:
    """
    Atomically allocates a free Nonce account with TTL auto-release.
    Supports parameterized db_path and optional active conn object.
    """
    with get_db_cursor(conn, db_path, commit=False) as cursor:
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS nonce_accounts (
                pubkey TEXT PRIMARY KEY,
                status TEXT NOT NULL DEFAULT 'free',
                locked_at TIMESTAMP
            )
        """)
        # 1. Auto-release locks hanging for >15 minutes
        cursor.execute(f"UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE status = 'locked' AND locked_at < datetime('now', '-{NONCE_TTL_MINUTES} minutes')")

        # 2. Check for RETURNING support (SQLite >= 3.35.0)
        sqlite_version = sqlite3.sqlite_version_info
        if sqlite_version >= (3, 35, 0):
            cursor.execute("""
                UPDATE nonce_accounts 
                SET status = 'locked', locked_at = CURRENT_TIMESTAMP 
                WHERE pubkey = (
                    SELECT pubkey FROM nonce_accounts WHERE status = 'free' LIMIT 1
                )
                RETURNING pubkey;
            """)
            row = cursor.fetchone()
            cursor.connection.commit()
            return row[0] if row else None
        else:
            # Fallback for older SQLite versions: ensure clean transaction state before BEGIN IMMEDIATE
            cursor.connection.commit()
            cursor.execute("BEGIN IMMEDIATE;")
            cursor.execute("SELECT pubkey FROM nonce_accounts WHERE status = 'free' LIMIT 1;")
            row = cursor.fetchone()
            if row:
                pubkey = row[0]
                cursor.execute("UPDATE nonce_accounts SET status = 'locked', locked_at = CURRENT_TIMESTAMP WHERE pubkey = ?;", (pubkey,))
                cursor.connection.commit()
                return pubkey
            cursor.connection.commit()
            return None

def release_nonce_account(conn: Optional[sqlite3.Connection] = None, pubkey: Optional[str] = None, db_path: str = DB_PATH) -> None:
    """
    Releases a locked Nonce account back to the free pool.
    """
    if not pubkey:
        return
    with get_db_cursor(conn, db_path) as cursor:
        cursor.execute("UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE pubkey = ?", (pubkey,))

def mark_nonce_account_stale(conn: Optional[sqlite3.Connection] = None, pubkey: Optional[str] = None, db_path: str = DB_PATH) -> None:
    """
    Marks a nonce account as stale_needs_refresh when an on-chain transaction reverts.
    """
    if not pubkey:
        return
    with get_db_cursor(conn, db_path) as cursor:
        cursor.execute("""
            UPDATE nonce_accounts 
            SET status = 'stale_needs_refresh', locked_at = CURRENT_TIMESTAMP 
            WHERE pubkey = ?
        """, (pubkey,))

def refresh_stale_nonce_account(conn: Optional[sqlite3.Connection] = None, pubkey: Optional[str] = None, new_nonce_hash: Optional[str] = None, db_path: str = DB_PATH) -> None:
    """
    Refreshes a stale nonce account after on-chain RPC getAccountInfo state fetch.
    """
    if not pubkey:
        return
    with get_db_cursor(conn, db_path) as cursor:
        cursor.execute("""
            UPDATE nonce_accounts 
            SET status = 'free', locked_at = NULL 
            WHERE pubkey = ?
        """, (pubkey,))
