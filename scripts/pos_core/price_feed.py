#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Multi-Tier Price Feed Fallback Circuit Breaker Module
"""

import datetime
from typing import Dict, Optional, Any

DEFAULT_STATIC_FIAT_RATES: Dict[str, float] = {
    # Major Global Currencies & Regional Markets
    "USD": 1.00,  # US Dollar
    "EUR": 0.92,  # Euro
    "BRL": 5.45,  # Brazilian Real (Bounty Primary Focus!)
    "UAH": 41.50,  # Ukrainian Hryvnia
    "GBP": 0.78,  # British Pound
    "CAD": 1.37,  # Canadian Dollar
    "JPY": 152.50,  # Japanese Yen
    "MXN": 19.80,  # Mexican Peso
    "PLN": 3.98,  # Polish Zloty
    "CHF": 0.88,  # Swiss Franc
    "AUD": 1.52,  # Australian Dollar
    "SEK": 10.45,  # Swedish Krona
    "NOK": 10.85,  # Norwegian Krone
    "DKK": 6.88,  # Danish Krone
    "NZD": 1.65,  # New Zealand Dollar
    "SGD": 1.34,  # Singapore Dollar
    "HKD": 7.81,  # Hong Kong Dollar
    "INR": 83.70,  # Indian Rupee
    "TRY": 33.10,  # Turkish Lira
    "ZAR": 18.20,  # South African Rand
    "AED": 3.67,  # UAE Dirham
    "CZK": 23.20,  # Czech Koruna
    "HUF": 365.00,  # Hungarian Forint
    "THB": 35.80,  # Thai Baht
    "PHP": 58.40,  # Philippine Peso
    "IDR": 16250.0,  # Indonesian Rupiah
    "ILS": 3.72,  # Israeli New Shekel
    "CLP": 940.00,  # Chilean Peso
    "COP": 4050.00,  # Colombian Peso
    "ARS": 930.00,  # Argentine Peso
}


def get_multitier_fiat_rate(
    fiat_currency: str,
    primary_data: Optional[Dict[str, Any]] = None,
    secondary_data: Optional[Dict[str, Any]] = None,
    cached_data: Optional[Dict[str, Any]] = None,
    current_ts: Optional[int] = None,
    allow_static_fallback: bool = True,
) -> Dict[str, Any]:
    """
    Multi-Tier Price Feed Fallback Circuit Breaker:
    1. Primary: Switchboard Crossbar API (valid if age <= 300s)
    2. Secondary: Pyth Hermes / REST Fiat API (valid if age <= 300s)
    3. Tertiary: Local Cached Rate (valid if age <= 900s with warning log)
    4. Quaternary: Static Offline Hardcoded Fallback Rate (guarantees offline availability for mapped currencies)
    """
    if current_ts is None:
        current_ts = int(datetime.datetime.now().timestamp())

    curr = (fiat_currency or "").upper().strip()

    # Tier 1: Primary Switchboard
    if primary_data and isinstance(primary_data, dict):
        ts = primary_data.get("timestamp", 0)
        if ts > 10**11:
            ts = ts // 1000
        rate = primary_data.get("rate")
        if rate and -15 <= (current_ts - ts) <= 300:
            return {"rate": float(rate), "tier": "primary_switchboard", "status": "OK"}

    # Tier 2: Secondary Pyth / REST Fiat API
    if secondary_data and isinstance(secondary_data, dict):
        ts = secondary_data.get("timestamp", 0)
        if ts > 10**11:
            ts = ts // 1000
        rate = secondary_data.get("rate")
        if rate and -15 <= (current_ts - ts) <= 300:
            return {"rate": float(rate), "tier": "secondary_pyth_hermes", "status": "OK"}

    # Tier 3: Tertiary Cached Fallback
    if cached_data and isinstance(cached_data, dict):
        ts = cached_data.get("timestamp", 0)
        if ts > 10**11:
            ts = ts // 1000
        rate = cached_data.get("rate")
        if rate and -15 <= (current_ts - ts) <= 900:
            return {"rate": float(rate), "tier": "tertiary_cache", "status": "WARNING_USING_CACHE"}

    # Tier 4: Quaternary Static Offline Fallback (only for mapped currencies)
    if allow_static_fallback:
        if curr in DEFAULT_STATIC_FIAT_RATES:
            return {"rate": float(DEFAULT_STATIC_FIAT_RATES[curr]), "tier": "quaternary_static_fallback", "status": "OFFLINE_STATIC_FALLBACK"}

    # Tier 5: Fail-Closed Guard (prevents 1.0 default fallback for unmapped currencies like JPY/MXN)
    raise ValueError(f"FAIL_CLOSED: Unsupported or unmapped fiat currency '{curr}'")
