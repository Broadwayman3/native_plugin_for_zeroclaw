mod suites;

use std::sync::atomic::{AtomicUsize, Ordering};

static TEST_COUNT: AtomicUsize = AtomicUsize::new(0);
static PASS_COUNT: AtomicUsize = AtomicUsize::new(0);

pub fn run_suite() -> usize {
    println!("═══════════════════════════════════════════════════════════");
    println!("🚀 ZeroClaw POS Backend Test Suite (Rust)");
    println!("═══════════════════════════════════════════════════════════");

    TEST_COUNT.store(0, Ordering::SeqCst);
    PASS_COUNT.store(0, Ordering::SeqCst);

    suites::test_token2022::run_suite();
    suites::test_solana_pay::run_suite();
    suites::test_pix_brl::run_suite();
    suites::test_price_feed::run_suite();
    suites::test_sanitizer::run_suite();
    suites::test_verification::run_suite();
    suites::test_i18n::run_suite();
    suites::test_keyboards::run_suite();
    suites::test_database::run_suite();
    suites::test_api::run_suite();
    suites::test_security::run_suite();
    suites::test_nonce_pools::run_suite();
    suites::test_squads_multisig::run_suite();
    suites::test_edge_storage::run_suite();
    suites::test_telegram_handlers::run_suite();
    suites::test_prompt_injection::run_suite();
    suites::test_qa_red_team::run_suite();
    suites::test_validators::run_suite();
    suites::test_zeroclaw_integration::run_suite();
    suites::test_error::run_suite();
    suites::test_config::run_suite();

    let total = TEST_COUNT.load(Ordering::SeqCst);
    let passed = PASS_COUNT.load(Ordering::SeqCst);
    let failed = total - passed;

    println!("═══════════════════════════════════════════════════════════");
    if failed == 0 {
        println!("✅ ALL {} TESTS PASSED", total);
    } else {
        println!("❌ {}/{} TESTS PASSED, {} FAILED", passed, total, failed);
    }
    println!("═══════════════════════════════════════════════════════════");

    failed
}

pub fn test_pass(name: &str) {
    TEST_COUNT.fetch_add(1, Ordering::SeqCst);
    PASS_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("  ✅ {}", name);
}

pub fn test_fail(name: &str, msg: &str) {
    TEST_COUNT.fetch_add(1, Ordering::SeqCst);
    println!("  ❌ {}: {}", name, msg);
}

#[test]
fn run_all_tests() {
    let failed = run_suite();
    assert_eq!(failed, 0, "{} tests failed", failed);
}
