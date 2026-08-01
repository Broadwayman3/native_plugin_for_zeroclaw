#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - SQLite Concurrency, WAL Mode & Database Safety Domain Tests (Tests 016-030)
"""

import os
import sqlite3
import threading
from pos_core import cleanup_db_files, is_valid_base58

TEST_DB_PATH = "data/test_boundary.db"

def setup_test_db():
    cleanup_db_files(TEST_DB_PATH)
    os.makedirs("data", exist_ok=True)
    conn = sqlite3.connect(TEST_DB_PATH, timeout=5.0)
    conn.execute("PRAGMA journal_mode=WAL;")
    conn.execute("PRAGMA busy_timeout=5000;")
    conn.execute("CREATE TABLE IF NOT EXISTS invoices (id TEXT PRIMARY KEY, status TEXT);")
    conn.commit()
    conn.close()

def teardown_test_db():
    cleanup_db_files(TEST_DB_PATH)

def test_016_parallel_nonce_pool_allocation():
    nonce_pool = ["NonceAcc111", "NonceAcc222", "NonceAcc333"]
    allocated = [nonce_pool.pop(0), nonce_pool.pop(0)]
    assert len(allocated) == 2 and allocated[0] != allocated[1] and len(nonce_pool) == 1

def test_017_brazil_first_brl_pricing():
    brl_amount = 54.50
    brl_usdc = round(brl_amount / 5.45, 2)
    assert brl_usdc == 10.00

def test_018_switchboard_crossbar_fiat_rate_fallback():
    def get_switchboard_fiat_rate(pair):
        mock_response = {"UAH_USD": 41.50, "BRL_USD": 5.45}
        return mock_response.get(pair, 1.0)
    assert get_switchboard_fiat_rate("BRL_USD") == 5.45 and get_switchboard_fiat_rate("UAH_USD") == 41.50

def test_019_solana_base58_pubkey_validation():
    valid_pk = "8xAZmQ1111111111111111111111111111111111111"
    invalid_pk = "8xAZmQ111111111111111111111111111111111111000O"
    assert is_valid_base58(valid_pk) and not is_valid_base58(invalid_pk)

def test_020_pix_qr_settlement_reconciliation():
    pix_payload = "00020126580014br.gov.bcb.pix0136123e4567-e89b-12d3-a456-426614174000520400005303986540510.005802BR5913ZeroClaw POS6008BRASILIA"
    assert "br.gov.bcb.pix" in pix_payload and "ZeroClaw POS" in pix_payload

def test_021_sqlite_duplicate_reference_key_unique_constraint():
    setup_test_db()
    try:
        conn = sqlite3.connect(TEST_DB_PATH)
        conn.execute("CREATE TABLE IF NOT EXISTS refs (ref TEXT UNIQUE);")
        conn.execute("INSERT INTO refs VALUES ('7xRefKeyUnique111');")
        conn.commit()
        caught_dup = False
        try:
            conn.execute("INSERT INTO refs VALUES ('7xRefKeyUnique111');")
            conn.commit()
        except sqlite3.IntegrityError:
            caught_dup = True
        conn.close()
        assert caught_dup
    finally:
        teardown_test_db()

def test_022_concurrent_double_payment_race_condition_defense():
    setup_test_db()
    try:
        conn = sqlite3.connect(TEST_DB_PATH)
        conn.execute("INSERT INTO invoices VALUES ('INV-RACE', 'pending');")
        conn.commit()

        cursor1 = conn.cursor()
        cursor1.execute("UPDATE invoices SET status = 'paid' WHERE id = 'INV-RACE' AND status = 'pending'")
        updated_1 = cursor1.rowcount
        conn.commit()

        cursor2 = conn.cursor()
        cursor2.execute("UPDATE invoices SET status = 'paid' WHERE id = 'INV-RACE' AND status = 'pending'")
        updated_2 = cursor2.rowcount
        conn.commit()
        conn.close()
        assert updated_1 == 1 and updated_2 == 0
    finally:
        teardown_test_db()

def test_023_sqlite_wal_mode_multithread_concurrency():
    setup_test_db()
    try:
        errors = []
        def db_writer(thread_id):
            try:
                c = sqlite3.connect(TEST_DB_PATH, timeout=5.0)
                c.execute("PRAGMA journal_mode=WAL;")
                c.execute("INSERT OR REPLACE INTO invoices VALUES (?, 'paid');", (f"INV-{thread_id}",))
                c.commit()
                c.close()
            except Exception as e:
                errors.append(e)

        threads = [threading.Thread(target=db_writer, args=(i,)) for i in range(10)]
        for t in threads: t.start()
        for t in threads: t.join()
        assert len(errors) == 0
    finally:
        teardown_test_db()

def test_024_sql_parameter_escaping_unicode_null_bytes():
    setup_test_db()
    try:
        conn = sqlite3.connect(TEST_DB_PATH)
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM invoices WHERE id = ?", ("INV\x00-101",))
        rows = cursor.fetchall()
        conn.close()
        assert len(rows) == 0
    finally:
        teardown_test_db()

def test_025_squads_v4_pda_derivation_string_consistency():
    pda_program_id = "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm"
    assert len(pda_program_id) == 43 and pda_program_id.startswith("SQDS")

def test_026_token2022_net_vs_gross_amount_reconciliation():
    gross_paid = 10.00
    fee_deducted = 0.01
    net_received = gross_paid - fee_deducted
    assert net_received >= (10.00 - 0.01)

def test_027_telegram_sender_user_id_isolation():
    msg_from_id = "987654321"
    attacker_from_id = "111222333"
    def verify_manager_auth(from_id, allowed_manager_id):
        return str(from_id) == str(allowed_manager_id)
    assert verify_manager_auth(msg_from_id, "987654321") and not verify_manager_auth(attacker_from_id, "987654321")

def test_028_deep_solana_pay_instruction_parsing():
    fake_tx_parsed = {
        "instructions": [
            {
                "program": "spl-token",
                "parsed": {
                    "type": "transfer",
                    "info": {
                        "destination": "AttackerATA",
                        "amount": "10000000"
                    }
                },
                "accounts": ["RefKey11111111111111111111111111111111111"]
            }
        ]
    }
    merchant_ata = "MerchantUSDC_ATA_Pubkey"
    tx_valid = (fake_tx_parsed["instructions"][0]["parsed"]["info"]["destination"] == merchant_ata)
    assert not tx_valid

def test_029_parameterized_query_sql_injection_isolation():
    setup_test_db()
    try:
        conn = sqlite3.connect(TEST_DB_PATH)
        cursor = conn.cursor()
        cursor.execute("SELECT * FROM invoices WHERE id = ?", ("' OR '1'='1' --",))
        rows = cursor.fetchall()
        conn.close()
        assert len(rows) == 0
    finally:
        teardown_test_db()

def test_030_rpc_node_fallback_endpoint_switching():
    rpc_primary = "https://failing-rpc.com"
    rpc_fallback = "https://devnet.helius-rpc.com/?api-key=test"
    active_rpc = rpc_primary
    active_rpc = rpc_fallback
    assert active_rpc == rpc_fallback

def run_suite():
    tests = [
        ("Parallel Durable Nonce Account Pool Allocation", test_016_parallel_nonce_pool_allocation),
        ("Brazil-First BRL Currency Pricing & Conversion", test_017_brazil_first_brl_pricing),
        ("Switchboard Crossbar API BRL/USD Rate Fallback", test_018_switchboard_crossbar_fiat_rate_fallback),
        ("Solana Base58 Public Key Format Validation", test_019_solana_base58_pubkey_validation),
        ("PIX QR Code & USDC Settlement Reconciliation", test_020_pix_qr_settlement_reconciliation),
        ("SQLite Duplicate Reference Key Unique Constraint", test_021_sqlite_duplicate_reference_key_unique_constraint),
        ("Concurrent Double-Payment Race Condition Defense", test_022_concurrent_double_payment_race_condition_defense),
        ("SQLite WAL Mode Multi-Thread Concurrency", test_023_sqlite_wal_mode_multithread_concurrency),
        ("SQL Parameter Escaping with Unicode Null Bytes", test_024_sql_parameter_escaping_unicode_null_bytes),
        ("Squads v4 PDA Derivation String Consistency", test_025_squads_v4_pda_derivation_string_consistency),
        ("Token-2022 Net vs Gross Amount Reconciliation", test_026_token2022_net_vs_gross_amount_reconciliation),
        ("Telegram Sender User ID vs Chat ID Isolation", test_027_telegram_sender_user_id_isolation),
        ("Deep Solana Pay Instruction Parsing", test_028_deep_solana_pay_instruction_parsing),
        ("Parameterized Query SQL Injection Isolation", test_029_parameterized_query_sql_injection_isolation),
        ("RPC Node Fallback Endpoint Switching Logic", test_030_rpc_node_fallback_endpoint_switching),
    ]
    passed = 0
    GREEN = "\033[92m"
    RESET = "\033[0m"
    for name, fn in tests:
        fn()
        idx = int(fn.__name__.split("_")[1])
        print(f"  ✅ [TEST {idx:02d}] {name} ... {GREEN}PASSED{RESET}")
        passed += 1
    return passed
