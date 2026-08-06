pub mod chat_action;
pub mod client;
pub mod client_queue;
pub mod events;
pub mod fsm;
pub mod fsm_store;
pub mod handlers;
pub mod locks;
pub mod orders;
pub mod polling;
pub mod qr;
pub mod state;
pub mod verifier;
pub mod webhook;
pub mod webhook_worker;

use crate::config::AppConfig;
use fsm::FsmStore;
use locks::ChatLocksManager;
use std::sync::Arc;

pub type ChatLocks = locks::ChatLocksManager;

use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};

static FAILED_WEBHOOK_ATTEMPTS: AtomicU32 = AtomicU32::new(0);
static WEBHOOK_COOLDOWN_UNTIL: AtomicU64 = AtomicU64::new(0);

pub const MAX_WEBHOOK_FAILURES: u32 = 3;
pub const WEBHOOK_COOLDOWN_SECS: u64 = 300;

/// Starts background Telegram listener (Webhook or Polling) and Solana RPC payment verifier services.
pub fn start_telegram_services(config: Arc<AppConfig>, db_pool: Option<deadpool_sqlite::Pool>) {
    let token = config.telegram_bot_token.clone();
    if token.is_empty() || token.contains("123456789:ABC") {
        tracing::warn!("Telegram Bot token not set or placeholder. Skipping Telegram services.");
        return;
    }

    // 1. Start Solana RPC payment verification background worker
    verifier::start_verifier_worker(config.clone());

    // 2. Instantiate SQLite-backed FSM store and Weak-ref ChatLocksManager
    let fsm_store = if let Some(ref pool) = db_pool {
        FsmStore::new_with_pool(config.db_path.clone(), pool.clone())
    } else {
        FsmStore::new_with_db(config.db_path.clone())
    };
    let chat_locks = ChatLocksManager::new();
    let in_flight = locks::InFlightTracker::new();

    // 3. Start Webhook queue worker for async DB queue processing
    webhook_worker::start_webhook_queue_worker(
        config.clone(),
        fsm_store.clone(),
        chat_locks.clone(),
        db_pool.clone(),
        tokio_util::sync::CancellationToken::new(),
    );

    // 4. Dual-mode Telegram update listener startup with Circuit Breaker and Cancellation Token
    let poller_config = config.clone();
    let poller_pool = db_pool.clone();
    let poller_cancel_token = tokio_util::sync::CancellationToken::new();

    let poller_cancel_token_clone = poller_cancel_token.clone();
    tokio::spawn(async move {
        if let Some(ref webhook_url) = poller_config.telegram_webhook_url {
            if !webhook_url.trim().is_empty() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let cooldown_until = WEBHOOK_COOLDOWN_UNTIL.load(Ordering::SeqCst);

                if now < cooldown_until {
                    tracing::warn!(
                        cooldown_secs = cooldown_until - now,
                        "Webhook circuit breaker active (cooldown). Falling back to Long Polling directly."
                    );
                    polling::start_poller_worker(
                        poller_config,
                        fsm_store,
                        chat_locks,
                        in_flight.clone(),
                        poller_pool,
                        poller_cancel_token_clone,
                    );
                    return;
                }

                if let Err(e) = webhook::register_telegram_webhook(&poller_config).await {
                    let failures = FAILED_WEBHOOK_ATTEMPTS.fetch_add(1, Ordering::SeqCst) + 1;
                    tracing::error!(
                        error = %e,
                        failures = failures,
                        "Failed to register Telegram Webhook"
                    );

                    if failures >= MAX_WEBHOOK_FAILURES {
                        let cooldown = now + WEBHOOK_COOLDOWN_SECS;
                        WEBHOOK_COOLDOWN_UNTIL.store(cooldown, Ordering::SeqCst);
                        tracing::error!(
                            cooldown_secs = WEBHOOK_COOLDOWN_SECS,
                            "Webhook registration failed 3 consecutive times. Circuit breaker TRIPPED! Falling back to Long Polling with 5-minute cooldown."
                        );

                        // Spawn background recovery task that waits for cooldown, then retries Webhook registration
                        let recovery_config = poller_config.clone();
                        let recovery_cancel_token = poller_cancel_token_clone.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(std::time::Duration::from_secs(
                                WEBHOOK_COOLDOWN_SECS,
                            ))
                            .await;
                            tracing::info!("Webhook circuit breaker cooldown expired. Attempting Webhook recovery...");
                            if let Ok(()) =
                                webhook::register_telegram_webhook(&recovery_config).await
                            {
                                FAILED_WEBHOOK_ATTEMPTS.store(0, Ordering::SeqCst);
                                WEBHOOK_COOLDOWN_UNTIL.store(0, Ordering::SeqCst);
                                recovery_cancel_token.cancel();
                                tracing::info!("Webhook registration recovered successfully! Webhook mode restored and Poller worker cleanly shut down.");
                            } else {
                                tracing::warn!("Webhook recovery re-attempt failed. Remaining in Polling mode.");
                            }
                        });
                    }
                    polling::start_poller_worker(
                        poller_config,
                        fsm_store,
                        chat_locks,
                        in_flight.clone(),
                        poller_pool,
                        poller_cancel_token_clone,
                    );
                    return;
                }

                // Reset failure counter on successful webhook registration
                FAILED_WEBHOOK_ATTEMPTS.store(0, Ordering::SeqCst);
                WEBHOOK_COOLDOWN_UNTIL.store(0, Ordering::SeqCst);
                return;
            }
        }

        // Fallback or primary Long Polling mode
        polling::start_poller_worker(
            poller_config,
            fsm_store,
            chat_locks,
            in_flight.clone(),
            poller_pool,
            poller_cancel_token_clone,
        );
    });
}

/// Helper function to process a single Telegram update with per-(chat_id, user_id) locking & async SQLite registration.
#[allow(clippy::too_many_arguments)]
pub async fn process_single_update(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    chat_locks: &ChatLocksManager,
    db_pool: Option<&deadpool_sqlite::Pool>,
    update: &serde_json::Value,
    update_id: i64,
) -> Result<(), String> {
    let (chat_id, user_id) = if let Some(msg) = update
        .get("message")
        .or_else(|| update.get("edited_message"))
    {
        let cid = msg
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64());
        let uid = msg
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        (cid, uid)
    } else if let Some(cb) = update.get("callback_query") {
        let cid = cb
            .get("message")
            .and_then(|m| m.get("chat"))
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64());
        let uid = cb
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        (cid, uid)
    } else {
        (None, 0)
    };

    let dispatch_res = if let Some(target_chat_id) = chat_id {
        let chat_lock = chat_locks.get_or_create(target_chat_id, user_id);
        let _guard = chat_lock.lock().await;
        dispatch_update_content(client, base_url, config, fsm, update).await
    } else {
        dispatch_update_content(client, base_url, config, fsm, update).await
    };

    if let Err(ref e) = dispatch_res {
        tracing::error!(update_id = update_id, error = %e, "Update content dispatch failed");
        return Err(e.clone());
    }

    if let Some(pool) = db_pool {
        if let Ok(conn) = pool.get().await {
            let _ = conn
                .interact(move |c| {
                    let _ = crate::db::updates::check_and_register(c, update_id);
                })
                .await;
            return Ok(());
        }
    }

    let db_path = config.db_path.clone();
    let _ = tokio::task::spawn_blocking(move || {
        if let Ok(conn) = crate::db::get_db_connection(&db_path) {
            let _ = crate::db::updates::check_and_register(&conn, update_id);
        }
    })
    .await;

    Ok(())
}

/// Dispatches update payload to message or callback query handlers.
pub async fn dispatch_update_content(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    update: &serde_json::Value,
) -> Result<(), String> {
    // Process Telegram system events (my_chat_member, migrate_to_chat_id)
    if events::handle_system_event(client, base_url, config, fsm, update).await? {
        return Ok(());
    }

    // Process Message
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
        let chat_type = msg
            .get("chat")
            .and_then(|c| c.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("private");
        let reply_to_text = msg
            .get("reply_to_message")
            .and_then(|r| r.get("text"))
            .and_then(|v| v.as_str());

        if let Some(chat_id) = chat_id {
            if let Some(text) = text {
                handlers::handle_user_message(
                    client,
                    base_url,
                    config,
                    fsm,
                    chat_id,
                    user_id,
                    chat_type,
                    text,
                    reply_to_text,
                )
                .await?;
            } else if chat_type == "private" {
                // In DM/private chats, send a helpful fallback message for non-text media (photos, voice, stickers)
                let help = crate::domain::sanitizer::escape_telegram_markdown_v2(
                    "⚠️ Only text commands and order amounts are supported.",
                );
                let payload = serde_json::json!({
                    "chat_id": chat_id,
                    "text": help,
                    "parse_mode": "MarkdownV2"
                });
                let _ = client::send_telegram_request(
                    client,
                    &format!("{}/sendMessage", base_url),
                    &payload,
                )
                .await;
            }
        }
    }

    // Process Callback Query
    if let Some(cb) = update.get("callback_query") {
        let cb_id = cb.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let data = cb.get("data").and_then(|v| v.as_str()).unwrap_or("");
        let user_id = cb
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let msg = cb.get("message");
        let chat_id = msg
            .and_then(|m| m.get("chat"))
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64());

        if let Some(chat_id) = chat_id {
            handlers::handle_callback_query(
                client,
                base_url,
                config,
                fsm.pool(),
                chat_id,
                user_id,
                cb_id,
                data,
            )
            .await?;
        }
    }

    Ok(())
}
