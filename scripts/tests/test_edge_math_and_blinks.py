#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Edge Math, Blinks & EMV UTF8 Safe Suite (Tests 251-255)
"""

from decimal import Decimal
from pos_core import (
    token_to_atomic_units,
    generate_pix_emv_payload,
    calculate_pix_crc16,
    format_itemized_receipt,
    t
)
from pos_backend import handle_actions_spec_json, handle_action_get_invoice, handle_action_post_invoice

def test_251_decimal_exact_micro_lamport_math():
    """Guarantees 0.29 USDC converts to exactly 290,000 atomic units without IEEE 754 float drift."""
    assert token_to_atomic_units("0.29", decimals=6) == 290000
    assert token_to_atomic_units(0.29, decimals=6) == 290000
    assert token_to_atomic_units(Decimal("0.29"), decimals=6) == 290000

def test_252_pix_tag59_multi_byte_emoji_overflow_truncation():
    """Guarantees Tag 59 truncates names gracefully when byte length exceeds 99 bytes."""
    huge_emoji_name = "Padaria " + "🇧🇷" * 40  # > 120 bytes
    payload = generate_pix_emv_payload("merchant@pix.br", 10.0, huge_emoji_name)
    tag59_part = payload.split("5802BR")[1].split("6009SAO PAULO")[0]
    tag59_len = int(tag59_part[2:4])
    assert tag59_len <= 99

def test_253_solana_actions_blinks_json_schema():
    """Verifies Blink Action response conforms to Solana Actions specification."""
    status, blink_resp, headers = handle_action_get_invoice(None, {"invoice_id": ["INV-101"]})
    assert status == 200
    assert "icon" in blink_resp and "label" in blink_resp and "title" in blink_resp
    assert headers.get("X-Action-Version") == "2.1.3"

def test_254_actions_json_discovery_mapping():
    """Verifies /actions.json response maps pathPattern for Blinks discovery."""
    status, actions_json = handle_actions_spec_json(None, {})
    assert status == 200
    assert "rules" in actions_json
    assert actions_json["rules"][0]["pathPattern"] == "/api/v1/actions/**"

def test_255_telegram_markdown_v2_receipt_formatting():
    """Verifies Telegram MarkdownV2 receipt formatting preserves bold wrappers while escaping dynamic content."""
    receipt = format_itemized_receipt("INV-102", "Coffee; Donut", 0.0, 5.0, lang="en")
    assert r"*☕ ZeroClaw POS Receipt \#INV\-102*" in receipt
    assert "• Coffee\n• Donut" in receipt
    assert r"*TOTAL: $5\.00 USDC*" in receipt



def run_suite():
    tests = [
        ("Decimal Exact Micro-Lamport Math (No Float Drift)", test_251_decimal_exact_micro_lamport_math),
        ("PIX Tag 59 Multi-byte Emoji Overflow Truncation", test_252_pix_tag59_multi_byte_emoji_overflow_truncation),
        ("Solana Actions / Blinks Spec Schema Compliance", test_253_solana_actions_blinks_json_schema),
        ("Blinks Actions.json Discovery Mapping", test_254_actions_json_discovery_mapping),
        ("Telegram MarkdownV2 Receipt Structural Formatting", test_255_telegram_markdown_v2_receipt_formatting)
    ]
    passed = 0
    GREEN = "\033[92m"
    RESET = "\033[0m"
    for name, fn in tests:
        fn()
        idx = int(fn.__name__.split("_")[1])
        print(f"  ✅ [TEST {idx:03d}] {name} ... {GREEN}PASSED{RESET}")
        passed += 1
    return passed

if __name__ == "__main__":
    run_suite()
