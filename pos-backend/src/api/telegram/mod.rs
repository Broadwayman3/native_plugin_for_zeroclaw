pub mod admin_session;
pub mod chat_action;
pub mod client;
pub mod client_queue;
pub mod events;
pub mod fsm;
pub mod fsm_store;
pub mod handlers;
pub mod lang_cache;
pub mod lifecycle;
pub mod locks;
pub mod orders;
pub mod polling;
pub mod qr;
pub mod rate_limiter;
pub mod state;
pub mod verifier;
pub mod webhook;
pub mod webhook_db;
pub mod webhook_worker;

use crate::config::AppConfig;
use fsm::FsmStore;
use locks::ChatLocksManager;

pub type ChatLocks = locks::ChatLocksManager;
pub use lifecycle::{start_telegram_services, TelegramServicesHandles};

/// Flexible helper to extract invoice ID (token starting with INV-) from any message or callback query payload.
pub fn extract_invoice_id(update: &serde_json::Value) -> Option<String> {
    let raw_str = update
        .get("callback_query")
        .and_then(|cb| cb.get("data"))
        .and_then(|v| v.as_str())
        .or_else(|| {
            update
                .get("message")
                .and_then(|m| m.get("text"))
                .and_then(|v| v.as_str())
        })?;

    if let Some(idx) = raw_str.find("INV-") {
        let candidate = &raw_str[idx..];
        let end = candidate
            .find(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
            .unwrap_or(candidate.len());
        let token = &candidate[..end];
        if token.len() > 4 {
            return Some(token.to_string());
        }
    }
    None
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
    // 1. Pre-dispatch idempotency check & registration with explicit DB failure propagation
    let db_check_res = if let Some(pool) = db_pool {
        match pool.get().await {
            Ok(conn) => conn
                .interact(move |c| crate::db::updates::check_and_register(c, update_id))
                .await
                .map_err(|e| e.to_string())
                .and_then(|r| r.map_err(|e| e.to_string())),
            Err(e) => Err(format!("DB pool acquisition error: {}", e)),
        }
    } else {
        let db_path = config.db_path.clone();
        tokio::task::spawn_blocking(move || {
            let conn = crate::db::get_db_connection(&db_path).map_err(|e| e.to_string())?;
            crate::db::updates::check_and_register(&conn, update_id).map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r)
    };

    let is_new = match db_check_res {
        Ok(is_new) => is_new,
        Err(e) => {
            tracing::error!(
                update_id = update_id,
                error = %e,
                "DB connection acquisition failed during update idempotency check"
            );
            return Err(format!("DB pool error: {}", e));
        }
    };

    if !is_new {
        tracing::debug!(
            update_id = update_id,
            "Update already registered in processed_updates, skipping duplicate dispatch"
        );
        return Ok(());
    }

    // 2. Dispatch content with single canonical per-session lock to prevent FSM race conditions & deadlocks
    let (chat_id, user_id) = admin_session::extract_effective_user_context(update);

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
