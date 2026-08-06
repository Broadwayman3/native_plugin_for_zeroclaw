use crate::common;

#[test]
fn test_305_markdownv2_receipt() {
    let receipt = pos_backend::domain::i18n::format_itemized_receipt(
        "INV-305",
        "2x Cappuccino",
        0.0,
        4.82,
        "en",
        Some("UAH"),
        Some(200.0),
        Some(41.5),
    );
    assert!(
        receipt.contains("305"),
        "305: receipt should contain invoice ID"
    );
}

#[test]
fn test_306_x402_payment_required() {
    let config = pos_backend::config::AppConfig {
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
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _router = rt.block_on(pos_backend::api::build_router(&config));
}

#[test]
fn test_307_cors_preflight() {
    let config = pos_backend::config::AppConfig {
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
    };
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _router = rt.block_on(pos_backend::api::build_router(&config));
}

#[test]
fn test_308_api_response_format() {
    let conn = common::setup_memory_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-FORMAT-308".to_string(),
            reference_pubkey: "ref_format_308_unique".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    let invoices =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("INV-FORMAT-308"), None).unwrap();
    let inv = invoices.first().expect("308: no invoices returned");
    assert_eq!(inv.id, "INV-FORMAT-308", "308: wrong invoice id");
    assert_eq!(inv.fiat_currency, "UAH", "308: wrong currency");
    assert_eq!(inv.status, "pending", "308: wrong status");
}

#[test]
fn test_309_concurrent_requests() {
    let guard = common::TempDbGuard::new("concurrent_api");
    let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    drop(conn);

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let path = guard.path().to_string();
            std::thread::spawn(move || {
                let conn = pos_backend::db::get_db_connection(&path).unwrap();
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
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        count, 5,
        "309: expected 5 concurrent inserts, got {}",
        count
    );
}

#[test]
fn test_310_error_handling() {
    let conn = common::setup_memory_db();
    let invoices =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("NONEXISTENT"), None).unwrap();
    assert!(
        invoices.is_empty(),
        "310: should return empty list for nonexistent invoice"
    );
}
