use super::locks::ChatLocksManager;
use super::process_single_update;
use super::webhook_db;
use crate::api::telegram::fsm::FsmStore;
use crate::config::AppConfig;
use crate::db;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

/// Background worker that processes pending webhook updates asynchronously from SQLite queue.
pub fn start_webhook_queue_worker(
    config: Arc<AppConfig>,
    fsm: FsmStore,
    chat_locks: ChatLocksManager,
    in_flight: super::locks::InFlightTracker,
    db_pool: Option<deadpool_sqlite::Pool>,
    notify: Arc<tokio::sync::Notify>,
    cancel_token: CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        tracing::info!("Telegram Webhook queue worker starting...");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        let base_url = format!("https://api.telegram.org/bot{}", config.telegram_bot_token);
        let semaphore = Arc::new(tokio::sync::Semaphore::new(50));

        loop {
            if cancel_token.is_cancelled() {
                tracing::info!("Webhook queue worker cancelled. Exiting.");
                break;
            }

            // 1. Drain all available batches before waiting on notification
            while !cancel_token.is_cancelled() {
                let batch_res = if let Some(ref pool) = db_pool {
                    if let Ok(conn) = pool.get().await {
                        conn.interact(|c| db::updates::fetch_pending_batch(c, 10))
                            .await
                            .unwrap_or(Ok(Vec::new()))
                    } else {
                        Ok(Vec::new())
                    }
                } else {
                    let db_path = config.db_path.clone();
                    tokio::task::spawn_blocking(move || {
                        if let Ok(conn) = db::get_db_connection(&db_path) {
                            db::updates::fetch_pending_batch(&conn, 10)
                        } else {
                            Ok(Vec::new())
                        }
                    })
                    .await
                    .unwrap_or(Ok(Vec::new()))
                };

                let batch = match batch_res {
                    Ok(b) => b,
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to fetch pending webhook batch from SQLite");
                        break;
                    }
                };

                if batch.is_empty() {
                    break;
                }

                // Process batch concurrently with Semaphore backpressure
                for (update_id, chat_id, payload_str) in batch {
                    let flight_guard = match in_flight.try_claim(update_id) {
                        Some(g) => g,
                        None => continue,
                    };

                    let permit = semaphore.clone().acquire_owned().await.ok();
                    let config_clone = config.clone();
                    let client_clone = client.clone();
                    let base_url_clone = base_url.clone();
                    let fsm_clone = fsm.clone();
                    let chat_locks_clone = chat_locks.clone();
                    let pool_clone = db_pool.clone();

                    tokio::spawn(async move {
                        let _permit = permit;
                        let _guard = flight_guard;

                        // Check if update_id was already completely processed
                        if webhook_db::is_update_processed(
                            pool_clone.as_ref(),
                            &config_clone.db_path,
                            update_id,
                        )
                        .await
                        {
                            let _ = webhook_db::mark_done(
                                pool_clone.as_ref(),
                                &config_clone.db_path,
                                update_id,
                            )
                            .await;
                            return;
                        }

                        let update_val: serde_json::Value = match serde_json::from_str(&payload_str)
                        {
                            Ok(v) => v,
                            Err(_) => serde_json::json!({ "update_id": update_id }),
                        };

                        let res = tokio::time::timeout(
                            Duration::from_secs(60),
                            process_single_update(
                                &client_clone,
                                &base_url_clone,
                                &config_clone,
                                &fsm_clone,
                                &chat_locks_clone,
                                pool_clone.as_ref(),
                                &update_val,
                                update_id,
                            ),
                        )
                        .await
                        .map_err(|_| "Webhook update execution timed out (60s)".to_string())
                        .and_then(|r| r);

                        if res.is_ok() {
                            let _ = webhook_db::mark_done(
                                pool_clone.as_ref(),
                                &config_clone.db_path,
                                update_id,
                            )
                            .await;
                        } else {
                            let err_msg = res.err().unwrap_or_default();
                            tracing::error!(
                                update_id = update_id,
                                error = %err_msg,
                                "Webhook update execution failed"
                            );

                            let reached_dlq = webhook_db::record_failure(
                                pool_clone.as_ref(),
                                &config_clone.db_path,
                                update_id,
                                chat_id,
                                &payload_str,
                                &err_msg,
                                3,
                            )
                            .await;

                            if reached_dlq {
                                if let Some(cid) = chat_id {
                                    let notice = crate::domain::sanitizer::escape_telegram_markdown_v2(
                                        "⚠️ Temporary network issue processing request. Please try again or type /cancel.",
                                    );
                                    let dlq_msg = serde_json::json!({
                                        "chat_id": cid,
                                        "text": notice,
                                        "parse_mode": "MarkdownV2"
                                    });
                                    let _ = super::client::send_telegram_request(
                                        &client_clone,
                                        &format!("{}/sendMessage", base_url_clone),
                                        &dlq_msg,
                                    )
                                    .await;
                                }
                            }
                        }
                    });
                }
            }

            // 2. Wait on notification, cancellation, or fallback sleep timeout
            tokio::select! {
                _ = cancel_token.cancelled() => {
                    tracing::info!("Webhook queue worker cancelled. Exiting.");
                    break;
                }
                _ = notify.notified() => {}
                _ = sleep(Duration::from_secs(2)) => {}
            }
        }
    })
}
