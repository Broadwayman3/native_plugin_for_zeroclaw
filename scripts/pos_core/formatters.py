#!/usr/bin/env python3
"""ZeroClaw Solana POS Agent - Formatting & Public Key Utilities Module"""
import os
import urllib.parse
from typing import Optional
from pos_core.constants import BASE58_ALPHABET

def format_pubkey_short(pubkey: str) -> str:
    """Truncates long Base58 pubkeys/signatures for clean display (e.g. 8xAZ...mQ11)."""
    if not pubkey or len(pubkey) < 12:
        return pubkey or ""
    return f"{pubkey[:4]}...{pubkey[-4:]}"

def get_solscan_tx_url(signature: str, network: Optional[str] = None) -> str:
    """Generates direct transaction link to Solscan Explorer with automatic network detection from environment RPC URL."""
    if not network:
        rpc_url = os.getenv("SOLANA_RPC_URL", "").lower()
        network = "devnet" if "devnet" in rpc_url else ("mainnet" if ("mainnet" in rpc_url or "helius" in rpc_url) else "devnet")
    cluster_param = f"?cluster={network}" if network in ("devnet", "testnet") else ""
    return f"https://solscan.io/tx/{signature}{cluster_param}"

def is_valid_base58(pubkey_str: str) -> bool:
    """Validates Solana Base58 public key format (32-44 chars, valid alphabet)."""
    if not isinstance(pubkey_str, str) or len(pubkey_str) < 32 or len(pubkey_str) > 44:
        return False
    return all(c in BASE58_ALPHABET for c in pubkey_str)

def generate_solana_pay_qr_image_url(solana_pay_url: str, size: int = 300) -> str:
    """Generates direct QR code image rendering URL for instant visualization in chat."""
    encoded_url = urllib.parse.quote(solana_pay_url, safe="")
    return f"https://api.qrserver.com/v1/create-qr-code/?size={size}x{size}&data={encoded_url}"
