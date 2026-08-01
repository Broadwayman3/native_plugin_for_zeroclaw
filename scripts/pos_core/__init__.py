#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Core Package Init
Exposes unified domain APIs from submodules.
"""

from pos_core.db import (
    DB_PATH,
    get_db_connection,
    cleanup_db_files,
    init_db,
    cleanup_expired_pending_invoices,
    check_and_register_telegram_update
)
from pos_core.nonce_pool import (
    allocate_free_nonce_account,
    release_nonce_account,
    mark_nonce_account_stale,
    refresh_stale_nonce_account
)
from pos_core.solana_pay import (
    token_to_atomic_units,
    usdc_to_atomic_units,
    calculate_token2022_fee,
    is_valid_base58,
    is_payment_amount_valid,
    generate_secure_reference_key,
    initiate_refund_request,
    handle_telegram_429_retry,
    load_wasm_binary_ram_cache,
    get_required_commitment_level,
    generate_atomic_refund_instructions,
    validate_squads_multisig_account,
    verify_solana_transaction_payload
)
from pos_core.pix_brl import (
    calculate_pix_crc16,
    generate_pix_emv_payload
)
from pos_core.price_feed import (
    get_multitier_fiat_rate
)
from pos_core.router import (
    route_get,
    route_post,
    dispatch_request,
    send_json_response
)

__all__ = [
    "DB_PATH",
    "get_db_connection",
    "cleanup_db_files",
    "init_db",
    "cleanup_expired_pending_invoices",
    "check_and_register_telegram_update",
    "allocate_free_nonce_account",
    "release_nonce_account",
    "mark_nonce_account_stale",
    "refresh_stale_nonce_account",
    "token_to_atomic_units",
    "usdc_to_atomic_units",
    "calculate_token2022_fee",
    "is_valid_base58",
    "is_payment_amount_valid",
    "generate_secure_reference_key",
    "initiate_refund_request",
    "handle_telegram_429_retry",
    "load_wasm_binary_ram_cache",
    "get_required_commitment_level",
    "generate_atomic_refund_instructions",
    "validate_squads_multisig_account",
    "verify_solana_transaction_payload",
    "calculate_pix_crc16",
    "generate_pix_emv_payload",
    "get_multitier_fiat_rate",
    "route_get",
    "route_post",
    "dispatch_request",
    "send_json_response"
]
