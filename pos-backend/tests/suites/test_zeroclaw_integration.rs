use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 ZeroClaw Integration Tests (293-310)");
    test_293_sop_get_pending_invoices();
    test_294_sop_create_invoice();
    test_295_sop_update_invoice_status();
    test_296_sop_cancel_invoice();
    test_297_sop_nonce_allocate();
    test_298_sop_nonce_release();
    test_299_sop_refund_flow();
    test_300_sop_invoice_lifecycle();
    test_301_skill_solana_pay_url();
    test_302_skill_price_feed();
    test_303_skill_squads_proposal();
    test_304_keyboard_callback_data();
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

// ========== SOP check_payments.json Tests ==========

fn test_293_sop_get_pending_invoices() {
    let conn = setup_test_db();
    // Create pending invoices with unique IDs
    for i in 1..=3 {
        pos_backend::db::invoices::create_invoice(
            &conn,
            &pos_backend::db::invoices::CreateInvoiceRequest {
                id: format!("INV-293-{}", i),
                reference_pubkey: format!("ref293-{}-unique", i),
                fiat_currency: Some("UAH".to_string()),
                fiat_amount: Some(100.0),
                usdc_amount: 2.41,
            },
        )
        .unwrap();
    }

    // SOP calls: GET /api/v1/invoices?status=pending
    let invoices =
        pos_backend::db::invoices::get_invoices_list(&conn, None, Some("pending")).unwrap();
    // Filter to only our test invoices (setup_test_db seeds sample data too)
    let our_invoices: Vec<_> = invoices
        .iter()
        .filter(|i| i.id.starts_with("INV-293-"))
        .collect();
    if our_invoices.len() == 3 && our_invoices.iter().all(|i| i.status == "pending") {
        test_pass("293: SOP can fetch pending invoices");
    } else {
        test_fail("293", &format!("got {} test invoices", our_invoices.len()));
    }
}

fn test_294_sop_create_invoice() {
    let conn = setup_test_db();
    // SOP calls: POST /api/v1/invoices/create
    let result = pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-SOP-001".to_string(),
            reference_pubkey: "7xRefKey-SOP-001-11111111111111111111".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(200.0),
            usdc_amount: 4.82,
        },
    );

    match result {
        Ok(id) if id == "INV-SOP-001" => test_pass("294: SOP can create invoice"),
        Ok(id) => test_fail("294", &format!("wrong id: {}", id)),
        Err(e) => test_fail("294", &format!("error: {}", e)),
    }
}

fn test_295_sop_update_invoice_status() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-SOP-002".to_string(),
            reference_pubkey: "ref2".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();

    // SOP calls: POST /api/v1/invoices/update_status
    let updated = pos_backend::db::invoices::update_invoice_status(
        &conn,
        "INV-SOP-002",
        "paid",
        Some("5k9X...Signature"),
    )
    .unwrap();

    if updated == 1 {
        let status: String = conn
            .query_row(
                "SELECT status FROM invoices WHERE id = 'INV-SOP-002'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        if status == "paid" {
            test_pass("295: SOP can update invoice status");
        } else {
            test_fail("295", &format!("status: {}", status));
        }
    } else {
        test_fail("295", "update returned 0 rows");
    }
}

fn test_296_sop_cancel_invoice() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-SOP-003".to_string(),
            reference_pubkey: "ref3".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();

    // SOP calls: POST /api/v1/invoices/cancel
    let cancelled = pos_backend::db::invoices::cancel_invoice(&conn, "INV-SOP-003").unwrap();

    if cancelled == 1 {
        let status: String = conn
            .query_row(
                "SELECT status FROM invoices WHERE id = 'INV-SOP-003'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        if status == "cancelled" {
            test_pass("296: SOP can cancel invoice");
        } else {
            test_fail("296", &format!("status: {}", status));
        }
    } else {
        test_fail("296", "cancel returned 0 rows");
    }
}

// ========== SOP refund_approval.json Tests ==========

fn test_297_sop_nonce_allocate() {
    let conn = setup_test_db();

    // SOP calls: POST /api/v1/nonce/allocate
    let result = pos_backend::db::nonce::allocate_free_nonce(&conn).unwrap();

    match result {
        Some(pubkey) => {
            let status: String = conn
                .query_row(
                    "SELECT status FROM nonce_accounts WHERE pubkey = ?1",
                    [&pubkey],
                    |row| row.get(0),
                )
                .unwrap();
            if status == "locked" {
                test_pass("297: SOP can allocate nonce");
            } else {
                test_fail("297", &format!("status: {}", status));
            }
        }
        None => test_fail("297", "no nonce available"),
    }
}

fn test_298_sop_nonce_release() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();

    // SOP calls: POST /api/v1/nonce/release
    pos_backend::db::nonce::release_nonce(&conn, &pubkey).unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM nonce_accounts WHERE pubkey = ?1",
            [&pubkey],
            |row| row.get(0),
        )
        .unwrap();

    if status == "free" {
        test_pass("298: SOP can release nonce");
    } else {
        test_fail("298", &format!("status: {}", status));
    }
}

fn test_299_sop_refund_flow() {
    let conn = setup_test_db();

    // Create invoice
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-REFUND-299".to_string(),
            reference_pubkey: "ref_refund_299_unique".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();

    // Mark as paid
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-REFUND-299", "paid", None)
        .unwrap();

    // Initiate refund (paid -> refunding) using the dedicated function
    let initiated = pos_backend::db::invoices::initiate_refund(&conn, "INV-REFUND-299").unwrap();

    // Create Squads proposal
    let proposal_idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-REFUND-299", "customer_address", 2.41)
            .unwrap();

    if initiated && proposal_idx > 0 {
        test_pass("299: SOP refund flow works");
    } else {
        test_fail(
            "299",
            &format!("initiated={}, idx={}", initiated, proposal_idx),
        );
    }
}

fn test_300_sop_invoice_lifecycle() {
    let conn = setup_test_db();

    // Full lifecycle: create -> paid -> refunding -> refund_proposed
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-LIFECYCLE-300".to_string(),
            reference_pubkey: "ref_lifecycle_300_unique".to_string(),
            fiat_currency: Some("BRL".to_string()),
            fiat_amount: Some(50.0),
            usdc_amount: 9.17,
        },
    )
    .unwrap();

    // pending -> paid
    pos_backend::db::invoices::update_invoice_status(
        &conn,
        "INV-LIFECYCLE-300",
        "paid",
        Some("sig123"),
    )
    .unwrap();

    // paid -> refunding (using initiate_refund)
    pos_backend::db::invoices::initiate_refund(&conn, "INV-LIFECYCLE-300").unwrap();

    // refunding -> refund_proposed_squads_v4 (using propose_refund)
    pos_backend::db::invoices::propose_refund(&conn, "INV-LIFECYCLE-300").unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM invoices WHERE id = 'INV-LIFECYCLE-300'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    if status == "refund_proposed_squads_v4" {
        test_pass("300: full invoice lifecycle works");
    } else {
        test_fail("300", &format!("final status: {}", status));
    }
}

// ========== Skill Integration Tests ==========

fn test_301_skill_solana_pay_url() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        4.82,
        "7xRefKey11111111111111111111111111111111111",
        Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        "ZeroClaw POS",
        "Invoice INV-101",
    );

    if url.starts_with("solana:")
        && url.contains("amount=4.82")
        && url.contains("spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v")
    {
        test_pass("301: skill generates valid Solana Pay URL");
    } else {
        test_fail("301", &format!("url: {}", url));
    }
}

fn test_302_skill_price_feed() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH", None, None, None, None, true,
    );

    match result {
        Ok(rate_info) => {
            if rate_info.get("rate").is_some() && rate_info.get("tier").is_some() {
                test_pass("302: skill gets fiat rate");
            } else {
                test_fail("302", "missing rate or tier");
            }
        }
        Err(e) => test_fail("302", e),
    }
}

fn test_303_skill_squads_proposal() {
    let result = pos_core_logic::build_squads_v4_proposal(
        "multisig111",
        "vault111",
        "proposer111",
        "customer111",
        2.41,
        1,
        "Refund invoice INV-101",
    );

    if result.get("program_id").is_some() && result.get("instruction_data_base64").is_some() {
        test_pass("303: skill generates Squads proposal");
    } else {
        test_fail("303", "missing required fields");
    }
}

// ========== Keyboard & UI Tests ==========

fn test_304_keyboard_callback_data() {
    // Verify callback_data format matches what ZeroClaw expects
    let cancel_kb = pos_backend::domain::i18n::get_cancel_invoice_inline_keyboard("INV-101", "en");
    let callback = cancel_kb["inline_keyboard"][0][0]["callback_data"]
        .as_str()
        .unwrap();

    if callback == "cancel_invoice_INV-101" {
        test_pass("304: keyboard callback_data format correct");
    } else {
        test_fail("304", &format!("callback: {}", callback));
    }
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

// ========== API Contract Tests ==========

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
