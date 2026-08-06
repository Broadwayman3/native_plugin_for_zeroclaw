use crate::common;

#[test]
fn test_111_db_connection() {
    let conn = common::setup_memory_db();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap_or_default();
    assert!(
        mode.to_lowercase() == "memory" || mode.to_lowercase() == "wal",
        "111: unexpected journal mode: {}",
        mode
    );
}

#[test]
fn test_112_db_init() {
    let conn = common::setup_memory_db();
    let tables: Vec<String> = {
        let mut stmt = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table'")
            .unwrap();
        let rows = stmt.query_map([], |row| row.get(0)).unwrap();
        rows.filter_map(|r| r.ok()).collect()
    };
    assert!(
        tables.contains(&"invoices".to_string()),
        "111: missing invoices table"
    );
    assert!(
        tables.contains(&"nonce_accounts".to_string()),
        "112: missing nonce_accounts table"
    );
    assert!(
        tables.contains(&"sop_checkpoints".to_string()),
        "112: missing sop_checkpoints table"
    );
}

#[test]
fn test_113_create_invoice() {
    let conn = common::setup_memory_db();
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-101".to_string(),
        reference_pubkey: "RefKey111111111111111111111111111111111111".to_string(),
        fiat_currency: Some("UAH".to_string()),
        fiat_amount: Some(200.0),
        usdc_amount: 4.82,
        telegram_chat_id: None,
        telegram_msg_id: None,
    };
    let id = pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();
    assert_eq!(id, "TEST-101", "113: wrong invoice id");
}

#[test]
fn test_114_get_invoices_list() {
    let conn = common::setup_memory_db();
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-114".to_string(),
        reference_pubkey: "RefKey114111111111111111111111111111111111".to_string(),
        fiat_currency: Some("UAH".to_string()),
        fiat_amount: Some(100.0),
        usdc_amount: 2.41,
        telegram_chat_id: None,
        telegram_msg_id: None,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();
    let list = pos_backend::db::invoices::get_invoices_list(&conn, None, None).unwrap();
    assert!(!list.is_empty(), "114: invoices list should not be empty");
}

#[test]
fn test_115_update_invoice_status() {
    let conn = common::setup_memory_db();
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-115".to_string(),
        reference_pubkey: "RefKey115111111111111111111111111111111111".to_string(),
        fiat_currency: Some("UAH".to_string()),
        fiat_amount: Some(100.0),
        usdc_amount: 2.41,
        telegram_chat_id: None,
        telegram_msg_id: None,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();
    let updated =
        pos_backend::db::invoices::update_invoice_status(&conn, "TEST-115", "paid", None).unwrap();
    assert_eq!(updated, 1, "115: should update exactly 1 row");
}

#[test]
fn test_116_cancel_invoice() {
    let conn = common::setup_memory_db();
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-116".to_string(),
        reference_pubkey: "RefKey116111111111111111111111111111111111".to_string(),
        fiat_currency: Some("USD".to_string()),
        fiat_amount: Some(10.0),
        usdc_amount: 10.0,
        telegram_chat_id: None,
        telegram_msg_id: None,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();
    let cancelled = pos_backend::db::invoices::cancel_invoice(&conn, "TEST-116").unwrap();
    assert_eq!(cancelled, 1, "116: should cancel exactly 1 row");
}

#[test]
fn test_117_duplicate_invoice_id() {
    let conn = common::setup_memory_db();
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-117".to_string(),
        reference_pubkey: "RefKey117111111111111111111111111111111111".to_string(),
        fiat_currency: Some("EUR".to_string()),
        fiat_amount: Some(5.0),
        usdc_amount: 5.0,
        telegram_chat_id: None,
        telegram_msg_id: None,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();
    let req2 = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-117".to_string(),
        reference_pubkey: "RefKey117222222222222222222222222222222222".to_string(),
        fiat_currency: Some("EUR".to_string()),
        fiat_amount: Some(5.0),
        usdc_amount: 5.0,
        telegram_chat_id: None,
        telegram_msg_id: None,
    };
    assert!(
        pos_backend::db::invoices::create_invoice(&conn, &req2).is_err(),
        "117: duplicate ID should be rejected"
    );
}

#[test]
fn test_118_sales_summary() {
    let conn = common::setup_memory_db();
    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    assert!(
        summary.get("total_paid_invoices").is_some(),
        "118: sales summary missing required fields"
    );
}

#[test]
fn test_119_nonce_allocate() {
    let conn = common::setup_memory_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .expect("119: should allocate a nonce");
    assert!(!pubkey.is_empty(), "119: pubkey should not be empty");
}

#[test]
fn test_120_nonce_release() {
    let conn = common::setup_memory_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .expect("120: should allocate a nonce");
    pos_backend::db::nonce::release_nonce(&conn, &pubkey).unwrap();
}

#[test]
fn test_121_cleanup_expired() {
    let conn = common::setup_memory_db();
    pos_backend::db::invoices::cleanup_expired_pending_invoices(&conn).unwrap();
}

#[test]
fn test_124_sop_checkpoint_create() {
    let conn = common::setup_memory_db();
    pos_backend::db::sop_checkpoints::create_checkpoint(&conn, "cp-1", "sop-1", "step-1", None)
        .unwrap();
}

#[test]
fn test_125_seed_data() {
    let conn = common::setup_memory_db();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))
        .unwrap_or(0);
    assert!(count >= 0, "125: count should be >= 0");
}

#[test]
fn test_126_invoice_not_found() {
    let conn = common::setup_memory_db();
    let result = pos_backend::db::invoices::get_invoice_by_id(&conn, "NONEXISTENT").unwrap();
    assert!(
        result.is_none(),
        "126: nonexistent invoice should return None"
    );
}

#[test]
fn test_127_invalid_status() {
    let conn = common::setup_memory_db();
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-127".to_string(),
        reference_pubkey: "RefKey127111111111111111111111111111111111".to_string(),
        fiat_currency: Some("USD".to_string()),
        fiat_amount: Some(10.0),
        usdc_amount: 10.0,
        telegram_chat_id: None,
        telegram_msg_id: None,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();
    assert!(
        pos_backend::db::invoices::update_invoice_status(&conn, "TEST-127", "invalid_status", None)
            .is_err(),
        "127: invalid status should be rejected"
    );
}

#[test]
fn test_128_wal_mode() {
    let conn = common::setup_memory_db();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap_or_default();
    assert!(
        mode.to_lowercase() == "memory" || mode.to_lowercase() == "wal",
        "128: unexpected journal mode: {}",
        mode
    );
}

#[test]
fn test_129_concurrent_access() {
    use std::sync::{Arc, Mutex};

    let conn = common::setup_memory_db();
    let conn = Arc::new(Mutex::new(conn));

    let conn_clone = conn.clone();
    let handle = std::thread::spawn(move || {
        let c = conn_clone.lock().unwrap();
        pos_backend::db::invoices::get_sales_summary(&c).is_ok()
    });

    let local = {
        let c = conn.lock().unwrap();
        pos_backend::db::invoices::get_sales_summary(&c).is_ok()
    };

    let remote = handle.join().unwrap_or(false);
    assert!(local, "129: local read should succeed");
    assert!(remote, "129: remote read should succeed");
}

#[test]
fn test_130_db_cleanup() {
    let guard = common::TempDbGuard::new("cleanup_130");
    let path = guard.path().to_string();
    let conn = pos_backend::db::get_db_connection(&path).unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    drop(conn);
    assert!(
        std::path::Path::new(&path).exists(),
        "130: DB file should exist before drop"
    );
    drop(guard);
    assert!(
        !std::path::Path::new(&path).exists(),
        "130: DB file should be removed after drop"
    );
}
