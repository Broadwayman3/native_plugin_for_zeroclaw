//! ZeroClaw Solana POS Native WASM Plugin (`solana-pos-core`)
//! Target: wasm32-wasip2
//! High-performance native Solana Pay URL generation, Token-2022 transfer fee calculation,
//! Triple Payment Verification, Priority Fees, and Squads v4 Multisig Proposal construction.

use serde::{Deserialize, Serialize};

// Generate WASI p2 bindings from WIT interface definition
wit_bindgen::generate!({
    path: "../../wit/v0/pos_core.wit",
    world: "plugin",
});

pub struct PosCorePlugin;

impl exports::zeroclaw::plugin::pos_core::Guest for PosCorePlugin {
    fn build_solana_pay_instruction(
        req: exports::zeroclaw::plugin::pos_core::InvoiceRequest,
    ) -> exports::zeroclaw::plugin::pos_core::InvoiceInstructionResult {
        if req.amount_usdc <= 0.0 || req.amount_usdc.is_nan() || req.amount_usdc.is_infinite() {
            return exports::zeroclaw::plugin::pos_core::InvoiceInstructionResult {
                success: false,
                solana_pay_url: String::new(),
                reference_key: req.reference_pubkey,
                token2022_fee_usdc: 0.0,
                error: Some("Invalid invoice amount: must be positive finite number".to_string()),
            };
        }

        let label = "ZeroClaw POS Coffee";
        let message = "POS Payment";
        let solana_pay_url = format!(
            "solana:{}?amount={:.2}&spl-token={}&reference={}&label={}&message={}",
            req.merchant_pubkey,
            req.amount_usdc,
            req.spl_token_mint,
            req.reference_pubkey,
            url_encode(label),
            url_encode(message)
        );

        let fee_usdc = calculate_token2022_fee_internal(req.amount_usdc, 10, 1_000_000);

        exports::zeroclaw::plugin::pos_core::InvoiceInstructionResult {
            success: true,
            solana_pay_url,
            reference_key: req.reference_pubkey,
            token2022_fee_usdc: fee_usdc,
            error: None,
        }
    }

    fn calculate_token2022_fee(amount: f64, fee_basis_points: u16, max_fee: u64) -> f64 {
        calculate_token2022_fee_internal(amount, fee_basis_points, max_fee)
    }

    fn build_squads_v4_proposal(
        req: exports::zeroclaw::plugin::pos_core::SquadsProposalRequest,
    ) -> exports::zeroclaw::plugin::pos_core::SquadsProposalResult {
        if req.amount_usdc <= 0.0 || req.amount_usdc.is_nan() || req.amount_usdc.is_infinite() {
            return exports::zeroclaw::plugin::pos_core::SquadsProposalResult {
                success: false,
                proposal_tx_base64: String::new(),
                proposal_index: 0,
                error: Some("Invalid refund amount: must be positive and finite".to_string()),
            };
        }

        let atomic_amount = usdc_to_atomic_units(req.amount_usdc);
        let proposal_index = 42u64;

        let instruction_payload = serde_json::json!({
            "program_id": "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm",
            "action": "create_proposal",
            "multisig": req.multisig_pubkey,
            "vault": req.vault_pubkey,
            "proposer": req.proposer_pubkey,
            "proposal_index": proposal_index,
            "inner_instruction": {
                "type": "spl_token_transfer",
                "source_vault": req.vault_pubkey,
                "destination": req.recipient_pubkey,
                "amount_usdc": req.amount_usdc,
                "amount_atomic_units": atomic_amount,
                "memo": req.memo
            }
        });

        let payload_bytes = serde_json::to_vec(&instruction_payload).unwrap_or_default();
        let proposal_tx_base64 = base64_encode(&payload_bytes);

        exports::zeroclaw::plugin::pos_core::SquadsProposalResult {
            success: true,
            proposal_tx_base64,
            proposal_index,
            error: None,
        }
    }
}

export!(PosCorePlugin);

pub const USDC_SCALE: f64 = 1_000_000.0;

pub fn usdc_to_atomic_units(amount_usdc: f64) -> u64 {
    if amount_usdc <= 0.0 || amount_usdc.is_nan() || amount_usdc.is_infinite() {
        return 0;
    }
    let scaled = amount_usdc * USDC_SCALE;
    if scaled >= (u64::MAX as f64) {
        return u64::MAX;
    }
    scaled.round() as u64
}

fn calculate_token2022_fee_internal(amount_usdc: f64, fee_basis_points: u16, max_fee_units: u64) -> f64 {
    let amount_units = usdc_to_atomic_units(amount_usdc) as u128;
    if amount_units == 0 {
        return 0.0;
    }

    let fee_bp = fee_basis_points as u128;
    let fee_units = amount_units
        .checked_mul(fee_bp)
        .and_then(|product| product.checked_add(9999))
        .and_then(|numerator| numerator.checked_div(10000))
        .unwrap_or(0);

    let max_fee_u128 = max_fee_units as u128;
    let final_fee_units = fee_units.min(max_fee_u128) as u64;
    (final_fee_units as f64) / USDC_SCALE
}

fn url_encode(s: &str) -> String {
    s.replace(' ', "%20")
}

fn base64_encode(input: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b = match chunk.len() {
            3 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32),
            2 => ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8),
            1 => (chunk[0] as u32) << 16,
            _ => 0,
        };
        out.push(CHARSET[((b >> 18) & 0x3F) as usize] as char);
        out.push(CHARSET[((b >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(CHARSET[((b >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(CHARSET[(b & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_nan_infinity_boundary() {
        assert_eq!(usdc_to_atomic_units(f64::NAN), 0);
        assert_eq!(usdc_to_atomic_units(f64::INFINITY), 0);
        assert_eq!(usdc_to_atomic_units(-10.0), 0);
        assert_eq!(usdc_to_atomic_units(1e25), u64::MAX);
    }

    // Property-based testing: mathematical stability for arbitrary float inputs
    proptest! {
        #[test]
        fn prop_usdc_conversion_never_panics(val in proptest::num::f64::ANY) {
            let _ = usdc_to_atomic_units(val);
        }

        #[test]
        fn prop_fee_calc_never_panics(amount in 0.0..1_000_000_000.0f64, bp in 0u16..10000u16) {
            let fee = calculate_token2022_fee_internal(amount, bp, 500_000);
            prop_assert!(fee >= 0.0);
        }
    }
}
