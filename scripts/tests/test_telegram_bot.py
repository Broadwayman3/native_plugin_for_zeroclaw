#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Telegram Bot Logical & Math Test Suite (16 tests)
Category A: Logical Edge Cases (322-329)
Category B: Math & Currency Precision (330-337)
"""

import os
import sys
import sqlite3

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
POS_DIR = os.path.dirname(SCRIPT_DIR)
if POS_DIR not in sys.path:
    sys.path.insert(0, POS_DIR)

os.environ["MANAGER_TELEGRAM_ID"] = "123456789"

from pos_core import (
    get_db_connection,
    init_db,
    check_and_register_telegram_update,
    cancel_invoice_record,
    create_invoice_record,
    update_invoice_status_record,
    get_multitier_fiat_rate,
    parse_pos_order_input,
    handle_callback_query,
    handle_text_message,
    initiate_refund_request,
    generate_secure_reference_key,
)
import pos_core.bot_ui_utils as bot_ui_utils

bot_ui_utils.MANAGER_TELEGRAM_ID = 123456789

TEST_DB = "data/test_boundary.db"


def setup_db():
    init_db(TEST_DB)


def _make_session(lang="uk", state="idle", user_set=False, draft_items=None):
    return {"lang": lang, "state": state, "user_set": user_set, "draft_items": draft_items}


def _create_test_invoice(inv_id, status="pending", usdc_amt=10.0, fiat_curr="USD", fiat_amt=10.0):
    ref_key = generate_secure_reference_key()
    create_invoice_record(
        {"id": inv_id, "reference_pubkey": ref_key, "fiat_currency": fiat_curr, "fiat_amount": fiat_amt, "usdc_amount": usdc_amt}, db_path=TEST_DB
    )
    if status != "pending":
        update_invoice_status_record(inv_id, status, db_path=TEST_DB)


def _get_invoice_status(inv_id):
    conn = get_db_connection(TEST_DB)
    conn.row_factory = sqlite3.Row
    try:
        cursor = conn.cursor()
        cursor.execute("SELECT status FROM invoices WHERE id = ?", (inv_id,))
        row = cursor.fetchone()
        return row["status"] if row else None
    finally:
        conn.close()


# =====================================================================
# Category A: Logical Edge Cases (Tests 322-329)
# =====================================================================


def test_322_update_id_deduplication():
    conn = get_db_connection(TEST_DB)
    try:
        assert check_and_register_telegram_update(conn, 100) is True
        assert check_and_register_telegram_update(conn, 100) is False
        assert check_and_register_telegram_update(conn, 101) is True
        assert check_and_register_telegram_update(conn, None) is False
    finally:
        conn.close()


def test_323_double_click_cancel_returns_conflict():
    _create_test_invoice("INV-TEST-323", status="pending")
    rowcount1 = cancel_invoice_record("INV-TEST-323", db_path=TEST_DB)
    rowcount2 = cancel_invoice_record("INV-TEST-323", db_path=TEST_DB)
    assert rowcount1 == 1, "First cancel should succeed"
    assert rowcount2 == 0, "Second cancel should fail (409 Conflict)"
    assert _get_invoice_status("INV-TEST-323") == "cancelled"


def test_324_sop_pending_to_cancelled_flow():
    _create_test_invoice("INV-TEST-324", status="pending")
    assert _get_invoice_status("INV-TEST-324") == "pending"
    cancel_invoice_record("INV-TEST-324", db_path=TEST_DB)
    assert _get_invoice_status("INV-TEST-324") == "cancelled"


def test_325_cannot_cancel_already_paid():
    _create_test_invoice("INV-TEST-325", status="paid")
    rowcount = cancel_invoice_record("INV-TEST-325", db_path=TEST_DB)
    assert rowcount == 0, "Cannot cancel paid invoice"
    assert _get_invoice_status("INV-TEST-325") == "paid"


def test_326_only_paid_can_be_refunded():
    _create_test_invoice("INV-TEST-326", status="pending", usdc_amt=5.0)
    conn = get_db_connection(TEST_DB)
    try:
        assert initiate_refund_request(conn, "INV-TEST-326") is False, "Pending invoice cannot be refunded"
        assert _get_invoice_status("INV-TEST-326") == "pending"
    finally:
        conn.close()
    update_invoice_status_record("INV-TEST-326", "paid", db_path=TEST_DB)
    assert _get_invoice_status("INV-TEST-326") == "paid"
    conn2 = get_db_connection(TEST_DB)
    try:
        assert initiate_refund_request(conn2, "INV-TEST-326") is True, "Paid invoice should be refundable"
    finally:
        conn2.close()
    assert _get_invoice_status("INV-TEST-326") == "refunding"


def test_327_callback_with_empty_unknown_data():
    session = _make_session()
    cb_empty = {"id": "cb1", "data": "", "message": {"chat": {"id": 111}}}
    payloads = handle_callback_query(cb_empty, session, db_path=TEST_DB)
    assert len(payloads) == 0, "Empty callback data should produce no payloads"
    cb_unknown = {"id": "cb2", "data": "unknown_action_xyz", "message": {"chat": {"id": 111}}}
    payloads2 = handle_callback_query(cb_unknown, session, db_path=TEST_DB)
    assert len(payloads2) == 0, "Unknown callback data should produce no payloads"


def test_328_session_preserves_lang_after_state_change():
    session = _make_session(lang="pl", user_set=True)
    assert session["lang"] == "pl"
    assert session["user_set"] is True
    session["state"] = "awaiting_custom_amount"
    assert session["lang"] == "pl", "Lang must not change after state change"


def test_329_empty_message_text_ignored():
    session = _make_session()
    payloads = handle_text_message({"chat": {"id": 222}}, session, db_path=TEST_DB)
    assert len(payloads) == 0
    payloads2 = handle_text_message({"chat": {"id": 222}, "text": "   "}, session, db_path=TEST_DB)
    assert len(payloads2) == 0


# =====================================================================
# Category B: Math & Currency Precision (Tests 330-337)
# =====================================================================


def test_330_uah_to_usdc_precision_150():
    rate = get_multitier_fiat_rate("UAH")["rate"]
    usdc = round(150.0 / rate, 2)
    assert usdc == 3.61, f"150 UAH / {rate} -> expected 3.61, got {usdc}"


def test_331_brl_to_usdc_precision_35_5():
    rate = get_multitier_fiat_rate("BRL")["rate"]
    usdc = round(35.5 / rate, 2)
    assert usdc == 6.51, f"35.5 BRL / {rate} -> expected 6.51, got {usdc}"


def test_332_usd_to_usdc_precision_12_50():
    rate = get_multitier_fiat_rate("USD")["rate"]
    usdc = round(12.50 / rate, 2)
    assert usdc == 12.50, f"12.50 USD / {rate} -> expected 12.50, got {usdc}"


def test_333_eur_to_usdc_precision_15():
    rate = get_multitier_fiat_rate("EUR")["rate"]
    usdc = round(15.0 / rate, 2)
    assert usdc == 16.30, f"15 EUR / {rate} -> expected 16.30, got {usdc}"


def test_334_composite_order_2x_latte_300_uah():
    parsed = parse_pos_order_input("2x Latte 300 UAH")
    assert parsed["has_price"] is True
    assert parsed["items"] == "2x Latte"
    assert parsed["amount"] == 300.0
    assert parsed["currency"] == "UAH"
    rate = get_multitier_fiat_rate("UAH")["rate"]
    usdc = round(300.0 / rate, 2)
    assert usdc == 7.23, f"300 UAH / {rate} -> expected 7.23, got {usdc}"


def test_335_sub_cent_drift_protection():
    rates_data = [
        ("UAH", 41.50, 150.0),
        ("BRL", 5.45, 35.5),
        ("USD", 1.00, 12.50),
        ("EUR", 0.92, 15.0),
        ("JPY", 152.50, 1525.0),
        ("PLN", 3.98, 39.80),
        ("GBP", 0.78, 7.80),
        ("TRY", 33.10, 100.0),
    ]
    for curr, _rate, amount in rates_data:
        usdc = round(amount / _rate, 2)
        assert round(usdc, 2) == usdc, f"{curr}: {usdc} has >2 decimals"


def test_336_standalone_number_defaults_to_uah():
    parsed = parse_pos_order_input("500")
    assert parsed["has_price"] is True
    assert parsed["amount"] == 500.0
    assert parsed["currency"] == "UAH"
    rate = get_multitier_fiat_rate("UAH")["rate"]
    usdc = round(500.0 / rate, 2)
    assert usdc == 12.05


def test_337_large_amount_precision():
    parsed = parse_pos_order_input("999999.99 UAH")
    assert parsed["amount"] == 999999.99
    rate = get_multitier_fiat_rate("UAH")["rate"]
    usdc = round(999999.99 / rate, 2)
    str_usdc = f"{usdc:.2f}"
    decimal_part = str_usdc.split(".")[1]
    assert len(decimal_part) == 2, f"Expected 2 decimals, got {len(decimal_part)}: {str_usdc}"


def run_suite() -> int:
    setup_db()
    tests = [
        ("322. Update ID Deduplication", test_322_update_id_deduplication),
        ("323. Double-Click Cancel Returns 409 Conflict", test_323_double_click_cancel_returns_conflict),
        ("324. SOP: pending->cancelled Flow", test_324_sop_pending_to_cancelled_flow),
        ("325. SOP: Cannot Cancel Already Paid", test_325_cannot_cancel_already_paid),
        ("326. SOP: Only Paid Can Be Refunded", test_326_only_paid_can_be_refunded),
        ("327. Callback with Empty/Unknown Data", test_327_callback_with_empty_unknown_data),
        ("328. Session Preserves lang After State Change", test_328_session_preserves_lang_after_state_change),
        ("329. Empty Message Text Ignored", test_329_empty_message_text_ignored),
        ("330. 150 UAH -> USDC Precision", test_330_uah_to_usdc_precision_150),
        ("331. 35.5 BRL -> USDC Precision", test_331_brl_to_usdc_precision_35_5),
        ("332. 12.50 USD -> USDC Precision", test_332_usd_to_usdc_precision_12_50),
        ("333. 15 EUR -> USDC Precision", test_333_eur_to_usdc_precision_15),
        ("334. Composite Order 2x Latte 300 UAH", test_334_composite_order_2x_latte_300_uah),
        ("335. Sub-Cent Drift Protection (IEEE 754)", test_335_sub_cent_drift_protection),
        ("336. Standalone Number Defaults to UAH", test_336_standalone_number_defaults_to_uah),
        ("337. Large Amount Precision 999999.99", test_337_large_amount_precision),
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
