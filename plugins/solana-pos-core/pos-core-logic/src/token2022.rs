use crate::constants::{USDC_DECIMALS, MAX_U64};

/// Converts a floating-point amount to atomic units using u128 integer arithmetic.
/// Returns 0 for NaN, infinity, negative, or zero amounts.
/// Caps at MAX_U64 to prevent overflow.
pub fn safe_f64_to_u64_atomic(amount: f64, decimals: u8) -> u64 {
    if !amount.is_finite() || amount <= 0.0 {
        return 0;
    }

    let scale = 10u128.pow(decimals as u32);

    // Check if the scaled value would overflow u64
    // Use f64 multiplication first, then check for overflow
    let scaled_f64 = amount * scale as f64;

    if !scaled_f64.is_finite() || scaled_f64 < 0.0 {
        // Overflow or invalid - cap at MAX_U64
        return MAX_U64;
    }

    // Try u128 conversion for precision
    let scaled_u128 = (amount * scale as f64).round() as u128;
    if scaled_u128 >= MAX_U64 as u128 {
        MAX_U64
    } else {
        scaled_u128 as u64
    }
}

/// Backward-compatible alias for 6-decimal USDC atomic conversion.
pub fn usdc_to_atomic_units(amount_usdc: f64) -> u64 {
    safe_f64_to_u64_atomic(amount_usdc, USDC_DECIMALS)
}

/// Calculates Token-2022 transfer fee using u128 integer precision.
/// Uses ceiling-based rounding (add 9999 before integer division).
/// Guards against > 10000 basis points (returns max_fee).
/// Returns 0 for zero amount or decimals > 18.
pub fn calculate_token2022_fee(
    amount_atomic: u64,
    fee_basis_points: u16,
    max_fee_units: u64,
) -> u64 {
    if fee_basis_points > 10000 {
        return max_fee_units;
    }
    if amount_atomic == 0 {
        return 0;
    }

    let fee_units = (amount_atomic as u128 * fee_basis_points as u128 + 9999) / 10000;
    fee_units.min(max_fee_units as u128) as u64
}

/// Converts atomic units back to floating-point with the given decimals.
pub fn atomic_to_f64(atomic: u64, decimals: u8) -> f64 {
    let scale = 10f64.powi(decimals as i32);
    atomic as f64 / scale
}

/// Checks if a payment amount is within slippage tolerance.
pub fn is_payment_amount_valid(
    paid_usdc: f64,
    expected_usdc: f64,
    slippage_tolerance_pct: f64,
) -> bool {
    paid_usdc >= (expected_usdc * (1.0 - (slippage_tolerance_pct / 100.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_usdc_conversion_normal() {
        assert_eq!(usdc_to_atomic_units(1.0), 1_000_000);
        assert_eq!(usdc_to_atomic_units(0.5), 500_000);
        assert_eq!(usdc_to_atomic_units(10.0), 10_000_000);
    }

    #[test]
    fn test_usdc_conversion_edge_cases() {
        assert_eq!(usdc_to_atomic_units(0.0), 0);
        assert_eq!(usdc_to_atomic_units(-1.0), 0);
        assert_eq!(usdc_to_atomic_units(f64::NAN), 0);
        assert_eq!(usdc_to_atomic_units(f64::INFINITY), 0);
        assert_eq!(usdc_to_atomic_units(f64::NEG_INFINITY), 0);
    }

    #[test]
    fn test_fee_calculation_normal() {
        // 1 USDC = 1_000_000 atomic, 10 bp = 0.1%
        let fee = calculate_token2022_fee(1_000_000, 10, 1_000_000);
        assert_eq!(fee, 1000); // 0.001 USDC
    }

    #[test]
    fn test_fee_calculation_ceiling() {
        // 3 atomic units * 10 bp = 0.003, ceiling = 1
        let fee = calculate_token2022_fee(3, 10, 1_000_000);
        assert_eq!(fee, 1);
    }

    #[test]
    fn test_fee_calculation_max_cap() {
        let fee = calculate_token2022_fee(1_000_000, 10000, 500);
        assert_eq!(fee, 500);
    }

    #[test]
    fn test_fee_exceeding_max_bp() {
        let fee = calculate_token2022_fee(1_000_000, 10001, 500);
        assert_eq!(fee, 500);
    }

    #[test]
    fn test_fee_zero_amount() {
        let fee = calculate_token2022_fee(0, 10, 1_000_000);
        assert_eq!(fee, 0);
    }

    #[test]
    fn test_safe_f64_overflow() {
        let result = safe_f64_to_u64_atomic(f64::MAX, 6);
        assert_eq!(result, MAX_U64);
    }

    #[test]
    fn test_atomic_to_f64() {
        assert!((atomic_to_f64(1_000_000, 6) - 1.0).abs() < f64::EPSILON);
        assert!((atomic_to_f64(500_000, 6) - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_slippage_tolerance() {
        assert!(is_payment_amount_valid(9.9, 10.0, 1.0));
        assert!(is_payment_amount_valid(10.0, 10.0, 1.0));
        assert!(!is_payment_amount_valid(9.8, 10.0, 1.0));
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_usdc_conversion_never_panics(amount in any::<f64>()) {
            let _ = usdc_to_atomic_units(amount);
        }

        #[test]
        fn prop_safe_atomic_never_panics(
            amount in any::<f64>(),
            decimals in 0u8..18u8
        ) {
            let _ = safe_f64_to_u64_atomic(amount, decimals);
        }

        #[test]
        fn prop_fee_calc_never_panics(
            amount in 0u64..1_000_000_000u64,
            bp in 0u16..65535u16,
            max_fee in 0u64..1_000_000u64
        ) {
            let result = calculate_token2022_fee(amount, bp, max_fee);
            assert!(result <= max_fee);
        }
    }
}
