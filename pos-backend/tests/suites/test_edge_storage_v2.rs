use crate::common::TempDbGuard;

fn setup_test_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    conn
}

#[test]
fn test_227_sales_summary_empty() {
    let conn = setup_test_db();
    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    assert_eq!(
        summary["total_paid_invoices"], 0,
        "227: expected empty sales summary"
    );
}

#[test]
fn test_228_sales_summary_with_data() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-SUM".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-SUM", "paid", None).unwrap();

    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    assert_eq!(
        summary["total_paid_invoices"], 1,
        "228: expected 1 paid invoice in summary"
    );
}

#[test]
fn test_229_sales_summary_by_currency() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-UAH".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-UAH", "paid", None).unwrap();

    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    let by_currency = summary["sales_by_currency"].as_object().unwrap();
    assert!(by_currency.contains_key("UAH"), "229: UAH not in breakdown");
}

#[test]
fn test_230_sales_summary_rounding() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-ROUND".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.416,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-ROUND", "paid", None).unwrap();

    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    let total = summary["total_sales_usdc"].as_f64().unwrap();
    let rounded = (total * 100.0).round() / 100.0;
    assert!(
        (total - rounded).abs() < f64::EPSILON,
        "230: sales summary not rounded to 2 decimals: {}",
        total
    );
}

#[test]
fn test_231_sales_summary_timestamp() {
    let conn = setup_test_db();
    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    let timestamp = summary["timestamp"].as_str().unwrap();
    assert!(
        timestamp.contains("T"),
        "231: timestamp missing T separator: {}",
        timestamp
    );
    assert!(
        !timestamp.contains("T00:00:00"),
        "231: timestamp should not be midnight: {}",
        timestamp
    );
}

#[test]
fn test_232_get_invoices_list_empty() {
    let conn = setup_test_db();
    let invoices = pos_backend::db::invoices::get_invoices_list(&conn, None, None).unwrap();
    assert!(
        invoices.is_empty(),
        "232: expected empty list, got: {}",
        invoices.len()
    );
}

#[test]
fn test_233_get_invoices_list_with_data() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-LIST".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();

    let invoices = pos_backend::db::invoices::get_invoices_list(&conn, None, None).unwrap();
    assert_eq!(
        invoices.len(),
        1,
        "233: expected 1 invoice, got: {}",
        invoices.len()
    );
}

#[test]
fn test_234_get_invoices_by_id() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-FIND".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();

    let invoices =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("INV-FIND"), None).unwrap();
    assert_eq!(
        invoices.len(),
        1,
        "234: expected 1 invoice, got: {}",
        invoices.len()
    );
    assert_eq!(invoices[0].id, "INV-FIND", "234: wrong invoice ID returned");
}

#[test]
fn test_235_get_invoices_by_id_not_found() {
    let conn = setup_test_db();
    let invoices =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("NONEXISTENT"), None).unwrap();
    assert!(
        invoices.is_empty(),
        "235: should return empty for nonexistent ID"
    );
}

#[test]
fn test_236_db_connection_wal() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))
        .unwrap();
    assert_eq!(count, 0, "236: expected 0 invoices, got: {}", count);
}

#[test]
fn test_237_db_connection_busy_timeout() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    let timeout: i64 = conn
        .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        timeout, 5000,
        "237: expected busy_timeout 5000, got: {}",
        timeout
    );
}

#[test]
fn test_238_db_connection_cache_size() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    let cache_size: i64 = conn
        .query_row("PRAGMA cache_size", [], |row| row.get(0))
        .unwrap();
    assert_eq!(
        cache_size, -64000,
        "238: expected cache_size -64000, got: {}",
        cache_size
    );
}

#[test]
fn test_239_invoice_status_all_allowed() {
    let allowed = pos_backend::db::invoices::ALLOWED_INVOICE_STATUSES;
    let expected = vec![
        "pending",
        "paid",
        "partially_paid",
        "cancelled",
        "refunding",
        "refund_proposed_squads_v4",
        "expired",
        "failed",
    ];
    assert_eq!(allowed.len(), expected.len(), "239: status count mismatch");
    for s in &expected {
        assert!(allowed.contains(s), "239: missing status: {}", s);
    }
}

#[test]
fn test_240_squads_proposal_create() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-240".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-240", "recipient1", 10.0).unwrap();
    assert!(idx > 0, "240: expected proposal index > 0, got: {}", idx);
}

#[test]
fn test_241_squads_proposal_update() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-241".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-241", "recipient1", 10.0).unwrap();
    let updated = pos_backend::db::squads::update_proposal_status(&conn, idx, "approved").unwrap();
    assert!(updated, "241: squads proposal update failed");
}

#[test]
fn test_242_processed_updates_dedup() {
    let conn = setup_test_db();
    let first = pos_backend::db::updates::check_and_register(&conn, 99999).unwrap();
    let second = pos_backend::db::updates::check_and_register(&conn, 99999).unwrap();
    assert!(
        first && !second,
        "242: expected first=true second=false, got first={} second={}",
        first,
        second
    );
}

#[test]
fn test_243_propose_refund_rejects_non_refunding() {
    let conn = setup_test_db();

    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-243".to_string(),
            reference_pubkey: "ref243".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    pos_backend::db::invoices::cancel_invoice(&conn, "INV-243").unwrap();

    let result = pos_backend::db::invoices::propose_refund(&conn, "INV-243").unwrap();
    assert!(
        !result,
        "243: propose_refund should have returned false for cancelled invoice"
    );
}

#[test]
fn test_244_db_cleanup_files() {
    let path: String;
    {
        let guard = TempDbGuard::new("cleanup_243");
        path = guard.path().to_string();
        let conn = pos_backend::db::get_db_connection(&path).unwrap();
        pos_backend::db::schema::init_db(&conn, false).unwrap();
        drop(conn);
        assert!(std::path::Path::new(&path).exists());
        // guard is dropped here, should cleanup files
    }
    assert!(
        !std::path::Path::new(&path).exists(),
        "244: main DB file should be removed after TempDbGuard drop"
    );
    assert!(
        !std::path::Path::new(&format!("{}-wal", path)).exists(),
        "244: WAL file should be removed after TempDbGuard drop"
    );
    assert!(
        !std::path::Path::new(&format!("{}-shm", path)).exists(),
        "244: SHM file should be removed after TempDbGuard drop"
    );
}
