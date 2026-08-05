#[test]
fn test_326_lang_keyboard_4_per_row() {
    let kb = pos_backend::domain::keyboards::generate_lang_inline_keyboard();
    let rows = kb["inline_keyboard"]
        .as_array()
        .expect("326: no inline_keyboard array");
    let total_buttons: usize = rows
        .iter()
        .map(|r| r.as_array().map_or(0, |a| a.len()))
        .sum();
    assert_eq!(
        total_buttons, 13,
        "326: expected 13 buttons, got {}",
        total_buttons
    );
}

#[test]
fn test_327_lang_keyboard_all_langs() {
    let kb = pos_backend::domain::keyboards::generate_lang_inline_keyboard();
    let expected = [
        "ar", "de", "en", "es", "fr", "hi", "it", "ja", "pl", "pt", "tr", "uk", "zh",
    ];
    let mut found = Vec::new();
    if let Some(rows) = kb["inline_keyboard"].as_array() {
        for row in rows {
            if let Some(btns) = row.as_array() {
                for btn in btns {
                    if let Some(cd) = btn["callback_data"].as_str() {
                        if let Some(code) = cd.strip_prefix("set_lang_") {
                            found.push(code.to_string());
                        }
                    }
                }
            }
        }
    }
    found.sort();
    assert_eq!(
        found, expected,
        "327: expected 13 languages {:?}, found {:?}",
        expected, found
    );
}

#[test]
fn test_328_build_send_message_basic() {
    let payload =
        pos_backend::domain::keyboards::build_send_message_payload(12345, "hello", None, None);
    assert_eq!(
        payload["text"].as_str().unwrap(),
        "hello",
        "328: wrong text"
    );
    assert_eq!(
        payload["chat_id"].as_i64().unwrap(),
        12345,
        "328: wrong chat_id"
    );
    assert!(
        payload.get("parse_mode").is_none(),
        "328: parse_mode should not exist"
    );
}

#[test]
fn test_329_build_send_message_with_parse_mode() {
    let payload = pos_backend::domain::keyboards::build_send_message_payload(
        1,
        "hello",
        Some("MarkdownV2"),
        None,
    );
    assert_eq!(
        payload["parse_mode"].as_str().unwrap(),
        "MarkdownV2",
        "329: wrong parse_mode"
    );
}

#[test]
fn test_330_build_send_message_with_reply_markup() {
    let markup = serde_json::json!({"inline_keyboard": []});
    let payload =
        pos_backend::domain::keyboards::build_send_message_payload(1, "hello", None, Some(&markup));
    assert!(
        payload.get("reply_markup").is_some(),
        "330: reply_markup missing"
    );
    assert!(
        payload["reply_markup"]["inline_keyboard"].is_array(),
        "330: inline_keyboard not array"
    );
}

#[test]
fn test_331_build_answer_callback_basic() {
    let payload =
        pos_backend::domain::keyboards::build_answer_callback_payload("cb_123", "ok", false);
    assert_eq!(
        payload["callback_query_id"].as_str().unwrap(),
        "cb_123",
        "331: wrong cb_id"
    );
    assert_eq!(payload["text"].as_str().unwrap(), "ok", "331: wrong text");
    assert!(
        payload.get("show_alert").is_none(),
        "331: show_alert should not exist"
    );
}

#[test]
fn test_332_build_answer_callback_show_alert() {
    let payload =
        pos_backend::domain::keyboards::build_answer_callback_payload("cb_456", "alert", true);
    assert!(
        payload["show_alert"].as_bool().unwrap(),
        "332: show_alert should be true"
    );
}

#[test]
fn test_333_is_btn_click_match() {
    let matched = pos_backend::domain::keyboards::is_btn_click("✅ Схвалити", "btn_approve");
    assert!(matched, "333: Ukrainian btn_approve should match");
}

#[test]
fn test_334_is_btn_click_no_match() {
    let matched = pos_backend::domain::keyboards::is_btn_click("Hello", "btn_approve");
    assert!(!matched, "334: unrelated text should not match");
}

#[test]
fn test_335_refund_keyboard_localized() {
    let kb = pos_backend::domain::i18n::get_refund_checkpoint_inline_keyboard(42, "uk");
    let approve = kb["inline_keyboard"][0][0]["text"].as_str().unwrap();
    let reject = kb["inline_keyboard"][0][1]["text"].as_str().unwrap();
    assert!(
        approve.contains("Схвалити"),
        "335: approve should contain Ukrainian text"
    );
    assert!(
        reject.contains("Відхилити"),
        "335: reject should contain Ukrainian text"
    );
}
