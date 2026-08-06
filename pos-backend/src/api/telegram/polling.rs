use super::locks::{ChatLocksManager, ChatQueueDispatcher, WatermarkTracker};
use super::process_single_update;
use crate::api::telegram::fsm::FsmStore;
use crate::config::AppConfig;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

/// Starts background Telegram long-polling update listener worker with Low Watermark tracking.
pub fn start_poller_worker(
    config: Arc<AppConfig>,
    fsm: FsmStore,
    chat_locks: ChatLocksManager,
    in_flight: super::locks::InFlightTracker,
    db_pool: Option<deadpool_sqlite::Pool>,
    cancel_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        tracing::info!("Telegram long-poller worker starting with Low Watermark tracker...");

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();

        let base_url = format!("https://api.telegram.org/bot{}", config.telegram_bot_token);

        // 1. Delete Webhook to avoid 409 Conflict
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

        // 2. Load persistent update offset from SQLite
        let db_path = config.db_path.clone();
        let mut offset =
            tokio::task::spawn_blocking(move || super::state::get_update_offset(&db_path))
                .await
                .unwrap_or(0);

        let watermark_tracker = WatermarkTracker::new();
        let queue_dispatcher = ChatQueueDispatcher::new();

        loop {
            if cancel_token.is_cancelled() {
                tracing::info!(
                    "Polling worker received cancellation signal. Shutting down cleanly."
                );
                break;
            }

            let poll_url = format!(
                "{}/getUpdates?offset={}&timeout=20&allowed_updates=%5B%22message%22%2C%22edited_message%22%2C%22callback_query%22%2C%22my_chat_member%22%5D",
                base_url, offset
            );

            let resp_res = tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Polling worker cancelled during getUpdates long poll.");
                    break;
                }
                res = client.get(&poll_url).send() => res,
            };

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
                            let mut batch_max_offset = offset;

                            for update in result {
                                let update_id =
                                    match update.get("update_id").and_then(|v| v.as_i64()) {
                                        Some(id) => id,
                                        None => continue,
                                    };

                                if update_id >= batch_max_offset {
                                    batch_max_offset = update_id + 1;
                                }

                                // Pre-dispatch idempotency check
                                let is_proc = super::webhook_db::is_update_processed(
                                    db_pool.as_ref(),
                                    &config.db_path,
                                    update_id,
                                )
                                .await;

                                if is_proc {
                                    continue;
                                }

                                let flight_guard = match in_flight.try_claim(update_id) {
                                    Some(g) => g,
                                    None => continue,
                                };

                                let (chat_id, user_id) =
                                    super::admin_session::extract_effective_user_context(update);
                                let target_chat_id = chat_id.unwrap_or(0);

                                // Register update in Low Watermark Tracker
                                let watermark_guard = watermark_tracker.register(update_id);
                                let watermark_guard_fallback = watermark_guard.clone();

                                let update_clone = update.clone();
                                let config_clone = config.clone();
                                let client_clone = client.clone();
                                let base_url_clone = base_url.clone();
                                let fsm_clone = fsm.clone();
                                let chat_locks_clone = chat_locks.clone();
                                let pool_clone = db_pool.clone();
                                let db_path_persist = config.db_path.clone();

                                let task_closure = Box::new(move || {
                                    let inner_client = client_clone.clone();
                                    let inner_base = base_url_clone.clone();
                                    let inner_config = config_clone.clone();
                                    let inner_fsm = fsm_clone.clone();
                                    let inner_locks = chat_locks_clone.clone();
                                    let inner_pool = pool_clone.clone();
                                    let inner_update = update_clone.clone();
                                    let db_path_p = db_path_persist.clone();

                                    tokio::spawn(async move {
                                        let inner_handle = tokio::spawn(async move {
                                            let _flight = flight_guard;

                                            let res = tokio::time::timeout(
                                                Duration::from_secs(30),
                                                process_single_update(
                                                    &inner_client,
                                                    &inner_base,
                                                    &inner_config,
                                                    &inner_fsm,
                                                    &inner_locks,
                                                    inner_pool.as_ref(),
                                                    &inner_update,
                                                    update_id,
                                                ),
                                            )
                                            .await;

                                            let mut reached_dlq = false;
                                            let exec_success = match res {
                                                Ok(Ok(())) => true,
                                                Ok(Err(ref err_msg)) => {
                                                    tracing::error!(
                                                        update_id = update_id,
                                                        error = %err_msg,
                                                        "Long polling update execution error"
                                                    );
                                                    reached_dlq =
                                                        super::webhook_db::record_failure(
                                                            inner_pool.as_ref(),
                                                            &inner_config.db_path,
                                                            update_id,
                                                            chat_id,
                                                            &inner_update.to_string(),
                                                            err_msg,
                                                            3,
                                                        )
                                                        .await;
                                                    false
                                                }
                                                Err(_) => {
                                                    tracing::error!(
                                                        update_id = update_id,
                                                        "Long polling update execution timed out (30s)"
                                                    );
                                                    reached_dlq =
                                                        super::webhook_db::record_failure(
                                                            inner_pool.as_ref(),
                                                            &inner_config.db_path,
                                                            update_id,
                                                            chat_id,
                                                            &inner_update.to_string(),
                                                            "Execution timeout (30s)",
                                                            3,
                                                        )
                                                        .await;
                                                    false
                                                }
                                            };

                                            if reached_dlq && target_chat_id != 0 {
                                                let notice = crate::domain::sanitizer::escape_telegram_markdown_v2(
                                                    "⚠️ Temporary network issue processing request. Please try again or type /cancel.",
                                                );
                                                let dlq_msg = serde_json::json!({
                                                    "chat_id": target_chat_id,
                                                    "text": notice,
                                                    "parse_mode": "MarkdownV2"
                                                });
                                                let _ = super::client::send_telegram_request(
                                                    &inner_client,
                                                    &format!("{}/sendMessage", inner_base),
                                                    &dlq_msg,
                                                )
                                                .await;
                                            }

                                            (exec_success, reached_dlq)
                                        });

                                        let (exec_success, reached_dlq) = match inner_handle.await {
                                            Ok((success, dlq)) => (success, dlq),
                                            Err(join_err) => {
                                                if join_err.is_panic() {
                                                    tracing::error!(
                                                        update_id = update_id,
                                                        "Task panicked during update execution! Force isolating in DLQ."
                                                    );
                                                    let dlq = super::webhook_db::record_failure(
                                                        pool_clone.as_ref(),
                                                        &config_clone.db_path,
                                                        update_id,
                                                        chat_id,
                                                        &update_clone.to_string(),
                                                        "Task panicked during execution",
                                                        1,
                                                    )
                                                    .await;
                                                    (false, dlq || true)
                                                } else {
                                                    (false, false)
                                                }
                                            }
                                        };

                                        // Complete Watermark ONLY if execution succeeded OR update was committed to DLQ in SQLite
                                        if exec_success || reached_dlq {
                                            if let Some(lw) = watermark_guard.complete() {
                                                super::state::set_update_offset_memory(lw);
                                                let _ = tokio::task::spawn_blocking(move || {
                                                    super::state::set_update_offset(&db_path_p, lw);
                                                })
                                                .await;
                                            }
                                        }
                                    })
                                });

                                // Non-blocking dispatch to session FIFO queue with bounded capacity (64)
                                if let Err(_full_err) = queue_dispatcher.try_enqueue(
                                    target_chat_id,
                                    user_id,
                                    task_closure,
                                ) {
                                    tracing::warn!(
                                        chat_id = target_chat_id,
                                        update_id = update_id,
                                        "Per-chat bounded queue full (capacity 64). Recording DLQ and rate-limiting user."
                                    );
                                    let pool_dlq = db_pool.clone();
                                    let db_path_dlq = config.db_path.clone();
                                    let update_str = update.to_string();
                                    let client_c = client.clone();
                                    let base_c = base_url.clone();
                                    let watermark_fb = watermark_guard_fallback;

                                    // Lightweight DLQ recording task without heavy business execution (OOM-safe)
                                    tokio::spawn(async move {
                                        let reached_dlq = super::webhook_db::record_failure(
                                            pool_dlq.as_ref(),
                                            &db_path_dlq,
                                            update_id,
                                            chat_id,
                                            &update_str,
                                            "Per-chat queue capacity full (64)",
                                            1,
                                        )
                                        .await;

                                        if reached_dlq {
                                            if let Some(lw) = watermark_fb.complete() {
                                                super::state::set_update_offset_memory(lw);
                                                let db_path_p = db_path_dlq.clone();
                                                let _ = tokio::task::spawn_blocking(move || {
                                                    super::state::set_update_offset(&db_path_p, lw);
                                                })
                                                .await;
                                            }
                                        }

                                        if target_chat_id != 0 {
                                            let notice = crate::domain::sanitizer::escape_telegram_markdown_v2(
                                                "⚠️ Too many commands in progress. Please wait a few seconds.",
                                            );
                                            let payload = serde_json::json!({
                                                "chat_id": target_chat_id,
                                                "text": notice,
                                                "parse_mode": "MarkdownV2"
                                            });
                                            let _ = super::client::send_telegram_request(
                                                &client_c,
                                                &format!("{}/sendMessage", base_c),
                                                &payload,
                                            )
                                            .await;
                                        }
                                    });
                                }
                            }

                            // If watermark tracker is empty, advance offset to batch_max_offset
                            if watermark_tracker.is_empty() && batch_max_offset > offset {
                                offset = batch_max_offset;
                                super::state::set_update_offset_memory(offset);
                                let db_p = config.db_path.clone();
                                let cur_off = offset;
                                let _ = tokio::task::spawn_blocking(move || {
                                    super::state::set_update_offset(&db_p, cur_off);
                                })
                                .await;
                            } else if let Some(lw) = watermark_tracker.low_watermark() {
                                if lw > offset {
                                    offset = lw;
                                    super::state::set_update_offset_memory(offset);
                                    let db_p = config.db_path.clone();
                                    let cur_off = offset;
                                    let _ = tokio::task::spawn_blocking(move || {
                                        super::state::set_update_offset(&db_p, cur_off);
                                    })
                                    .await;
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::debug!(error = %e, "Telegram getUpdates request failed, retrying...");
                    sleep(Duration::from_secs(3)).await;
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
