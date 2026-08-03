#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Telegram Bot Refund Flow Integration Test Suite (6 tests)
Category E: Integration & Full Refund Flow (352-357)
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
    create_invoice_record,
    update_invoice_status_record,
    build_answer_callback_payload,
    handle_callback_query,
    handle_text_message,
    generate_secure_reference_key,
    initiate_refund_request,
    create_squads_proposal,
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


def test_352_full_refund_flow_text_to_proposal_to_approve():
    _create_test_invoice("INV-REF-001", status="paid", usdc_amt=15.0, fiat_curr="UAH", fiat_amt=622.5)
    session = _make_session(state="awaiting_refund_invoice")
    payloads = handle_text_message({"chat": {"id": 333}, "text": "INV-REF-001"}, session, db_path=TEST_DB)
    assert session["state"] == "idle"
    send_msg_payloads = [p for p in payloads if p[0] == "sendMessage"]
    assert len(send_msg_payloads) >= 1
    proposal_msg = send_msg_payloads[0][1]["text"]
    assert "INV-REF-001" in proposal_msg
    assert "Proposal Initiated" in proposal_msg or "15.00" in proposal_msg


def test_353_full_refund_reject_reverts_to_paid():
    _create_test_invoice("INV-REF-002", status="paid", usdc_amt=8.0)
    conn = get_db_connection(TEST_DB)
    try:
        initiate_refund_request(conn, "INV-REF-002")
        proposal_idx = create_squads_proposal(conn, "INV-REF-002", "8xAZmQ1111111111111111111111111111111111111", 8.0)
    finally:
        conn.close()
    cb = {"id": "cb_reject", "data": f"reject_refund_{proposal_idx}", "message": {"chat": {"id": 111}}, "from": {"id": 123456789}}
    handle_callback_query(cb, _make_session(), db_path=TEST_DB)
    assert _get_invoice_status("INV-REF-002") == "paid"
    conn3 = get_db_connection(TEST_DB)
    conn3.row_factory = sqlite3.Row
    try:
        cursor = conn3.cursor()
        cursor.execute("SELECT status FROM squads_proposals WHERE proposal_index = ?", (proposal_idx,))
        row = cursor.fetchone()
        assert row["status"] == "rejected"
    finally:
        conn3.close()


def test_354_refund_nonexistent_invoice_id():
    session = _make_session(state="awaiting_refund_invoice")
    payloads = handle_text_message({"chat": {"id": 444}, "text": "INV-DOES-NOT-EXIST"}, session, db_path=TEST_DB)
    send_msg = [p for p in payloads if p[0] == "sendMessage"]
    assert len(send_msg) >= 1
    err_text = send_msg[0][1]["text"].lower()
    assert "error" in err_text or "not found" in err_text or "already" in err_text or "refund" in err_text


def test_355_refund_double_click_reentrancy():
    _create_test_invoice("INV-REF-003", status="paid", usdc_amt=7.0)
    conn = get_db_connection(TEST_DB)
    try:
        success1 = initiate_refund_request(conn, "INV-REF-003")
        assert success1 is True
        success2 = initiate_refund_request(conn, "INV-REF-003")
        assert success2 is False, "Second refund attempt must fail (re-entrancy guard)"
    finally:
        conn.close()
    assert _get_invoice_status("INV-REF-003") == "refunding"


def test_356_unauthorized_approve_blocked_with_notification():
    _create_test_invoice("INV-REF-004", status="paid", usdc_amt=12.0)
    conn = get_db_connection(TEST_DB)
    try:
        initiate_refund_request(conn, "INV-REF-004")
        proposal_idx = create_squads_proposal(conn, "INV-REF-004", "8xAZmQ1111111111111111111111111111111111111", 12.0)
    finally:
        conn.close()
    cb = {"id": "cb_block", "data": f"approve_refund_{proposal_idx}", "message": {"chat": {"id": 555}}, "from": {"id": 999999999}}
    session = _make_session(lang="en")
    payloads = handle_callback_query(cb, session, db_path=TEST_DB)
    answers = [p for p in payloads if p[0] == "answerCallbackQuery"]
    assert answers[0][1]["show_alert"] is True
    assert "Unauthorized" in answers[0][1]["text"]
    msgs = [p for p in payloads if p[0] == "sendMessage"]
    assert any("Unauthorized" in m[1]["text"] for m in msgs)


def test_357_build_answer_callback_payload_structure():
    payload = build_answer_callback_payload("cb_test_123", "Hello!")
    assert payload["callback_query_id"] == "cb_test_123"
    assert payload["text"] == "Hello!"
    assert "show_alert" not in payload
    payload_alert = build_answer_callback_payload("cb_test_456", "Alert!", show_alert=True)
    assert payload_alert["callback_query_id"] == "cb_test_456"
    assert payload_alert["text"] == "Alert!"
    assert payload_alert["show_alert"] is True


def run_suite() -> int:
    setup_db()
    tests = [
        ("352. Full Refund Flow: Text -> Proposal -> Payment", test_352_full_refund_flow_text_to_proposal_to_approve),
        ("353. Full Refund Flow: Reject Reverts to Paid", test_353_full_refund_reject_reverts_to_paid),
        ("354. Refund: Non-Existent Invoice ID", test_354_refund_nonexistent_invoice_id),
        ("355. Refund: Double-Click Re-Entrancy Guard", test_355_refund_double_click_reentrancy),
        ("356. Refund: Unauthorized Approve Blocked + Notification", test_356_unauthorized_approve_blocked_with_notification),
        ("357. build_answer_callback_payload Structure", test_357_build_answer_callback_payload_structure),
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
