#[test]
fn test_288_cors_headers() {
    let config =
        pos_backend::config::AppConfig::from_env().unwrap_or(pos_backend::config::AppConfig {
            manager_telegram_id: 0,
            telegram_bot_token: String::new(),
            merchant_wallet_pubkey: "test".to_string(),
            solana_rpc_url: "https://api.devnet.solana.com".to_string(),
            fallback_rpc_url: "https://api.devnet.solana.com".to_string(),
            usdc_mint_address: "test".to_string(),
            nonce_account_pubkey: "test".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            db_path: ":memory:".to_string(),
            rate_limit_rps: 100,
            telegram_bot_secret_token: None,
            telegram_webhook_url: None,
            api_keys: vec![],
            quick_receipt_amount: 200.0,
            quick_receipt_currency: "UAH".into(),
            allow_local_rpc: false,
            stale_update_ttl_secs: 300,
        });
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _router = rt.block_on(pos_backend::api::build_router(&config));
}

#[test]
fn test_289_pubkey_formatting() {
    let short = pos_backend::domain::formatters::format_pubkey_short(
        "8xAZmQ1111111111111111111111111111111111111",
    );
    assert!(short.starts_with("8xAZ"), "289: should start with 8xAZ");
    assert!(short.ends_with("1111"), "289: should end with 1111");
    assert!(short.contains("..."), "289: should contain ...");
}

#[test]
fn test_290_receipt_formatting() {
    let receipt = pos_backend::domain::i18n::format_itemized_receipt(
        "INV-101",
        "2x Cappuccino",
        0.0,
        4.82,
        "en",
        Some("UAH"),
        Some(200.0),
        Some(41.5),
    );
    assert!(
        receipt.contains("101"),
        "290: receipt should contain invoice ID"
    );
}

#[test]
fn test_291_cancel_idempotency() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-IDEM".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    let first = pos_backend::db::invoices::cancel_invoice(&conn, "INV-IDEM").unwrap();
    let second = pos_backend::db::invoices::cancel_invoice(&conn, "INV-IDEM").unwrap();
    assert_eq!(first, 1, "291: first cancel should affect 1 row");
    assert_eq!(second, 0, "291: second cancel should affect 0 rows");
}

#[test]
fn test_292_invoice_id_filtering() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    for i in 1..=5 {
        pos_backend::db::invoices::create_invoice(
            &conn,
            &pos_backend::db::invoices::CreateInvoiceRequest {
                id: format!("INV-{}", i),
                reference_pubkey: format!("ref{}", i),
                fiat_currency: Some("UAH".to_string()),
                fiat_amount: Some(100.0),
                usdc_amount: 2.41,
                telegram_chat_id: None,
                telegram_msg_id: None,
            },
        )
        .unwrap();
    }
    let filtered =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("INV-3"), None).unwrap();
    let all = pos_backend::db::invoices::get_invoices_list(&conn, None, None).unwrap();
    assert_eq!(filtered.len(), 1, "292: filtered should have 1 result");
    assert_eq!(filtered[0].id, "INV-3", "292: filtered should be INV-3");
    assert_eq!(all.len(), 5, "292: all should have 5 results");
}
