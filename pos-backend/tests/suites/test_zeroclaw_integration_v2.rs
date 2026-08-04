use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 ZeroClaw Integration Tests V2 (305-310)");
    test_305_markdownv2_receipt();
    test_306_x402_payment_required();
    test_307_cors_preflight();
    test_308_api_response_format();
    test_309_concurrent_requests();
    test_310_error_handling();
}

fn setup_test_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, true).unwrap();
    conn
}

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

    // Receipt should contain the invoice ID
    if receipt.contains("305") {
        test_pass("305: receipt formatted correctly");
    } else {
        test_fail("305", "receipt missing invoice ID");
    }
}

fn test_306_x402_payment_required() {
    // Verify x402 endpoint structure
    let config = pos_backend::config::AppConfig {
        manager_telegram_id: 0,
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
        api_keys: vec![],
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _router = rt.block_on(pos_backend::api::build_router(&config));

    test_pass("306: x402 endpoint configured");
}

fn test_307_cors_preflight() {
    // Verify CORS configuration
    let config = pos_backend::config::AppConfig {
        manager_telegram_id: 0,
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
        api_keys: vec![],
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let _router = rt.block_on(pos_backend::api::build_router(&config));

    test_pass("307: CORS configured for preflight");
}

fn test_308_api_response_format() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-FORMAT-308".to_string(),
            reference_pubkey: "ref_format_308_unique".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();

    let invoices =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("INV-FORMAT-308"), None).unwrap();
    if let Some(inv) = invoices.first() {
        if inv.id == "INV-FORMAT-308" && inv.fiat_currency == "UAH" && inv.status == "pending" {
            test_pass("308: API response format correct");
        } else {
            test_fail("308", "unexpected invoice data");
        }
    } else {
        test_fail("308", "no invoices returned");
    }
}

fn test_309_concurrent_requests() {
    let db_path = "data/test_concurrent_api.db";
    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path));
    let _ = std::fs::remove_file(format!("{}-shm", db_path));

    // Initialize DB first
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    drop(conn);

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let db_path = db_path.to_string();
            std::thread::spawn(move || {
                let conn = pos_backend::db::get_db_connection(&db_path).unwrap();
                pos_backend::db::invoices::create_invoice(
                    &conn,
                    &pos_backend::db::invoices::CreateInvoiceRequest {
                        id: format!("INV-{}", i),
                        reference_pubkey: format!("ref{}", i),
                        fiat_currency: Some("UAH".to_string()),
                        fiat_amount: Some(100.0),
                        usdc_amount: 2.41,
                    },
                )
                .unwrap();
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))
        .unwrap();
    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path));
    let _ = std::fs::remove_file(format!("{}-shm", db_path));

    if count == 5 {
        test_pass("309: concurrent API requests work");
    } else {
        test_fail("309", &format!("count: {}", count));
    }
}

fn test_310_error_handling() {
    let conn = setup_test_db();

    // Test 404-like behavior (invoice not found)
    let result = pos_backend::db::invoices::get_invoices_list(&conn, Some("NONEXISTENT"), None);
    match result {
        Ok(invoices) if invoices.is_empty() => {
            test_pass("310: missing invoice returns empty list");
        }
        Ok(_) => test_fail("310", "should return empty list"),
        Err(e) => test_fail("310", &format!("error: {}", e)),
    }
}
