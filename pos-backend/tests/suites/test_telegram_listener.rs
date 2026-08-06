use crate::common;
use pos_backend::api::telegram::state::{get_user_lang, set_user_lang};
use pos_backend::domain::i18n::get_main_reply_keyboard;
use pos_backend::domain::keyboards::is_btn_click;
use pos_backend::domain::sanitizer::escape_telegram_markdown_v2;
use pos_backend::domain::verification::verify_solana_transaction_with_reference;

#[test]
fn test_telegram_language_state_persistence() {
    let guard = common::TempDbGuard::new("tg_state");
    let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
    pos_backend::db::init_db(&conn, false).unwrap();

    let chat_id = 987654321;
    assert_eq!(get_user_lang(guard.path(), chat_id), "en");

    set_user_lang(guard.path(), chat_id, "uk");
    assert_eq!(get_user_lang(guard.path(), chat_id), "uk");

    set_user_lang(guard.path(), chat_id, "de");
    assert_eq!(get_user_lang(guard.path(), chat_id), "de");
}

#[test]
fn test_telegram_reply_keyboard_multilingual_generation() {
    let uk_kbd = get_main_reply_keyboard("uk", 200.0, "UAH");
    let uk_str = serde_json::to_string(&uk_kbd).unwrap();
    assert!(uk_str.contains("Ввести довільну суму") || uk_str.contains("Швидкий чек"));

    let de_kbd = get_main_reply_keyboard("de", 200.0, "UAH");
    let de_str = serde_json::to_string(&de_kbd).unwrap();
    assert!(de_str.contains("Betrag eingeben") || de_str.contains("Sprachen"));

    let pt_kbd = get_main_reply_keyboard("pt", 200.0, "UAH");
    let pt_str = serde_json::to_string(&pt_kbd).unwrap();
    assert!(pt_str.contains("Digitar valor") || pt_str.contains("Idiomas"));
}

#[test]
fn test_telegram_button_click_matching_across_languages() {
    assert!(is_btn_click("🌐 Languages (13)", "btn_lang"));
    assert!(is_btn_click("🌐 Idiomas (13)", "btn_lang"));
    assert!(is_btn_click("✍️ Digitar valor personalizado", "btn_custom"));
    assert!(is_btn_click("🔄 Reembolso", "btn_refund"));
}

#[test]
fn test_telegram_markdown_escaping_precision() {
    let item = "2x Cappuccino + Croissant";
    let esc = escape_telegram_markdown_v2(item);
    assert_eq!(esc, "2x Cappuccino \\+ Croissant");

    let text_template = format!("*Total:* {}", esc);
    assert!(text_template.starts_with("*Total:*"));
    assert!(text_template.contains("\\+"));
}

#[test]
fn test_telegram_invoice_schema_migrations() {
    let guard = common::TempDbGuard::new("tg_schema");
    let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
    pos_backend::db::init_db(&conn, false).unwrap();

    let inv_id = "INV-TEST-TG-101";
    let ref_key = "RefKey111111111111111111111111111111111111";

    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: inv_id.to_string(),
            reference_pubkey: ref_key.to_string(),
            fiat_currency: Some("USD".into()),
            fiat_amount: Some(10.0),
            usdc_amount: 10.0,
            telegram_chat_id: Some(123456789),
            telegram_msg_id: Some(445566),
        },
    )
    .unwrap();

    let inv = pos_backend::db::invoices::get_invoice_by_id(&conn, inv_id)
        .unwrap()
        .unwrap();
    assert_eq!(inv.telegram_chat_id, Some(123456789));
    assert_eq!(inv.telegram_msg_id, Some(445566));
    assert_eq!(inv.telegram_expired_notified, Some(0));

    pos_backend::db::invoices::update_invoice_telegram_context(&conn, inv_id, 987654321, 998877)
        .unwrap();
    pos_backend::db::invoices::mark_invoice_expired_notified(&conn, inv_id).unwrap();

    let updated = pos_backend::db::invoices::get_invoice_by_id(&conn, inv_id)
        .unwrap()
        .unwrap();
    assert_eq!(updated.telegram_chat_id, Some(987654321));
    assert_eq!(updated.telegram_msg_id, Some(998877));
    assert_eq!(updated.telegram_expired_notified, Some(1));
}

#[test]
fn test_cross_invoice_reference_key_isolation() {
    let merchant_ata = "MerchantATA111111111111111111111111111111111";
    let usdc_mint = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";

    let ref_key_a = "RefKeyAAAAA111111111111111111111111111111111";
    let ref_key_b = "RefKeyBBBBB222222222222222222222222222222222";

    // Transaction containing ONLY ref_key_a
    let tx_for_a = serde_json::json!({
        "meta": {
            "err": null,
            "preTokenBalances": [{"accountIndex": 0, "mint": usdc_mint, "uiTokenAmount": {"amount": "0"}}],
            "postTokenBalances": [{"accountIndex": 0, "mint": usdc_mint, "uiTokenAmount": {"amount": "10000000"}}]
        },
        "transaction": {
            "message": {
                "accountKeys": [
                    {"pubkey": merchant_ata},
                    {"pubkey": ref_key_a}
                ]
            }
        }
    });

    // 1. Verify transaction passes for Invoice A (ref_key_a)
    let res_a = verify_solana_transaction_with_reference(
        &tx_for_a,
        merchant_ata,
        10_000_000,
        usdc_mint,
        Some(ref_key_a),
    );
    assert_eq!(res_a.get("is_valid").and_then(|v| v.as_bool()), Some(true));

    // 2. Verify transaction FAILS for Invoice B (ref_key_b) — Guards against double spend / cross-invoice false positive!
    let res_b = verify_solana_transaction_with_reference(
        &tx_for_a,
        merchant_ata,
        10_000_000,
        usdc_mint,
        Some(ref_key_b),
    );
    assert_eq!(res_b.get("is_valid").and_then(|v| v.as_bool()), Some(false));
    assert!(res_b
        .get("error")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("Reference key missing"));
}

#[test]
fn test_system_settings_persistence() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    // 1. Initial state is None
    let val =
        pos_backend::db::invoices::get_system_setting(&conn, "solana_last_seen_signature").unwrap();
    assert_eq!(val, None);

    // 2. Set signature and retrieve across server restart simulation
    pos_backend::db::invoices::set_system_setting(
        &conn,
        "solana_last_seen_signature",
        "5Pbug4Lm...",
    )
    .unwrap();
    let restored =
        pos_backend::db::invoices::get_system_setting(&conn, "solana_last_seen_signature").unwrap();
    assert_eq!(restored, Some("5Pbug4Lm...".to_string()));
}
