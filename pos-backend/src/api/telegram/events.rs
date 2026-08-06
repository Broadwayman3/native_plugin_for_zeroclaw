use crate::api::telegram::client::send_telegram_request;
use crate::api::telegram::fsm::FsmStore;
use crate::config::AppConfig;
use crate::db;

/// Processes Telegram system events such as `my_chat_member` and group chat migrations (`migrate_to_chat_id`).
pub async fn handle_system_event(
    client: &reqwest::Client,
    base_url: &str,
    config: &AppConfig,
    fsm: &FsmStore,
    update: &serde_json::Value,
) -> Result<bool, String> {
    // 1. Handle `my_chat_member`: Bot blocked or removed from chat
    if let Some(my_cm) = update.get("my_chat_member") {
        let chat_id = my_cm
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64());
        let new_status = my_cm
            .get("new_chat_member")
            .and_then(|m| m.get("status"))
            .and_then(|s| s.as_str());

        if let (Some(cid), Some(status)) = (chat_id, new_status) {
            if status == "kicked" || status == "left" {
                tracing::info!(
                    chat_id = cid,
                    status = status,
                    "Bot was removed/kicked from chat. Purging all chat FSM sessions."
                );
                // Purge all FSM sessions for this chat_id (all users)
                fsm.clear_chat(cid).await;
            }
        }
        return Ok(true);
    }

    // 2. Handle group chat migration to supergroup (`migrate_to_chat_id`)
    if let Some(msg) = update.get("message") {
        let old_chat_id = msg
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64());
        let new_chat_id = msg.get("migrate_to_chat_id").and_then(|v| v.as_i64());

        if let (Some(old_id), Some(new_id)) = (old_chat_id, new_chat_id) {
            tracing::info!(
                old_chat_id = old_id,
                new_chat_id = new_id,
                "Group chat migrated to supergroup. Updating database records."
            );

            let old_key = format!("lang_{}", old_id);
            let new_key = format!("lang_{}", new_id);

            if let Some(pool) = fsm.pool() {
                if let Ok(conn) = pool.get().await {
                    let _ = conn
                        .interact(move |c| {
                            let _ = c.execute(
                                "UPDATE invoices SET telegram_chat_id = ?1 WHERE telegram_chat_id = ?2 AND status = 'pending'",
                                rusqlite::params![new_id, old_id],
                            );
                            let _ = c.execute(
                                "UPDATE telegram_fsm_sessions SET chat_id = ?1 WHERE chat_id = ?2",
                                rusqlite::params![new_id, old_id],
                            );
                            let _ = c.execute(
                                "UPDATE system_settings SET key = ?1 WHERE key = ?2",
                                rusqlite::params![new_key, old_key],
                            );
                        })
                        .await;
                }
            } else {
                let db_path = config.db_path.clone();
                let _ = tokio::task::spawn_blocking(move || {
                    if let Ok(conn) = db::get_db_connection(&db_path) {
                        let _ = conn.execute(
                            "UPDATE invoices SET telegram_chat_id = ?1 WHERE telegram_chat_id = ?2 AND status = 'pending'",
                            rusqlite::params![new_id, old_id],
                        );
                        let _ = conn.execute(
                            "UPDATE telegram_fsm_sessions SET chat_id = ?1 WHERE chat_id = ?2",
                            rusqlite::params![new_id, old_id],
                        );
                        let _ = conn.execute(
                            "UPDATE system_settings SET key = ?1 WHERE key = ?2",
                            rusqlite::params![new_key, old_key],
                        );
                    }
                })
                .await;
            }

            let notice = crate::domain::sanitizer::escape_telegram_markdown_v2(
                "ℹ️ Group updated to supergroup. Settings migrated successfully.",
            );
            let payload = serde_json::json!({
                "chat_id": new_id,
                "text": notice,
                "parse_mode": "MarkdownV2"
            });
            let _ =
                send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload).await;
            return Ok(true);
        }
    }

    // 3. Handle edited_message
    if let Some(msg) = update.get("edited_message") {
        let chat_id = msg
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64());
        let user_id = msg
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let text = msg.get("text").and_then(|v| v.as_str()).unwrap_or("");
        if let Some(chat_id) = chat_id {
            if text.trim() == "/cancel" {
                fsm.clear(chat_id, user_id).await;
                let payload = serde_json::json!({
                    "chat_id": chat_id,
                    "text": "❌ Action cancelled. Current session reset.",
                });
                let _ =
                    send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload)
                        .await;
            } else {
                let notice = crate::domain::sanitizer::escape_telegram_markdown_v2(
                    "⚠️ Editing previous messages does not modify existing invoices. Use /cancel to reset or type a new amount.",
                );
                let payload = serde_json::json!({
                    "chat_id": chat_id,
                    "text": notice,
                    "parse_mode": "MarkdownV2"
                });
                let _ =
                    send_telegram_request(client, &format!("{}/sendMessage", base_url), &payload)
                        .await;
            }
        }
        return Ok(true);
    }

    Ok(false)
}
