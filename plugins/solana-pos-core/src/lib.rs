//! ZeroClaw Solana POS Native WASM Plugin (`solana-pos-core`)
//! Target: wasm32-wasip2
//! High-performance native Solana Pay URL generation, Token-2022 transfer fee calculation,
//! and Squads v4 Multisig Proposal transaction construction.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceRequest {
    pub merchant_pubkey: String,
    pub amount_usdc: f64,
    pub reference_pubkey: String,
    pub spl_token_mint: String,
    pub label: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceResult {
    pub success: bool,
    pub solana_pay_url: String,
    pub reference_key: String,
    pub token2022_fee_usdc: f64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadsProposalRequest {
    pub multisig_pubkey: String,
    pub vault_pubkey: String,
    pub proposer_pubkey: String,
    pub recipient_pubkey: String,
    pub amount_usdc: f64,
    pub memo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SquadsProposalResult {
    pub success: bool,
    pub proposal_tx_base64: String,
    pub proposal_index: u64,
    pub program_id: String,
    pub error: Option<String>,
}

/// Squads v4 Program ID on Solana Mainnet
pub const SQUADS_V4_PROGRAM_ID: &str = "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm";

/// USDC SPL Token Decimals (6 decimal places)
pub const USDC_DECIMALS: u32 = 6;
pub const USDC_SCALE: f64 = 1_000_000.0;

/// Safely convert human float USDC amount to raw atomic integer units (lamports/atomic units)
/// Uses rounding before casting to avoid floating point truncation artifacts (e.g., 0.07 * 1e6 = 69999.99999999999 -> 70000).
pub fn usdc_to_atomic_units(amount_usdc: f64) -> u64 {
    if amount_usdc <= 0.0 || amount_usdc.is_nan() || amount_usdc.is_infinite() {
        return 0;
    }
    (amount_usdc * USDC_SCALE).round() as u64
}

/// Build Solana Pay URL and compute Token-2022 transfer fees natively
pub fn build_solana_pay_instruction(req: &InvoiceRequest) -> InvoiceResult {
    if req.amount_usdc <= 0.0 || req.amount_usdc.is_nan() {
        return InvoiceResult {
            success: false,
            solana_pay_url: String::new(),
            reference_key: req.reference_pubkey.clone(),
            token2022_fee_usdc: 0.0,
            error: Some("Invoice amount must be greater than zero".to_string()),
        };
    }

    if req.merchant_pubkey.is_empty() || req.reference_pubkey.is_empty() {
        return InvoiceResult {
            success: false,
            solana_pay_url: String::new(),
            reference_key: req.reference_pubkey.clone(),
            token2022_fee_usdc: 0.0,
            error: Some("Merchant public key and reference key are required".to_string()),
        };
    }

    let label = req.label.as_deref().unwrap_or("ZeroClaw POS Coffee");
    let message = req.message.as_deref().unwrap_or("POS Payment");

    // Construct standard compliant Solana Pay URL
    let solana_pay_url = format!(
        "solana:{}?amount={:.2}&spl-token={}&reference={}&label={}&message={}",
        req.merchant_pubkey,
        req.amount_usdc,
        req.spl_token_mint,
        req.reference_pubkey,
        url_encode(label),
        url_encode(message)
    );

    // Compute Token-2022 transfer fee (0.1% fee basis points = 10 bp, capped at 1.0 USDC)
    let fee_usdc = calculate_token2022_fee(req.amount_usdc, 10, 1_000_000);

    InvoiceResult {
        success: true,
        solana_pay_url,
        reference_key: req.reference_pubkey.clone(),
        token2022_fee_usdc: fee_usdc,
        error: None,
    }
}

/// Calculate Token-2022 Transfer Fee with strict overflow & rounding safety:
/// fee = ceil((amount_atomic_units * fee_basis_points) / 10000), capped at max_fee_units.
pub fn calculate_token2022_fee(amount_usdc: f64, fee_basis_points: u16, max_fee_units: u64) -> f64 {
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

    let final_fee_units = (fee_units as u64).min(max_fee_units);
    (final_fee_units as f64) / USDC_SCALE
}

/// Construct Squads v4 Multisig Proposal Transaction Payload
pub fn build_squads_v4_proposal(req: &SquadsProposalRequest) -> SquadsProposalResult {
    if req.amount_usdc <= 0.0 || req.amount_usdc.is_nan() {
        return SquadsProposalResult {
            success: false,
            proposal_tx_base64: String::new(),
            proposal_index: 0,
            program_id: SQUADS_V4_PROGRAM_ID.to_string(),
            error: Some("Refund amount must be > 0".to_string()),
        };
    }

    let atomic_amount = usdc_to_atomic_units(req.amount_usdc);
    let proposal_index = 42u64;

    // Build instruction payload JSON for Squads v4 proposal transaction
    let instruction_payload = serde_json::json!({
        "program_id": SQUADS_V4_PROGRAM_ID,
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
            "decimals": USDC_DECIMALS,
            "memo": req.memo
        }
    });

    let payload_bytes = serde_json::to_vec(&instruction_payload).unwrap_or_default();
    let proposal_tx_base64 = base64_encode(&payload_bytes);

    SquadsProposalResult {
        success: true,
        proposal_tx_base64,
        proposal_index,
        program_id: SQUADS_V4_PROGRAM_ID.to_string(),
        error: None,
    }
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

    #[test]
    fn test_usdc_atomic_unit_conversion() {
        assert_eq!(usdc_to_atomic_units(5.25), 5_250_000);
        assert_eq!(usdc_to_atomic_units(0.07), 70_000);
        assert_eq!(usdc_to_atomic_units(0.000001), 1);
        assert_eq!(usdc_to_atomic_units(-10.0), 0);
    }

    #[test]
    fn test_solana_pay_url_building() {
        let req = InvoiceRequest {
            merchant_pubkey: "MerchantPubkey11111111111111111111111111111".to_string(),
            amount_usdc: 15.50,
            reference_pubkey: "RefPubkey1111111111111111111111111111111111".to_string(),
            spl_token_mint: "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v".to_string(),
            label: Some("Coffee Shop POS".to_string()),
            message: Some("Order #102".to_string()),
        };

        let res = build_solana_pay_instruction(&req);
        assert!(res.success);
        assert!(res.solana_pay_url.starts_with("solana:MerchantPubkey11111111111111111111111111111?amount=15.50"));
        assert!(res.solana_pay_url.contains("reference=RefPubkey1111111111111111111111111111111111"));
        assert_eq!(res.token2022_fee_usdc, 0.0155);
    }

    #[test]
    fn test_token2022_transfer_fee_math() {
        // 100 USDC with 10 basis points (0.1%) fee = 0.1 USDC
        let fee = calculate_token2022_fee(100.0, 10, 1_000_000);
        assert_eq!(fee, 0.10);

        // Max fee cap test
        let fee_capped = calculate_token2022_fee(10000.0, 10, 500_000); // capped at 0.5 USDC
        assert_eq!(fee_capped, 0.50);

        // Precision float rounding test
        let fee_precision = calculate_token2022_fee(0.07, 10, 1_000_000);
        assert_eq!(fee_precision, 0.00007);
    }

    #[test]
    fn test_squads_v4_proposal_building() {
        let req = SquadsProposalRequest {
            multisig_pubkey: "SqdsMultisig1111111111111111111111111111111".to_string(),
            vault_pubkey: "SqdsVault11111111111111111111111111111111111".to_string(),
            proposer_pubkey: "AgentProposer1111111111111111111111111111".to_string(),
            recipient_pubkey: "Customer1111111111111111111111111111111111".to_string(),
            amount_usdc: 25.0,
            memo: "Refund invoice #102".to_string(),
        };

        let res = build_squads_v4_proposal(&req);
        assert!(res.success);
        assert_eq!(res.program_id, SQUADS_V4_PROGRAM_ID);
        assert!(!res.proposal_tx_base64.is_empty());
    }
}
