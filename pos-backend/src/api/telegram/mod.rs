pub mod client;
pub mod fsm;
pub mod fsm_store;
pub mod handlers;
pub mod locks;
pub mod orders;
pub mod polling;
pub mod state;
pub mod verifier;
pub mod webhook;

use crate::config::AppConfig;
use fsm::FsmStore;
use locks::ChatLocksManager;
use std::sync::Arc;

pub type ChatLocks = locks::ChatLocksManager;

/// Starts background Telegram listener (Webhook or Polling) and Solana RPC payment verifier services.
pub fn start_telegram_services(config: Arc<AppConfig>) {
    let token = config.telegram_bot_token.clone();
    if token.is_empty() || token.contains("123456789:ABC") {
        tracing::warn!("Telegram Bot token not set or placeholder. Skipping Telegram services.");
        return;
    }

    // 1. Start Solana RPC payment verification background worker
    verifier::start_verifier_worker(config.clone());

    // 2. Instantiate SQLite-backed FSM store and Weak-ref ChatLocksManager
    let fsm_store = FsmStore::new_with_db(config.db_path.clone());
    let chat_locks = ChatLocksManager::new();

    // 3. Dual-mode Telegram update listener startup
    let poller_config = config.clone();
    tokio::spawn(async move {
        if let Some(ref webhook_url) = poller_config.telegram_webhook_url {
            if !webhook_url.trim().is_empty() {
                if let Err(e) = webhook::register_telegram_webhook(&poller_config).await {
                    tracing::error!(error = %e, "Failed to register Telegram Webhook; falling back to Long Polling");
                    polling::start_poller_worker(poller_config, fsm_store, chat_locks);
                }
                return;
            }
        }

        // Fallback or primary Long Polling mode
        polling::start_poller_worker(poller_config, fsm_store, chat_locks);
    });
}

/// Helper function to process a single Telegram update with per-chat locking & async SQLite registration.
pub async fn process_single_update(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    chat_locks: &ChatLocksManager,
    update: &serde_json::Value,
    update_id: i64,
) {
    let chat_id = if let Some(msg) = update
        .get("message")
        .or_else(|| update.get("edited_message"))
    {
        msg.get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64())
    } else if let Some(cb) = update.get("callback_query") {
        cb.get("message")
            .and_then(|m| m.get("chat"))
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64())
    } else if let Some(my_chat) = update.get("my_chat_member") {
        my_chat
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64())
    } else {
        None
    };

    if let Some(target_chat_id) = chat_id {
        let chat_lock = chat_locks.get_or_create(target_chat_id);
        let _guard = chat_lock.lock().await;
        dispatch_update_content(client, base_url, config, fsm, update).await;
    } else {
        dispatch_update_content(client, base_url, config, fsm, update).await;
    }

    // Register update_id as processed in SQLite via spawn_blocking
    let db_path = config.db_path.clone();
    let _ = tokio::task::spawn_blocking(move || match crate::db::get_db_connection(&db_path) {
        Ok(conn) => {
            if let Err(e) = crate::db::updates::check_and_register(&conn, update_id) {
                tracing::warn!(update_id = update_id, error = %e, "Failed to register processed update_id");
            }
        }
        Err(e) => {
            tracing::error!(db_path = %db_path, error = %e, "Failed to connect to SQLite in process_single_update");
        }
    })
    .await;
}

/// Dispatches update payload to message or callback query handlers.
pub async fn dispatch_update_content(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    update: &serde_json::Value,
) {
    // Process Message or Edited Message
    if let Some(msg) = update
        .get("message")
        .or_else(|| update.get("edited_message"))
    {
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
        let reply_to_text = msg
            .get("reply_to_message")
            .and_then(|r| r.get("text"))
            .and_then(|v| v.as_str());

        if let (Some(chat_id), Some(text)) = (chat_id, text) {
            handlers::handle_user_message(
                client,
                base_url,
                config,
                fsm,
                chat_id,
                user_id,
                text,
                reply_to_text,
            )
            .await;
        }
    }

    // Process Callback Query
    if let Some(cb) = update.get("callback_query") {
        let cb_id = cb.get("id").and_then(|v| v.as_str()).unwrap_or("");
        let data = cb.get("data").and_then(|v| v.as_str()).unwrap_or("");
        let msg = cb.get("message");
        let chat_id = msg
            .and_then(|m| m.get("chat"))
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64());

        if let Some(chat_id) = chat_id {
            handlers::handle_callback_query(client, base_url, config, chat_id, cb_id, data).await;
        }
    }
}
