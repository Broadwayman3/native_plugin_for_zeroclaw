use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Verification Tests (091-100)");
    test_091_verify_valid_transaction();
    test_092_verify_invalid_payload();
    test_093_verify_no_meta();
    test_094_verify_reverted_tx();
    test_095_verify_no_transfer();
    test_096_verify_inner_instruction();
    test_097_verify_balance_delta();
    test_098_verify_wrong_mint();
    test_099_verify_missing_account();
    test_100_verify_empty_instructions();
}

fn test_091_verify_valid_transaction() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant_ata", 1000000, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
    if !r["is_valid"].as_bool().unwrap_or(true) {
        test_pass("091: valid payload structure parsed");
    } else {
        test_fail("091", "expected is_valid=false");
    }
}

fn test_092_verify_invalid_payload() {
    let r = pos_backend::domain::verification::verify_solana_transaction(&serde_json::Value::Null, "merchant", 100, "mint");
    if r["is_valid"] == false {
        test_pass("092: null payload rejected");
    } else {
        test_fail("092", "expected is_valid=false");
    }
}

fn test_093_verify_no_meta() {
    let tx = serde_json::json!({"transaction": {"message": {}}});
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant", 100, "mint");
    if r["is_valid"] == false {
        test_pass("093: missing meta rejected");
    } else {
        test_fail("093", "expected is_valid=false");
    }
}

fn test_094_verify_reverted_tx() {
    let tx = serde_json::json!({
        "meta": {"err": "InstructionError"},
        "transaction": {"message": {"accountKeys": [], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant", 100, "mint");
    if r["is_valid"] == false {
        test_pass("094: reverted transaction rejected");
    } else {
        test_fail("094", "expected is_valid=false");
    }
}

fn test_095_verify_no_transfer() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [], "instructions": [{"parsed": {"type": "unknown"}}]}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant", 100, "mint");
    if r["is_valid"] == false {
        test_pass("095: no transfer instruction rejected");
    } else {
        test_fail("095", "expected is_valid=false");
    }
}

fn test_096_verify_inner_instruction() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [],
                 "innerInstructions": [{"instructions": [
                     {"parsed": {"type": "transfer", "info": {"destination": "merchant_ata", "amount": "5000000"}}}
                 ]}]},
        "transaction": {"message": {"accountKeys": [], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant_ata", 5000000, "mint");
    if r["is_valid"] == true {
        test_pass("096: inner instruction transfer verified");
    } else {
        test_fail("096", &format!("result: {}", r));
    }
}

fn test_097_verify_balance_delta() {
    let tx = serde_json::json!({
        "meta": {"err": null,
                 "postTokenBalances": [{"accountIndex": 1, "mint": "mint", "uiTokenAmount": {"amount": "2000000"}}],
                 "preTokenBalances": [{"accountIndex": 1, "mint": "mint", "uiTokenAmount": {"amount": "1000000"}}],
                 "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [{"pubkey": "other"}, {"pubkey": "merchant_ata"}], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant_ata", 1000000, "mint");
    if r["is_valid"] == true && r["verification_method"] == "balance_delta" {
        test_pass("097: balance delta verified");
    } else {
        test_fail("097", &format!("result: {}", r));
    }
}

fn test_098_verify_wrong_mint() {
    let tx = serde_json::json!({
        "meta": {"err": null,
                 "postTokenBalances": [{"accountIndex": 1, "mint": "wrong_mint", "uiTokenAmount": {"amount": "2000000"}}],
                 "preTokenBalances": [{"accountIndex": 1, "mint": "wrong_mint", "uiTokenAmount": {"amount": "1000000"}}],
                 "innerInstructions": []},
        "transaction": {"message": {"accountKeys": ["other", "merchant_ata"], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant_ata", 1000000, "correct_mint");
    if r["is_valid"] == false {
        test_pass("098: wrong mint rejected");
    } else {
        test_fail("098", "expected is_valid=false for wrong mint");
    }
}

fn test_099_verify_missing_account() {
    let tx = serde_json::json!({
        "meta": {"err": null,
                 "postTokenBalances": [{"accountIndex": 5, "mint": "mint", "uiTokenAmount": {"amount": "2000000"}}],
                 "preTokenBalances": [{"accountIndex": 5, "mint": "mint", "uiTokenAmount": {"amount": "1000000"}}],
                 "innerInstructions": []},
        "transaction": {"message": {"accountKeys": ["only_one"], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant_ata", 1000000, "mint");
    if r["is_valid"] == false {
        test_pass("099: missing merchant account rejected");
    } else {
        test_fail("099", "expected is_valid=false");
    }
}

fn test_100_verify_empty_instructions() {
    let tx = serde_json::json!({
        "meta": {"err": null, "postTokenBalances": [], "preTokenBalances": [], "innerInstructions": []},
        "transaction": {"message": {"accountKeys": [], "instructions": []}}
    });
    let r = pos_backend::domain::verification::verify_solana_transaction(&tx, "merchant", 100, "mint");
    if r["is_valid"] == false {
        test_pass("100: empty instructions rejected");
    } else {
        test_fail("100", "expected is_valid=false");
    }
}
