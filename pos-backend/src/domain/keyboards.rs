use crate::domain::i18n_strings::LANG_META;

/// Generates Telegram inline keyboard with 13 language buttons (4 per row).
pub fn generate_lang_inline_keyboard() -> serde_json::Value {
    let mut rows = Vec::new();
    let mut row = Vec::new();

    for (code, (flag, _)) in LANG_META.iter() {
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
pub fn is_btn_click(text: &str, key: &str) -> bool {
    let text_clean = text.trim().to_lowercase();
    use crate::domain::i18n_strings::TRANSLATIONS;

    for (_, trans_dict) in TRANSLATIONS.iter() {
        if let Some(target) = trans_dict.get(key) {
            let target_clean = target.trim().to_lowercase();
            if text_clean == target_clean || text_clean.contains(&target_clean) {
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
