use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Token-2022 Math Tests (001-030)");
    test_001_usdc_conversion_normal();
    test_002_usdc_conversion_zero();
    test_003_usdc_conversion_negative();
    test_004_usdc_conversion_nan();
    test_005_usdc_conversion_infinity();
    test_006_usdc_conversion_overflow();
    test_007_fee_calculation_normal();
    test_008_fee_calculation_ceiling();
    test_009_fee_calculation_max_cap();
    test_010_fee_exceeding_max_bp();
    test_011_fee_zero_amount();
    test_012_fee_zero_bp();
    test_013_atomic_to_f64();
    test_014_slippage_tolerance_valid();
    test_015_slippage_tolerance_invalid();
    test_016_token2022_fee_large_amount();
    test_017_token2022_fee_small_amount();
    test_018_token2022_fee_custom_decimals();
    test_019_atomic_conversion_max_u64();
    test_020_atomic_conversion_fractional();
    test_021_fee_calculation_100_percent();
    test_022_fee_calculation_zero_max_fee();
    test_023_atomic_conversion_6_decimals();
    test_024_atomic_conversion_9_decimals();
    test_025_fee_calculation_symmetry();
    test_026_atomic_conversion_precision();
    test_027_fee_calculation_stress();
    test_028_atomic_conversion_large_values();
    test_029_fee_calculation_boundary();
    test_030_atomic_conversion_consistency();
}

fn test_001_usdc_conversion_normal() {
    let r = pos_core_logic::usdc_to_atomic_units(1.0);
    if r == 1_000_000 { test_pass("001: 1.0 USDC = 1000000 atomic"); } else { test_fail("001", &format!("expected 1000000, got {}", r)); }
}

fn test_002_usdc_conversion_zero() {
    let r = pos_core_logic::usdc_to_atomic_units(0.0);
    if r == 0 { test_pass("002: 0.0 USDC = 0 atomic"); } else { test_fail("002", &format!("expected 0, got {}", r)); }
}

fn test_003_usdc_conversion_negative() {
    let r = pos_core_logic::usdc_to_atomic_units(-5.0);
    if r == 0 { test_pass("003: negative USDC = 0 atomic"); } else { test_fail("003", &format!("expected 0, got {}", r)); }
}

fn test_004_usdc_conversion_nan() {
    let r = pos_core_logic::usdc_to_atomic_units(f64::NAN);
    if r == 0 { test_pass("004: NaN USDC = 0 atomic"); } else { test_fail("004", &format!("expected 0, got {}", r)); }
}

fn test_005_usdc_conversion_infinity() {
    let r = pos_core_logic::usdc_to_atomic_units(f64::INFINITY);
    if r == 0 { test_pass("005: infinity USDC = 0 atomic"); } else { test_fail("005", &format!("expected 0, got {}", r)); }
}

fn test_006_usdc_conversion_overflow() {
    let r = pos_core_logic::usdc_to_atomic_units(1e25);
    if r == u64::MAX { test_pass("006: overflow USDC = MAX_U64"); } else { test_fail("006", &format!("expected MAX_U64, got {}", r)); }
}

fn test_007_fee_calculation_normal() {
    let fee = pos_core_logic::calculate_token2022_fee(1_000_000, 10, 1_000_000);
    if fee == 1000 { test_pass("007: 1 USDC, 10bp = 1000 atomic"); } else { test_fail("007", &format!("expected 1000, got {}", fee)); }
}

fn test_008_fee_calculation_ceiling() {
    let fee = pos_core_logic::calculate_token2022_fee(3, 10, 1_000_000);
    if fee == 1 { test_pass("008: ceiling rounding works"); } else { test_fail("008", &format!("expected 1, got {}", fee)); }
}

fn test_009_fee_calculation_max_cap() {
    let fee = pos_core_logic::calculate_token2022_fee(1_000_000, 10000, 500);
    if fee == 500 { test_pass("009: max fee cap enforced"); } else { test_fail("009", &format!("expected 500, got {}", fee)); }
}

fn test_010_fee_exceeding_max_bp() {
    let fee = pos_core_logic::calculate_token2022_fee(1_000_000, 10001, 500);
    if fee == 500 { test_pass("010: >10000bp uses max_fee"); } else { test_fail("010", &format!("expected 500, got {}", fee)); }
}

fn test_011_fee_zero_amount() {
    let fee = pos_core_logic::calculate_token2022_fee(0, 10, 1_000_000);
    if fee == 0 { test_pass("011: zero amount = zero fee"); } else { test_fail("011", &format!("expected 0, got {}", fee)); }
}

fn test_012_fee_zero_bp() {
    let fee = pos_core_logic::calculate_token2022_fee(1_000_000, 0, 1_000_000);
    if fee == 0 { test_pass("012: zero bp = zero fee"); } else { test_fail("012", &format!("expected 0, got {}", fee)); }
}

fn test_013_atomic_to_f64() {
    let r = pos_core_logic::atomic_to_f64(1_000_000, 6);
    if (r - 1.0).abs() < f64::EPSILON { test_pass("013: atomic_to_f64(1000000, 6) = 1.0"); } else { test_fail("013", &format!("expected 1.0, got {}", r)); }
}

fn test_014_slippage_tolerance_valid() {
    let r = pos_core_logic::is_payment_amount_valid(9.9, 10.0, 1.0);
    if r { test_pass("014: 9.9 USDC within 1% of 10.0"); } else { test_fail("014", "expected true"); }
}

fn test_015_slippage_tolerance_invalid() {
    let r = pos_core_logic::is_payment_amount_valid(9.8, 10.0, 1.0);
    if !r { test_pass("015: 9.8 USDC outside 1% of 10.0"); } else { test_fail("015", "expected false"); }
}

fn test_016_token2022_fee_large_amount() {
    let fee = pos_core_logic::calculate_token2022_fee(100_000_000, 10, 1_000_000_000);
    if fee == 100_000 { test_pass("016: large amount fee = 100000 (10bp of 100M)"); } else { test_fail("016", &format!("expected 100000, got {}", fee)); }
}

fn test_017_token2022_fee_small_amount() {
    let fee = pos_core_logic::calculate_token2022_fee(1, 10, 1_000_000);
    if fee == 1 { test_pass("017: 1 atomic unit fee = 1"); } else { test_fail("017", &format!("expected 1, got {}", fee)); }
}

fn test_018_token2022_fee_custom_decimals() {
    let atomic = pos_core_logic::safe_f64_to_u64_atomic(100.0, 18);
    let fee = pos_core_logic::calculate_token2022_fee(atomic, 100, 1_000_000);
    if fee > 0 { test_pass("018: custom decimals fee > 0"); } else { test_fail("018", "expected fee > 0"); }
}

fn test_019_atomic_conversion_max_u64() {
    let r = pos_core_logic::safe_f64_to_u64_atomic(f64::MAX, 6);
    if r == u64::MAX { test_pass("019: f64::MAX -> MAX_U64"); } else { test_fail("019", &format!("expected MAX_U64, got {}", r)); }
}

fn test_020_atomic_conversion_fractional() {
    let r = pos_core_logic::usdc_to_atomic_units(0.000001);
    if r == 1 { test_pass("020: 0.000001 USDC = 1 atomic"); } else { test_fail("020", &format!("expected 1, got {}", r)); }
}

fn test_021_fee_calculation_100_percent() {
    let fee = pos_core_logic::calculate_token2022_fee(1_000_000, 10000, 1_000_000);
    if fee == 1_000_000 { test_pass("021: 100% fee = full amount"); } else { test_fail("021", &format!("expected 1000000, got {}", fee)); }
}

fn test_022_fee_calculation_zero_max_fee() {
    let fee = pos_core_logic::calculate_token2022_fee(1_000_000, 10, 0);
    if fee == 0 { test_pass("022: zero max_fee = zero fee"); } else { test_fail("022", &format!("expected 0, got {}", fee)); }
}

fn test_023_atomic_conversion_6_decimals() {
    let r = pos_core_logic::usdc_to_atomic_units(42.50);
    if r == 42_500_000 { test_pass("023: 42.50 USDC = 42500000 atomic"); } else { test_fail("023", &format!("expected 42500000, got {}", r)); }
}

fn test_024_atomic_conversion_9_decimals() {
    let r = pos_core_logic::safe_f64_to_u64_atomic(1.0, 9);
    if r == 1_000_000_000 { test_pass("024: 1.0 SOL = 1000000000 atomic"); } else { test_fail("024", &format!("expected 1000000000, got {}", r)); }
}

fn test_025_fee_calculation_symmetry() {
    let fee1 = pos_core_logic::calculate_token2022_fee(1_000_000, 10, 10_000_000);
    let fee2 = pos_core_logic::calculate_token2022_fee(1_000_000, 10, 10_000_000);
    if fee1 == fee2 { test_pass("025: fee calculation is deterministic"); } else { test_fail("025", &format!("{} != {}", fee1, fee2)); }
}

fn test_026_atomic_conversion_precision() {
    let r = pos_core_logic::usdc_to_atomic_units(0.1 + 0.2);
    let expected = pos_core_logic::usdc_to_atomic_units(0.3);
    if r == expected { test_pass("026: 0.1+0.2 == 0.3 in atomic"); } else { test_fail("026", &format!("{} != {}", r, expected)); }
}

fn test_027_fee_calculation_stress() {
    let fee = pos_core_logic::calculate_token2022_fee(u64::MAX / 2, 5000, u64::MAX);
    if fee > 0 && fee <= u64::MAX { test_pass("027: stress test no overflow"); } else { test_fail("027", &format!("fee = {}", fee)); }
}

fn test_028_atomic_conversion_large_values() {
    let r = pos_core_logic::usdc_to_atomic_units(1_000_000.0);
    if r == 1_000_000_000_000 { test_pass("028: 1M USDC = 1T atomic"); } else { test_fail("028", &format!("expected 1000000000000, got {}", r)); }
}

fn test_029_fee_calculation_boundary() {
    let fee = pos_core_logic::calculate_token2022_fee(0, 0, 0);
    if fee == 0 { test_pass("029: all zeros = zero fee"); } else { test_fail("029", &format!("expected 0, got {}", fee)); }
}

fn test_030_atomic_conversion_consistency() {
    let r1 = pos_core_logic::usdc_to_atomic_units(10.0);
    let r2 = pos_core_logic::usdc_to_atomic_units(10.0);
    if r1 == r2 { test_pass("030: conversion is consistent"); } else { test_fail("030", &format!("{} != {}", r1, r2)); }
}
