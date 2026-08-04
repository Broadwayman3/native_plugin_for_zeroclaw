#[test]
fn test_101_t_function_basic() {
    let r = pos_backend::domain::i18n::t("welcome", Some("en"), &[]);
    assert!(r.contains("ZeroClaw"), "result: {}", &r[..50.min(r.len())]);
}

#[test]
fn test_102_t_function_with_kwargs() {
    let r = pos_backend::domain::i18n::t("receipt_title", Some("en"), &[("invoice_id", "123")]);
    assert!(r.contains("123"), "result: {}", r);
}

#[test]
fn test_103_t_function_escape_markdown() {
    let r = pos_backend::domain::i18n::t("welcome", Some("en"), &[]);
    // After fix: bold markers preserved, template special chars escaped
    assert!(
        r.contains("*Welcome") && r.contains("\\("),
        "result: {}",
        &r[..120.min(r.len())]
    );
}

#[test]
fn test_103b_t_function_bold_preserved() {
    let r = pos_backend::domain::i18n::t("welcome", Some("en"), &[]);
    // Bold markers must be preserved (not escaped to \*...\*)
    let has_bold = r.contains("*Welcome") && r.contains("Terminal");
    // Backticks must be preserved (not escaped to \`...\`)
    let has_code = r.contains("`150 UAH`") || r.contains("`35.5 BRL`");
    assert!(has_bold && has_code, "bold={}, code={}", has_bold, has_code);
}

#[test]
fn test_103c_t_function_kwargs_substituted() {
    let r = pos_backend::domain::i18n::t("price_needed", Some("en"), &[("items", "2x Cappuccino")]);
    // Kwargs must be substituted (escaped: spaces become \s... no, spaces aren't escaped)
    // But the kwarg value itself is escaped: "2x Cappuccino" has no special chars
    assert!(
        r.contains("2x Cappuccino") && !r.contains("{items}"),
        "kwargs not substituted: {}",
        &r[..120.min(r.len())]
    );
}

#[test]
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
    assert!(
        has_bold && has_invoice && has_amount && has_proposal,
        "bold={}, invoice={}, amount={}, proposal={}",
        has_bold,
        has_invoice,
        has_amount,
        has_proposal
    );
}

#[test]
fn test_104_t_function_fallback_to_english() {
    let r = pos_backend::domain::i18n::t("welcome", Some("xx"), &[]);
    let en = pos_backend::domain::i18n::t("welcome", Some("en"), &[]);
    assert_eq!(r, en, "fallback mismatch");
}

#[test]
fn test_105_get_lang_meta_uk() {
    let (_flag, name) = pos_backend::domain::i18n::get_lang_meta("uk");
    assert!(name.contains("Українська"), "name: {}", name);
}

#[test]
fn test_106_get_lang_meta_unknown() {
    let (_, name) = pos_backend::domain::i18n::get_lang_meta("xx");
    assert_eq!(name, "English", "name: {}", name);
}

#[test]
fn test_107_get_localized_confirmation() {
    let r = pos_backend::domain::i18n::get_localized_confirmation("uk");
    assert!(
        r.contains("Українська") || r.contains("🇺🇦"),
        "result: {}",
        r
    );
}

#[test]
fn test_108_get_main_reply_keyboard() {
    let kb = pos_backend::domain::i18n::get_main_reply_keyboard("en", 200.0, "UAH");
    assert!(
        kb.get("keyboard").is_some() && kb["keyboard"].is_array(),
        "kb: {}",
        kb
    );
    // Verify quick receipt button contains configured amount and currency
    let btn_text = kb["keyboard"][0][1]["text"].as_str().unwrap_or("");
    assert!(
        btn_text.contains("200") && btn_text.contains("UAH"),
        "quick btn: {}",
        btn_text
    );
}

#[test]
fn test_109_get_cancel_invoice_keyboard() {
    let kb = pos_backend::domain::i18n::get_cancel_invoice_inline_keyboard("INV-101", "en");
    let callback = kb["inline_keyboard"][0][0]["callback_data"]
        .as_str()
        .unwrap_or("");
    assert!(
        callback.contains("cancel_invoice_INV-101"),
        "callback: {}",
        callback
    );
}

#[test]
fn test_110_get_refund_checkpoint_keyboard() {
    let kb = pos_backend::domain::i18n::get_refund_checkpoint_inline_keyboard(42, "en");
    let approve = kb["inline_keyboard"][0][0]["callback_data"]
        .as_str()
        .unwrap_or("");
    let reject = kb["inline_keyboard"][0][1]["callback_data"]
        .as_str()
        .unwrap_or("");
    assert!(
        approve.contains("approve_refund_42") && reject.contains("reject_refund_42"),
        "approve: {}, reject: {}",
        approve,
        reject
    );
}

#[test]
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
    assert!(
        has_id && has_items && has_total && has_fiat,
        "id={}, items={}, total={}, fiat={}",
        has_id,
        has_items,
        has_total,
        has_fiat
    );
}
