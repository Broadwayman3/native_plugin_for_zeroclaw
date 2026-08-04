fn setup_test_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, true).unwrap();
    conn
}

// ========== SOP check_payments.json Tests ==========

#[test]
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
    assert_eq!(
        our_invoices.len(),
        3,
        "got {} test invoices",
        our_invoices.len()
    );
    assert!(
        our_invoices.iter().all(|i| i.status == "pending"),
        "not all invoices pending"
    );
}

#[test]
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
        Ok(id) => assert_eq!(id, "INV-SOP-001", "wrong id: {}", id),
        Err(e) => panic!("294: error: {}", e),
    }
}

#[test]
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

    assert_eq!(updated, 1, "update returned 0 rows");
    let status: String = conn
        .query_row(
            "SELECT status FROM invoices WHERE id = 'INV-SOP-002'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "paid", "status: {}", status);
}

#[test]
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

    assert_eq!(cancelled, 1, "cancel returned 0 rows");
    let status: String = conn
        .query_row(
            "SELECT status FROM invoices WHERE id = 'INV-SOP-003'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "cancelled", "status: {}", status);
}

// ========== SOP refund_approval.json Tests ==========

#[test]
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
            assert_eq!(status, "locked", "status: {}", status);
        }
        None => panic!("297: no nonce available"),
    }
}

#[test]
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

    assert_eq!(status, "free", "status: {}", status);
}

#[test]
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

    assert!(initiated, "initiate_refund returned false");
    assert!(proposal_idx > 0, "proposal_idx={} not > 0", proposal_idx);
}

#[test]
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

    assert_eq!(
        status, "refund_proposed_squads_v4",
        "final status: {}",
        status
    );
}

// ========== Skill Integration Tests ==========

#[test]
fn test_301_skill_solana_pay_url() {
    let url = pos_core_logic::build_solana_pay_url(
        "8xAZmQ1111111111111111111111111111111111111",
        4.82,
        "7xRefKey11111111111111111111111111111111111",
        Some("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        "ZeroClaw POS",
        "Invoice INV-101",
    );

    assert!(
        url.starts_with("solana:"),
        "url does not start with solana: {}",
        url
    );
    assert!(
        url.contains("amount=4.82"),
        "url missing amount=4.82: {}",
        url
    );
    assert!(
        url.contains("spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        "url missing spl-token: {}",
        url
    );
}

#[test]
fn test_302_skill_price_feed() {
    let result = pos_backend::domain::price_feed::get_multitier_fiat_rate(
        "UAH", None, None, None, None, true,
    );

    match result {
        Ok(rate_info) => {
            assert!(rate_info.get("rate").is_some(), "missing rate");
            assert!(rate_info.get("tier").is_some(), "missing tier");
        }
        Err(e) => panic!("302: {}", e),
    }
}

#[test]
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

    assert!(result.get("program_id").is_some(), "missing program_id");
    assert!(
        result.get("instruction_data_base64").is_some(),
        "missing instruction_data_base64"
    );
}

// ========== Keyboard & UI Tests ==========

#[test]
fn test_304_keyboard_callback_data() {
    // Verify callback_data format matches what ZeroClaw expects
    let cancel_kb = pos_backend::domain::i18n::get_cancel_invoice_inline_keyboard("INV-101", "en");
    let callback = cancel_kb["inline_keyboard"][0][0]["callback_data"]
        .as_str()
        .unwrap();

    assert_eq!(callback, "cancel_invoice_INV-101", "callback: {}", callback);
}
