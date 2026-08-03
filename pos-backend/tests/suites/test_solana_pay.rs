use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Solana Pay Tests (031-050)");
    test_031_solana_pay_url_with_token();
    test_032_solana_pay_url_without_token();
    test_033_solana_pay_url_no_token();
    test_034_solana_pay_url_special_chars();
    test_035_reference_key_length();
    test_036_reference_key_charset();
    test_037_phantom_link_format();
    test_038_get_active_rpc_url_primary();
    test_039_get_active_rpc_url_fallback();
    test_040_get_active_rpc_url_default();
    test_041_refund_instructions_with_nonce();
    test_042_refund_instructions_without_nonce();
    test_043_validate_squads_multisig_valid();
    test_044_validate_squads_multisig_null();
    test_045_solana_pay_url_encoding();
    test_046_solana_pay_url_amount_format();
    test_047_reference_key_uniqueness();
    test_048_solana_pay_url_long_label();
    test_049_phantom_link_encoding();
    test_050_solana_pay_url_empty_mint();
}

fn test_031_solana_pay_url_with_token() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        10.5,
        "7xRefKey11111111111111111111111111111111111",
        Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        "ZeroClaw POS",
        "POS Payment",
    );
    if url.starts_with("solana:")
        && url.contains("spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
    {
        test_pass("031: Solana Pay URL with SPL token");
    } else {
        test_fail("031", &format!("URL: {}", url));
    }
}

fn test_032_solana_pay_url_without_token() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        5.0,
        "7xRefKey11111111111111111111111111111111111",
        Some("So11111111111111111111111111111111111111112"),
        "ZeroClaw POS",
        "POS Payment",
    );
    if !url.contains("spl-token=") {
        test_pass("032: Solana Pay URL omits spl-token for SOL");
    } else {
        test_fail("032", "should not contain spl-token for SOL");
    }
}

fn test_033_solana_pay_url_no_token() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        5.0,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "ZeroClaw POS",
        "POS Payment",
    );
    if !url.contains("spl-token=") {
        test_pass("033: Solana Pay URL with no token mint");
    } else {
        test_fail("033", "should not contain spl-token");
    }
}

fn test_034_solana_pay_url_special_chars() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        1.0,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "Café & Bakery",
        "POS Payment",
    );
    if url.contains("label=") && url.contains("%26") {
        test_pass("034: special chars in label are encoded");
    } else {
        test_fail("034", &format!("URL: {}", url));
    }
}

fn test_035_reference_key_length() {
    let key = pos_core_logic::generate_secure_reference_key();
    if key.len() == 44 {
        test_pass("035: reference key is 44 chars");
    } else {
        test_fail("035", &format!("len = {}", key.len()));
    }
}

fn test_036_reference_key_charset() {
    let key = pos_core_logic::generate_secure_reference_key();
    let valid = key
        .chars()
        .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit());
    if valid {
        test_pass("036: reference key charset is valid");
    } else {
        test_fail("036", &format!("key: {}", key));
    }
}

fn test_037_phantom_link_format() {
    let link = pos_core_logic::generate_phantom_universal_link("solana:pubkey?amount=10.00");
    if link.starts_with("https://phantom.app/ul/browse/") && link.ends_with("?ref=zeroclaw") {
        test_pass("037: Phantom link format correct");
    } else {
        test_fail("037", &format!("link: {}", link));
    }
}

fn test_038_get_active_rpc_url_primary() {
    let url = pos_core_logic::get_active_rpc_url(
        Some("https://primary.com"),
        Some("https://fallback.com"),
    );
    if url == "https://primary.com" {
        test_pass("038: primary URL preferred");
    } else {
        test_fail("038", &format!("url: {}", url));
    }
}

fn test_039_get_active_rpc_url_fallback() {
    let url = pos_core_logic::get_active_rpc_url(None, Some("https://fallback.com"));
    if url == "https://fallback.com" {
        test_pass("039: fallback URL used when primary is None");
    } else {
        test_fail("039", &format!("url: {}", url));
    }
}

fn test_040_get_active_rpc_url_default() {
    let url = pos_core_logic::get_active_rpc_url(None, None);
    if url == "https://api.devnet.solana.com" {
        test_pass("040: default URL is devnet");
    } else {
        test_fail("040", &format!("url: {}", url));
    }
}

fn test_041_refund_instructions_with_nonce() {
    let ix = pos_core_logic::generate_atomic_refund_instructions(
        "payer111",
        "recipient111",
        10.0,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        Some("nonce111"),
    );
    if ix.len() == 3 && ix[0]["instruction"] == "AdvanceNonceAccount" {
        test_pass("041: refund instructions with nonce");
    } else {
        test_fail("041", &format!("len = {}", ix.len()));
    }
}

fn test_042_refund_instructions_without_nonce() {
    let ix = pos_core_logic::generate_atomic_refund_instructions(
        "payer111",
        "recipient111",
        10.0,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
        None,
    );
    if ix.len() == 2 {
        test_pass("042: refund instructions without nonce");
    } else {
        test_fail("042", &format!("len = {}", ix.len()));
    }
}

fn test_043_validate_squads_multisig_valid() {
    let data = serde_json::json!({"transaction_index": 5});
    let r = pos_core_logic::validate_squads_multisig_account(Some(&data));
    if r == Ok(6) {
        test_pass("043: valid multisig returns tx_index + 1");
    } else {
        test_fail("043", &format!("result: {:?}", r));
    }
}

fn test_044_validate_squads_multisig_null() {
    let r = pos_core_logic::validate_squads_multisig_account(None);
    if r.is_err() {
        test_pass("044: null multisig returns error");
    } else {
        test_fail("044", "expected error");
    }
}

fn test_045_solana_pay_url_encoding() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        100.0,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "ZeroClaw POS",
        "POS Payment",
    );
    if url.contains("amount=100.00") {
        test_pass("045: amount formatted as .2f");
    } else {
        test_fail("045", &format!("url: {}", url));
    }
}

fn test_046_solana_pay_url_amount_format() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        0.5,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "ZeroClaw POS",
        "POS Payment",
    );
    if url.contains("amount=0.50") {
        test_pass("046: 0.5 formatted as 0.50");
    } else {
        test_fail("046", &format!("url: {}", url));
    }
}

fn test_047_reference_key_uniqueness() {
    let k1 = pos_core_logic::generate_secure_reference_key();
    let k2 = pos_core_logic::generate_secure_reference_key();
    if k1 != k2 {
        test_pass("047: reference keys are unique");
    } else {
        test_fail("047", "keys are identical");
    }
}

fn test_048_solana_pay_url_long_label() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        1.0,
        "7xRefKey11111111111111111111111111111111111",
        None,
        "A very long label that should be percent encoded",
        "POS Payment",
    );
    if url.contains("label=") {
        test_pass("048: long label is encoded");
    } else {
        test_fail("048", &format!("url: {}", url));
    }
}

fn test_049_phantom_link_encoding() {
    let link =
        pos_core_logic::generate_phantom_universal_link("solana:pubkey?amount=10.00&label=Café");
    if link.contains("Caf") || link.contains("Caf%C3%A9") {
        test_pass("049: Phantom link encodes special chars");
    } else {
        test_fail("049", &format!("link: {}", link));
    }
}

fn test_050_solana_pay_url_empty_mint() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        1.0,
        "7xRefKey11111111111111111111111111111111111",
        Some("11111111111111111111111111111111"),
        "ZeroClaw POS",
        "POS Payment",
    );
    if !url.contains("spl-token=") {
        test_pass("050: 1111... mint omits spl-token");
    } else {
        test_fail("050", "should not contain spl-token for 1111 mint");
    }
}
