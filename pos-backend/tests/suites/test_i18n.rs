use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 i18n Tests (101-111)");
    test_101_t_function_basic();
    test_102_t_function_with_kwargs();
    test_103_t_function_escape_markdown();
    test_103b_t_function_bold_preserved();
    test_103c_t_function_kwargs_substituted();
    test_103d_t_function_complex_template();
    test_104_t_function_fallback_to_english();
    test_105_get_lang_meta_uk();
    test_106_get_lang_meta_unknown();
    test_107_get_localized_confirmation();
    test_108_get_main_reply_keyboard();
    test_109_get_cancel_invoice_keyboard();
    test_110_get_refund_checkpoint_keyboard();
    test_111_format_itemized_receipt();
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
    // After fix: bold markers preserved, template special chars escaped
    if r.contains("*Welcome") && r.contains("\\(") {
        test_pass("103: bold preserved, parens escaped");
    } else {
        test_fail("103", &format!("result: {}", &r[..120.min(r.len())]));
    }
}

fn test_103b_t_function_bold_preserved() {
    let r = pos_backend::domain::i18n::t("welcome", Some("en"), &[]);
    // Bold markers must be preserved (not escaped to \*...\*)
    let has_bold = r.contains("*Welcome") && r.contains("Terminal");
    // Backticks must be preserved (not escaped to \`...\`)
    let has_code = r.contains("`150 UAH`") || r.contains("`35.5 BRL`");
    if has_bold && has_code {
        test_pass("103b: bold and code formatting preserved");
    } else {
        test_fail("103b", &format!("bold={}, code={}", has_bold, has_code));
    }
}

fn test_103c_t_function_kwargs_substituted() {
    let r = pos_backend::domain::i18n::t("price_needed", Some("en"), &[("items", "2x Cappuccino")]);
    // Kwargs must be substituted (escaped: spaces become \s... no, spaces aren't escaped)
    // But the kwarg value itself is escaped: "2x Cappuccino" has no special chars
    if r.contains("2x Cappuccino") && !r.contains("{items}") {
        test_pass("103c: kwargs substituted correctly");
    } else {
        test_fail(
            "103c",
            &format!("kwargs not substituted: {}", &r[..120.min(r.len())]),
        );
    }
}

fn test_103d_t_function_complex_template() {
    let r = pos_backend::domain::i18n::t(
        "squads_refund_initiated",
        Some("en"),
        &[
            ("invoice_id", "INV-T1"),
            ("amount_usdc", "10.00"),
            ("proposal_index", "42"),
        ],
    );
    // Bold markers preserved
    let has_bold = r.contains("*Squads") && r.contains("USDC*");
    // Kwargs substituted (values are escaped: "-" → "\-", "." → "\.")
    let has_invoice = r.contains("INV\\-T1");
    let has_amount = r.contains("10\\.00 USDC");
    let has_proposal = r.contains("\\#42");
    if has_bold && has_invoice && has_amount && has_proposal {
        test_pass("103d: complex template with bold + kwargs works");
    } else {
        test_fail(
            "103d",
            &format!(
                "bold={}, invoice={}, amount={}, proposal={}",
                has_bold, has_invoice, has_amount, has_proposal
            ),
        );
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
    let (_flag, name) = pos_backend::domain::i18n::get_lang_meta("uk");
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

fn test_111_format_itemized_receipt() {
    let receipt = pos_backend::domain::i18n::format_itemized_receipt(
        "INV-TEST",
        "2x Cappuccino; Croissant",
        10.0,
        5.0,
        "en",
        Some("UAH"),
        Some(200.0),
        Some(41.5),
    );
    // Values are escaped: "-" → "\-", "." → "\."
    let has_id = receipt.contains("INV\\-TEST");
    let has_items = receipt.contains("2x Cappuccino") && receipt.contains("Croissant");
    let has_total = receipt.contains("5\\.00");
    let has_fiat =
        receipt.contains("UAH") && receipt.contains("200\\.00") && receipt.contains("41\\.50");
    if has_id && has_items && has_total && has_fiat {
        test_pass("111: format_itemized_receipt renders all fields");
    } else {
        test_fail(
            "111",
            &format!(
                "id={}, items={}, total={}, fiat={}",
                has_id, has_items, has_total, has_fiat
            ),
        );
    }
}
