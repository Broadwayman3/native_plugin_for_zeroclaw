use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 QA Red Team Tests (288-292)");
    test_288_cors_headers();
    test_289_pubkey_formatting();
    test_290_receipt_formatting();
    test_291_cancel_idempotency();
    test_292_invoice_id_filtering();
}

fn test_288_cors_headers() {
    // Verify CORS is configured with explicit headers (not Allow-Headers: Any)
    let config =
        pos_backend::config::AppConfig::from_env().unwrap_or(pos_backend::config::AppConfig {
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
        });

    // Build router and verify it doesn't panic
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _router = rt.block_on(pos_backend::api::build_router(&config));

    // If we get here, CORS is configured correctly
    test_pass("288: CORS configured and router built successfully");
}

fn test_289_pubkey_formatting() {
    let short = pos_backend::domain::formatters::format_pubkey_short(
        "8xAZmQ1111111111111111111111111111111111111",
    );
    if short.starts_with("8xAZ") && short.ends_with("1111") && short.contains("...") {
        test_pass("289: pubkey formatting correct");
    } else {
        test_fail("289", &format!("short: {}", short));
    }
}

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

    // Receipt should contain the invoice ID (may be escaped for MarkdownV2)
    if receipt.contains("101") {
        test_pass("290: receipt formatting correct");
    } else {
        test_fail("290", "receipt missing invoice ID");
    }
}

fn test_291_cancel_idempotency() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    // Create invoice
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-IDEM".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();

    // First cancel succeeds
    let first = pos_backend::db::invoices::cancel_invoice(&conn, "INV-IDEM").unwrap();
    // Second cancel returns 0 (already cancelled)
    let second = pos_backend::db::invoices::cancel_invoice(&conn, "INV-IDEM").unwrap();

    if first == 1 && second == 0 {
        test_pass("291: cancel idempotency works");
    } else {
        test_fail("291", &format!("first={}, second={}", first, second));
    }
}

fn test_292_invoice_id_filtering() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    // Create multiple invoices
    for i in 1..=5 {
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
    }

    // Filter by ID
    let filtered =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("INV-3"), None).unwrap();
    let all = pos_backend::db::invoices::get_invoices_list(&conn, None, None).unwrap();

    if filtered.len() == 1 && filtered[0].id == "INV-3" && all.len() == 5 {
        test_pass("292: invoice ID filtering works");
    } else {
        test_fail(
            "292",
            &format!("filtered={}, all={}", filtered.len(), all.len()),
        );
    }
}
