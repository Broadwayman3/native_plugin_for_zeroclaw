#[test]
fn test_091_verify_valid_transaction() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        1000000,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    );
    assert!(
        !r["is_valid"].as_bool().unwrap_or(true),
        "091: expected is_valid=false"
    );
}

#[test]
fn test_092_verify_invalid_payload() {
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &serde_json::Value::Null,
        "merchant",
        100,
        "mint",
    );
    assert_eq!(r["is_valid"], false, "092: expected is_valid=false");
}

#[test]
fn test_093_verify_no_meta() {
    let tx = serde_json::json!({"transaction": {"message": {}}});
    let r =
        pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant", 100, "mint");
    assert_eq!(r["is_valid"], false, "093: expected is_valid=false");
}

#[test]
fn test_094_verify_reverted_tx() {
    let tx = serde_json::json!({
        "meta": {"err": "InstructionError"},
        "transaction": {"message": {"accountKeys": [], "instructions": []}}
    });
    let r =
        pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant", 100, "mint");
    assert_eq!(r["is_valid"], false, "094: expected is_valid=false");
}

#[test]
fn test_095_verify_no_transfer() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [], "instructions": [{"parsed": {"type": "unknown"}}]}}
    });
    let r =
        pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant", 100, "mint");
    assert_eq!(r["is_valid"], false, "095: expected is_valid=false");
}

#[test]
fn test_096_verify_inner_instruction() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [],
                 "innerInstructions": [{"instructions": [
                     {"parsed": {"type": "transfer", "info": {"destination": "merchant_ata", "amount": "5000000"}}}
                 ]}]},
        "transaction": {"message": {"accountKeys": [], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        5000000,
        "mint",
    );
    assert_eq!(r["is_valid"], true, "096: result: {}", r);
}

#[test]
fn test_097_verify_balance_delta() {
    let tx = serde_json::json!({
        "meta": {"err": null,
                 "postTokenBalances": [{"accountIndex": 1, "mint": "mint", "uiTokenAmount": {"amount": "2000000"}}],
                 "preTokenBalances": [{"accountIndex": 1, "mint": "mint", "uiTokenAmount": {"amount": "1000000"}}],
                 "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [{"pubkey": "other"}, {"pubkey": "merchant_ata"}], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        1000000,
        "mint",
    );
    assert!(
        r["is_valid"] == true && r["verification_method"] == "balance_delta",
        "097: result: {}",
        r
    );
}

#[test]
fn test_098_verify_wrong_mint() {
    let tx = serde_json::json!({
        "meta": {"err": null,
                 "postTokenBalances": [{"accountIndex": 1, "mint": "wrong_mint", "uiTokenAmount": {"amount": "2000000"}}],
                 "preTokenBalances": [{"accountIndex": 1, "mint": "wrong_mint", "uiTokenAmount": {"amount": "1000000"}}],
                 "innerInstructions": []},
        "transaction": {"message": {"accountKeys": ["other", "merchant_ata"], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        1000000,
        "correct_mint",
    );
    assert_eq!(
        r["is_valid"], false,
        "098: expected is_valid=false for wrong mint"
    );
}

#[test]
fn test_099_verify_missing_account() {
    let tx = serde_json::json!({
        "meta": {"err": null,
                 "postTokenBalances": [{"accountIndex": 5, "mint": "mint", "uiTokenAmount": {"amount": "2000000"}}],
                 "preTokenBalances": [{"accountIndex": 5, "mint": "mint", "uiTokenAmount": {"amount": "1000000"}}],
                 "innerInstructions": []},
        "transaction": {"message": {"accountKeys": ["only_one"], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        1000000,
        "mint",
    );
    assert_eq!(r["is_valid"], false, "099: expected is_valid=false");
}

#[test]
fn test_100_verify_empty_instructions() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [], "instructions": []}}
    });
    let r =
        pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant", 100, "mint");
    assert_eq!(r["is_valid"], false, "100: expected is_valid=false");
}

#[test]
fn test_101_verify_transfer_checked_type() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {
            "message": {
                "accountKeys": [
                    {"pubkey": "payer"},
                    {"pubkey": "merchant_ata"}
                ],
                "instructions": [
                    {
                        "parsed": {
                            "type": "transferChecked",
                            "info": {
                                "destination": "merchant_ata",
                                "amount": "1000000",
                                "source": "payer"
                            }
                        }
                    }
                ]
            }
        }
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        1000000,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    );
    assert!(
        r["is_valid"] == true && r["verification_method"] == "top_level_instruction",
        "101: valid={}, method={}",
        r["is_valid"],
        r["verification_method"]
    );
}

// --- Boundary tests (390-394) ---

#[test]
fn test_390_verify_amount_zero() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [{"pubkey": "merchant_ata"}], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        0,
        "mint",
    );
    // With empty balances, delta=0 for merchant_ata. 0>=0 should be valid.
    assert_eq!(
        r["is_valid"], true,
        "390: expected valid for zero amount, got: {}",
        r
    );
}

#[test]
fn test_391_verify_invalid_amount_string() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [{"accountIndex": 0, "mint": "mint", "uiTokenAmount": {"amount": "not_a_number"}}], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [{"pubkey": "merchant_ata"}], "instructions": [
            {"parsed": {"type": "transfer", "info": {"destination": "merchant_ata", "amount": "not_a_number"}}}
        ]}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        100,
        "mint",
    );
    // Unparseable amounts are treated as 0, so 0 < 100 → invalid
    assert_eq!(r["is_valid"], false, "391: result: {}", r);
}

#[test]
fn test_392_verify_empty_pre_post_token_balances() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [{"pubkey": "merchant_ata"}], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        1000000,
        "mint",
    );
    // Empty balances → delta = 0 → 0 < 1000000 → invalid
    assert_eq!(r["is_valid"], false, "392: result: {}", r);
}

#[test]
fn test_393_verify_multiple_transfers_one_match() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [{"pubkey": "payer"}, {"pubkey": "merchant_ata"}], "instructions": [
            {"parsed": {"type": "transfer", "info": {"destination": "wrong_dest", "amount": "5000000"}}},
            {"parsed": {"type": "transfer", "info": {"destination": "merchant_ata", "amount": "1000000"}}}
        ]}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        1000000,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    );
    assert!(
        r["is_valid"] == true && r["verification_method"] == "top_level_instruction",
        "393: result: {}",
        r
    );
}

#[test]
fn test_394_verify_nested_instructions() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [{"pubkey": "payer"}, {"pubkey": "merchant_ata"}], "instructions": [
            {"instructions": [
                {"parsed": {"type": "transfer", "info": {"destination": "merchant_ata", "amount": "2000000"}}}
            ]}
        ]}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(
        &tx,
        "merchant_ata",
        2000000,
        "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
    );
    assert!(
        r["is_valid"] == true && r["verification_method"] == "top_level_instruction",
        "394: result: {}",
        r
    );
}

#[test]
fn test_329_verify_token2022_transfer_fee_net_amount() {
    let tx = serde_json::json!({
        "meta": {
            "err": null,
            "preTokenBalances": [{"accountIndex": 1, "mint": "mint2022", "uiTokenAmount": {"amount": "0"}}],
            "postTokenBalances": [{"accountIndex": 1, "mint": "mint2022", "uiTokenAmount": {"amount": "999000"}}],
            "innerInstructions": []
        },
        "transaction": {"message": {"accountKeys": [{"pubkey": "payer"}, {"pubkey": "merchant_ata"}]}}
    });
    // Expected gross: 1_000_000 (1 USDC). Fee: 10 bp = 1000 atomic units. Net: 999_000 atomic units.
    let r = pos_backend::domain::verification::verify_solana_transaction_with_fee_bps(
        &tx,
        "merchant_ata",
        1000000,
        "mint2022",
        10,
        1000000,
    );
    assert_eq!(
        r["is_valid"], true,
        "329: token2022 net amount verification should succeed, got: {}",
        r
    );
}
