pub mod client;
pub mod fsm;
pub mod handlers;
pub mod orders;
pub mod state;
pub mod verifier;

use crate::config::AppConfig;
use fsm::FsmStore;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;
use tokio::time::{sleep, Duration};

pub type ChatLocks = Arc<std::sync::Mutex<HashMap<i64, Arc<AsyncMutex<()>>>>>;

/// Starts background Telegram long-poller and Solana RPC payment verifier services.
pub fn start_telegram_services(config: Arc<AppConfig>) {
    let token = config.telegram_bot_token.clone();
    if token.is_empty() || token.contains("123456789:ABC") {
        tracing::warn!("Telegram Bot token not set or placeholder. Skipping Telegram services.");
        return;
    }

    // 1. Start Solana RPC payment verification background worker
    verifier::start_verifier_worker(config.clone());

    // 2. Instantiate in-memory FSM store with 5-minute TTL
    let fsm_store = FsmStore::new();

    // 3. Chat-level concurrency locks map: chat_id -> Mutex<()>
    let chat_locks: ChatLocks = Arc::new(std::sync::Mutex::new(HashMap::new()));

    // 4. Start Telegram long-polling update listener background worker
    let poller_config = config.clone();
    tokio::spawn(async move {
        tracing::info!("Telegram long-poller worker started");
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
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
                            let mut handles = Vec::new();

                            for update in result {
                                let update_id =
                                    match update.get("update_id").and_then(|v| v.as_i64()) {
                                        Some(id) => id,
                                        None => continue,
                                    };

                                if update_id >= offset {
                                    offset = update_id + 1;
                                }

                                // Fast pre-check: skip if update_id was already fully processed
                                if let Ok(conn) =
                                    crate::db::get_db_connection(&poller_config.db_path)
                                {
                                    if crate::db::updates::is_processed(&conn, update_id)
                                        .unwrap_or(false)
                                    {
                                        continue;
                                    }
                                }

                                let update_clone = update.clone();
                                let poller_config_clone = poller_config.clone();
                                let client_clone = client.clone();
                                let base_url_clone = base_url.clone();
                                let fsm_clone = fsm_store.clone();
                                let chat_locks_clone = chat_locks.clone();

                                // Spawn per-update task with per-chat ordering lock
                                let handle = tokio::spawn(async move {
                                    process_single_update(
                                        &client_clone,
                                        &base_url_clone,
                                        &poller_config_clone,
                                        &fsm_clone,
                                        &chat_locks_clone,
                                        &update_clone,
                                        update_id,
                                    )
                                    .await;
                                });
                                handles.push(handle);
                            }

                            // Wait for all updates in batch to finish before persisting offset to SQLite
                            for handle in handles {
                                let _ = handle.await;
                            }

                            // Persist max update offset to SQLite
                            state::set_update_offset(&poller_config.db_path, offset);
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

/// Helper function to process a single Telegram update with per-chat locking & memory GC.
async fn process_single_update(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    chat_locks: &ChatLocks,
    update: &serde_json::Value,
    update_id: i64,
) {
    let chat_id = if let Some(msg) = update.get("message") {
        msg.get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64())
    } else if let Some(cb) = update.get("callback_query") {
        cb.get("message")
            .and_then(|m| m.get("chat"))
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64())
    } else {
        None
    };

    let target_chat_id = chat_id.unwrap_or(0);

    // Safely acquire per-chat lock (without holding the map lock across .await!)
    let chat_lock = {
        let mut map = chat_locks.lock().unwrap_or_else(|e| e.into_inner());
        map.entry(target_chat_id)
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    };

    {
        let _guard = chat_lock.lock().await;

        // Process Message (with reply_to_message support)
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
                handlers::handle_callback_query(client, base_url, config, chat_id, cb_id, data)
                    .await;
            }
        }
    } // Lock guard dropped here!

    // Mark update_id as processed in SQLite ONLY AFTER processing completes
    if let Ok(conn) = crate::db::get_db_connection(&config.db_path) {
        let _ = crate::db::updates::check_and_register(&conn, update_id);
    }

    // Memory GC: Remove chat_id from chat_locks if no other task is waiting for this chat
    {
        let mut map = chat_locks.lock().unwrap_or_else(|e| e.into_inner());
        if Arc::strong_count(&chat_lock) <= 2 {
            map.remove(&target_chat_id);
        }
    }
}
