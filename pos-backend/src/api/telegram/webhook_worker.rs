use super::locks::ChatLocksManager;
use super::process_single_update;
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
    db_pool: Option<deadpool_sqlite::Pool>,
    cancel_token: CancellationToken,
) {
    tokio::spawn(async move {
        tracing::info!("Telegram Webhook queue worker starting...");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        let base_url = format!("https://api.telegram.org/bot{}", config.telegram_bot_token);

        loop {
            if cancel_token.is_cancelled() {
                tracing::info!("Webhook queue worker cancelled. Exiting.");
                break;
            }

            // 1. Fetch pending batch from SQLite queue
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
                    sleep(Duration::from_millis(500)).await;
                    continue;
                }
            };

            // 2. CPU Saver: If queue is empty, sleep for 200ms
            if batch.is_empty() {
                sleep(Duration::from_millis(200)).await;
                continue;
            }

            // 3. Process batch concurrently without Head-of-Line Blocking
            for (update_id, chat_id, payload_str) in batch {
                let config_clone = config.clone();
                let client_clone = client.clone();
                let base_url_clone = base_url.clone();
                let fsm_clone = fsm.clone();
                let chat_locks_clone = chat_locks.clone();
                let pool_clone = db_pool.clone();

                tokio::spawn(async move {
                    // Check if update_id was already completely processed to avoid re-execution on late Telegram retry
                    let is_already_processed = if let Some(ref pool) = pool_clone {
                        if let Ok(conn) = pool.get().await {
                            conn.interact(move |c| {
                                db::updates::is_processed(c, update_id).unwrap_or(false)
                            })
                            .await
                            .unwrap_or(false)
                        } else {
                            false
                        }
                    } else {
                        let db_path_check = config_clone.db_path.clone();
                        tokio::task::spawn_blocking(move || {
                            if let Ok(conn) = db::get_db_connection(&db_path_check) {
                                db::updates::is_processed(&conn, update_id).unwrap_or(false)
                            } else {
                                false
                            }
                        })
                        .await
                        .unwrap_or(false)
                    };

                    if is_already_processed {
                        if let Some(ref pool) = pool_clone {
                            if let Ok(conn) = pool.get().await {
                                let _ = conn
                                    .interact(move |c| {
                                        db::updates::mark_webhook_update_done(c, update_id)
                                    })
                                    .await;
                            }
                        } else {
                            let db_path_del = config_clone.db_path.clone();
                            let _ = tokio::task::spawn_blocking(move || {
                                if let Ok(conn) = db::get_db_connection(&db_path_del) {
                                    let _ = db::updates::mark_webhook_update_done(&conn, update_id);
                                }
                            })
                            .await;
                        }
                        return;
                    }

                    let update_val: serde_json::Value = match serde_json::from_str(&payload_str) {
                        Ok(v) => v,
                        Err(_) => serde_json::json!({ "update_id": update_id }),
                    };

                    let res = process_single_update(
                        &client_clone,
                        &base_url_clone,
                        &config_clone,
                        &fsm_clone,
                        &chat_locks_clone,
                        pool_clone.as_ref(),
                        &update_val,
                        update_id,
                    )
                    .await;

                    if res.is_ok() {
                        // Mark done in pending queue
                        if let Some(ref pool) = pool_clone {
                            if let Ok(conn) = pool.get().await {
                                let _ = conn
                                    .interact(move |c| {
                                        db::updates::mark_webhook_update_done(c, update_id)
                                    })
                                    .await;
                            }
                        } else {
                            let db_path = config_clone.db_path.clone();
                            let _ = tokio::task::spawn_blocking(move || {
                                if let Ok(conn) = db::get_db_connection(&db_path) {
                                    let _ = db::updates::mark_webhook_update_done(&conn, update_id);
                                }
                            })
                            .await;
                        }
                    } else {
                        let err_msg = res.err().unwrap_or_default();
                        tracing::error!(
                            update_id = update_id,
                            error = %err_msg,
                            "Webhook update execution failed"
                        );

                        // Record failure and check if reached max retries (3)
                        let payload_for_dlq = payload_str.clone();
                        let err_for_dlq = err_msg.clone();

                        let reached_dlq = if let Some(ref pool) = pool_clone {
                            if let Ok(conn) = pool.get().await {
                                conn.interact(move |c| {
                                    let reached =
                                        db::updates::record_failure_and_check_max_retries(
                                            c, update_id, 3,
                                        )
                                        .unwrap_or(false);

                                    if reached {
                                        let _ = db::updates::move_to_dlq(
                                            c,
                                            update_id,
                                            chat_id,
                                            &payload_for_dlq,
                                            &err_for_dlq,
                                            3,
                                        );
                                    }
                                    reached
                                })
                                .await
                                .unwrap_or(false)
                            } else {
                                false
                            }
                        } else {
                            let db_path = config_clone.db_path.clone();
                            tokio::task::spawn_blocking(move || {
                                if let Ok(conn) = db::get_db_connection(&db_path) {
                                    let reached =
                                        db::updates::record_failure_and_check_max_retries(
                                            &conn, update_id, 3,
                                        )
                                        .unwrap_or(false);

                                    if reached {
                                        let _ = db::updates::move_to_dlq(
                                            &conn,
                                            update_id,
                                            chat_id,
                                            &payload_for_dlq,
                                            &err_for_dlq,
                                            3,
                                        );
                                    }
                                    reached
                                } else {
                                    false
                                }
                            })
                            .await
                            .unwrap_or(false)
                        };

                        // DLQ UX Notification: inform user if max retries exceeded
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
    });
}
