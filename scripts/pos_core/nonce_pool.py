#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Durable Nonce Account Pool Management Core Module
"""

import sqlite3
from pos_core.db import DB_PATH, get_db_connection

def allocate_free_nonce_account(conn: sqlite3.Connection = None, db_path: str = DB_PATH) -> str:
    """
    Atomically allocates a free Nonce account with 15-minute TTL auto-release.
    Supports parameterized db_path and optional active conn object.
    """
    close_conn = False
    if conn is None:
        conn = get_db_connection(db_path)
        close_conn = True

    try:
        cursor = conn.cursor()
        cursor.execute("""
            CREATE TABLE IF NOT EXISTS nonce_accounts (
                pubkey TEXT PRIMARY KEY,
                status TEXT NOT NULL DEFAULT 'free',
                locked_at TIMESTAMP
            )
        """)
        # 1. Auto-release locks hanging for >15 minutes
        cursor.execute("UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE status = 'locked' AND locked_at < datetime('now', '-15 minutes')")

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
            conn.commit()
            return row[0] if row else None
        else:
            # Fallback for older SQLite versions
            cursor.execute("BEGIN IMMEDIATE;")
            cursor.execute("SELECT pubkey FROM nonce_accounts WHERE status = 'free' LIMIT 1;")
            row = cursor.fetchone()
            if row:
                pubkey = row[0]
                cursor.execute("UPDATE nonce_accounts SET status = 'locked', locked_at = CURRENT_TIMESTAMP WHERE pubkey = ?;", (pubkey,))
                conn.commit()
                return pubkey
            conn.commit()
            return None
    finally:
        if close_conn:
            conn.close()

def release_nonce_account(conn: sqlite3.Connection = None, pubkey: str = None, db_path: str = DB_PATH):
    """
    Releases a locked Nonce account back to the free pool.
    """
    if not pubkey:
        return
    close_conn = False
    if conn is None:
        conn = get_db_connection(db_path)
        close_conn = True

    try:
        cursor = conn.cursor()
        cursor.execute("UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE pubkey = ?", (pubkey,))
        conn.commit()
    finally:
        if close_conn:
            conn.close()

def mark_nonce_account_stale(conn: sqlite3.Connection = None, pubkey: str = None, db_path: str = DB_PATH):
    """
    Marks a nonce account as stale_needs_refresh when an on-chain transaction reverts.
    """
    if not pubkey:
        return
    close_conn = False
    if conn is None:
        conn = get_db_connection(db_path)
        close_conn = True

    try:
        cursor = conn.cursor()
        cursor.execute("""
            UPDATE nonce_accounts 
            SET status = 'stale_needs_refresh', locked_at = CURRENT_TIMESTAMP 
            WHERE pubkey = ?
        """, (pubkey,))
        conn.commit()
    finally:
        if close_conn:
            conn.close()

def refresh_stale_nonce_account(conn: sqlite3.Connection = None, pubkey: str = None, new_nonce_hash: str = None, db_path: str = DB_PATH):
    """
    Refreshes a stale nonce account after on-chain RPC getAccountInfo state fetch.
    """
    if not pubkey:
        return
    close_conn = False
    if conn is None:
        conn = get_db_connection(db_path)
        close_conn = True

    try:
        cursor = conn.cursor()
        cursor.execute("""
            UPDATE nonce_accounts 
            SET status = 'free', locked_at = NULL 
            WHERE pubkey = ?
        """, (pubkey,))
        conn.commit()
    finally:
        if close_conn:
            conn.close()
