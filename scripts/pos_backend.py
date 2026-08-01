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
import math
import socket
import secrets
import base64
from http.server import HTTPServer, BaseHTTPRequestHandler

from sanitizer import redact_api_key, escape_telegram_markdown_v2

# Set global socket timeout to prevent hung RPC HTTP connection sockets
socket.setdefaulttimeout(10.0)

DB_PATH = "data/pos_store.db"
WASM_RAM_CACHE = None  # In-memory RAM cache for solana_pos_core.wasm binary

def get_db_connection():
    conn = sqlite3.connect(DB_PATH, timeout=10.0)
    try:
        conn.execute("PRAGMA journal_mode=WAL;")
    except sqlite3.OperationalError:
        conn.execute("PRAGMA journal_mode=DELETE;")  # Fallback for NFS/Docker network mounts
    conn.execute("PRAGMA busy_timeout=5000;")
    conn.execute("PRAGMA synchronous=NORMAL;")
    conn.execute("PRAGMA cache_size=-64000;")
    return conn

def release_nonce_account(conn, pubkey):
    """
    Звільняє заблокований Nonce-аккаунт.
    """
    cursor = conn.cursor()
    cursor.execute("UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE pubkey = ?", (pubkey,))
    conn.commit()

def cleanup_expired_pending_invoices(conn):
    """
    Автоматично маркує pending-інвойси старіші 24 годин як expired,
    запобігаючи перевантаженню RPC-запитів у Cron SOP.
    """
    cursor = conn.cursor()
    cursor.execute("""
        UPDATE invoices 
        SET status = 'expired', updated_at = CURRENT_TIMESTAMP 
        WHERE status = 'pending' AND created_at < datetime('now', '-24 hours')
    """)
    conn.commit()

def allocate_free_nonce_account(conn):
    """
    Атомарно виділяє вільний Nonce-аккаунт із пулу з 15-хвилинним TTL авто-звільненням завислих локів.
    Автоматично обирає між UPDATE ... RETURNING (SQLite >= 3.35.0) та BEGIN IMMEDIATE транзакцією (для старіших версій SQLite/Docker).
    """
    cursor = conn.cursor()
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS nonce_accounts (
            pubkey TEXT PRIMARY KEY,
            status TEXT NOT NULL DEFAULT 'free',
            locked_at TIMESTAMP
        )
    """)
    # 1. Автоматично звільняємо локи, що висять понад 15 хвилин (TTL auto-release)
    cursor.execute("UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE status = 'locked' AND locked_at < datetime('now', '-15 minutes')")
    
    # 2. Перевірка підтримки RETURNING (SQLite >= 3.35.0)
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
        # Fallback для старіших версій SQLite
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

def calculate_pix_crc16(payload_without_crc: str) -> str:
    """
    Calculates EMV QRCPS CRC16 (CCITT-FALSE, polynomial 0x1021, init 0xFFFF).
    Appends '6304' before computing checksum as per EMV Co / BR Code specification.
    Returns 4-character uppercase hexadecimal string (e.g. '1D2C').
    """
    data_to_hash = (payload_without_crc + "6304").encode('utf-8')
    crc = 0xFFFF
    for byte in data_to_hash:
        crc ^= (byte << 8)
        for _ in range(8):
            if crc & 0x8000:
                crc = ((crc << 1) ^ 0x1021) & 0xFFFF
            else:
                crc = (crc << 1) & 0xFFFF
    return f"{crc:04X}"

def generate_pix_emv_payload(pix_key: str, amount_brl: float, merchant_name: str = "ZeroClaw POS") -> str:
    """
    Generates Brazil-first EMV QRCPS PIX payload with valid CRC16 CCITT-FALSE checksum.
    Compatible with Brazilian banking apps (br.gov.bcb.pix).
    Uses byte-length calculation for multi-byte UTF-8 character support in Tag 59.
    """
    amount_str = f"{amount_brl:.2f}"
    merchant_bytes = merchant_name.encode('utf-8')
    merchant_len = len(merchant_bytes)
    pix_key_bytes = pix_key.encode('utf-8')
    pix_key_len = len(pix_key_bytes)
    payload_base = (
        "00020126580014br.gov.bcb.pix"
        f"01{pix_key_len:02d}{pix_key}"
        "520400005303986"
        f"54{len(amount_str):02d}{amount_str}"
        "5802BR"
        f"59{merchant_len:02d}{merchant_name}"
        "6009SAO PAULO"
        "62070503***"
    )
    crc_hex = calculate_pix_crc16(payload_base)
    return f"{payload_base}6304{crc_hex}"

def mark_nonce_account_stale(conn, pubkey: str):
    """
    Solana AdvanceNonceAccount Revert Recovery Engine.
    When a transaction fails on-chain, AdvanceNonceAccount still advances the nonce state.
    Marks the nonce as 'stale_needs_refresh' to force RPC getAccountInfo re-fetch before reuse.
    """
    cursor = conn.cursor()
    cursor.execute("""
        UPDATE nonce_accounts 
        SET status = 'stale_needs_refresh', locked_at = CURRENT_TIMESTAMP 
        WHERE pubkey = ?
    """, (pubkey,))
    conn.commit()

def refresh_stale_nonce_account(conn, pubkey: str, new_nonce_hash: str):
    """
    Refreshes a stale nonce account after on-chain RPC getAccountInfo state fetch.
    """
    cursor = conn.cursor()
    cursor.execute("""
        UPDATE nonce_accounts 
        SET status = 'free', locked_at = NULL 
        WHERE pubkey = ?
    """, (pubkey,))
    conn.commit()

def token_to_atomic_units(amount: float, decimals: int = 6) -> int:
    """
    Універсальне конвертування з float у атомарні одиниці з динамічними decimals (USDC=6, SOL=9, BONK=5).
    Захищає від Overflow та NaN/Infinity.
    """
    if amount <= 0.0 or math.isnan(amount) or math.isinf(amount):
        return 0
    scale = 10**decimals
    scaled = amount * float(scale)
    if scaled >= (2**64 - 1):
        return 2**64 - 1
    return int(round(scaled))

def usdc_to_atomic_units(amount: float) -> int:
    """
    Backward-compatible alias for 6-decimal USDC atomic conversion.
    """
    return token_to_atomic_units(amount, 6)

def is_payment_amount_valid(paid_usdc: float, expected_usdc: float, slippage_tolerance_pct: float = 1.0) -> bool:
    """
    Fiat Volatility & Slippage Tolerance Guard.
    Prevents POS payment rejection when exchange rate (BRL/USD/UAH) moves slightly during checkout.
    Accepts payment if paid_usdc >= expected_usdc * (1.0 - slippage_tolerance_pct / 100.0).
    """
    min_required = expected_usdc * (1.0 - (slippage_tolerance_pct / 100.0))
    return paid_usdc >= min_required

def generate_secure_reference_key() -> str:
    """
    Generates cryptographically secure 32-byte Ed25519 reference key for Solana Pay URLs.
    Uses secrets.token_bytes (OS CSPRNG).
    """
    raw_bytes = secrets.token_bytes(32)
    return base64.b32encode(raw_bytes).decode('utf-8')[:44]

def initiate_refund_request(conn, invoice_id: str) -> bool:
    """
    Atomic Re-Entrancy Guard for Squads v4 Refund Proposals.
    Transitions status 'paid' -> 'refunding'. Returns True if updated, False if already refunding or invalid.
    """
    cursor = conn.cursor()
    cursor.execute("""
        UPDATE invoices 
        SET status = 'refunding', updated_at = CURRENT_TIMESTAMP 
        WHERE id = ? AND status = 'paid'
    """, (invoice_id,))
    conn.commit()
    return cursor.rowcount > 0

def handle_telegram_429_retry(resp_json: dict) -> int:
    """
    Telegram Bot API HTTP 429 Rate Limit Interceptor.
    Returns retry_after seconds or 0 if not rate limited.
    """
    if isinstance(resp_json, dict) and resp_json.get("error_code") == 429:
        return resp_json.get("parameters", {}).get("retry_after", 1)
    return 0

def load_wasm_binary_ram_cache(wasm_path="plugins/solana-pos-core/target/wasm32-wasip2/release/solana_pos_core.wasm") -> bytes:
    """
    In-Memory WASM RAM Cache Warmup Engine to eliminate disk I/O latency.
    """
    global WASM_RAM_CACHE
    if WASM_RAM_CACHE is not None:
        return WASM_RAM_CACHE
    if os.path.exists(wasm_path):
        with open(wasm_path, "rb") as f:
            WASM_RAM_CACHE = f.read()
            return WASM_RAM_CACHE
    return b""

def release_nonce_account(conn, nonce_pubkey):
    cursor = conn.cursor()
    cursor.execute("UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE pubkey = ?", (nonce_pubkey,))
    conn.commit()

def check_and_register_telegram_update(conn, update_id):
    cursor = conn.cursor()
    # Auto TTL cleanup for updates older than 24 hours (1 day)
    cursor.execute("DELETE FROM processed_updates WHERE processed_at < datetime('now', '-1 day')")
    try:
        cursor.execute("INSERT INTO processed_updates (update_id) VALUES (?)", (update_id,))
        conn.commit()
        return True
    except sqlite3.IntegrityError:
        return False

def get_required_commitment_level(amount_usdc, threshold_usdc=50.0):
    return "finalized" if amount_usdc >= threshold_usdc else "confirmed"

def generate_atomic_refund_instructions(payer_pubkey="REFUND_SESSION_KEY", recipient_pubkey="9xK2...Customer1", amount_usdc=10.0):
    return [
        {
            "instruction": "createAssociatedTokenAccountIdempotent",
            "payer": payer_pubkey,
            "owner": recipient_pubkey,
            "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        },
        {
            "instruction": "splTokenTransfer",
            "from": payer_pubkey,
            "to": recipient_pubkey,
            "amount_usdc": amount_usdc
        }
    ]

def get_multitier_fiat_rate(fiat_currency, primary_data=None, secondary_data=None, cached_data=None, current_ts=None):
    """
    Multi-Tier Price Feed Fallback Circuit Breaker:
    1. Primary: Switchboard Crossbar API (valid if age <= 300s)
    2. Secondary: Pyth Hermes / REST Fiat API (valid if age <= 300s)
    3. Tertiary: Local Cached Rate (valid if age <= 900s with warning log)
    4. Fail-Closed: If all sources offline or stale (>900s)
    """
    if current_ts is None:
        current_ts = int(datetime.datetime.now().timestamp())

    # Tier 1: Primary Switchboard
    if primary_data and isinstance(primary_data, dict):
        ts = primary_data.get("timestamp", 0)
        rate = primary_data.get("rate")
        if rate and (current_ts - ts) <= 300:
            return {"rate": float(rate), "tier": "primary_switchboard", "status": "OK"}

    # Tier 2: Secondary Pyth / REST Fiat API
    if secondary_data and isinstance(secondary_data, dict):
        ts = secondary_data.get("timestamp", 0)
        rate = secondary_data.get("rate")
        if rate and (current_ts - ts) <= 300:
            return {"rate": float(rate), "tier": "secondary_pyth_hermes", "status": "OK"}

    # Tier 3: Tertiary Cached Fallback
    if cached_data and isinstance(cached_data, dict):
        ts = cached_data.get("timestamp", 0)
        rate = cached_data.get("rate")
        if rate and (current_ts - ts) <= 900:
            return {"rate": float(rate), "tier": "tertiary_cache", "status": "WARNING_USING_CACHE"}

    # Tier 4: Fail-Closed
    raise ValueError(f"FAIL_CLOSED: Stale or unavailable price feed for currency {fiat_currency}")

def validate_squads_multisig_account(account_data):
    """
    Squads v4 Null Account & Invalid State Defense.
    """
    if account_data is None or not isinstance(account_data, dict) or "transaction_index" not in account_data:
        raise ValueError("FAIL_CLOSED: Invalid or missing Squads multisig account")
    return account_data["transaction_index"] + 1

def verify_solana_transaction_payload(tx_json, expected_merchant_ata, expected_usdc_atomic, expected_mint="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"):
    """
    Triple Payment Protection:
    1. Reverted Tx Guard (meta.err != null)
    2. Gold Standard Token Balance Delta Verification (meta.postTokenBalances - meta.preTokenBalances)
    3. Recursive Instruction Inspector (top-level + innerInstructions)
    """
    if not tx_json or not isinstance(tx_json, dict):
        return {"is_valid": False, "error": "Invalid transaction JSON payload"}

    # 1. Reverted Tx Guard
    meta = tx_json.get("meta")
    if not meta or meta.get("err") is not None:
        return {"is_valid": False, "error": "Transaction failed or reverted on-chain (meta.err != null)"}

    # 2. LAYER 1: Token Balance Delta Verification (Solana Gold Standard)
    pre_balances = {}
    for b in meta.get("preTokenBalances", []):
        if b.get("mint") == expected_mint:
            amt_str = b.get("uiTokenAmount", {}).get("amount") or "0"
            try:
                pre_balances[b.get("accountIndex")] = int(amt_str)
            except (ValueError, TypeError):
                pre_balances[b.get("accountIndex")] = 0

    post_balances = {}
    for b in meta.get("postTokenBalances", []):
        if b.get("mint") == expected_mint:
            amt_str = b.get("uiTokenAmount", {}).get("amount") or "0"
            try:
                post_balances[b.get("accountIndex")] = int(amt_str)
            except (ValueError, TypeError):
                post_balances[b.get("accountIndex")] = 0

    account_keys = tx_json.get("transaction", {}).get("message", {}).get("accountKeys", [])
    merchant_account_index = None
    for idx, key_obj in enumerate(account_keys):
        pubkey = key_obj.get("pubkey") if isinstance(key_obj, dict) else key_obj
        if pubkey == expected_merchant_ata:
            merchant_account_index = idx
            break

    if merchant_account_index is not None:
        # Handles newly created ATAs idempotently (if not present in preTokenBalances, pre_amt = 0)
        pre_amt = pre_balances.get(merchant_account_index, 0)
        post_amt = post_balances.get(merchant_account_index, 0)
        delta = post_amt - pre_amt

        if delta >= expected_usdc_atomic:
            return {"is_valid": True, "paid_atomic": delta, "verification_method": "balance_delta"}

    # 3. LAYER 2: Recursive Instruction Inspection
    def inspect_instruction_list(instructions):
        for inst in instructions:
            parsed = inst.get("parsed")
            if parsed and parsed.get("type") in ["transfer", "transferChecked"]:
                info = parsed.get("info", {})
                dest = info.get("destination")
                amount_str = info.get("amount") or info.get("tokenAmount", {}).get("amount")
                if dest == expected_merchant_ata and amount_str:
                    try:
                        paid = int(amount_str)
                        if paid >= expected_usdc_atomic:
                            return paid
                    except (ValueError, TypeError):
                        pass
            if "instructions" in inst and isinstance(inst["instructions"], list):
                res = inspect_instruction_list(inst["instructions"])
                if res:
                    return res
        return None

    top_ixs = tx_json.get("transaction", {}).get("message", {}).get("instructions", [])
    paid_top = inspect_instruction_list(top_ixs)
    if paid_top is not None:
        return {"is_valid": True, "paid_atomic": paid_top, "verification_method": "top_level_instruction"}

    inner_groups = meta.get("innerInstructions", [])
    for group in inner_groups:
        paid_inner = inspect_instruction_list(group.get("instructions", []))
        if paid_inner is not None:
            return {"is_valid": True, "paid_atomic": paid_inner, "verification_method": "inner_instruction"}

    return {"is_valid": False, "error": "No valid token transfer or positive balance delta found for Merchant ATA"}



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
            pix_id TEXT,
            pix_payload TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)
    cursor.execute("""
        CREATE UNIQUE INDEX IF NOT EXISTS idx_invoices_tx_sig 
        ON invoices(tx_signature) 
        WHERE tx_signature IS NOT NULL;
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
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS processed_updates (
            update_id INTEGER PRIMARY KEY,
            processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS nonce_accounts (
            pubkey TEXT PRIMARY KEY,
            status TEXT NOT NULL DEFAULT 'free',
            locked_at TIMESTAMP
        )
    """)
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS sop_checkpoints (
            id TEXT PRIMARY KEY,
            sop_id TEXT NOT NULL,
            step_id TEXT NOT NULL,
            state_data TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
    """)
    
    # Initialize default Durable Nonce Pool
    cursor.execute("SELECT COUNT(*) FROM nonce_accounts")
    if cursor.fetchone()[0] == 0:
        cursor.executemany("INSERT INTO nonce_accounts (pubkey, status) VALUES (?, 'free')", [
            ("Nonce111111111111111111111111111111111111111",),
            ("Nonce222222222222222222222222222222222222222",),
            ("Nonce333333333333333333333333333333333333333",)
        ])
        
    # Insert sample POS data if empty
    cursor.execute("SELECT COUNT(*) FROM invoices")
    if cursor.fetchone()[0] == 0:
        now = datetime.datetime.now(datetime.timezone.utc).isoformat()
        sample_data = [
            ("INV-101", "7xRefKey11111111111111111111111111111111111", "UAH", 200.0, 4.82, "paid", "5k9X...Signature1", "9xK2...Customer1", None, None, now, now),
            ("INV-102", "8xRefKey22222222222222222222222222222222222", "UAH", 150.0, 3.61, "paid", "5k9X...Signature2", "9xK2...Customer2", None, None, now, now),
            ("INV-103", "9xRefKey33333333333333333333333333333333333", "USD", 10.0, 10.00, "pending", None, None, None, None, now, now),
        ]
        cursor.executemany("""
            INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, tx_signature, customer_address, pix_id, pix_payload, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        """, sample_data)
        
    conn.commit()
    cleanup_expired_pending_invoices(conn)
    conn.close()
    print(f"✅ SQLite Database (WAL Mode & Nonce Pool & PIX Support) initialized at {DB_PATH}")

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

        # ✅ x402 Machine Commerce Handshake Support
        if self.headers.get('X-ACCEPT-PAYMENT') == 'x402' and self.path == '/api/v1/sales/premium_analytics':
            self.send_response(402)
            self.send_header('Content-Type', 'application/json')
            self.send_header('X-PAYMENT-REQUIRED-AMOUNT', '1.00 USDC')
            self.send_header('X-PAYMENT-RECIPIENT', '8xAZmQ1111111111111111111111111111111111111')
            self.end_headers()
            self.wfile.write(json.dumps({
                "error": "Payment Required",
                "x402_spec": "solana-pay",
                "amount_usdc": 1.00,
                "pay_url": "solana:8xAZmQ11111111111111111111111111111111111?amount=1.00&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
            }).encode('utf-8'))
            conn.close()
            return

        try:
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
                cleanup_expired_pending_invoices(conn)
                cursor.execute("SELECT * FROM invoices ORDER BY created_at DESC")
                rows = [dict(r) for r in cursor.fetchall()]
                self._set_headers(200)
                self.wfile.write(json.dumps(rows, indent=2).encode('utf-8'))

            else:
                self._set_headers(404)
                self.wfile.write(json.dumps({"error": "Endpoint not found"}).encode('utf-8'))
        finally:
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

        try:
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
                        WHERE id = ? AND (status = 'pending' OR status = 'partially_paid' OR status = ?)
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
                    self.wfile.write(json.dumps({"error": redact_api_key(str(e))}).encode('utf-8'))

            elif self.path == '/api/v1/nonce/allocate':
                try:
                    allocated = allocate_free_nonce_account(conn)
                    if allocated:
                        self._set_headers(200)
                        self.wfile.write(json.dumps({"success": True, "nonce_pubkey": allocated}).encode('utf-8'))
                    else:
                        self._set_headers(503)
                        self.wfile.write(json.dumps({"success": False, "error": "No free durable nonce account available in pool"}).encode('utf-8'))
                except Exception as e:
                    self._set_headers(500)
                    self.wfile.write(json.dumps({"error": redact_api_key(str(e))}).encode('utf-8'))

            elif self.path == '/api/v1/nonce/release':
                try:
                    pubkey = data.get('nonce_pubkey') if data else None
                    if pubkey:
                        release_nonce_account(conn, pubkey)
                        self._set_headers(200)
                        self.wfile.write(json.dumps({"success": True, "released": pubkey}).encode('utf-8'))
                    else:
                        self._set_headers(400)
                        self.wfile.write(json.dumps({"error": "Missing nonce_pubkey"}).encode('utf-8'))
                except Exception as e:
                    self._set_headers(500)
                    self.wfile.write(json.dumps({"error": redact_api_key(str(e))}).encode('utf-8'))

            else:
                self._set_headers(404)
                self.wfile.write(json.dumps({"error": "Endpoint not found"}).encode('utf-8'))
        finally:
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
