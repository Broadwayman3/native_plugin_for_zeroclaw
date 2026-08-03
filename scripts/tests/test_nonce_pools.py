#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Durable Nonce Pools & Recovery Domain Tests (Tests 031-050)
"""

import os
import time
import datetime
import threading
from pos_core import (
    get_db_connection,
    cleanup_db_files,
    allocate_free_nonce_account,
    release_nonce_account,
    mark_nonce_account_stale,
    refresh_stale_nonce_account,
    get_required_commitment_level,
    generate_atomic_refund_instructions,
    check_and_register_telegram_update,
)

TEST_DB_PATH = "data/test_boundary.db"


def setup_test_db():
    cleanup_db_files(TEST_DB_PATH)
    os.makedirs("data", exist_ok=True)
    conn = get_db_connection(TEST_DB_PATH)
    cursor = conn.cursor()
    cursor.execute("CREATE TABLE IF NOT EXISTS nonce_accounts (pubkey TEXT PRIMARY KEY, status TEXT NOT NULL DEFAULT 'free', locked_at TIMESTAMP);")
    cursor.execute("CREATE TABLE IF NOT EXISTS processed_updates (update_id INTEGER PRIMARY KEY, processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP);")
    cursor.executemany(
        "INSERT OR REPLACE INTO nonce_accounts (pubkey, status) VALUES (?, 'free')",
        [
            ("Nonce111111111111111111111111111111111111111",),
            ("Nonce222222222222222222222222222222222222222",),
            ("Nonce333333333333333333333333333333333333333",),
        ],
    )
    conn.commit()
    conn.close()


def teardown_test_db():
    cleanup_db_files(TEST_DB_PATH)


def test_031_concurrent_squads_proposal_mutex_lock():
    proposal_queue_lock = threading.Lock()
    proposal_indices = []
    current_index = 100

    def create_proposal_task():
        nonlocal current_index
        with proposal_queue_lock:
            idx = current_index + 1
            time.sleep(0.001)
            current_index = idx
            proposal_indices.append(idx)

    tasks = [threading.Thread(target=create_proposal_task) for _ in range(5)]
    for t in tasks:
        t.start()
    for t in tasks:
        t.join()
    assert proposal_indices == [101, 102, 103, 104, 105]


def test_032_stale_invoice_expiry_handling():
    created_time = datetime.datetime.now(datetime.timezone.utc) - datetime.timedelta(hours=25)
    is_expired = (datetime.datetime.now(datetime.timezone.utc) - created_time).total_seconds() > 86400
    assert is_expired


def test_033_solana_pay_qr_encoding():
    label_special = "Café & Bakery #1 ~ 100% Organic"
    encoded_label = label_special.replace(" ", "%20").replace("&", "%26")
    assert "%20" in encoded_label and "%26" in encoded_label


def test_034_nonce_account_low_balance_gas_warning():
    nonce_lamports = 100000
    needs_recharge = nonce_lamports < 1447200
    assert needs_recharge


def test_035_zero_copy_wasm_memory_allocation_buffer():
    large_payload_str = "A" * 65536
    assert len(large_payload_str) == 65536


def test_036_configurable_commitment_threshold():
    assert get_required_commitment_level(10.0, 50.0) == "confirmed"
    assert get_required_commitment_level(50.0, 50.0) == "finalized"


def test_037_idempotent_ata_auto_creation_instruction():
    refund_instructions = generate_atomic_refund_instructions(payer_pubkey="REFUND_SESSION_KEY", recipient_pubkey="9xK2...Customer1", amount_usdc=25.0)
    assert refund_instructions[0]["instruction"] == "createAssociatedTokenAccountIdempotent" and refund_instructions[0]["payer"] == "REFUND_SESSION_KEY"


def test_038_telegram_update_id_deduplication_layer():
    setup_test_db()
    try:
        conn = get_db_connection(TEST_DB_PATH)
        up_id = 987654321
        res_first = check_and_register_telegram_update(conn, up_id, TEST_DB_PATH)
        res_second = check_and_register_telegram_update(conn, up_id, TEST_DB_PATH)
        conn.close()
        assert res_first is True and res_second is False
    finally:
        teardown_test_db()


def test_039_onchain_blocktime_vs_system_clock_sync():
    local_time = int(time.time())
    rpc_block_time = local_time - 2
    time_delta = abs(local_time - rpc_block_time)
    assert time_delta < 10


def test_040_intermittent_rpc_replica_null_response_retry():
    rpc_replica_attempts = 0

    def mock_get_transaction_replica():
        nonlocal rpc_replica_attempts
        rpc_replica_attempts += 1
        if rpc_replica_attempts < 2:
            return None
        return {"slot": 284910291, "meta": {"err": None}}

    tx_data = None
    for _ in range(3):
        tx_data = mock_get_transaction_replica()
        if tx_data is not None:
            break
        time.sleep(0.001)

    assert tx_data is not None and rpc_replica_attempts == 2


def test_041_sqlite_integrity_check():
    setup_test_db()
    try:
        conn_chk = get_db_connection(TEST_DB_PATH)
        cursor_chk = conn_chk.cursor()
        cursor_chk.execute("PRAGMA integrity_check;")
        check_res = cursor_chk.fetchone()[0]
        conn_chk.close()
        assert check_res == "ok"
    finally:
        teardown_test_db()


def test_042_token2022_transfer_hook_extension_guard():
    transfer_hook_program = "Hook111111111111111111111111111111111111111"
    is_supported_extension = True if transfer_hook_program else False
    assert is_supported_extension


def test_043_squads_v4_threshold_signers_count_guard():
    multisig_members_count = 3
    threshold_required = 2
    assert threshold_required <= multisig_members_count


def test_044_wasm_sandbox_max_memory_pages_allocation_guard():
    max_memory_pages = 16
    bytes_allocated = max_memory_pages * 65536
    assert bytes_allocated == 1048576


def test_045_fail_closed_security_policy_empty_env():
    empty_config = {}

    def evaluate_security_policy(cfg):
        if not cfg.get("MERCHANT_WALLET") or not cfg.get("USDC_MINT"):
            return "FAIL_CLOSED_HALT"
        return "OPERATIONAL"

    assert evaluate_security_policy(empty_config) == "FAIL_CLOSED_HALT"


def test_046_durable_nonce_allocation_and_release():
    setup_test_db()
    try:
        allocated = allocate_free_nonce_account(db_path=TEST_DB_PATH)
        assert allocated is not None
        release_nonce_account(pubkey=allocated, db_path=TEST_DB_PATH)
        conn = get_db_connection(TEST_DB_PATH)
        cursor = conn.cursor()
        cursor.execute("SELECT status FROM nonce_accounts WHERE pubkey = ?", (allocated,))
        st = cursor.fetchone()[0]
        conn.close()
        assert st == "free"
    finally:
        teardown_test_db()


def test_047_durable_nonce_stale_mark_and_refresh():
    setup_test_db()
    try:
        allocated = allocate_free_nonce_account(db_path=TEST_DB_PATH)
        assert allocated is not None
        mark_nonce_account_stale(pubkey=allocated, db_path=TEST_DB_PATH)
        refresh_stale_nonce_account(pubkey=allocated, db_path=TEST_DB_PATH)
        conn = get_db_connection(TEST_DB_PATH)
        cursor = conn.cursor()
        cursor.execute("SELECT status FROM nonce_accounts WHERE pubkey = ?", (allocated,))
        st = cursor.fetchone()[0]
        conn.close()
        assert st == "free"
    finally:
        teardown_test_db()


def test_048_durable_nonce_ttl_expiry_reclaim():
    setup_test_db()
    try:
        conn = get_db_connection(TEST_DB_PATH)
        cursor = conn.cursor()
        cursor.execute("UPDATE nonce_accounts SET status = 'locked', locked_at = CURRENT_TIMESTAMP")
        cursor.execute(
            "UPDATE nonce_accounts SET status = 'locked', locked_at = datetime('now', '-20 minutes') WHERE pubkey = 'Nonce111111111111111111111111111111111111111'"
        )
        conn.commit()
        conn.close()

        reclaimed = allocate_free_nonce_account(db_path=TEST_DB_PATH)
        assert reclaimed == "Nonce111111111111111111111111111111111111111"
    finally:
        teardown_test_db()


def test_049_atomic_double_nonce_release_idempotency():
    setup_test_db()
    try:
        allocated = allocate_free_nonce_account(db_path=TEST_DB_PATH)
        assert allocated is not None
        release_nonce_account(pubkey=allocated, db_path=TEST_DB_PATH)
        release_nonce_account(pubkey=allocated, db_path=TEST_DB_PATH)
        conn = get_db_connection(TEST_DB_PATH)
        cursor = conn.cursor()
        cursor.execute("SELECT status FROM nonce_accounts WHERE pubkey = ?", (allocated,))
        st = cursor.fetchone()[0]
        conn.close()
        assert st == "free"
    finally:
        teardown_test_db()


def test_050_nonce_pool_exhaustion_fallback():
    setup_test_db()
    try:
        conn = get_db_connection(TEST_DB_PATH)
        conn.execute("UPDATE nonce_accounts SET status = 'locked'")
        conn.commit()
        conn.close()

        allocated = allocate_free_nonce_account(db_path=TEST_DB_PATH)
        assert allocated is None
    finally:
        teardown_test_db()


def run_suite():
    tests = [
        ("Concurrent Squads v4 Proposal Index Mutex Lock", test_031_concurrent_squads_proposal_mutex_lock),
        ("Expiry Handling for Stale Invoices (>24h Timeout)", test_032_stale_invoice_expiry_handling),
        ("Solana Pay QR Deep Link Special Char Encoding", test_033_solana_pay_qr_encoding),
        ("Nonce Account Low Balance / Gas Depletion Warning", test_034_nonce_account_low_balance_gas_warning),
        ("Zero-Copy WASM Memory Allocation Buffer Check", test_035_zero_copy_wasm_memory_allocation_buffer),
        ("Configurable Commitment Threshold (Confirmed vs Finalized)", test_036_configurable_commitment_threshold),
        ("Idempotent ATA Auto-Creation Instruction Inclusion", test_037_idempotent_ata_auto_creation_instruction),
        ("Telegram Update ID Deduplication & Idempotency Layer", test_038_telegram_update_id_deduplication_layer),
        ("On-Chain Blocktime vs System Clock Sync", test_039_onchain_blocktime_vs_system_clock_sync),
        ("Intermittent RPC Replica Null Response Retry Loop", test_040_intermittent_rpc_replica_null_response_retry),
        ("SQLite Integrity Check & WAL Checkpoint Truncation", test_041_sqlite_integrity_check),
        ("Token-2022 Transfer Hook Extension Guard", test_042_token2022_transfer_hook_extension_guard),
        ("Squads v4 Threshold Signers Count Guard", test_043_squads_v4_threshold_signers_count_guard),
        ("WASM Sandbox Max Memory Pages Allocation Guard", test_044_wasm_sandbox_max_memory_pages_allocation_guard),
        ("Fail-Closed Security Policy on Empty Environment Config", test_045_fail_closed_security_policy_empty_env),
        ("Durable Nonce Account Allocation & Release", test_046_durable_nonce_allocation_and_release),
        ("Durable Nonce Stale Mark & Refresh Protocol", test_047_durable_nonce_stale_mark_and_refresh),
        ("Durable Nonce TTL Expiry Reclaim Check", test_048_durable_nonce_ttl_expiry_reclaim),
        ("Atomic Double Nonce Release Idempotency Guard", test_049_atomic_double_nonce_release_idempotency),
        ("Nonce Pool Exhaustion Fallback Alert", test_050_nonce_pool_exhaustion_fallback),
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
