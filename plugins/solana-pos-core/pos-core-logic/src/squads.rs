/// Anchor discriminator for Squads v4 `create_proposal` instruction.
/// Computed as sha256("global:create_proposal")[..8].
pub const ANCHOR_DISCRIMINATOR: [u8; 8] = [132, 116, 68, 174, 216, 160, 198, 22];

/// Builds raw Squads v4 instruction data with Anchor discriminator and Borsh encoding.
///
/// Layout (18 bytes total):
/// - 8 bytes: Anchor discriminator (sha256("global:create_proposal")[..8])
/// - 8 bytes: proposal_index (u64 LE)
/// - 1 byte:  execution_type (u8)
/// - 1 byte:  draft (bool as u8: 0 or 1)
pub fn build_squads_v4_instruction_data(
    proposal_index: u64,
    execution_type: u8,
    draft: bool,
) -> Vec<u8> {
    let mut data = Vec::with_capacity(18);
    data.extend_from_slice(&ANCHOR_DISCRIMINATOR);
    data.extend_from_slice(&proposal_index.to_le_bytes());
    data.push(execution_type);
    data.push(draft as u8);
    data
}

/// Builds a Squads v4 proposal JSON payload with program_id and hex-encoded instruction data.
pub fn build_squads_v4_proposal(
    multisig_pubkey: &str,
    vault_pubkey: &str,
    proposer_pubkey: &str,
    recipient_pubkey: &str,
    amount_usdc: f64,
    proposal_index: u64,
    memo: &str,
) -> serde_json::Value {
    let instruction_data = build_squads_v4_instruction_data(proposal_index, 0, false);
    let hex_discriminator = hex_encode(&ANCHOR_DISCRIMINATOR);
    let hex_instruction_data = hex_encode(&instruction_data);

    serde_json::json!({
        "program_id": "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm",
        "multisig_pubkey": multisig_pubkey,
        "vault_pubkey": vault_pubkey,
        "proposer_pubkey": proposer_pubkey,
        "recipient_pubkey": recipient_pubkey,
        "amount_usdc": amount_usdc,
        "proposal_index": proposal_index,
        "memo": memo,
        "anchor_discriminator": hex_discriminator,
        "instruction_data_hex": hex_instruction_data,
        "instruction_data_base64": base64_encode(&instruction_data),
    })
}

/// Hex-encodes bytes to a lowercase hex string.
pub fn hex_encode(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Base64-encodes bytes (RFC 4648).
pub fn base64_encode(input: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((input.len() + 2) / 3 * 4);

    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = if chunk.len() > 1 { chunk[1] as u32 } else { 0 };
        let b2 = if chunk.len() > 2 { chunk[2] as u32 } else { 0 };
        let triple = (b0 << 16) | (b1 << 8) | b2;

        result.push(CHARSET[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARSET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARSET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARSET[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anchor_discriminator() {
        assert_eq!(ANCHOR_DISCRIMINATOR, [132, 116, 68, 174, 216, 160, 198, 22]);
        assert_eq!(ANCHOR_DISCRIMINATOR.len(), 8);
    }

    #[test]
    fn test_build_instruction_data() {
        let data = build_squads_v4_instruction_data(42, 0, false);
        assert_eq!(data.len(), 18);
        assert_eq!(&data[..8], &ANCHOR_DISCRIMINATOR);
        assert_eq!(data[8..16], 42u64.to_le_bytes());
        assert_eq!(data[16], 0); // execution_type
        assert_eq!(data[17], 0); // draft = false
    }

    #[test]
    fn test_build_instruction_data_draft() {
        let data = build_squads_v4_instruction_data(1, 1, true);
        assert_eq!(data[16], 1); // execution_type
        assert_eq!(data[17], 1); // draft = true
    }

    #[test]
    fn test_hex_encode() {
        assert_eq!(hex_encode(&[0x84, 0x74, 0x44, 0xae]), "847444ae");
        assert_eq!(hex_encode(&ANCHOR_DISCRIMINATOR), "847444aed8a0c616");
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"Hello, World!"), "SGVsbG8sIFdvcmxkIQ==");
        assert_eq!(base64_encode(b"Man"), "TWFu");
        assert_eq!(base64_encode(b"Ma"), "TWE=");
        assert_eq!(base64_encode(b"M"), "TQ==");
        assert_eq!(base64_encode(b""), "");
    }

    #[test]
    fn test_base64_encode_instruction_data() {
        let data = build_squads_v4_instruction_data(0, 0, false);
        let encoded = base64_encode(&data);
        assert!(!encoded.is_empty());
        assert!(encoded.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '='));
    }

    #[test]
    fn test_build_proposal_json() {
        let proposal = build_squads_v4_proposal(
            "multisig111",
            "vault111",
            "proposer111",
            "recipient111",
            10.0,
            5,
            "Refund proposal",
        );

        assert_eq!(proposal["program_id"], "SQDS4ep65T869rmQrGGsybZb26a6Uq3vig54W62pkhm");
        assert_eq!(proposal["multisig_pubkey"], "multisig111");
        assert_eq!(proposal["amount_usdc"], 10.0);
        assert_eq!(proposal["proposal_index"], 5);
        assert!(!proposal["instruction_data_base64"].as_str().unwrap().is_empty());
    }

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_fee_calc_never_panics(
            idx in 0u64..u64::MAX,
            exec in 0u8..2u8,
            draft in any::<bool>()
        ) {
            let data = build_squads_v4_instruction_data(idx, exec, draft);
            assert_eq!(data.len(), 18);
        }
    }
}
