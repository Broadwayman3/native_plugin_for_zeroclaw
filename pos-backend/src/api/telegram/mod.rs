pub mod handlers;
pub mod orders;
pub mod state;
pub mod verifier;

use crate::config::AppConfig;
use std::sync::Arc;

/// Starts background Telegram long-poller and Solana RPC payment verifier services.
pub fn start_telegram_services(config: Arc<AppConfig>) {
    let token = config.telegram_bot_token.clone();
    if token.is_empty() || token.contains("123456789:ABC") {
        tracing::warn!("Telegram Bot token not set or placeholder. Skipping Telegram services.");
        return;
    }

    // 1. Start Solana RPC payment verification background worker
    verifier::start_verifier_worker(config.clone());

    // 2. Start Telegram long-polling update listener background worker
    let poller_config = config.clone();
    tokio::spawn(async move {
        tracing::info!("Telegram long-poller worker started");
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        let base_url = format!(
            "https://api.telegram.org/bot{}",
            poller_config.telegram_bot_token
        );
        let mut offset = state::get_update_offset(&poller_config.db_path);

        loop {
            let poll_url = format!("{}/getUpdates?offset={}&timeout=10", base_url, offset);
            match client.get(&poll_url).send().await {
                Ok(resp) => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(result) = json.get("result").and_then(|v| v.as_array()) {
                            for update in result {
                                if let Some(update_id) =
                                    update.get("update_id").and_then(|v| v.as_i64())
                                {
                                    offset = update_id + 1;
                                    state::set_update_offset(offset);
                                }

                                // Process update
                                if let Some(msg) = update.get("message") {
                                    let chat_id = msg
                                        .get("chat")
                                        .and_then(|c| c.get("id"))
                                        .and_then(|v| v.as_i64());
                                    let text = msg.get("text").and_then(|v| v.as_str());
                                    let user_id = msg
                                        .get("from")
                                        .and_then(|f| f.get("id"))
                                        .and_then(|v| v.as_i64())
                                        .unwrap_or(0);

                                    if let (Some(chat_id), Some(text)) = (chat_id, text) {
                                        handlers::handle_user_message(
                                            &client,
                                            &base_url,
                                            &poller_config,
                                            chat_id,
                                            user_id,
                                            text,
                                        )
                                        .await;
                                    }
                                }

                                if let Some(cb) = update.get("callback_query") {
                                    let cb_id = cb.get("id").and_then(|v| v.as_str()).unwrap_or("");
                                    let data =
                                        cb.get("data").and_then(|v| v.as_str()).unwrap_or("");
                                    let msg = cb.get("message");
                                    let chat_id = msg
                                        .and_then(|m| m.get("chat"))
                                        .and_then(|c| c.get("id"))
                                        .and_then(|v| v.as_i64());

                                    if let Some(chat_id) = chat_id {
                                        handlers::handle_callback_query(
                                            &client,
                                            &base_url,
                                            &poller_config,
                                            chat_id,
                                            cb_id,
                                            data,
                                        )
                                        .await;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "Telegram getUpdates request failed, retrying...");
                    tokio::time::sleep(std::time::Duration::from_secs(3)).await;
                }
            }
        }
    });
}
