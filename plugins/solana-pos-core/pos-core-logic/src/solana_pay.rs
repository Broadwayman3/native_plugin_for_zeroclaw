use crate::constants::SOL_MINT;

/// Generates a cryptographically secure 32-byte reference key encoded as base32 (truncated to 44 chars).
pub fn generate_secure_reference_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    let full = base32_encode(&bytes);
    full[..44].to_string()
}

/// Encodes bytes to uppercase base32 (RFC 4648) without padding.
fn base32_encode(input: &[u8]) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::with_capacity((input.len() * 8 + 4) / 5);
    let mut bits: u32 = 0;
    let mut bits_left: i32 = 0;

    for &byte in input {
        bits = (bits << 8) | byte as u32;
        bits_left += 8;
        while bits_left >= 5 {
            bits_left -= 5;
            result.push(CHARSET[((bits >> bits_left) & 0x1F) as usize] as char);
        }
    }

    if bits_left > 0 {
        result.push(CHARSET[((bits << (5 - bits_left)) & 0x1F) as usize] as char);
    }

    result
}

/// SIP-0001 compliant Solana Pay URL generator.
/// - Omits spl-token for Native SOL (SOL_MINT or "11111111...").
/// - Percent-encodes label and message parameters.
pub fn build_solana_pay_url(
    merchant_pubkey: &str,
    amount: f64,
    reference_pubkey: &str,
    spl_token_mint: Option<&str>,
    label: &str,
    message: &str,
) -> String {
    let encoded_label = url_encode(label);
    let encoded_message = url_encode(message);

    let mut url = format!(
        "solana:{}?amount={:.2}&reference={}&label={}&message={}",
        merchant_pubkey, amount, reference_pubkey, encoded_label, encoded_message
    );

    if let Some(mint) = spl_token_mint {
        if mint != SOL_MINT && mint != "11111111111111111111111111111111" {
            url.push_str(&format!("&spl-token={}", mint));
        }
    }

    url
}

/// Generates Phantom Universal HTTPS Deep Link for 1-tap mobile wallet opening.
pub fn generate_phantom_universal_link(solana_pay_url: &str) -> String {
    let encoded = url_encode(solana_pay_url);
    format!("https://phantom.app/ul/browse/{}?ref=zeroclaw", encoded)
}

/// Minimal percent-encoder (space → %20, other special chars).
fn url_encode(input: &str) -> String {
    let mut result = String::with_capacity(input.len() * 3);
    for byte in input.bytes() {
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

/// Returns the active RPC URL, preferring primary and falling back if needed.
pub fn get_active_rpc_url(primary: Option<&str>, fallback: Option<&str>) -> String {
    primary
        .filter(|s| !s.is_empty())
        .or(fallback)
        .unwrap_or("https://api.devnet.solana.com")
        .to_string()
}

/// Generates atomic refund instructions (ATA + SPL transfer).
pub fn generate_atomic_refund_instructions(
    payer_pubkey: &str,
    recipient_pubkey: &str,
    amount_usdc: f64,
    mint: &str,
    nonce_pubkey: Option<&str>,
) -> Vec<serde_json::Value> {
    let mut instructions = Vec::new();

    if let Some(nonce) = nonce_pubkey {
        instructions.push(serde_json::json!({
            "instruction": "AdvanceNonceAccount",
            "nonce_account": nonce,
            "authority": payer_pubkey
        }));
    }

    instructions.push(serde_json::json!({
        "instruction": "createAssociatedTokenAccountIdempotent",
        "payer": payer_pubkey,
        "owner": recipient_pubkey,
        "mint": mint
    }));

    instructions.push(serde_json::json!({
        "instruction": "splTokenTransfer",
        "from": payer_pubkey,
        "to": recipient_pubkey,
        "amount_usdc": amount_usdc
    }));

    instructions
}

/// Validates Squads v4 multisig account data (null account defense).
/// Returns the next proposal index (transaction_index + 1).
pub fn validate_squads_multisig_account(
    account_data: Option<&serde_json::Value>,
) -> Result<u64, &'static str> {
    let data = account_data.ok_or("FAIL_CLOSED: Missing Squads multisig account")?;
    let tx_index = data
        .get("transaction_index")
        .and_then(|v| v.as_u64())
        .ok_or("FAIL_CLOSED: Invalid or missing transaction_index")?;
    Ok(tx_index + 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_key_length() {
        let key = generate_secure_reference_key();
        assert_eq!(key.len(), 44);
        assert!(key.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()));
    }

    #[test]
    fn test_solana_pay_url_with_token() {
        let url = build_solana_pay_url(
            "8xAZmQ1111111111111111111111111111111111111",
            10.5,
            "7xRefKey11111111111111111111111111111111111",
            Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
            "ZeroClaw POS",
            "POS Payment",
        );
        assert!(url.starts_with("solana:8xAZmQ1111111111111111111111111111111111111?amount=10.50"));
        assert!(url.contains("spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"));
    }

    #[test]
    fn test_solana_pay_url_without_sol_token() {
        let url = build_solana_pay_url(
            "8xAZmQ1111111111111111111111111111111111111",
            5.0,
            "7xRefKey11111111111111111111111111111111111",
            Some("So11111111111111111111111111111111111111112"),
            "ZeroClaw POS",
            "POS Payment",
        );
        assert!(!url.contains("spl-token="));
    }

    #[test]
    fn test_solana_pay_url_no_token() {
        let url = build_solana_pay_url(
            "8xAZmQ1111111111111111111111111111111111111",
            5.0,
            "7xRefKey11111111111111111111111111111111111",
            None,
            "ZeroClaw POS",
            "POS Payment",
        );
        assert!(!url.contains("spl-token="));
    }

    #[test]
    fn test_phantom_link() {
        let link = generate_phantom_universal_link("solana:pubkey?amount=10.00");
        assert!(link.starts_with("https://phantom.app/ul/browse/"));
        assert!(link.ends_with("?ref=zeroclaw"));
    }

    #[test]
    fn test_get_active_rpc_url() {
        assert_eq!(
            get_active_rpc_url(Some("https://primary.com"), Some("https://fallback.com")),
            "https://primary.com"
        );
        assert_eq!(
            get_active_rpc_url(None, Some("https://fallback.com")),
            "https://fallback.com"
        );
        assert_eq!(
            get_active_rpc_url(Some(""), Some("https://fallback.com")),
            "https://fallback.com"
        );
        assert_eq!(
            get_active_rpc_url(None, None),
            "https://api.devnet.solana.com"
        );
    }

    #[test]
    fn test_validate_squads_multisig_account() {
        let data = serde_json::json!({"transaction_index": 5});
        assert_eq!(validate_squads_multisig_account(Some(&data)).unwrap(), 6);
        assert!(validate_squads_multisig_account(None).is_err());
    }

    #[test]
    fn test_generate_refund_instructions() {
        let ix = generate_atomic_refund_instructions(
            "payer111",
            "recipient111",
            10.0,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            Some("nonce111"),
        );
        assert_eq!(ix.len(), 3);
        assert_eq!(ix[0]["instruction"], "AdvanceNonceAccount");
    }

    #[test]
    fn test_generate_refund_instructions_no_nonce() {
        let ix = generate_atomic_refund_instructions(
            "payer111",
            "recipient111",
            10.0,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            None,
        );
        assert_eq!(ix.len(), 2);
    }
}
