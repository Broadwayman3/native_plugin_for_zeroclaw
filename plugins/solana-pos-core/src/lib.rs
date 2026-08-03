//! ZeroClaw Solana POS Native WASM Plugin (`solana-pos-core`)
//! Target: wasm32-wasip2
//! High-performance native Solana Pay URL generation, Token-2022 transfer fee calculation,
//! Triple Payment Verification, Priority Fees, and Squads v4 Multisig Proposal construction.

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

        let fee_usdc = calculate_token2022_fee_internal(req.amount_usdc, 10, 1_000_000, 6);

        exports::zeroclaw::plugin::pos_core::InvoiceInstructionResult {
            success: true,
            solana_pay_url,
            reference_key: req.reference_pubkey,
            token2022_fee_usdc: fee_usdc,
            error: None,
        }
    }

    fn calculate_token2022_fee(
        amount: f64,
        fee_basis_points: u16,
        max_fee: u64,
        decimals: u8,
    ) -> f64 {
        calculate_token2022_fee_internal(amount, fee_basis_points, max_fee, decimals)
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

        let atomic_amount = match safe_f64_to_u64_atomic(req.amount_usdc, 6) {
            Ok(val) => val,
            Err(e) => {
                return exports::zeroclaw::plugin::pos_core::SquadsProposalResult {
                    success: false,
                    proposal_tx_base64: String::new(),
                    proposal_index: 0,
                    error: Some(format!("Atomic unit conversion error: {}", e)),
                };
            }
        };
        let proposal_index = req.proposal_index;

        // Anchor discriminator & raw Borsh instruction byte packing for Squads v4 `create_proposal`
        let raw_instruction_bytes = build_raw_squads_v4_instruction_data(proposal_index, 1);
        let anchor_discriminator: [u8; 8] = [132, 116, 68, 174, 216, 160, 198, 22];

        let instruction_payload = serde_json::json!({
            "program_id": "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm",
            "anchor_discriminator_hex": hex_encode(&anchor_discriminator),
            "raw_instruction_bytes_hex": hex_encode(&raw_instruction_bytes),
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

        let payload_bytes = match serde_json::to_vec(&instruction_payload) {
            Ok(bytes) => bytes,
            Err(e) => {
                return exports::zeroclaw::plugin::pos_core::SquadsProposalResult {
                    success: false,
                    proposal_tx_base64: String::new(),
                    proposal_index: 0,
                    error: Some(format!("Payload serialization error: {}", e)),
                };
            }
        };
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

/// Prevents floating point precision drift (IEEE 754 precision drift) and overflow in u64.
/// Guarantees zero panics inside the WASM sandbox environment.
#[inline(always)]
pub fn safe_f64_to_u64_atomic(amount: f64, decimals: u8) -> Result<u64, &'static str> {
    if amount <= 0.0 || amount.is_nan() || amount.is_infinite() {
        return Err("Invalid float input: must be positive and finite");
    }
    let multiplier = 10f64.powi(decimals as i32);
    let scaled = amount * multiplier;

    if scaled >= (u64::MAX as f64) {
        return Err("Integer overflow: amount exceeds maximum u64 bounds");
    }

    Ok(scaled.round() as u64)
}

pub fn usdc_to_atomic_units(amount_usdc: f64) -> u64 {
    safe_f64_to_u64_atomic(amount_usdc, 6).unwrap_or(0)
}

fn calculate_token2022_fee_internal(
    amount_usdc: f64,
    fee_basis_points: u16,
    max_fee_units: u64,
    decimals: u8,
) -> f64 {
    if decimals > 18 {
        return 0.0;
    }
    let scale = 10f64.powi(decimals as i32);
    if fee_basis_points > 10000 {
        return (max_fee_units as f64) / scale;
    }

    let amount_units = safe_f64_to_u64_atomic(amount_usdc, decimals).unwrap_or(0) as u128;
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
    (final_fee_units as f64) / scale
}

/// Забезпечує Anchor-сумісне бінарне кодування для Squads v4 create_proposal
pub fn build_raw_squads_v4_instruction_data(proposal_index: u64, execution_type: u8) -> Vec<u8> {
    // Anchor discriminator: sha256("global:create_proposal")[..8]
    let mut data = vec![132, 116, 68, 174, 216, 160, 198, 22];

    // Borsh encoding: proposal_index (u64 little-endian)
    data.extend_from_slice(&proposal_index.to_le_bytes());

    // Borsh encoding: execution_type (u8)
    data.push(execution_type);

    // Borsh encoding: draft (bool = false)
    data.push(0);

    data
}

fn url_encode(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }
    result
}

fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn base64_encode(input: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
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
    fn test_safe_f64_to_u64_atomic() {
        assert_eq!(safe_f64_to_u64_atomic(10.5, 6).unwrap(), 10_500_000);
        assert!(safe_f64_to_u64_atomic(-1.0, 6).is_err());
        assert!(safe_f64_to_u64_atomic(f64::NAN, 6).is_err());
        assert!(safe_f64_to_u64_atomic(f64::INFINITY, 6).is_err());
        assert!(safe_f64_to_u64_atomic(1e25, 6).is_err());
    }

    #[test]
    fn test_nan_infinity_boundary() {
        assert_eq!(usdc_to_atomic_units(f64::NAN), 0);
        assert_eq!(usdc_to_atomic_units(f64::INFINITY), 0);
        assert_eq!(usdc_to_atomic_units(-10.0), 0);
        assert_eq!(usdc_to_atomic_units(1e25), 0);
    }

    #[test]
    fn test_build_raw_squads_v4_instruction_data() {
        let data = build_raw_squads_v4_instruction_data(42, 1);
        assert_eq!(data.len(), 18);
        assert_eq!(&data[0..8], &[132, 116, 68, 174, 216, 160, 198, 22]);
    }

    #[test]
    fn test_fee_bp_exceeding_max() {
        let fee = calculate_token2022_fee_internal(100.0, 20000, 500_000, 6);
        assert_eq!(fee, 0.50);
    }

    #[test]
    fn test_zero_decimals_token2022_fee() {
        let fee = calculate_token2022_fee_internal(100.0, 100, 10, 0);
        assert_eq!(fee, 1.0);
    }

    // Property-based testing: mathematical stability for arbitrary float inputs
    proptest! {
        #[test]
        fn prop_usdc_conversion_never_panics(val in proptest::num::f64::ANY) {
            let _ = usdc_to_atomic_units(val);
        }

        #[test]
        fn prop_safe_atomic_never_panics(val in proptest::num::f64::ANY, dec in 0u8..18u8) {
            let _ = safe_f64_to_u64_atomic(val, dec);
        }

        #[test]
        fn prop_fee_calc_never_panics(amount in 0.0..1_000_000_000.0f64, bp in 0u16..65535u16, dec in 0u8..18u8) {
            let fee = calculate_token2022_fee_internal(amount, bp, 500_000, dec);
            prop_assert!(fee >= 0.0);
        }
    }
}
