use crate::constants::SOL_MINT;

/// Generates a cryptographically secure 32-byte reference key encoded as Base58 (valid Solana Public Key format).
pub fn generate_secure_reference_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    bs58::encode(bytes).into_string()
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

/// Decodes a Base58 string into a 32-byte array.
fn decode_bs58_32(input: &str) -> Result<[u8; 32], &'static str> {
    const ALPHABET: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";
    let mut bytes = Vec::new();
    for c in input.bytes() {
        let val = match ALPHABET.iter().position(|&b| b == c) {
            Some(v) => v as u64,
            None => return Err("Invalid Base58 character"),
        };
        let mut carry = val;
        for byte in bytes.iter_mut().rev() {
            let temp = (*byte as u64) * 58 + carry;
            *byte = (temp & 0xFF) as u8;
            carry = temp >> 8;
        }
        while carry > 0 {
            bytes.insert(0, (carry & 0xFF) as u8);
            carry >>= 8;
        }
    }
    for c in input.bytes() {
        if c == b'1' {
            bytes.insert(0, 0);
        } else {
            break;
        }
    }
    if bytes.len() > 32 {
        return Err("Base58 string exceeds 32 bytes");
    }
    let mut out = [0u8; 32];
    let offset = 32 - bytes.len();
    out[offset..].copy_from_slice(&bytes);
    Ok(out)
}

/// Builds an unsigned Solana Actions (Blink) wire transaction for USDC SPL Token transfer.
///
/// Features:
/// - User wallet set as fee_payer at index 0 (num_required_signatures = 1)
/// - Single signature slot reserved with 64 zero bytes
/// - Reference key included as non-signer, non-writable account for Triple Payment Protection
/// - Encoded to Base64 (Solana Actions spec v2.1.3 compliant)
pub fn build_actions_payment_transaction(
    user_wallet_pubkey: &str,
    merchant_ata_pubkey: &str,
    amount_usdc: f64,
    usdc_mint_pubkey: &str,
    reference_pubkey: &str,
    recent_blockhash: &str,
) -> Result<String, &'static str> {
    let user_pk = decode_bs58_32(user_wallet_pubkey)?;
    let merchant_pk = decode_bs58_32(merchant_ata_pubkey)?;
    let reference_pk = decode_bs58_32(reference_pubkey)?;
    let mint_pk = decode_bs58_32(usdc_mint_pubkey)?;
    let blockhash_bytes = decode_bs58_32(recent_blockhash)?;

    let token_program_pk = decode_bs58_32("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")?;
    let usdc_atomic = crate::token2022::safe_f64_to_u64_atomic(amount_usdc, 6);

    let mut ix_data = Vec::with_capacity(10);
    ix_data.push(12u8);
    ix_data.extend_from_slice(&usdc_atomic.to_le_bytes());
    ix_data.push(6u8);

    let mut message = Vec::new();
    message.push(1u8);
    message.push(0u8);
    message.push(3u8);

    message.push(5u8);
    message.extend_from_slice(&user_pk);
    message.extend_from_slice(&merchant_pk);
    message.extend_from_slice(&reference_pk);
    message.extend_from_slice(&mint_pk);
    message.extend_from_slice(&token_program_pk);

    message.extend_from_slice(&blockhash_bytes);

    message.push(1u8);
    message.push(4u8);

    message.push(5u8);
    message.extend_from_slice(&[0u8, 3u8, 1u8, 0u8, 2u8]);

    message.push(10u8);
    message.extend_from_slice(&ix_data);

    let mut tx_bytes = Vec::with_capacity(1 + 64 + message.len());
    tx_bytes.push(1u8);
    tx_bytes.extend_from_slice(&[0u8; 64]);
    tx_bytes.extend_from_slice(&message);

    Ok(crate::squads::base64_encode(&tx_bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reference_key_length() {
        let key = generate_secure_reference_key();
        assert!(key.len() >= 43 && key.len() <= 44);
        assert!(key.chars().all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c)));
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

    #[test]
    fn test_build_actions_payment_transaction_valid() {
        let tx_base64 = build_actions_payment_transaction(
            "8xAZmQ1111111111111111111111111111111111111",
            "8xAZmQ1111111111111111111111111111111111111",
            15.5,
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "7xRefKey11111111111111111111111111111111111",
            "4vJ9JU1bJJE96FWSXTvBxF2vT7JhRReB88vC17A88vC1",
        )
        .unwrap();
        assert!(!tx_base64.is_empty(), "base64 transaction string should not be empty");
    }
}
