use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Keyboards Tests (326-335)");
    test_326_lang_keyboard_4_per_row();
    test_327_lang_keyboard_all_langs();
    test_328_build_send_message_basic();
    test_329_build_send_message_with_parse_mode();
    test_330_build_send_message_with_reply_markup();
    test_331_build_answer_callback_basic();
    test_332_build_answer_callback_show_alert();
    test_333_is_btn_click_match();
    test_334_is_btn_click_no_match();
    test_335_refund_keyboard_localized();
}

fn test_326_lang_keyboard_4_per_row() {
    let kb = pos_backend::domain::keyboards::generate_lang_inline_keyboard();
    let rows = kb["inline_keyboard"].as_array();
    if let Some(rows) = rows {
        let total_buttons: usize = rows
            .iter()
            .map(|r| r.as_array().map_or(0, |a| a.len()))
            .sum();
        let all_four = rows
            .iter()
            .all(|r| r.as_array().map_or(false, |a| a.len() == 4))
            || rows
                .last()
                .map_or(false, |r| r.as_array().map_or(false, |a| a.len() <= 4));
        if total_buttons == 13 && all_four {
            test_pass("326: 13 buttons in rows of 4");
        } else {
            test_fail("326", &format!("total={}", total_buttons));
        }
    } else {
        test_fail("326", "no inline_keyboard array");
    }
}

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
    if found.len() == 13 && found.iter().zip(expected.iter()).all(|(a, b)| a == *b) {
        test_pass("327: all 13 language codes present");
    } else {
        test_fail("327", &format!("found: {:?}", found));
    }
}

fn test_328_build_send_message_basic() {
    let payload =
        pos_backend::domain::keyboards::build_send_message_payload(12345, "hello", None, None);
    let text = payload["text"].as_str().unwrap_or("");
    let chat_id = payload["chat_id"].as_i64().unwrap_or(0);
    if text == "hello" && chat_id == 12345 && payload.get("parse_mode").is_none() {
        test_pass("328: basic sendMessage payload correct");
    } else {
        test_fail("328", &format!("payload: {}", payload));
    }
}

fn test_329_build_send_message_with_parse_mode() {
    let payload = pos_backend::domain::keyboards::build_send_message_payload(
        1,
        "hello",
        Some("MarkdownV2"),
        None,
    );
    let mode = payload["parse_mode"].as_str().unwrap_or("");
    if mode == "MarkdownV2" {
        test_pass("329: parse_mode included");
    } else {
        test_fail("329", &format!("parse_mode: {}", mode));
    }
}

fn test_330_build_send_message_with_reply_markup() {
    let markup = serde_json::json!({"inline_keyboard": []});
    let payload =
        pos_backend::domain::keyboards::build_send_message_payload(1, "hello", None, Some(&markup));
    if payload.get("reply_markup").is_some()
        && payload["reply_markup"]["inline_keyboard"].is_array()
    {
        test_pass("330: reply_markup included");
    } else {
        test_fail("330", &format!("payload: {}", payload));
    }
}

fn test_331_build_answer_callback_basic() {
    let payload =
        pos_backend::domain::keyboards::build_answer_callback_payload("cb_123", "ok", false);
    let cb_id = payload["callback_query_id"].as_str().unwrap_or("");
    let text = payload["text"].as_str().unwrap_or("");
    let has_alert = payload.get("show_alert").is_some();
    if cb_id == "cb_123" && text == "ok" && !has_alert {
        test_pass("331: basic answerCallbackQuery payload");
    } else {
        test_fail("331", &format!("payload: {}", payload));
    }
}

fn test_332_build_answer_callback_show_alert() {
    let payload =
        pos_backend::domain::keyboards::build_answer_callback_payload("cb_456", "alert", true);
    let show_alert = payload["show_alert"].as_bool().unwrap_or(false);
    if show_alert {
        test_pass("332: show_alert=true set correctly");
    } else {
        test_fail("332", &format!("payload: {}", payload));
    }
}

fn test_333_is_btn_click_match() {
    // is_btn_click checks if user text matches a translation across all languages
    // The Ukrainian translation of btn_approve is "✅ Схвалити" (with emoji)
    let matched = pos_backend::domain::keyboards::is_btn_click("✅ Схвалити", "btn_approve");
    if matched {
        test_pass("333: Ukrainian btn_approve matched");
    } else {
        test_fail("333", "expected match for Ukrainian approve");
    }
}

fn test_334_is_btn_click_no_match() {
    let matched = pos_backend::domain::keyboards::is_btn_click("Hello", "btn_approve");
    if !matched {
        test_pass("334: unrelated text does not match");
    } else {
        test_fail("334", "unexpected match");
    }
}

fn test_335_refund_keyboard_localized() {
    let kb = pos_backend::domain::i18n::get_refund_checkpoint_inline_keyboard(42, "uk");
    let approve = kb["inline_keyboard"][0][0]["text"].as_str().unwrap_or("");
    let reject = kb["inline_keyboard"][0][1]["text"].as_str().unwrap_or("");
    if approve.contains("Схвалити") && reject.contains("Відхилити") {
        test_pass("335: Ukrainian refund keyboard labels correct");
    } else {
        test_fail("335", &format!("approve: {}, reject: {}", approve, reject));
    }
}
