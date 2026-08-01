#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Multi-Tier Price Feed Fallback Circuit Breaker Module
"""

import datetime

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
