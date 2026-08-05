/// Triple Payment Verification for Solana transactions.
use serde_json::Value;

/// Extracts token balance deltas from transaction metadata.
fn extract_token_balance_deltas(
    meta: &Value,
    expected_mint: &str,
) -> std::collections::HashMap<i64, i64> {
    let mut pre_balances = std::collections::HashMap::new();
    let mut post_balances = std::collections::HashMap::new();

    if let Some(pre_token) = meta.get("preTokenBalances").and_then(|v| v.as_array()) {
        for b in pre_token {
            if let Some(mint) = b.get("mint").and_then(|v| v.as_str()) {
                if mint == expected_mint {
                    let idx = b.get("accountIndex").and_then(|v| v.as_i64()).unwrap_or(0);
                    let amount = b
                        .get("uiTokenAmount")
                        .and_then(|t| t.get("amount"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i64>().ok())
                        .unwrap_or(0);
                    pre_balances.insert(idx, amount);
                }
            }
        }
    }

    if let Some(post_token) = meta.get("postTokenBalances").and_then(|v| v.as_array()) {
        for b in post_token {
            if let Some(mint) = b.get("mint").and_then(|v| v.as_str()) {
                if mint == expected_mint {
                    let idx = b.get("accountIndex").and_then(|v| v.as_i64()).unwrap_or(0);
                    let amount = b
                        .get("uiTokenAmount")
                        .and_then(|t| t.get("amount"))
                        .and_then(|v| v.as_str())
                        .and_then(|s| s.parse::<i64>().ok())
                        .unwrap_or(0);
                    post_balances.insert(idx, amount);
                }
            }
        }
    }

    let mut deltas = std::collections::HashMap::new();
    for idx in pre_balances.keys().chain(post_balances.keys()) {
        let pre = pre_balances.get(idx).unwrap_or(&0);
        let post = post_balances.get(idx).unwrap_or(&0);
        deltas.insert(*idx, post - pre);
    }
    deltas
}

/// Recursively inspects instructions for token transfers.
fn inspect_instructions_for_transfer(
    instructions: &[Value],
    expected_merchant_ata: &str,
    expected_usdc_atomic: i64,
) -> Option<i64> {
    for inst in instructions {
        if let Some(parsed) = inst.get("parsed") {
            if let Some(type_) = parsed.get("type").and_then(|v| v.as_str()) {
                if type_ == "transfer" || type_ == "transferChecked" {
                    if let Some(info) = parsed.get("info") {
                        if let Some(dest) = info.get("destination").and_then(|v| v.as_str()) {
                            if dest == expected_merchant_ata {
                                let amount = info
                                    .get("amount")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| s.parse::<i64>().ok())
                                    .or_else(|| {
                                        info.get("tokenAmount")
                                            .and_then(|t| t.get("amount"))
                                            .and_then(|v| v.as_str())
                                            .and_then(|s| s.parse::<i64>().ok())
                                    });
                                if let Some(amt) = amount {
                                    if amt >= expected_usdc_atomic {
                                        return Some(amt);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Check nested instructions
        if let Some(nested) = inst.get("instructions").and_then(|v| v.as_array()) {
            if let Some(result) = inspect_instructions_for_transfer(
                nested,
                expected_merchant_ata,
                expected_usdc_atomic,
            ) {
                return Some(result);
            }
        }
    }
    None
}

/// Triple Payment Protection: Reverted Tx Guard, Balance Delta, Recursive Instruction Inspection.
pub fn verify_solana_transaction(
    tx_json: &Value,
    expected_merchant_ata: &str,
    expected_usdc_atomic: i64,
    expected_mint: &str,
) -> Value {
    verify_solana_transaction_with_fee_bps(
        tx_json,
        expected_merchant_ata,
        expected_usdc_atomic,
        expected_mint,
        0,
        0,
    )
}

/// Triple Payment Protection with Token-2022 Transfer Fee Net Amount Reconciliation.
pub fn verify_solana_transaction_with_fee_bps(
    tx_json: &Value,
    expected_merchant_ata: &str,
    expected_usdc_atomic: i64,
    expected_mint: &str,
    fee_basis_points: u16,
    max_fee_units: u64,
) -> Value {
    // Calculate expected net atomic amount after Token-2022 transfer fee deduction
    let expected_net_atomic = if expected_usdc_atomic > 0 {
        let fee_units = pos_core_logic::token2022::calculate_token2022_fee(
            expected_usdc_atomic as u64,
            fee_basis_points,
            if max_fee_units > 0 {
                max_fee_units
            } else {
                expected_usdc_atomic as u64
            },
        ) as i64;
        expected_usdc_atomic - fee_units
    } else {
        expected_usdc_atomic
    };

    // Invalid payload
    if tx_json.is_null() || !tx_json.is_object() {
        return serde_json::json!({
            "is_valid": false,
            "error": "Invalid transaction JSON payload"
        });
    }

    // Check meta.err (reverted transaction)
    let meta = match tx_json.get("meta") {
        Some(m) if m.is_object() => m,
        _ => {
            return serde_json::json!({
                "is_valid": false,
                "error": "Missing or invalid transaction metadata"
            });
        }
    };

    if meta.get("err").is_some() && !meta.get("err").unwrap().is_null() {
        return serde_json::json!({
            "is_valid": false,
            "error": "Transaction failed or reverted on-chain"
        });
    }

    // Balance delta verification
    let deltas = extract_token_balance_deltas(meta, expected_mint);
    let transaction = tx_json.get("transaction").and_then(|t| t.as_object());
    let message = transaction
        .and_then(|t| t.get("message"))
        .and_then(|m| m.as_object());

    if let Some(msg) = message {
        let account_keys = msg
            .get("accountKeys")
            .or_else(|| msg.get("staticAccountKeys"))
            .and_then(|k| k.as_array())
            .map(|a| a.as_slice())
            .unwrap_or(&[]);

        // Find merchant ATA index
        for (i, key) in account_keys.iter().enumerate() {
            let pubkey = key.get("pubkey").and_then(|v| v.as_str()).unwrap_or("");
            if pubkey == expected_merchant_ata {
                let delta = deltas.get(&(i as i64)).copied().unwrap_or(0);
                if delta >= expected_net_atomic {
                    return serde_json::json!({
                        "is_valid": true,
                        "paid_atomic": delta,
                        "verification_method": "balance_delta"
                    });
                }
                break;
            }
        }

        // Top-level instruction inspection
        if let Some(instructions) = msg.get("instructions").and_then(|v| v.as_array()) {
            if let Some(paid) = inspect_instructions_for_transfer(
                instructions,
                expected_merchant_ata,
                expected_net_atomic,
            ) {
                return serde_json::json!({
                    "is_valid": true,
                    "paid_atomic": paid,
                    "verification_method": "top_level_instruction"
                });
            }
        }
    }

    // Inner instruction inspection
    if let Some(inner_instructions) = meta.get("innerInstructions").and_then(|v| v.as_array()) {
        for group in inner_instructions {
            if let Some(instructions) = group.get("instructions").and_then(|v| v.as_array()) {
                if let Some(paid) = inspect_instructions_for_transfer(
                    instructions,
                    expected_merchant_ata,
                    expected_net_atomic,
                ) {
                    return serde_json::json!({
                        "is_valid": true,
                        "paid_atomic": paid,
                        "verification_method": "inner_instruction"
                    });
                }
            }
        }
    }

    serde_json::json!({
        "is_valid": false,
        "error": "No valid token transfer or positive balance delta found for Merchant ATA"
    })
}
