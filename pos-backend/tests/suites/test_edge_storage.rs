use crate::common::TempDbGuard;

fn setup_test_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    conn
}

#[test]
fn test_214_db_wal_mode() {
    let guard = TempDbGuard::new("wal_mode");
    let conn = pos_backend::db::get_db_connection(guard.path()).unwrap();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap();
    assert_eq!(mode, "wal", "214: expected WAL mode, got: {}", mode);
}

#[test]
fn test_215_schema_migration_tax_rate() {
    let conn = setup_test_db();
    let columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(invoices)").unwrap();
        let rows = stmt.query_map([], |row| row.get::<_, String>(1)).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    };
    assert!(
        columns.contains(&"tax_rate_pct".to_string()),
        "215: tax_rate_pct column missing"
    );
}

#[test]
fn test_216_schema_migration_items_breakdown() {
    let conn = setup_test_db();
    let columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(invoices)").unwrap();
        let rows = stmt.query_map([], |row| row.get::<_, String>(1)).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    };
    assert!(
        columns.contains(&"items_breakdown".to_string()),
        "216: items_breakdown column missing"
    );
}

#[test]
fn test_217_cleanup_expired_pending() {
    let conn = setup_test_db();

    conn.execute(
        "INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, created_at)
         VALUES ('INV-OLD', 'ref1', 'UAH', 100.0, 2.41, 'pending', datetime('now', '-48 hours'))",
        [],
    )
    .unwrap();

    pos_backend::db::invoices::cleanup_expired_pending_invoices(&conn).unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM invoices WHERE id = 'INV-OLD'",
            [],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(status, "expired", "217: expected expired, got: {}", status);
}

#[test]
fn test_220_concurrent_db_writes() {
    let guard = TempDbGuard::new("concurrent_storage");
    let db_path = guard.path().to_string();

    let conn = pos_backend::db::get_db_connection(&db_path).unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    drop(conn);

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let db_path = db_path.clone();
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

    let conn = pos_backend::db::get_db_connection(&db_path).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))
        .unwrap();

    assert_eq!(
        count, 5,
        "220: expected 5 concurrent writes, got: {}",
        count
    );
    // TempDbGuard will cleanup files on drop
}

#[test]
fn test_221_invoice_status_transitions() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-TRANS".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();

    let updated = pos_backend::db::invoices::update_invoice_status(
        &conn,
        "INV-TRANS",
        "paid",
        Some("sig123"),
    )
    .unwrap();
    assert_eq!(updated, 1);

    let updated =
        pos_backend::db::invoices::update_invoice_status(&conn, "INV-TRANS", "refunding", None)
            .unwrap();
    assert_eq!(updated, 0, "221: paid invoice should not be transitionable");
}

#[test]
fn test_224_invoice_cancel_paid() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-PAID".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-PAID", "paid", None).unwrap();

    let cancelled = pos_backend::db::invoices::cancel_invoice(&conn, "INV-PAID").unwrap();
    assert_eq!(cancelled, 0, "224: paid invoice should not be cancelable");
}

#[test]
fn test_226_invoice_update_status_invalid() {
    let conn = setup_test_db();
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-INVALID".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-INVALID", "paid", None).unwrap();

    let updated =
        pos_backend::db::invoices::update_invoice_status(&conn, "INV-INVALID", "cancelled", None)
            .unwrap();
    assert_eq!(
        updated, 0,
        "226: should not update paid invoice to cancelled"
    );
}
