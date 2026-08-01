#!/usr/bin/env python3
"""ZeroClaw Solana POS Agent - Solana Pay Verification & Settlement Core Module"""
import os, math, secrets, base64, sqlite3

WASM_RAM_CACHE = None

def token_to_atomic_units(amount: float, decimals: int = 6) -> int:
    """Converts float/string amount to atomic units with dynamic decimals (USDC=6, SOL=9)."""
    try:
        val = float(amount)
    except (ValueError, TypeError):
        return 0
    if val <= 0.0 or math.isnan(val) or math.isinf(val): return 0
    scale = 10**decimals
    scaled = val * float(scale)
    return (2**64 - 1) if scaled >= (2**64 - 1) else int(round(scaled))

def usdc_to_atomic_units(amount: float) -> int:
    """Backward-compatible alias for 6-decimal USDC atomic conversion."""
    return token_to_atomic_units(amount, 6)

def calculate_token2022_fee(amount_usdc: float, fee_basis_points: int, max_fee_units: int, decimals: int = 6) -> float:
    """Calculates Token-2022 transfer fee with ceiling rounding and max fee cap."""
    scale = 10**decimals
    if fee_basis_points > 10000: return max_fee_units / float(scale)
    amount_units = 0 if (amount_usdc <= 0.0 or math.isnan(amount_usdc) or math.isinf(amount_usdc)) else int(round(amount_usdc * float(scale)))
    if amount_units == 0: return 0.0
    fee_units = (amount_units * fee_basis_points + 9999) // 10000
    return min(fee_units, max_fee_units) / float(scale)

def is_valid_base58(pubkey_str: str) -> bool:
    """Validates Solana Base58 public key format (32-44 chars, valid alphabet)."""
    if not isinstance(pubkey_str, str) or len(pubkey_str) < 32 or len(pubkey_str) > 44: return False
    BASE58_ALPHABET = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"
    return all(c in BASE58_ALPHABET for c in pubkey_str)

def is_payment_amount_valid(paid_usdc: float, expected_usdc: float, slippage_tolerance_pct: float = 1.0) -> bool:
    """Fiat Volatility & Slippage Tolerance Guard."""
    return paid_usdc >= (expected_usdc * (1.0 - (slippage_tolerance_pct / 100.0)))

def generate_secure_reference_key() -> str:
    """Generates cryptographically secure 32-byte Ed25519 reference key."""
    return base64.b32encode(secrets.token_bytes(32)).decode('utf-8')[:44]

def initiate_refund_request(conn: sqlite3.Connection, invoice_id: str) -> bool:
    """Atomic Re-Entrancy Guard for Squads v4 Refund Proposals."""
    cursor = conn.cursor()
    cursor.execute("UPDATE invoices SET status = 'refunding', updated_at = CURRENT_TIMESTAMP WHERE id = ? AND status = 'paid'", (invoice_id,))
    conn.commit()
    return cursor.rowcount > 0

def handle_telegram_429_retry(resp_json: dict) -> int:
    """Telegram Bot API HTTP 429 Rate Limit Interceptor."""
    if isinstance(resp_json, dict) and resp_json.get("error_code") == 429:
        return resp_json.get("parameters", {}).get("retry_after", 1)
    return 0

def load_wasm_binary_ram_cache(wasm_path="plugins/solana-pos-core/target/wasm32-wasip2/release/solana_pos_core.wasm") -> bytes:
    """In-Memory WASM RAM Cache Warmup Engine."""
    global WASM_RAM_CACHE
    if WASM_RAM_CACHE is not None: return WASM_RAM_CACHE
    if os.path.exists(wasm_path):
        with open(wasm_path, "rb") as f:
            WASM_RAM_CACHE = f.read()
            return WASM_RAM_CACHE
    return b""

def get_required_commitment_level(amount_usdc, threshold_usdc=50.0):
    return "finalized" if amount_usdc >= threshold_usdc else "confirmed"

def generate_atomic_refund_instructions(payer_pubkey="REFUND_SESSION_KEY", recipient_pubkey="9xK2...Customer1", amount_usdc=10.0):
    return [
        {"instruction": "createAssociatedTokenAccountIdempotent", "payer": payer_pubkey, "owner": recipient_pubkey, "mint": "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"},
        {"instruction": "splTokenTransfer", "from": payer_pubkey, "to": recipient_pubkey, "amount_usdc": amount_usdc}
    ]

def validate_squads_multisig_account(account_data):
    """Squads v4 Null Account & Invalid State Defense."""
    if account_data is None or not isinstance(account_data, dict) or "transaction_index" not in account_data:
        raise ValueError("FAIL_CLOSED: Invalid or missing Squads multisig account")
    return account_data["transaction_index"] + 1

def verify_solana_transaction_payload(tx_json, expected_merchant_ata, expected_usdc_atomic, expected_mint="EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"):
    """Triple Payment Protection: Reverted Tx Guard, Balance Delta Verification, Recursive Instruction Inspection."""
    if not tx_json or not isinstance(tx_json, dict): return {"is_valid": False, "error": "Invalid transaction JSON payload"}
    meta = tx_json.get("meta")
    if not meta or meta.get("err") is not None: return {"is_valid": False, "error": "Transaction failed or reverted on-chain (meta.err != null)"}

    pre_balances, post_balances = {}, {}
    for b in meta.get("preTokenBalances", []):
        if b.get("mint") == expected_mint:
            try: pre_balances[b.get("accountIndex")] = int(b.get("uiTokenAmount", {}).get("amount") or "0")
            except (ValueError, TypeError): pre_balances[b.get("accountIndex")] = 0

    for b in meta.get("postTokenBalances", []):
        if b.get("mint") == expected_mint:
            try: post_balances[b.get("accountIndex")] = int(b.get("uiTokenAmount", {}).get("amount") or "0")
            except (ValueError, TypeError): post_balances[b.get("accountIndex")] = 0

    account_keys = tx_json.get("transaction", {}).get("message", {}).get("accountKeys", [])
    merchant_idx = next((i for i, k in enumerate(account_keys) if (k.get("pubkey") if isinstance(k, dict) else k) == expected_merchant_ata), None)

    if merchant_idx is not None:
        delta = post_balances.get(merchant_idx, 0) - pre_balances.get(merchant_idx, 0)
        if delta >= expected_usdc_atomic: return {"is_valid": True, "paid_atomic": delta, "verification_method": "balance_delta"}

    def inspect_instructions(instructions):
        for inst in instructions:
            parsed = inst.get("parsed")
            if parsed and parsed.get("type") in ["transfer", "transferChecked"]:
                info = parsed.get("info", {})
                if info.get("destination") == expected_merchant_ata:
                    amt_str = info.get("amount") or info.get("tokenAmount", {}).get("amount")
                    if amt_str and int(amt_str) >= expected_usdc_atomic: return int(amt_str)
            if "instructions" in inst and isinstance(inst["instructions"], list):
                res = inspect_instructions(inst["instructions"])
                if res: return res
        return None

    top_ixs = tx_json.get("transaction", {}).get("message", {}).get("instructions", [])
    paid_top = inspect_instructions(top_ixs)
    if paid_top is not None: return {"is_valid": True, "paid_atomic": paid_top, "verification_method": "top_level_instruction"}

    for group in meta.get("innerInstructions", []):
        paid_inner = inspect_instructions(group.get("instructions", []))
        if paid_inner is not None: return {"is_valid": True, "paid_atomic": paid_inner, "verification_method": "inner_instruction"}

    return {"is_valid": False, "error": "No valid token transfer or positive balance delta found for Merchant ATA"}
