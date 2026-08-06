use super::fetcher::{delete_webhook_with_retry, fetch_updates_batch, FetchResult};
use super::watermark;
use crate::api::telegram::fsm::FsmStore;
use crate::api::telegram::locks::{
    ChatLocksManager, ChatQueueDispatcher, InFlightTracker, WatermarkTracker,
};
use crate::api::telegram::process_single_update;
use crate::config::AppConfig;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

/// Starts background Telegram long-polling update listener worker with Low Watermark tracking.
pub fn start_poller_worker(
    config: Arc<AppConfig>,
    fsm: FsmStore,
    chat_locks: ChatLocksManager,
    in_flight: InFlightTracker,
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

        // 1. Reset Telegram Webhook to avoid 409 Conflict
        if !delete_webhook_with_retry(&client, &base_url, &cancel_token).await
            && cancel_token.is_cancelled()
        {
            return;
        }

        // 2. Load persistent update offset from SQLite
        let db_path = config.db_path.clone();
        let mut offset = tokio::task::spawn_blocking(move || {
            crate::api::telegram::state::get_update_offset(&db_path)
        })
        .await
        .unwrap_or(0);
        watermark::set_offset(offset);

        let watermark_tracker = WatermarkTracker::new();
        let queue_dispatcher = ChatQueueDispatcher::new();
        let dispatch_semaphore = Arc::new(tokio::sync::Semaphore::new(100));

        loop {
            if cancel_token.is_cancelled() {
                tracing::info!(
                    "Polling worker received cancellation signal. Shutting down cleanly."
                );
                break;
            }

            let fetch_res =
                match fetch_updates_batch(&client, &base_url, offset, &cancel_token).await {
                    Some(res) => res,
                    None => break,
                };

            match fetch_res {
                FetchResult::Conflict => {
                    tracing::error!("Telegram HTTP 409 Conflict: duplicate bot instance running. Retrying in 10s...");
                    sleep(Duration::from_secs(10)).await;
                    continue;
                }
                FetchResult::RateLimited(retry_secs) => {
                    tracing::warn!(
                        retry_secs = retry_secs,
                        "Telegram getUpdates HTTP 429 rate limit hit. Backing off..."
                    );
                    sleep(Duration::from_secs(retry_secs + 1)).await;
                    continue;
                }
                FetchResult::Error => {
                    sleep(Duration::from_secs(3)).await;
                    continue;
                }
                FetchResult::Success(result) => {
                    let mut batch_max_offset = offset;

                    for update in &result {
                        let update_id = match update.get("update_id").and_then(|v| v.as_i64()) {
                            Some(id) => id,
                            None => continue,
                        };

                        if update_id >= batch_max_offset {
                            batch_max_offset = update_id + 1;
                        }

                        // Pre-dispatch idempotency check
                        let is_proc = crate::api::telegram::webhook_db::is_update_processed(
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
                            crate::api::telegram::admin_session::extract_effective_user_context(
                                update,
                            );
                        let target_chat_id = chat_id.unwrap_or(0);

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
                                        Duration::from_secs(60),
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
                                                crate::api::telegram::webhook_db::record_failure(
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
                                                "Long polling update execution timed out (60s)"
                                            );
                                            reached_dlq =
                                                crate::api::telegram::webhook_db::record_failure(
                                                    inner_pool.as_ref(),
                                                    &inner_config.db_path,
                                                    update_id,
                                                    chat_id,
                                                    &inner_update.to_string(),
                                                    "Execution timeout (60s)",
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
                                        let _ =
                                            crate::api::telegram::client::send_telegram_request(
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
                                            let dlq =
                                                crate::api::telegram::webhook_db::record_failure(
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

                                if exec_success || reached_dlq {
                                    if let Some(lw) = watermark_guard.complete() {
                                        crate::api::telegram::state::set_update_offset_memory(lw);
                                        let _ = tokio::task::spawn_blocking(move || {
                                            crate::api::telegram::state::set_update_offset(
                                                &db_path_p, lw,
                                            );
                                        })
                                        .await;
                                    }
                                }
                            })
                        });

                        let dispatcher_clone = queue_dispatcher.clone();
                        let pool_dlq = db_pool.clone();
                        let db_path_dlq = config.db_path.clone();
                        let update_str = update.to_string();
                        let client_c = client.clone();
                        let base_c = base_url.clone();
                        let watermark_fb = watermark_guard_fallback;
                        let sem_clone = dispatch_semaphore.clone();

                        tokio::spawn(async move {
                            let _permit = sem_clone.acquire_owned().await.ok();
                            let enqueue_res = dispatcher_clone
                                .enqueue_timeout(
                                    target_chat_id,
                                    user_id,
                                    task_closure,
                                    Duration::from_secs(2),
                                )
                                .await;

                            if let Err(_timeout_err) = enqueue_res {
                                tracing::warn!(
                                    chat_id = target_chat_id,
                                    update_id = update_id,
                                    "Per-chat bounded queue full (capacity 64) after 2s backpressure. Recording DLQ and notifying user."
                                );

                                let _reached_dlq =
                                    crate::api::telegram::webhook_db::record_failure(
                                        pool_dlq.as_ref(),
                                        &db_path_dlq,
                                        update_id,
                                        chat_id,
                                        &update_str,
                                        "Per-chat queue capacity full (64) after 2s backpressure",
                                        1,
                                    )
                                    .await;

                                if let Some(lw) = watermark_fb.complete() {
                                    crate::api::telegram::state::set_update_offset_memory(lw);
                                    let db_path_p = db_path_dlq.clone();
                                    let _ = tokio::task::spawn_blocking(move || {
                                        crate::api::telegram::state::set_update_offset(
                                            &db_path_p, lw,
                                        );
                                    })
                                    .await;
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
                                    let _ = crate::api::telegram::client::send_telegram_request(
                                        &client_c,
                                        &format!("{}/sendMessage", base_c),
                                        &payload,
                                    )
                                    .await;
                                }
                            }
                        });
                    }

                    if watermark_tracker.is_empty() && batch_max_offset > offset {
                        watermark::advance_offset_if_greater(&mut offset, batch_max_offset);
                        crate::api::telegram::state::set_update_offset_memory(offset);
                        let db_p = config.db_path.clone();
                        let cur_off = offset;
                        let _ = tokio::task::spawn_blocking(move || {
                            crate::api::telegram::state::set_update_offset(&db_p, cur_off);
                        })
                        .await;
                    } else if let Some(lw) = watermark_tracker.low_watermark() {
                        if watermark::advance_offset_if_greater(&mut offset, lw) {
                            crate::api::telegram::state::set_update_offset_memory(offset);
                            let db_p = config.db_path.clone();
                            let cur_off = offset;
                            let _ = tokio::task::spawn_blocking(move || {
                                crate::api::telegram::state::set_update_offset(&db_p, cur_off);
                            })
                            .await;
                        }
                    }
                }
            }
        }

        let db_path_flush = config.db_path.clone();
        let _ = tokio::task::spawn_blocking(move || {
            crate::api::telegram::state::flush_offset_to_db(&db_path_flush);
        })
        .await;
    })
}
