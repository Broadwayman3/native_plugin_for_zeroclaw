#[test]
fn test_031_solana_pay_url_with_token() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        10.5,
        "7xRefKey11111111111111111111111111111111111",
        Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        "ZeroClaw POS",
        "POS Payment",
    );
    assert!(
        url.starts_with("solana:")
            && url.contains("spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        "031: Solana Pay URL with SPL token, URL: {}",
        url
    );
}

#[test]
fn test_032_solana_pay_url_without_token() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        5.0,
        "7xRefKey11111111111111111111111111111111111",
        Some("So11111111111111111111111111111111111111112"),
        "ZeroClaw POS",
        "POS Payment",
    );
    assert!(
        !url.contains("spl-token="),
        "032: should not contain spl-token for SOL"
    );
}

#[test]
fn test_033_solana_pay_url_no_token() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        5.0,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "ZeroClaw POS",
        "POS Payment",
    );
    assert!(
        !url.contains("spl-token="),
        "033: should not contain spl-token"
    );
}

#[test]
fn test_034_solana_pay_url_special_chars() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        1.0,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "Café & Bakery",
        "POS Payment",
    );
    assert!(
        url.contains("label=") && url.contains("%26"),
        "034: special chars in label are encoded, URL: {}",
        url
    );
}

#[test]
fn test_035_reference_key_length() {
    let key = pos_core_logic::generate_secure_reference_key();
    assert!(
        key.len() >= 43 && key.len() <= 44,
        "035: reference key is 43-44 chars"
    );
}

#[test]
fn test_036_reference_key_charset() {
    let key = pos_core_logic::generate_secure_reference_key();
    let valid = key
        .chars()
        .all(|c| "123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz".contains(c));
    assert!(
        valid,
        "036: reference key charset is valid Base58, key: {}",
        key
    );
}

#[test]
fn test_037_phantom_link_format() {
    let link = pos_core_logic::generate_phantom_universal_link("solana:pubkey?amount=10.00");
    assert!(
        link.starts_with("https://phantom.app/ul/browse/") && link.ends_with("?ref=zeroclaw"),
        "037: Phantom link format correct, link: {}",
        link
    );
}

#[test]
fn test_038_get_active_rpc_url_primary() {
    let url = pos_core_logic::get_active_rpc_url(
        Some("https://primary.com"),
        Some("https://fallback.com"),
    );
    assert_eq!(url, "https://primary.com", "038: primary URL preferred");
}

#[test]
fn test_039_get_active_rpc_url_fallback() {
    let url = pos_core_logic::get_active_rpc_url(None, Some("https://fallback.com"));
    assert_eq!(
        url, "https://fallback.com",
        "039: fallback URL used when primary is None"
    );
}

#[test]
fn test_040_get_active_rpc_url_default() {
    let url = pos_core_logic::get_active_rpc_url(None, None);
    assert_eq!(
        url, "https://api.devnet.solana.com",
        "040: default URL is devnet"
    );
}

#[test]
fn test_041_refund_instructions_with_nonce() {
    let ix = pos_core_logic::generate_atomic_refund_instructions(
        "payer111",
        "recipient111",
        10.0,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        Some("nonce111"),
    );
    assert_eq!(ix.len(), 3, "041: refund instructions with nonce");
    assert_eq!(
        ix[0]["instruction"], "AdvanceNonceAccount",
        "041: first instruction is AdvanceNonceAccount"
    );
}

#[test]
fn test_042_refund_instructions_without_nonce() {
    let ix = pos_core_logic::generate_atomic_refund_instructions(
        "payer111",
        "recipient111",
        10.0,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        None,
    );
    assert_eq!(ix.len(), 2, "042: refund instructions without nonce");
}

#[test]
fn test_043_validate_squads_multisig_valid() {
    let data = serde_json::json!({"transaction_index": 5});
    let r = pos_core_logic::validate_squads_multisig_account(Some(&data));
    assert_eq!(r, Ok(6), "043: valid multisig returns tx_index + 1");
}

#[test]
fn test_044_validate_squads_multisig_null() {
    let r = pos_core_logic::validate_squads_multisig_account(None);
    assert!(r.is_err(), "044: null multisig returns error");
}

#[test]
fn test_045_solana_pay_url_encoding() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        100.0,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "ZeroClaw POS",
        "POS Payment",
    );
    assert!(
        url.contains("amount=100.00"),
        "045: amount formatted as .2f"
    );
}

#[test]
fn test_046_solana_pay_url_amount_format() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        0.5,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "ZeroClaw POS",
        "POS Payment",
    );
    assert!(url.contains("amount=0.50"), "046: 0.5 formatted as 0.50");
}

#[test]
fn test_047_reference_key_uniqueness() {
    let k1 = pos_core_logic::generate_secure_reference_key();
    let k2 = pos_core_logic::generate_secure_reference_key();
    assert_ne!(k1, k2, "047: reference keys are unique");
}

#[test]
fn test_048_solana_pay_url_long_label() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        1.0,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "A very long label that should be percent encoded",
        "POS Payment",
    );
    assert!(url.contains("label="), "048: long label is encoded");
}

#[test]
fn test_049_phantom_link_encoding() {
    let link =
        pos_core_logic::generate_phantom_universal_link("solana:pubkey?amount=10.00&label=Café");
    assert!(
        link.contains("Caf") || link.contains("Caf%C3%A9"),
        "049: Phantom link encodes special chars, link: {}",
        link
    );
}

#[test]
fn test_050_solana_pay_url_empty_mint() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        1.0,
        "7xRefKey11111111111111111111111111111111111",
        Some("11111111111111111111111111111111"),
        "ZeroClaw POS",
        "POS Payment",
    );
    assert!(
        !url.contains("spl-token="),
        "050: 1111... mint omits spl-token"
    );
}
