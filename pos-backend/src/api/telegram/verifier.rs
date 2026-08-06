use serde_json::Value;
use std::collections::HashSet;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

use crate::config::AppConfig;
use crate::db;
use crate::domain::sanitizer::escape_telegram_markdown_v2;
use crate::domain::verification::verify_solana_transaction_with_reference;

/// Starts background Solana RPC invoice payment verification worker loop.
pub fn start_verifier_worker(config: Arc<AppConfig>) {
    tokio::spawn(async move {
        tracing::info!("Solana payment verifier worker started");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_default();

        // 1. Restore persistent last_seen_signature from SQLite system_settings table across restarts
        let mut last_seen_signature: Option<String> = match db::get_db_connection(&config.db_path) {
            Ok(conn) => db::invoices::get_system_setting(&conn, "solana_last_seen_signature")
                .unwrap_or(None),
            Err(_) => None,
        };

        let base_url = format!("https://api.telegram.org/bot{}", config.telegram_bot_token);

        loop {
            sleep(Duration::from_secs(4)).await;

            // 2. Handle expired invoices ONCE (filtering telegram_expired_notified == 0)
            handle_expired_invoices(&client, &base_url, &config.db_path).await;

            // 3. Fetch active pending invoices from SQLite first
            let pending_invoices = match db::get_db_connection(&config.db_path) {
                Ok(conn) => db::invoices::get_invoices_list(&conn, None, Some("pending"))
                    .unwrap_or_default(),
                Err(_) => continue,
            };

            if pending_invoices.is_empty() {
                continue;
            }

            // 4. Single RPC Signature Query for Merchant Wallet (limit: 10)
            let rpc_url = if config.solana_rpc_url.is_empty() {
                &config.fallback_rpc_url
            } else {
                &config.solana_rpc_url
            };

            let mut params_vec = serde_json::json!([
                config.merchant_wallet_pubkey,
                { "limit": 10 }
            ]);

            if let Some(ref until_sig) = last_seen_signature {
                if let Some(opts) = params_vec.get_mut(1).and_then(|v| v.as_object_mut()) {
                    opts.insert("until".to_string(), serde_json::json!(until_sig));
                }
            }

            let rpc_body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getSignaturesForAddress",
                "params": params_vec
            });

            let resp = match client.post(rpc_url).json(&rpc_body).send().await {
                Ok(r) => r,
                Err(_) => {
                    if let Ok(fallback_resp) = client
                        .post(&config.fallback_rpc_url)
                        .json(&rpc_body)
                        .send()
                        .await
                    {
                        fallback_resp
                    } else {
                        continue;
                    }
                }
            };

            let json: Value = match resp.json().await {
                Ok(j) => j,
                Err(_) => continue,
            };

            let sigs = match json.get("result").and_then(|r| r.as_array()) {
                Some(s) if !s.is_empty() => s,
                _ => continue,
            };

            // Reconcile ALL signatures in returned batch (limit 10)
            let mut newest_processed_sig: Option<String> = None;

            for sig_item in sigs {
                // Ignore failed transactions on-chain (err != null)
                if sig_item.get("err").map(|e| !e.is_null()).unwrap_or(false) {
                    if newest_processed_sig.is_none() {
                        if let Some(s) = sig_item.get("signature").and_then(|v| v.as_str()) {
                            newest_processed_sig = Some(s.to_string());
                        }
                    }
                    continue;
                }

                let tx_sig = match sig_item.get("signature").and_then(|v| v.as_str()) {
                    Some(s) => s,
                    None => continue,
                };

                // Throttling pause (50ms) between getTransaction calls to respect RPC rate limits
                sleep(Duration::from_millis(50)).await;

                let tx_req = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": 1,
                    "method": "getTransaction",
                    "params": [tx_sig, { "encoding": "jsonParsed", "maxSupportedTransactionVersion": 0 }]
                });

                let tx_json: Value = match client.post(rpc_url).json(&tx_req).send().await {
                    Ok(r) => match r.json().await {
                        Ok(j) => j,
                        Err(_) => break, // Network JSON parse error: stop batch, do NOT advance last_seen_signature past here!
                    },
                    Err(_) => break, // RPC network error: stop batch, do NOT advance last_seen_signature past here!
                };

                let result = tx_json.get("result").unwrap_or(&Value::Null);
                if result.is_null() {
                    continue;
                }

                // Track newest successfully fetched signature
                if newest_processed_sig.is_none() {
                    newest_processed_sig = Some(tx_sig.to_string());
                }

                // O(1) Pre-filtering: Extract accountKeys into HashSet to skip non-merchant transactions
                let message_obj = result
                    .get("transaction")
                    .and_then(|t| t.get("message"))
                    .and_then(|m| m.as_object());

                let mut account_set = HashSet::new();
                if let Some(msg) = message_obj {
                    if let Some(keys) = msg
                        .get("accountKeys")
                        .or_else(|| msg.get("staticAccountKeys"))
                        .and_then(|k| k.as_array())
                    {
                        for k in keys {
                            let pk = if k.is_string() {
                                k.as_str().unwrap_or("")
                            } else {
                                k.get("pubkey").and_then(|v| v.as_str()).unwrap_or("")
                            };
                            if !pk.is_empty() {
                                account_set.insert(pk);
                            }
                        }
                    }
                }

                if !account_set.contains(config.merchant_wallet_pubkey.as_str()) {
                    continue; // Skip transaction if merchant wallet is not involved
                }

                for inv in &pending_invoices {
                    let usdc_atomic = (inv.usdc_amount * 1_000_000.0) as i64;
                    let sol_atomic = (inv.fiat_amount * 1_000_000_000.0) as i64;
                    let target_atomic = if inv.fiat_currency == "SOL" {
                        sol_atomic
                    } else {
                        usdc_atomic
                    };

                    // Primary check: Strict Reference Key Matching
                    let mut verification = verify_solana_transaction_with_reference(
                        result,
                        &config.merchant_wallet_pubkey,
                        target_atomic,
                        &config.usdc_mint_address,
                        Some(&inv.reference_pubkey),
                    );

                    // Native SOL Fallback: Standard wallet transfers omit reference keys on System Program transfers
                    if !verification
                        .get("is_valid")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                        && inv.fiat_currency == "SOL"
                    {
                        verification = verify_solana_transaction_with_reference(
                            result,
                            &config.merchant_wallet_pubkey,
                            target_atomic,
                            &config.usdc_mint_address,
                            None,
                        );
                    }

                    if verification
                        .get("is_valid")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        // Mark invoice as PAID in SQLite
                        if let Ok(conn) = db::get_db_connection(&config.db_path) {
                            let _ = db::invoices::update_invoice_status(
                                &conn,
                                &inv.id,
                                "paid",
                                Some(tx_sig),
                            );
                        }

                        // Send Telegram Notification (with fallback to sendMessage if msg_id is None)
                        if let Some(chat_id) = inv.telegram_chat_id {
                            let esc_inv = escape_telegram_markdown_v2(&inv.id);
                            let esc_amt =
                                escape_telegram_markdown_v2(&format!("{:.2}", inv.usdc_amount));

                            // Base58 Solana signature contains only [1-9A-HJ-NP-Za-km-z].
                            // Placing tx_sig raw inside `code` block satisfies Telegram MarkdownV2 spec without raw \ backslashes!
                            let new_caption = format!("✅ *Invoice \\#{} PAID ✓*\n─────────────────\n• Amount: *{} USDC*\n• Tx: `{}`\n\nThank you for your payment\\!", esc_inv, esc_amt, tx_sig);

                            let mut notified = false;
                            if let Some(msg_id) = inv.telegram_msg_id {
                                let edit_payload = serde_json::json!({
                                    "chat_id": chat_id,
                                    "message_id": msg_id,
                                    "caption": new_caption,
                                    "parse_mode": "MarkdownV2",
                                    "reply_markup": { "inline_keyboard": [] }
                                });
                                if let Ok(resp) = client
                                    .post(format!("{}/editMessageCaption", base_url))
                                    .json(&edit_payload)
                                    .send()
                                    .await
                                {
                                    if resp.status().is_success() {
                                        notified = true;
                                    }
                                }
                            }

                            // Fallback notification via sendMessage if editMessageCaption failed or msg_id was None
                            if !notified {
                                let msg_payload = serde_json::json!({
                                    "chat_id": chat_id,
                                    "text": new_caption,
                                    "parse_mode": "MarkdownV2"
                                });
                                let _ = client
                                    .post(format!("{}/sendMessage", base_url))
                                    .json(&msg_payload)
                                    .send()
                                    .await;
                            }
                        }
                    }
                }
            }

            // Advance and persist last_seen_signature to SQLite system_settings across server restarts!
            if let Some(new_sig) = newest_processed_sig {
                last_seen_signature = Some(new_sig.clone());
                if let Ok(conn) = db::get_db_connection(&config.db_path) {
                    let _ = db::invoices::set_system_setting(
                        &conn,
                        "solana_last_seen_signature",
                        &new_sig,
                    );
                }
            }
        }
    });
}

async fn handle_expired_invoices(client: &reqwest::Client, base_url: &str, db_path: &str) {
    if let Ok(conn) = db::get_db_connection(db_path) {
        let expired_invoices =
            db::invoices::get_invoices_list(&conn, None, Some("expired")).unwrap_or_default();
        for inv in expired_invoices {
            // Only notify ONCE per expired invoice
            if inv.telegram_expired_notified.unwrap_or(0) == 1 {
                continue;
            }

            if let Some(chat_id) = inv.telegram_chat_id {
                let esc_inv = escape_telegram_markdown_v2(&inv.id);
                let new_caption = format!("❌ *Invoice \\#{} EXPIRED*\n─────────────────\nPayment window timed out \\(15 mins\\)\\.", esc_inv);

                if let Some(msg_id) = inv.telegram_msg_id {
                    let edit_payload = serde_json::json!({
                        "chat_id": chat_id,
                        "message_id": msg_id,
                        "caption": new_caption,
                        "parse_mode": "MarkdownV2",
                        "reply_markup": { "inline_keyboard": [] }
                    });
                    let _ = client
                        .post(format!("{}/editMessageCaption", base_url))
                        .json(&edit_payload)
                        .send()
                        .await;
                } else {
                    let msg_payload = serde_json::json!({
                        "chat_id": chat_id,
                        "text": new_caption,
                        "parse_mode": "MarkdownV2"
                    });
                    let _ = client
                        .post(format!("{}/sendMessage", base_url))
                        .json(&msg_payload)
                        .send()
                        .await;
                }

                // Mark expired notification sent in SQLite
                let _ = db::invoices::mark_invoice_expired_notified(&conn, &inv.id);
            }
        }
    }
}
