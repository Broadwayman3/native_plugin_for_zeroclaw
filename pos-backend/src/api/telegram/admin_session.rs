use serde_json::Value;

/// Returns true if update is a regular message sent by an anonymous group admin or linked channel forward (where `from` object is missing).
pub fn is_anonymous_admin_message(update: &Value) -> bool {
    if let Some(msg) = update
        .get("message")
        .or_else(|| update.get("edited_message"))
    {
        let has_from = msg.get("from").is_some();
        if !has_from {
            return true;
        }
    }
    false
}

/// Extracts effective user_id and chat_id from update.
/// For callback_query: returns (Some(chat_id), from.id) because callback queries ALWAYS include the real user ID in `from`.
/// For anonymous admin messages: returns (Some(chat_id), 0) to signal stateless handling.
/// For regular user messages: returns (Some(chat_id), from.id).
pub fn extract_effective_user_context(update: &Value) -> (Option<i64>, i64) {
    if let Some(cb) = update.get("callback_query") {
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
        return (cid, uid);
    }

    if let Some(msg) = update
        .get("message")
        .or_else(|| update.get("edited_message"))
    {
        let cid = msg
            .get("chat")
            .and_then(|c| c.get("id"))
            .and_then(|v| v.as_i64());

        if is_anonymous_admin_message(update) {
            return (cid, 0);
        }

        let uid = msg
            .get("from")
            .and_then(|f| f.get("id"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        return (cid, uid);
    }

    (None, 0)
}
