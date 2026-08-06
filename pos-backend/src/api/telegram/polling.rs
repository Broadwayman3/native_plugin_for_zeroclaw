use super::locks::ChatLocksManager;
use super::process_single_update;
use crate::api::telegram::fsm::FsmStore;
use crate::config::AppConfig;
use crate::db;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

/// Starts background Telegram long-polling update listener worker.
/// Automatically calls `deleteWebhook?drop_pending_updates=false` first to eliminate 409 Conflict.
pub fn start_poller_worker(
    config: Arc<AppConfig>,
    fsm: FsmStore,
    chat_locks: ChatLocksManager,
    in_flight: super::locks::InFlightTracker,
    db_pool: Option<deadpool_sqlite::Pool>,
    cancel_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
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
            if cancel_token.is_cancelled() {
                tracing::info!("Polling worker cancelled during webhook reset. Exiting.");
                return;
            }
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

        let semaphore = Arc::new(tokio::sync::Semaphore::new(20));

        loop {
            if cancel_token.is_cancelled() {
                tracing::info!("Polling worker received cancellation signal. Shutting down poller worker cleanly.");
                break;
            }

            let poll_url = format!(
                "{}/getUpdates?offset={}&timeout=20&allowed_updates=%5B%22message%22%2C%22edited_message%22%2C%22callback_query%22%2C%22my_chat_member%22%5D",
                base_url, offset
            );
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Polling worker received cancellation signal during getUpdates. Shutting down poller worker cleanly.");
                    break;
                }
                resp_res = client.get(&poll_url).send() => {
                    match resp_res {
                        Ok(resp) => {
                            let status = resp.status();
                            if status.as_u16() == 409 {
                                tracing::error!("Telegram HTTP 409 Conflict: duplicate bot instance running. Retrying in 10s...");
                                sleep(Duration::from_secs(10)).await;
                                continue;
                            } else if status.as_u16() == 429 {
                                let retry_secs = if let Ok(json) = resp.json::<serde_json::Value>().await {
                                    json.get("parameters")
                                        .and_then(|p| p.get("retry_after"))
                                        .and_then(|v| v.as_u64())
                                        .unwrap_or(3)
                                } else {
                                    3
                                };
                                tracing::warn!(
                                    retry_secs = retry_secs,
                                    "Telegram getUpdates HTTP 429 rate limit hit. Backing off..."
                                );
                                sleep(Duration::from_secs(retry_secs + 1)).await;
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

                                        let already_processed = if let Some(ref pool) = db_pool {
                                            if let Ok(conn) = pool.get().await {
                                                let res = conn
                                                    .interact(move |c| {
                                                        db::updates::is_processed(c, update_id)
                                                            .unwrap_or(false)
                                                    })
                                                    .await;
                                                res.unwrap_or(false)
                                            } else {
                                                false
                                            }
                                        } else {
                                            let db_path_check = config.db_path.clone();
                                            tokio::task::spawn_blocking(move || {
                                                if let Ok(conn) = db::get_db_connection(&db_path_check) {
                                                    db::updates::is_processed(&conn, update_id)
                                                        .unwrap_or(false)
                                                } else {
                                                    false
                                                }
                                            })
                                            .await
                                            .unwrap_or(false)
                                        };

                                        if already_processed {
                                            continue;
                                        }

                                        let flight_guard = match in_flight.try_claim(update_id) {
                                            Some(g) => g,
                                            None => continue,
                                        };

                                        let update_clone = update.clone();
                                        let config_clone = config.clone();
                                        let client_clone = client.clone();
                                        let base_url_clone = base_url.clone();
                                        let fsm_clone = fsm.clone();
                                        let chat_locks_clone = chat_locks.clone();
                                        let sem_clone = semaphore.clone();
                                        let pool_clone = db_pool.clone();

                                        let handle = tokio::spawn(async move {
                                            let _guard = flight_guard;
                                            let _permit = sem_clone.acquire_owned().await.ok();
                                            match tokio::time::timeout(
                                                Duration::from_secs(30),
                                                process_single_update(
                                                    &client_clone,
                                                    &base_url_clone,
                                                    &config_clone,
                                                    &fsm_clone,
                                                    &chat_locks_clone,
                                                    pool_clone.as_ref(),
                                                    &update_clone,
                                                    update_id,
                                                ),
                                            )
                                            .await
                                            {
                                                Ok(res) => res,
                                                Err(_) => Err("Update processing timed out (30s)".to_string()),
                                            }
                                        });
                                        handles.push((update_id, handle));
                                    }

                                    // Wait for all updates in batch to finish via join handles
                                    let mut min_failed_offset: Option<i64> = None;
                                    for (uid, handle) in handles {
                                        if let Ok(Err(err_msg)) = handle.await {
                                            let pool_clone = db_pool.clone();
                                            let config_clone = config.clone();
                                            let mut reached_dlq = false;
                                            let mut db_op_success = false;

                                            // Retry DB recording up to 3 times to avoid false offset rollbacks
                                            for retry_attempt in 0..3 {
                                                let p_clone = pool_clone.clone();
                                                let c_clone = config_clone.clone();
                                                let res = if let Some(ref pool) = p_clone {
                                                    if let Ok(conn) = pool.get().await {
                                                        conn.interact(move |c| {
                                                            db::updates::record_failure_and_check_max_retries(c, uid, 3)
                                                        })
                                                        .await
                                                        .unwrap_or(Err(rusqlite::Error::QueryReturnedNoRows))
                                                    } else {
                                                        Err(rusqlite::Error::QueryReturnedNoRows)
                                                    }
                                                } else {
                                                    let db_path_check = c_clone.db_path.clone();
                                                    tokio::task::spawn_blocking(move || {
                                                        if let Ok(conn) = db::get_db_connection(&db_path_check) {
                                                            db::updates::record_failure_and_check_max_retries(&conn, uid, 3)
                                                        } else {
                                                            Err(rusqlite::Error::QueryReturnedNoRows)
                                                        }
                                                    })
                                                    .await
                                                    .unwrap_or(Err(rusqlite::Error::QueryReturnedNoRows))
                                                };

                                                if let Ok(dlq_status) = res {
                                                    reached_dlq = dlq_status;
                                                    db_op_success = true;
                                                    break;
                                                }
                                                sleep(Duration::from_millis(50 * (1 << retry_attempt))).await;
                                            }

                                            if db_op_success && !reached_dlq {
                                                min_failed_offset = Some(match min_failed_offset {
                                                    Some(curr) => curr.min(uid),
                                                    None => uid,
                                                });
                                            } else {
                                                tracing::warn!(
                                                    update_id = uid,
                                                    error = %err_msg,
                                                    "Long Polling update reached max retries or DB retry exhausted; advancing offset past update_id."
                                                );
                                            }
                                        }
                                    }

                                    offset = min_failed_offset.unwrap_or(next_offset);

                                    // Update offset monotonically in memory and flush to SQLite on batch completion
                                    super::state::set_update_offset_memory(offset);
                                    let db_path_persist = config.db_path.clone();
                                    let current_offset = offset;
                                    tokio::task::spawn_blocking(move || {
                                        super::state::set_update_offset(&db_path_persist, current_offset);
                                    })
                                    .await
                                    .ok();
                                }
                            }
                        }
                        Err(e) => {
                            tracing::debug!(error = %e, "Telegram getUpdates request failed, retrying...");
                            sleep(Duration::from_secs(3)).await;
                        }
                    }
                }
            }
        }

        let db_path_flush = config.db_path.clone();
        let _ = tokio::task::spawn_blocking(move || {
            super::state::flush_offset_to_db(&db_path_flush);
        })
        .await;
    })
}
