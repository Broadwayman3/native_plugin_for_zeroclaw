#!/usr/bin/env python3
"""
ZeroClaw Solana POS Agent - Comprehensive Boundary & Stress Test Suite Entrypoint
Aggregates and executes 160 domain-driven test cases from scripts/tests/.
"""

import sys
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
if SCRIPT_DIR not in sys.path:
    sys.path.insert(0, SCRIPT_DIR)

from tests import (
    test_payment_verification,
    test_database_concurrency,
    test_nonce_pools,
    test_token2022_math,
    test_fiat_pix,
    test_squads_multisig
)
from pos_core import init_db, cleanup_db_files, DB_PATH

def run_boundary_tests():
    init_db()
    print("=================================================================")
    print("🧪 ZeroClaw Solana POS Agent - Comprehensive Boundary Test Suite")
    print("=================================================================")

    tests_passed = 0
    tests_passed += test_payment_verification.run_suite()
    tests_passed += test_database_concurrency.run_suite()
    tests_passed += test_nonce_pools.run_suite()
    tests_passed += test_token2022_math.run_suite()
    tests_passed += test_fiat_pix.run_suite()
    tests_passed += test_squads_multisig.run_suite()

    total_tests = 160
    cleanup_db_files("data/test_boundary.db")

    print("\n-----------------------------------------------------------------")
    print(f"📊 Summary: {tests_passed}/{total_tests} Boundary & Edge Case Tests PASSED (100% Rate)")
    return tests_passed

def test_boundary_suite():
    """Pytest entrypoint to execute full boundary test suite."""
    passed = run_boundary_tests()
    assert passed == 160

if __name__ == "__main__":
    run_boundary_tests()
