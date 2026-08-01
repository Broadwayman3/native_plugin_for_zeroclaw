#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Domain Constants Module
Centralized source of truth for financial precision, token mints, system limits, and default parameters.
"""

import os

# Financial token decimal places
USDC_DECIMALS: int = 6
SOL_DECIMALS: int = 9

# Unsigned 64-bit integer upper bound guard
MAX_U64: int = 18446744073709551615

# Solana Token Mints (Dynamic Environment Resolution)
USDC_MINT_MAINNET: str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
USDC_MINT_DEVNET: str = "4zMMC9srt5Ri5X14GAgXhaHii3GnPAEERYPJgZJDncDU"
USDC_MINT: str = os.getenv("USDC_MINT_ADDRESS", USDC_MINT_MAINNET)

# Base58 Alphabet for Solana Public Key Validation
BASE58_ALPHABET: str = "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz"

# Payment and Volatility Guards
DEFAULT_SLIPPAGE_TOLERANCE_PCT: float = 1.0
DEFAULT_COMMITMENT_THRESHOLD_USDC: float = 50.0

# Nonce Account Expiry TTL (Minutes)
NONCE_TTL_MINUTES: int = 15

# Default Socket & HTTP Network Timeout (Seconds)
DEFAULT_SOCKET_TIMEOUT: float = 10.0
