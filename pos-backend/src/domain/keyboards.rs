use crate::domain::i18n_strings::LANG_META;

/// Generates Telegram inline keyboard with 13 language buttons (4 per row).
pub fn generate_lang_inline_keyboard() -> serde_json::Value {
    let mut rows = Vec::new();
    let mut row = Vec::new();

    // Use sorted keys for deterministic button order
    let mut sorted_keys: Vec<&str> = LANG_META.keys().copied().collect();
    sorted_keys.sort();

    for code in sorted_keys {
        let (flag, _) = LANG_META[code];
        row.push(serde_json::json!({
            "text": format!("{} {}", flag, code.to_uppercase()),
            "callback_data": format!("set_lang_{}", code)
        }));

        if row.len() == 4 {
            rows.push(serde_json::Value::Array(row));
            row = Vec::new();
        }
    }

    if !row.is_empty() {
        rows.push(serde_json::Value::Array(row));
    }

    serde_json::json!({
        "inline_keyboard": rows
    })
}

/// Checks if user text matches any translation of a given button key across all 13 languages.
///
/// NOTE: User text must start with the full button label (including emoji) to match.
/// Partial text like "I don't approve" will NOT match "✅ Approve".
/// Text without emoji like "Approve" will NOT match "✅ Approve".
/// This is intentional — prevents false positives on substring matches.
pub fn is_btn_click(text: &str, key: &str) -> bool {
    let text_clean = text.trim().to_lowercase();
    use crate::domain::i18n_strings::TRANSLATIONS;

    for trans_dict in TRANSLATIONS.values() {
        if let Some(target) = trans_dict.get(key) {
            let target_clean = target.trim().to_lowercase();
            if text_clean == target_clean || text_clean.starts_with(&target_clean) {
                return true;
            }
        }
    }
    false
}

/// Builds a Telegram sendMessage JSON payload.
pub fn build_send_message_payload(
    chat_id: i64,
    text: &str,
    parse_mode: Option<&str>,
    reply_markup: Option<&serde_json::Value>,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text
    });

    if let Some(mode) = parse_mode {
        payload["parse_mode"] = serde_json::Value::String(mode.to_string());
    }

    if let Some(markup) = reply_markup {
        payload["reply_markup"] = markup.clone();
    }

    payload
}

/// Builds a Telegram answerCallbackQuery JSON payload.
pub fn build_answer_callback_payload(
    callback_query_id: &str,
    text: &str,
    show_alert: bool,
) -> serde_json::Value {
    let mut payload = serde_json::json!({
        "callback_query_id": callback_query_id,
        "text": text
    });

    if show_alert {
        payload["show_alert"] = serde_json::Value::Bool(true);
    }

    payload
}

/// Builds a Telegram getUpdates JSON payload for long-polling.
pub fn build_get_updates_payload(offset: i64, timeout: i64) -> serde_json::Value {
    serde_json::json!({
        "offset": offset,
        "timeout": timeout
    })
}
