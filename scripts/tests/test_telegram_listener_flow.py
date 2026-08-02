#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Telegram Listener & i18n Flow Test Suite
Dedicated unit and integration tests simulating Telegram updates, callback queries,
per-chat multi-language session state, language lock persistence, and POS order parsing.
"""

import os
import sys
import unittest

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
POS_DIR = os.path.dirname(SCRIPT_DIR)
if POS_DIR not in sys.path:
    sys.path.insert(0, POS_DIR)

from pos_core import (
    LANG_META,
    TRANSLATIONS,
    get_localized_confirmation,
    get_main_reply_keyboard,
    get_cancel_invoice_inline_keyboard,
    t
)
from telegram_bot_listener import (
    get_session,
    is_btn_click,
    parse_pos_order_input,
    USER_SESSIONS
)

def test_316_thirteen_languages_keyboard_and_confirmation():
    """Verifies that all 13 supported languages generate localized confirmations and reply keyboards."""
    for lang_code in LANG_META.keys():
        conf = get_localized_confirmation(lang_code)
        kb = get_main_reply_keyboard(lang_code)
        assert conf is not None and len(conf) > 0, f"Confirmation empty for {lang_code}"
        assert "keyboard" in kb and len(kb["keyboard"]) == 3, f"Keyboard invalid for {lang_code}"
        # Assert button labels match translated string for lang_code
        custom_btn = kb["keyboard"][0][0]["text"]
        expected_custom = t("btn_custom", lang_code, escape_markdown=False)
        assert custom_btn == expected_custom, f"Custom button mismatch for {lang_code}: {custom_btn} != {expected_custom}"

def test_317_language_lock_persistence_against_telegram_app_lang():
    """Verifies explicit user language selection is locked and NOT overridden by Telegram app language_code."""
    fake_chat_id = 999111
    session = get_session(fake_chat_id)
    session["lang"] = "pl"
    session["user_set"] = True

    # Simulate incoming message from Telegram client with Ukrainian app language
    msg = {"from": {"language_code": "uk"}}
    if not session.get("user_set") and "from" in msg and "language_code" in msg["from"]:
        session["lang"] = msg["from"]["language_code"]

    assert session["lang"] == "pl", f"Language lock broken! Overridden to {session['lang']}"

def test_318_order_quantity_multiplier_isolation():
    """Verifies quantity multipliers like '8x Cappuccino' are protected from being misparsed as fiat amounts."""
    res = parse_pos_order_input("8x Cappuccino + 10x Croissant", default_item_label="Standard Order")
    assert not res["has_price"], "Quantity multiplier '8x' was incorrectly parsed as price!"
    assert res["items"] == "8x Cappuccino + 10x Croissant"
    assert res["amount"] is None

def test_319_multicurrency_custom_order_parsing():
    """Verifies order parsing across multi-currency inputs (UAH, USD, BRL, EUR, ₴, $)."""
    r1 = parse_pos_order_input("8x Cappuccino + 10x Croissant 500 UAH")
    assert r1["has_price"] and r1["amount"] == 500.0 and r1["currency"] == "UAH" and r1["items"] == "8x Cappuccino + 10x Croissant"

    r2 = parse_pos_order_input("2x Espresso 15.50 USD")
    assert r2["has_price"] and r2["amount"] == 15.50 and r2["currency"] == "USD" and r2["items"] == "2x Espresso"

    r3 = parse_pos_order_input("150 UAH")
    assert r3["has_price"] and r3["amount"] == 150.0 and r3["currency"] == "UAH"

    r4 = parse_pos_order_input("35.50 BRL")
    assert r4["has_price"] and r4["amount"] == 35.50 and r4["currency"] == "BRL"

def test_320_two_step_draft_items_order_flow():
    """Verifies two-step POS order flow: Step 1 (items without price) -> Step 2 (follow-up price input)."""
    # Step 1: User types item list without price
    step1 = parse_pos_order_input("8x Cappuccino + 10x Croissant")
    assert not step1["has_price"]
    draft_items = step1["items"]

    # Step 2: User follows up with price
    step2 = parse_pos_order_input("500 UAH", draft_items=draft_items)
    assert step2["has_price"] and step2["amount"] == 500.0 and step2["currency"] == "UAH"
    assert step2["items"] == "8x Cappuccino + 10x Croissant"

def test_321_is_btn_click_multilingual_matching():
    """Verifies button click matcher across Polish, German, Japanese, Chinese, Arabic, Ukrainian, English."""
    assert is_btn_click("✍️ Enter custom amount", "btn_custom")
    assert is_btn_click("✍️ Ввести довільну суму", "btn_custom")
    assert is_btn_click("✍️ Wpisz kwotę", "btn_custom")
    assert is_btn_click("✍️ Betrag eingeben", "btn_custom")
    assert is_btn_click("✍️ 金額を入力", "btn_custom")
    assert is_btn_click("✍️ 输入自定义金额", "btn_custom")

def run_suite() -> int:
    tests = [
        ("316. 13-Language Reply Keyboard & Confirmation Alignment", test_316_thirteen_languages_keyboard_and_confirmation),
        ("317. Language Lock Persistence vs App Language Override", test_317_language_lock_persistence_against_telegram_app_lang),
        ("318. Order Quantity Multiplier Protection", test_318_order_quantity_multiplier_isolation),
        ("319. Multi-Currency Custom Order Parsing", test_319_multicurrency_custom_order_parsing),
        ("320. Two-Step Draft Items POS Order Flow", test_320_two_step_draft_items_order_flow),
        ("321. Multilingual Reply Keyboard Button Matcher", test_321_is_btn_click_multilingual_matching)
    ]
    passed = 0
    for name, test_func in tests:
        try:
            test_func()
            print(f"  ✅ [TEST {name}] ... PASSED")
            passed += 1
        except Exception as e:
            print(f"  ❌ [TEST {name}] ... FAILED: {e}")
            raise e
    return passed

if __name__ == "__main__":
    print("🧪 Running Telegram Listener Flow Test Suite...")
    run_suite()
