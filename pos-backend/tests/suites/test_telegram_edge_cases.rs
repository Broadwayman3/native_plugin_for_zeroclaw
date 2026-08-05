use crate::common;

#[test]
fn test_380_order_parser_negative_price() {
    let default_label = "Item";
    let parsed_neg1 =
        pos_backend::domain::order_parser::parse_pos_order_input("-50 UAH", default_label, None);
    assert!(
        !parsed_neg1.has_price,
        "380: negative price -50 UAH should be rejected"
    );

    let parsed_neg2 =
        pos_backend::domain::order_parser::parse_pos_order_input("-$20", default_label, None);
    assert!(
        !parsed_neg2.has_price,
        "380: negative price -$20 should be rejected"
    );

    let parsed_neg3 = pos_backend::domain::order_parser::parse_pos_order_input(
        "-2x Latte 40 UAH",
        default_label,
        None,
    );
    assert!(
        !parsed_neg3.has_price,
        "380: negative quantity -2x should be rejected"
    );
}

#[test]
fn test_381_cancel_invoice_idempotent() {
    let conn = common::setup_memory_db();

    // Create pending invoice
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "INV-IDEM-381".to_string(),
        reference_pubkey: "RefKey381111111111111111111111111111111111".to_string(),
        fiat_currency: Some("USD".to_string()),
        fiat_amount: Some(10.0),
        usdc_amount: 10.0,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();

    // First cancel
    let cancelled1 = pos_backend::db::invoices::cancel_invoice(&conn, "INV-IDEM-381").unwrap();
    assert_eq!(cancelled1, 1, "381: first cancel should update 1 row");

    // Second cancel attempt on DB directly returns 0 updated rows
    let cancelled2 = pos_backend::db::invoices::cancel_invoice(&conn, "INV-IDEM-381").unwrap();
    assert_eq!(cancelled2, 0, "381: second cancel should update 0 rows");

    // Verify invoice status in DB
    let inv = pos_backend::db::invoices::get_invoice_by_id(&conn, "INV-IDEM-381")
        .unwrap()
        .unwrap();
    assert_eq!(
        inv.status, "cancelled",
        "381: status should remain cancelled"
    );
}

#[test]
fn test_382_cancel_paid_invoice_conflict() {
    let conn = common::setup_memory_db();

    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "INV-PAID-382".to_string(),
        reference_pubkey: "RefKey382111111111111111111111111111111111".to_string(),
        fiat_currency: Some("USD".to_string()),
        fiat_amount: Some(15.0),
        usdc_amount: 15.0,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-PAID-382", "paid", Some("sig382"))
        .unwrap();

    // Cancel on paid invoice should fail
    let cancelled = pos_backend::db::invoices::cancel_invoice(&conn, "INV-PAID-382").unwrap();
    assert_eq!(
        cancelled, 0,
        "382: cancel on paid invoice should return 0 updated rows"
    );

    let inv = pos_backend::db::invoices::get_invoice_by_id(&conn, "INV-PAID-382")
        .unwrap()
        .unwrap();
    assert_eq!(inv.status, "paid", "382: status should remain paid");
}

#[test]
fn test_383_webhook_secret_token_auth() {
    let auth_config = pos_backend::api::middleware::AuthConfig {
        telegram_bot_secret_token: Some("my_secret_token_123".to_string()),
        api_keys: vec![],
        manager_telegram_id: None,
    };

    assert_eq!(
        auth_config.telegram_bot_secret_token.as_deref(),
        Some("my_secret_token_123"),
        "383: secret token should be stored in AuthConfig"
    );
}

#[test]
fn test_384_receipt_multilingual_rate() {
    let receipt_en = pos_backend::domain::i18n::format_itemized_receipt(
        "INV-384",
        "Coffee",
        0.0,
        5.0,
        "en",
        Some("UAH"),
        Some(200.0),
        Some(40.0),
    );
    assert!(
        receipt_en.contains("Fiat: 200\\.00 UAH"),
        "384: EN receipt should contain Fiat rate"
    );

    let receipt_uk = pos_backend::domain::i18n::format_itemized_receipt(
        "INV-384",
        "Кава",
        0.0,
        5.0,
        "uk",
        Some("UAH"),
        Some(200.0),
        Some(40.0),
    );
    assert!(
        receipt_uk.contains("Фіат: 200\\.00 UAH"),
        "384: UK receipt should contain localized Фіат rate"
    );
}

#[test]
fn test_385_order_parser_decimal_comma() {
    let parsed1 = pos_backend::domain::order_parser::parse_pos_order_input(
        "2x Cappuccino R$ 15,50",
        "Item",
        None,
    );
    assert!(parsed1.has_price, "385: R$ 15,50 should parse with price");
    assert_eq!(parsed1.amount, Some(15.50), "385: amount should be 15.50");
    assert_eq!(
        parsed1.currency.as_deref(),
        Some("BRL"),
        "385: currency should be BRL"
    );

    let parsed2 = pos_backend::domain::order_parser::parse_pos_order_input(
        "1,5x Croissant 40,50 UAH",
        "Item",
        None,
    );
    assert!(parsed2.has_price, "385: 1,5x with 40,50 UAH should parse");
    assert_eq!(
        parsed2.amount,
        Some(40.50),
        "385: line price should be 40.50"
    );
    assert_eq!(
        parsed2.currency.as_deref(),
        Some("UAH"),
        "385: currency should be UAH"
    );

    // Adversarial Check 1: Thousands separator 1,000 USD must not be corrupted to 1.000 USD
    let parsed3 =
        pos_backend::domain::order_parser::parse_pos_order_input("1,000 USD", "Item", None);
    assert!(parsed3.has_price, "385: 1,000 USD should parse with price");
    assert_eq!(
        parsed3.amount,
        Some(1000.0),
        "385: 1,000 USD should parse as 1000.0, NOT 1.0"
    );

    // Adversarial Check 2: Comma list Coffee 1, Tea 2 must not corrupt text
    let norm = pos_backend::domain::order_parser::normalize_numeric_commas("Coffee 1, Tea 2");
    assert_eq!(
        norm, "Coffee 1, Tea 2",
        "385: comma list text must remain intact"
    );
}
