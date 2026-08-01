#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Multi-Tier Price Feed Fallback Circuit Breaker Module
"""

import datetime
from typing import Dict, Optional, Any

DEFAULT_STATIC_FIAT_RATES: Dict[str, float] = {
    "BRL": 5.45,
    "UAH": 41.50,
    "USD": 1.00,
    "EUR": 0.92
}

def get_multitier_fiat_rate(
    fiat_currency: str,
    primary_data: Optional[Dict[str, Any]] = None,
    secondary_data: Optional[Dict[str, Any]] = None,
    cached_data: Optional[Dict[str, Any]] = None,
    current_ts: Optional[int] = None,
    allow_static_fallback: bool = True
) -> Dict[str, Any]:
    """
    Multi-Tier Price Feed Fallback Circuit Breaker:
    1. Primary: Switchboard Crossbar API (valid if age <= 300s)
    2. Secondary: Pyth Hermes / REST Fiat API (valid if age <= 300s)
    3. Tertiary: Local Cached Rate (valid if age <= 900s with warning log)
    4. Quaternary: Static Offline Hardcoded Fallback Rate (guarantees offline availability)
    """
    if current_ts is None:
        current_ts = int(datetime.datetime.now().timestamp())

    # Tier 1: Primary Switchboard
    if primary_data and isinstance(primary_data, dict):
        ts = primary_data.get("timestamp", 0)
        if ts > 10**11:
            ts = ts // 1000
        rate = primary_data.get("rate")
        if rate and -5 <= (current_ts - ts) <= 300:
            return {"rate": float(rate), "tier": "primary_switchboard", "status": "OK"}

    # Tier 2: Secondary Pyth / REST Fiat API
    if secondary_data and isinstance(secondary_data, dict):
        ts = secondary_data.get("timestamp", 0)
        if ts > 10**11:
            ts = ts // 1000
        rate = secondary_data.get("rate")
        if rate and -5 <= (current_ts - ts) <= 300:
            return {"rate": float(rate), "tier": "secondary_pyth_hermes", "status": "OK"}

    # Tier 3: Tertiary Cached Fallback
    if cached_data and isinstance(cached_data, dict):
        ts = cached_data.get("timestamp", 0)
        if ts > 10**11:
            ts = ts // 1000
        rate = cached_data.get("rate")
        if rate and -5 <= (current_ts - ts) <= 900:
            return {"rate": float(rate), "tier": "tertiary_cache", "status": "WARNING_USING_CACHE"}

    # Tier 4: Quaternary Static Offline Fallback
    if allow_static_fallback:
        fallback_rate = DEFAULT_STATIC_FIAT_RATES.get((fiat_currency or "").upper(), 1.0)
        return {
            "rate": float(fallback_rate),
            "tier": "quaternary_static_fallback",
            "status": "OFFLINE_STATIC_FALLBACK"
        }

    # Tier 5: Fail-Closed
    raise ValueError(f"FAIL_CLOSED: Stale or unavailable price feed for currency {fiat_currency}")
