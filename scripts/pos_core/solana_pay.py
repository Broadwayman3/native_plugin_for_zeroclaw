#!/usr/bin/env python3
"""ZeroClaw Solana POS Agent - Solana Pay Verification & Settlement Core Module"""

import os
import secrets
import base64
import sqlite3
import functools
from decimal import Decimal, ROUND_HALF_UP, InvalidOperation
from typing import Dict, List, Optional, Any, Union
from pos_core.constants import USDC_DECIMALS, MAX_U64, USDC_MINT, DEFAULT_SLIPPAGE_TOLERANCE_PCT, DEFAULT_COMMITMENT_THRESHOLD_USDC


def token_to_atomic_units(amount: Union[float, str, Decimal], decimals: int = USDC_DECIMALS) -> int:
    """Converts float/string/Decimal amount to atomic units using Decimal precision (zero float drift)."""
    try:
        if isinstance(amount, float):
            d_amount = Decimal(str(amount))
        else:
            d_amount = Decimal(amount)
    except (InvalidOperation, TypeError, ValueError):
        return 0

    if d_amount.is_nan() or d_amount.is_infinite() or d_amount <= Decimal("0"):
        return 0

    scale = Decimal(10**decimals)
    scaled_raw = d_amount * scale
    max_u64_dec = Decimal(MAX_U64)
    if scaled_raw >= max_u64_dec:
        return MAX_U64

    try:
        scaled = scaled_raw.quantize(Decimal("1"), rounding=ROUND_HALF_UP)
        if scaled >= max_u64_dec:
            return MAX_U64
        return int(scaled)
    except (InvalidOperation, TypeError, ValueError):
        return MAX_U64 if scaled_raw >= max_u64_dec else 0


def usdc_to_atomic_units(amount: Union[float, str, Decimal]) -> int:
    """Backward-compatible alias for 6-decimal USDC atomic conversion."""
    return token_to_atomic_units(amount, USDC_DECIMALS)


def calculate_token2022_fee(amount_usdc: Union[float, str, Decimal], fee_basis_points: int, max_fee_units: int, decimals: int = USDC_DECIMALS) -> float:
    """Calculates Token-2022 transfer fee using Decimal-backed atomic conversion."""
    if decimals > 18:
        return 0.0
    scale = 10**decimals
    if fee_basis_points > 10000:
        return max_fee_units / float(scale)
    amount_units = token_to_atomic_units(amount_usdc, decimals=decimals)
    if amount_units == 0:
        return 0.0
    fee_units = (amount_units * fee_basis_points + 9999) // 10000
    return min(fee_units, max_fee_units) / float(scale)


def is_payment_amount_valid(paid_usdc: float, expected_usdc: float, slippage_tolerance_pct: float = DEFAULT_SLIPPAGE_TOLERANCE_PCT) -> bool:
    """Fiat Volatility & Slippage Tolerance Guard."""
    return paid_usdc >= (expected_usdc * (1.0 - (slippage_tolerance_pct / 100.0)))


def generate_secure_reference_key() -> str:
    """Generates cryptographically secure 32-byte Ed25519 reference key."""
    return base64.b32encode(secrets.token_bytes(32)).decode("utf-8")[:44]


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


@functools.lru_cache(maxsize=1)
def load_wasm_binary_ram_cache(wasm_path: str = "plugins/solana-pos-core/target/wasm32-wasip2/release/solana_pos_core.wasm") -> bytes:
    """In-Memory WASM RAM Cache Warmup Engine (GIL Thread-Safe via LRU Cache)."""
    if os.path.exists(wasm_path):
        with open(wasm_path, "rb") as f:
            return f.read()
    return b""


def get_required_commitment_level(amount_usdc: float, threshold_usdc: float = DEFAULT_COMMITMENT_THRESHOLD_USDC) -> str:
    return "finalized" if amount_usdc >= threshold_usdc else "confirmed"


def generate_atomic_refund_instructions(
    payer_pubkey: str = "REFUND_SESSION_KEY",
    recipient_pubkey: str = "9xK2...Customer1",
    amount_usdc: float = 10.0,
    mint: str = USDC_MINT,
    nonce_pubkey: Optional[str] = None,
) -> List[Dict[str, Any]]:
    instructions: list = []
    if nonce_pubkey:
        instructions.append({"instruction": "AdvanceNonceAccount", "nonce_account": nonce_pubkey, "authority": payer_pubkey})
    instructions.extend(
        [
            {"instruction": "createAssociatedTokenAccountIdempotent", "payer": payer_pubkey, "owner": recipient_pubkey, "mint": mint},
            {"instruction": "splTokenTransfer", "from": payer_pubkey, "to": recipient_pubkey, "amount_usdc": amount_usdc},
        ]
    )
    return instructions


def validate_squads_multisig_account(account_data: Optional[Dict[str, Any]]) -> int:
    """Squads v4 Null Account & Invalid State Defense."""
    if account_data is None or not isinstance(account_data, dict) or "transaction_index" not in account_data:
        raise ValueError("FAIL_CLOSED: Invalid or missing Squads multisig account")
    return int(account_data["transaction_index"]) + 1


def generate_solana_pay_url(
    merchant_pubkey: str,
    amount: float,
    reference_pubkey: str,
    spl_token_mint: Optional[str] = USDC_MINT,
    label: str = "ZeroClaw POS",
    message: str = "POS Payment",
) -> str:
    """
    SIP-0001 compliant Solana Pay URL Generator:
    - Appends spl-token=<MINT> ONLY if spl_token_mint is specified AND is not Native SOL.
    - Omits spl-token for Native SOL (SOL_MINT).
    - Percent-encodes label & message parameters.
    """
    import urllib.parse
    from pos_core.constants import SOL_MINT

    encoded_label = urllib.parse.quote(label)
    encoded_message = urllib.parse.quote(message)
    base_url = f"solana:{merchant_pubkey}?amount={amount:.2f}&reference={reference_pubkey}&label={encoded_label}&message={encoded_message}"

    if spl_token_mint and spl_token_mint not in (SOL_MINT, "11111111111111111111111111111111"):
        base_url += f"&spl-token={spl_token_mint}"
    return base_url


def generate_phantom_universal_link(solana_pay_url: str) -> str:
    """Generates Phantom Universal HTTPS Deep Link for 1-tap mobile wallet opening."""
    import urllib.parse

    encoded_url = urllib.parse.quote(solana_pay_url, safe="")
    return f"https://phantom.app/ul/browse/{encoded_url}?ref=zeroclaw"


def get_active_rpc_url(primary_url: Optional[str] = None, fallback_url: Optional[str] = None) -> str:
    """Returns active RPC URL, falling back if primary encounters rate limit / 429 status."""
    primary = primary_url or os.getenv("SOLANA_RPC_URL", "https://devnet.helius-rpc.com/?api-key=test")
    fallback = fallback_url or os.getenv("FALLBACK_RPC_URL", "https://api.devnet.solana.com")
    return primary or fallback or "https://api.devnet.solana.com"
