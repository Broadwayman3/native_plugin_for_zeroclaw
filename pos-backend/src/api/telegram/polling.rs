use super::locks::ChatLocksManager;
use super::process_single_update;
use crate::api::telegram::fsm::FsmStore;
use crate::config::AppConfig;
use crate::db;
use std::sync::Arc;
use tokio::time::{sleep, Duration};

/// Starts background Telegram long-polling update listener worker.
/// Automatically calls `deleteWebhook?drop_pending_updates=false` first to eliminate 409 Conflict.
pub fn start_poller_worker(config: Arc<AppConfig>, fsm: FsmStore, chat_locks: ChatLocksManager) {
    tokio::spawn(async move {
        tracing::info!("Telegram long-poller worker starting...");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let base_url = format!("https://api.telegram.org/bot{}", config.telegram_bot_token);

        // 1. Delete Webhook (without dropping pending updates) to ensure 409 Conflict is avoided
        let delete_url = format!("{}/deleteWebhook?drop_pending_updates=false", base_url);
        let mut del_attempts = 0;
        loop {
            del_attempts += 1;
            match client.get(&delete_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    tracing::info!("Successfully reset Telegram webhook for long polling");
                    break;
                }
                Ok(resp) => {
                    tracing::warn!(status = %resp.status(), "deleteWebhook returned non-success status");
                }
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to call deleteWebhook");
                }
            }
            if del_attempts >= 3 {
                break;
            }
            sleep(Duration::from_secs(2)).await;
        }

        // 2. Load offset from SQLite via spawn_blocking
        let db_path = config.db_path.clone();
        let mut offset =
            tokio::task::spawn_blocking(move || super::state::get_update_offset(&db_path))
                .await
                .unwrap_or(0);

        loop {
            let poll_url = format!("{}/getUpdates?offset={}&timeout=20", base_url, offset);
            match client.get(&poll_url).send().await {
                Ok(resp) => {
                    let status = resp.status();
                    if status.as_u16() == 409 {
                        tracing::error!("Telegram HTTP 409 Conflict: duplicate bot instance running. Retrying in 10s...");
                        sleep(Duration::from_secs(10)).await;
                        continue;
                    }

                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        if let Some(result) = json.get("result").and_then(|v| v.as_array()) {
                            let mut next_offset = offset;
                            let mut handles = Vec::new();

                            for update in result {
                                let update_id =
                                    match update.get("update_id").and_then(|v| v.as_i64()) {
                                        Some(id) => id,
                                        None => continue,
                                    };

                                if update_id >= next_offset {
                                    next_offset = update_id + 1;
                                }

                                // Fast pre-check: skip if update_id was already processed
                                let db_path_check = config.db_path.clone();
                                let already_processed = tokio::task::spawn_blocking(move || {
                                    if let Ok(conn) = db::get_db_connection(&db_path_check) {
                                        db::updates::is_processed(&conn, update_id).unwrap_or(false)
                                    } else {
                                        false
                                    }
                                })
                                .await
                                .unwrap_or(false);

                                if already_processed {
                                    continue;
                                }

                                let update_clone = update.clone();
                                let config_clone = config.clone();
                                let client_clone = client.clone();
                                let base_url_clone = base_url.clone();
                                let fsm_clone = fsm.clone();
                                let chat_locks_clone = chat_locks.clone();

                                let handle = tokio::spawn(async move {
                                    process_single_update(
                                        &client_clone,
                                        &base_url_clone,
                                        &config_clone,
                                        &fsm_clone,
                                        &chat_locks_clone,
                                        &update_clone,
                                        update_id,
                                    )
                                    .await;
                                });
                                handles.push(handle);
                            }

                            // Wait for all updates in batch to finish before advancing & persisting offset to SQLite
                            for handle in handles {
                                let _ = handle.await;
                            }

                            offset = next_offset;

                            // Persist max update offset to SQLite via spawn_blocking
                            let db_path_persist = config.db_path.clone();
                            let current_offset = offset;
                            let _ = tokio::task::spawn_blocking(move || {
                                super::state::set_update_offset(&db_path_persist, current_offset);
                            })
                            .await;
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "Telegram getUpdates request failed, retrying...");
                    sleep(Duration::from_secs(3)).await;
                }
            }
        }
    });
}
