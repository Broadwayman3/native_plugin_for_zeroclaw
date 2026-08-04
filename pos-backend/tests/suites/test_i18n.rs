use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 i18n Tests (101-110)");
    test_101_t_function_basic();
    test_102_t_function_with_kwargs();
    test_103_t_function_escape_markdown();
    test_104_t_function_fallback_to_english();
    test_105_get_lang_meta_uk();
    test_106_get_lang_meta_unknown();
    test_107_get_localized_confirmation();
    test_108_get_main_reply_keyboard();
    test_109_get_cancel_invoice_keyboard();
    test_110_get_refund_checkpoint_keyboard();
}

fn test_101_t_function_basic() {
    let r = pos_backend::domain::i18n::t("welcome", Some("en"), &[]);
    if r.contains("ZeroClaw") {
        test_pass("101: welcome message contains ZeroClaw");
    } else {
        test_fail("101", &format!("result: {}", &r[..50.min(r.len())]));
    }
}

fn test_102_t_function_with_kwargs() {
    let r = pos_backend::domain::i18n::t("receipt_title", Some("en"), &[("invoice_id", "123")]);
    if r.contains("123") {
        test_pass("102: kwargs substituted in template");
    } else {
        test_fail("102", &format!("result: {}", r));
    }
}

fn test_103_t_function_escape_markdown() {
    let r = pos_backend::domain::i18n::t("welcome", Some("en"), &[]);
    // MarkdownV2 escaping should add backslashes
    if r.contains("\\") || !r.contains("*") {
        test_pass("103: MarkdownV2 escaping applied");
    } else {
        test_fail("103", "expected escaping");
    }
}

fn test_104_t_function_fallback_to_english() {
    let r = pos_backend::domain::i18n::t("welcome", Some("xx"), &[]);
    let en = pos_backend::domain::i18n::t("welcome", Some("en"), &[]);
    if r == en {
        test_pass("104: unknown lang falls back to English");
    } else {
        test_fail("104", "fallback mismatch");
    }
}

fn test_105_get_lang_meta_uk() {
    let (flag, name) = pos_backend::domain::i18n::get_lang_meta("uk");
    if name.contains("Українська") {
        test_pass("105: Ukrainian language name correct");
    } else {
        test_fail("105", &format!("name: {}", name));
    }
}

fn test_106_get_lang_meta_unknown() {
    let (_, name) = pos_backend::domain::i18n::get_lang_meta("xx");
    if name == "English" {
        test_pass("106: unknown lang returns English");
    } else {
        test_fail("106", &format!("name: {}", name));
    }
}

fn test_107_get_localized_confirmation() {
    let r = pos_backend::domain::i18n::get_localized_confirmation("uk");
    if r.contains("Українська") || r.contains("🇺🇦") {
        test_pass("107: Ukrainian confirmation message");
    } else {
        test_fail("107", &format!("result: {}", r));
    }
}

fn test_108_get_main_reply_keyboard() {
    let kb = pos_backend::domain::i18n::get_main_reply_keyboard("en");
    if kb.get("keyboard").is_some() && kb["keyboard"].is_array() {
        test_pass("108: reply keyboard has keyboard array");
    } else {
        test_fail("108", &format!("kb: {}", kb));
    }
}

fn test_109_get_cancel_invoice_keyboard() {
    let kb = pos_backend::domain::i18n::get_cancel_invoice_inline_keyboard("INV-101", "en");
    let callback = kb["inline_keyboard"][0][0]["callback_data"]
        .as_str()
        .unwrap_or("");
    if callback.contains("cancel_invoice_INV-101") {
        test_pass("109: cancel keyboard callback data correct");
    } else {
        test_fail("109", &format!("callback: {}", callback));
    }
}

fn test_110_get_refund_checkpoint_keyboard() {
    let kb = pos_backend::domain::i18n::get_refund_checkpoint_inline_keyboard(42, "en");
    let approve = kb["inline_keyboard"][0][0]["callback_data"]
        .as_str()
        .unwrap_or("");
    let reject = kb["inline_keyboard"][0][1]["callback_data"]
        .as_str()
        .unwrap_or("");
    if approve.contains("approve_refund_42") && reject.contains("reject_refund_42") {
        test_pass("110: refund keyboard approve/reject correct");
    } else {
        test_fail("110", &format!("approve: {}, reject: {}", approve, reject));
    }
}
