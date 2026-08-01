#!/usr/bin/env python3
"""ZeroClaw Solana POS Agent - Solana Pay Verification & Settlement Core Module"""
import os, sys, math, secrets, base64, sqlite3
from typing import Dict, List, Optional, Any, Union
from pos_core.constants import (
    USDC_DECIMALS,
    MAX_U64,
    USDC_MINT,
    BASE58_ALPHABET,
    DEFAULT_SLIPPAGE_TOLERANCE_PCT,
    DEFAULT_COMMITMENT_THRESHOLD_USDC
)

WASM_RAM_CACHE: Optional[bytes] = None

def token_to_atomic_units(amount: Union[float, str], decimals: int = USDC_DECIMALS) -> int:
    """Converts float/string amount to atomic units with dynamic decimals (USDC=6, SOL=9)."""
    try:
        val = float(amount)
    except (ValueError, TypeError):
        return 0
    if val <= 0.0 or math.isnan(val) or math.isinf(val): return 0
    scale = 10**decimals
    scaled = val * float(scale)
    return MAX_U64 if scaled >= MAX_U64 else int(round(scaled))

def usdc_to_atomic_units(amount: Union[float, str]) -> int:
    """Backward-compatible alias for 6-decimal USDC atomic conversion."""
    return token_to_atomic_units(amount, USDC_DECIMALS)

def calculate_token2022_fee(amount_usdc: float, fee_basis_points: int, max_fee_units: int, decimals: int = USDC_DECIMALS) -> float:
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
    return all(c in BASE58_ALPHABET for c in pubkey_str)

def is_payment_amount_valid(paid_usdc: float, expected_usdc: float, slippage_tolerance_pct: float = DEFAULT_SLIPPAGE_TOLERANCE_PCT) -> bool:
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

def handle_telegram_429_retry(resp_json: Dict[str, Any]) -> int:
    """Telegram Bot API HTTP 429 Rate Limit Interceptor."""
    if isinstance(resp_json, dict) and resp_json.get("error_code") == 429:
        params = resp_json.get("parameters")
        if isinstance(params, dict):
            return int(params.get("retry_after", 1))
        return 1
    return 0

def load_wasm_binary_ram_cache(wasm_path: str = "plugins/solana-pos-core/target/wasm32-wasip2/release/solana_pos_core.wasm") -> bytes:
    """In-Memory WASM RAM Cache Warmup Engine."""
    global WASM_RAM_CACHE
    if WASM_RAM_CACHE is not None: return WASM_RAM_CACHE
    if os.path.exists(wasm_path):
        with open(wasm_path, "rb") as f:
            WASM_RAM_CACHE = f.read()
            return WASM_RAM_CACHE
    return b""

def get_required_commitment_level(amount_usdc: float, threshold_usdc: float = DEFAULT_COMMITMENT_THRESHOLD_USDC) -> str:
    return "finalized" if amount_usdc >= threshold_usdc else "confirmed"

def generate_atomic_refund_instructions(payer_pubkey: str = "REFUND_SESSION_KEY", recipient_pubkey: str = "9xK2...Customer1", amount_usdc: float = 10.0, mint: str = USDC_MINT) -> List[Dict[str, Any]]:
    return [
        {"instruction": "createAssociatedTokenAccountIdempotent", "payer": payer_pubkey, "owner": recipient_pubkey, "mint": mint},
        {"instruction": "splTokenTransfer", "from": payer_pubkey, "to": recipient_pubkey, "amount_usdc": amount_usdc}
    ]

def validate_squads_multisig_account(account_data: Optional[Dict[str, Any]]) -> int:
    """Squads v4 Null Account & Invalid State Defense."""
    if account_data is None or not isinstance(account_data, dict) or "transaction_index" not in account_data:
        raise ValueError("FAIL_CLOSED: Invalid or missing Squads multisig account")
    return int(account_data["transaction_index"]) + 1

def _extract_token_balance_deltas(meta: Dict[str, Any], expected_mint: str, debug_log: bool = False) -> Dict[int, int]:
    """Extracts balance differences (post - pre) for specified mint indexed by account index."""
    pre_balances: Dict[int, int] = {}
    post_balances: Dict[int, int] = {}
    
    for b in (meta.get("preTokenBalances") or []):
        if isinstance(b, dict) and b.get("mint") == expected_mint:
            try:
                ui_amt = b.get("uiTokenAmount") or {}
                amt_val = int((ui_amt.get("amount") if isinstance(ui_amt, dict) else None) or "0")
                idx = int(b.get("accountIndex"))
                pre_balances[idx] = amt_val
            except (ValueError, TypeError, AttributeError) as e:
                if debug_log:
                    print(f"[DEBUG][solana_pay] Suppressed preTokenBalance parsing error: {e}", file=sys.stderr)

    for b in (meta.get("postTokenBalances") or []):
        if isinstance(b, dict) and b.get("mint") == expected_mint:
            try:
                ui_amt = b.get("uiTokenAmount") or {}
                amt_val = int((ui_amt.get("amount") if isinstance(ui_amt, dict) else None) or "0")
                idx = int(b.get("accountIndex"))
                post_balances[idx] = amt_val
            except (ValueError, TypeError, AttributeError) as e:
                if debug_log:
                    print(f"[DEBUG][solana_pay] Suppressed postTokenBalance parsing error: {e}", file=sys.stderr)

    deltas: Dict[int, int] = {}
    all_indices = set(pre_balances.keys()) | set(post_balances.keys())
    for idx in all_indices:
        deltas[idx] = post_balances.get(idx, 0) - pre_balances.get(idx, 0)
    return deltas

def _inspect_instructions_for_transfer(instructions: Optional[List[Any]], expected_merchant_ata: str, expected_usdc_atomic: int, debug_log: bool = False) -> Optional[int]:
    """Recursively inspects instructions for token transfers matching Merchant ATA and expected amount."""
    for inst in (instructions or []):
        if not isinstance(inst, dict):
            continue
        parsed = inst.get("parsed")
        if isinstance(parsed, dict) and parsed.get("type") in ["transfer", "transferChecked"]:
            info = parsed.get("info") or {}
            if isinstance(info, dict) and info.get("destination") == expected_merchant_ata:
                token_amt = info.get("tokenAmount") or {}
                amt_str = info.get("amount") or (token_amt.get("amount") if isinstance(token_amt, dict) else None)
                if amt_str:
                    try:
                        amt_val = int(amt_str)
                        if amt_val >= expected_usdc_atomic:
                            return amt_val
                    except (ValueError, TypeError, AttributeError) as e:
                        if debug_log:
                            print(f"[DEBUG][solana_pay] Suppressed transfer amount parsing error: {e}", file=sys.stderr)
        nested_ixs = inst.get("instructions")
        if isinstance(nested_ixs, list):
            res = _inspect_instructions_for_transfer(nested_ixs, expected_merchant_ata, expected_usdc_atomic, debug_log=debug_log)
            if res is not None:
                return res
    return None

def verify_solana_transaction_payload(tx_json: Any, expected_merchant_ata: str, expected_usdc_atomic: int, expected_mint: str = USDC_MINT, debug_log: bool = False) -> Dict[str, Any]:
    """Triple Payment Protection: Reverted Tx Guard, Balance Delta Verification, Recursive Instruction Inspection."""
    if not tx_json or not isinstance(tx_json, dict): return {"is_valid": False, "error": "Invalid transaction JSON payload"}
    meta = tx_json.get("meta")
    if not meta or not isinstance(meta, dict) or meta.get("err") is not None: return {"is_valid": False, "error": "Transaction failed or reverted on-chain (meta.err != null)"}

    deltas = _extract_token_balance_deltas(meta, expected_mint, debug_log=debug_log)
    transaction = tx_json.get("transaction") or {}
    message = (transaction.get("message") if isinstance(transaction, dict) else {}) or {}
    account_keys = (message.get("accountKeys") if isinstance(message, dict) else []) or []
    
    merchant_idx = next((i for i, k in enumerate(account_keys) if (k.get("pubkey") if isinstance(k, dict) else k) == expected_merchant_ata), None)

    if merchant_idx is not None:
        delta = deltas.get(merchant_idx, 0)
        if delta >= expected_usdc_atomic: return {"is_valid": True, "paid_atomic": delta, "verification_method": "balance_delta"}

    top_ixs = (message.get("instructions") if isinstance(message, dict) else []) or []
    paid_top = _inspect_instructions_for_transfer(top_ixs, expected_merchant_ata, expected_usdc_atomic, debug_log=debug_log)
    if paid_top is not None: return {"is_valid": True, "paid_atomic": paid_top, "verification_method": "top_level_instruction"}

    inner_ixs = meta.get("innerInstructions") or []
    for group in (inner_ixs if isinstance(inner_ixs, list) else []):
        if isinstance(group, dict):
            paid_inner = _inspect_instructions_for_transfer(group.get("instructions"), expected_merchant_ata, expected_usdc_atomic, debug_log=debug_log)
            if paid_inner is not None: return {"is_valid": True, "paid_atomic": paid_inner, "verification_method": "inner_instruction"}

    return {"is_valid": False, "error": "No valid token transfer or positive balance delta found for Merchant ATA"}
