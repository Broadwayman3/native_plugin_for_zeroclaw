#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Core Package Init
Exposes unified domain APIs from submodules.
"""

from pos_core.constants import (
    USDC_DECIMALS,
    SOL_DECIMALS,
    MAX_U64,
    USDC_MINT_MAINNET,
    USDC_MINT_DEVNET,
    USDC_MINT,
    BASE58_ALPHABET,
    DEFAULT_SLIPPAGE_TOLERANCE_PCT,
    DEFAULT_COMMITMENT_THRESHOLD_USDC,
    NONCE_TTL_MINUTES,
    DEFAULT_SOCKET_TIMEOUT
)
from pos_core.db import (
    DB_PATH,
    get_db_connection,
    get_db_cursor,
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
    verify_solana_transaction_payload,
    generate_solana_pay_qr_image_url,
    generate_solana_pay_url,
    generate_phantom_universal_link,
    get_active_rpc_url,
    format_pubkey_short,
    get_solscan_tx_url,
    _extract_token_balance_deltas,
    _inspect_instructions_for_transfer
)
from pos_core.i18n import (
    TRANSLATIONS,
    get_localized_message,
    t,
    format_itemized_receipt,
    get_refund_checkpoint_inline_keyboard
)

from pos_core.pix_brl import (
    calculate_pix_crc16,
    generate_pix_emv_payload
)
from pos_core.price_feed import (
    DEFAULT_STATIC_FIAT_RATES,
    get_multitier_fiat_rate
)
from pos_core.router import (
    route_get,
    route_post,
    dispatch_request,
    send_json_response
)

__all__ = [
    "USDC_DECIMALS",
    "SOL_DECIMALS",
    "MAX_U64",
    "USDC_MINT_MAINNET",
    "USDC_MINT_DEVNET",
    "USDC_MINT",
    "BASE58_ALPHABET",
    "DEFAULT_SLIPPAGE_TOLERANCE_PCT",
    "DEFAULT_COMMITMENT_THRESHOLD_USDC",
    "NONCE_TTL_MINUTES",
    "DEFAULT_SOCKET_TIMEOUT",
    "DB_PATH",
    "get_db_connection",
    "get_db_cursor",
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
    "generate_solana_pay_qr_image_url",
    "generate_solana_pay_url",
    "generate_phantom_universal_link",
    "get_active_rpc_url",
    "format_pubkey_short",
    "get_solscan_tx_url",
    "TRANSLATIONS",
    "get_localized_message",
    "t",
    "format_itemized_receipt",
    "get_refund_checkpoint_inline_keyboard",

    "_extract_token_balance_deltas",
    "_inspect_instructions_for_transfer",
    "calculate_pix_crc16",
    "generate_pix_emv_payload",
    "DEFAULT_STATIC_FIAT_RATES",
    "get_multitier_fiat_rate",
    "route_get",
    "route_post",
    "dispatch_request",
    "send_json_response"
]
