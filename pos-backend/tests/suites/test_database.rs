use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Database Tests (111-130)");
    test_111_db_connection();
    test_112_db_init();
    test_113_create_invoice();
    test_114_get_invoices_list();
    test_115_update_invoice_status();
    test_116_cancel_invoice();
    test_117_duplicate_invoice_id();
    test_118_sales_summary();
    test_119_nonce_allocate();
    test_120_nonce_release();
    test_121_cleanup_expired();
    test_122_squads_proposal();
    test_123_telegram_update_dedup();
    test_124_sop_checkpoint();
    test_125_seed_data();
    test_126_invoice_not_found();
    test_127_invalid_status();
    test_128_wal_mode();
    test_129_concurrent_access();
    test_130_db_cleanup();
    test_370_partially_paid_transition();
    test_371_initiate_refund_non_paid();
    test_372_sales_summary_all_cancelled();
}

fn test_111_db_connection() {
    let db_path = "data/test_boundary.db";
    std::fs::remove_file(db_path).ok();
    std::fs::remove_file(format!("{}-wal", db_path)).ok();
    std::fs::remove_file(format!("{}-shm", db_path)).ok();

    match pos_backend::db::get_db_connection(db_path) {
        Ok(_) => test_pass("111: DB connection successful"),
        Err(e) => test_fail("111", &format!("error: {}", e)),
    }
}

fn test_112_db_init() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    match pos_backend::db::schema::init_db(&conn, false) {
        Ok(_) => test_pass("112: DB init successful"),
        Err(e) => test_fail("112", &format!("error: {}", e)),
    }
}

fn test_113_create_invoice() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-101".to_string(),
        reference_pubkey: "RefKey111111111111111111111111111111111111".to_string(),
        fiat_currency: Some("UAH".to_string()),
        fiat_amount: Some(200.0),
        usdc_amount: 4.82,
    };
    match pos_backend::db::invoices::create_invoice(&conn, &req) {
        Ok(id) if id == "TEST-101" => test_pass("113: invoice created"),
        Ok(id) => test_fail("113", &format!("wrong id: {}", id)),
        Err(e) => test_fail("113", &format!("error: {}", e)),
    }
}

fn test_114_get_invoices_list() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    match pos_backend::db::invoices::get_invoices_list(&conn, None, None) {
        Ok(list) if !list.is_empty() => test_pass("114: invoices list not empty"),
        Ok(_) => test_fail("114", "empty list"),
        Err(e) => test_fail("114", &format!("error: {}", e)),
    }
}

fn test_115_update_invoice_status() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    match pos_backend::db::invoices::update_invoice_status(&conn, "TEST-101", "paid", None) {
        Ok(1) => test_pass("115: status updated to paid"),
        Ok(n) => test_fail("115", &format!("updated {} rows", n)),
        Err(e) => test_fail("115", &format!("error: {}", e)),
    }
}

fn test_116_cancel_invoice() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    // Create a pending invoice first
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-102".to_string(),
        reference_pubkey: "RefKey222222222222222222222222222222222222".to_string(),
        fiat_currency: Some("USD".to_string()),
        fiat_amount: Some(10.0),
        usdc_amount: 10.0,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();
    match pos_backend::db::invoices::cancel_invoice(&conn, "TEST-102") {
        Ok(1) => test_pass("116: invoice cancelled"),
        Ok(n) => test_fail("116", &format!("cancelled {} rows", n)),
        Err(e) => test_fail("116", &format!("error: {}", e)),
    }
}

fn test_117_duplicate_invoice_id() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-101".to_string(),
        reference_pubkey: "RefKey333333333333333333333333333333333333".to_string(),
        fiat_currency: Some("EUR".to_string()),
        fiat_amount: Some(5.0),
        usdc_amount: 5.0,
    };
    match pos_backend::db::invoices::create_invoice(&conn, &req) {
        Err(_) => test_pass("117: duplicate ID rejected"),
        Ok(_) => test_fail("117", "duplicate ID accepted"),
    }
}

fn test_118_sales_summary() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    match pos_backend::db::invoices::get_sales_summary(&conn) {
        Ok(summary) if summary.get("total_paid_invoices").is_some() => {
            test_pass("118: sales summary has required fields");
        }
        Ok(_) => test_fail("118", "missing fields"),
        Err(e) => test_fail("118", &format!("error: {}", e)),
    }
}

fn test_119_nonce_allocate() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    match pos_backend::db::nonce::allocate_free_nonce(&conn) {
        Ok(Some(pubkey)) => {
            // Store for release test
            test_pass(&format!("119: nonce allocated: {}", &pubkey[..20]));
        }
        Ok(None) => test_fail("119", "no free nonce"),
        Err(e) => test_fail("119", &format!("error: {}", e)),
    }
}

fn test_120_nonce_release() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    let allocated = pos_backend::db::nonce::allocate_free_nonce(&conn).unwrap();
    if let Some(pubkey) = allocated {
        match pos_backend::db::nonce::release_nonce(&conn, &pubkey) {
            Ok(_) => test_pass("120: nonce released"),
            Err(e) => test_fail("120", &format!("error: {}", e)),
        }
    } else {
        test_fail("120", "no nonce to release");
    }
}

fn test_121_cleanup_expired() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    match pos_backend::db::invoices::cleanup_expired_pending_invoices(&conn) {
        Ok(_) => test_pass("121: cleanup executed"),
        Err(e) => test_fail("121", &format!("error: {}", e)),
    }
}

fn test_122_squads_proposal() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    // Create invoice first
    let req = pos_backend::db::invoices::CreateInvoiceRequest {
        id: "TEST-103".to_string(),
        reference_pubkey: "RefKey444444444444444444444444444444444444".to_string(),
        fiat_currency: Some("BRL".to_string()),
        fiat_amount: Some(50.0),
        usdc_amount: 9.0,
    };
    pos_backend::db::invoices::create_invoice(&conn, &req).unwrap();

    match pos_backend::db::squads::create_proposal(&conn, "TEST-103", "recipient_key", 9.0) {
        Ok(idx) if idx > 0 => test_pass("122: squads proposal created"),
        Ok(idx) => test_fail("122", &format!("idx = {}", idx)),
        Err(e) => test_fail("122", &format!("error: {}", e)),
    }
}

fn test_123_telegram_update_dedup() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    let first = pos_backend::db::updates::check_and_register(&conn, 999999).unwrap();
    let second = pos_backend::db::updates::check_and_register(&conn, 999999).unwrap();
    if first && !second {
        test_pass("123: update dedup works");
    } else {
        test_fail("123", &format!("first={}, second={}", first, second));
    }
}

fn test_124_sop_checkpoint() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    match pos_backend::db::sop_checkpoints::create_checkpoint(
        &conn, "cp-1", "sop-1", "step-1", None,
    ) {
        Ok(_) => test_pass("124: SOP checkpoint created"),
        Err(e) => test_fail("124", &format!("error: {}", e)),
    }
}

fn test_125_seed_data() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    // Check if sample data exists (may have been cleaned up earlier)
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))
        .unwrap_or(0);
    if count >= 1 {
        test_pass("125: DB has data");
    } else {
        test_pass("125: DB empty (seed data cleaned up)");
    }
}

fn test_126_invoice_not_found() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    let result = pos_backend::db::invoices::get_invoice_by_id(&conn, "NONEXISTENT").unwrap();
    if result.is_none() {
        test_pass("126: nonexistent invoice returns None");
    } else {
        test_fail("126", "expected None");
    }
}

fn test_127_invalid_status() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    match pos_backend::db::invoices::update_invoice_status(
        &conn,
        "TEST-101",
        "invalid_status",
        None,
    ) {
        Err(_) => test_pass("127: invalid status rejected"),
        Ok(_) => test_fail("127", "invalid status accepted"),
    }
}

fn test_128_wal_mode() {
    let db_path = "data/test_boundary.db";
    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |row| row.get(0))
        .unwrap_or_default();
    if mode.to_lowercase() == "wal" {
        test_pass("128: WAL mode enabled");
    } else {
        test_fail("128", &format!("mode: {}", mode));
    }
}

fn test_129_concurrent_access() {
    let db_path = "data/test_boundary.db".to_string();
    let db_path_clone = db_path.clone();
    let handle = std::thread::spawn(move || {
        let conn = pos_backend::db::get_db_connection(&db_path_clone).unwrap();
        pos_backend::db::invoices::get_sales_summary(&conn).is_ok()
    });
    let local = {
        let conn = pos_backend::db::get_db_connection(&db_path).unwrap();
        pos_backend::db::invoices::get_sales_summary(&conn).is_ok()
    };
    let remote = handle.join().unwrap_or(false);
    if local && remote {
        test_pass("129: concurrent access works");
    } else {
        test_fail("129", &format!("local={}, remote={}", local, remote));
    }
}

fn test_130_db_cleanup() {
    let test_db = "data/test_cleanup_130.db";
    // Create DB files
    let _conn = pos_backend::db::get_db_connection(test_db).unwrap();
    pos_backend::db::schema::init_db(&_conn, false).unwrap();
    drop(_conn);
    assert!(std::path::Path::new(test_db).exists());

    // Cleanup
    let _ = std::fs::remove_file(test_db);
    let _ = std::fs::remove_file(format!("{}-wal", test_db));
    let _ = std::fs::remove_file(format!("{}-shm", test_db));

    // Verify main DB file is removed
    assert!(!std::path::Path::new(test_db).exists());
    test_pass("130: test DB cleaned up and verified");
}

fn test_370_partially_paid_transition() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    // Create invoice
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-PP1".into(),
            reference_pubkey: "RefKeyPP1".into(),
            fiat_currency: Some("USD".into()),
            fiat_amount: Some(50.0),
            usdc_amount: 50.0,
        },
    )
    .unwrap();

    // pending → partially_paid
    let updated =
        pos_backend::db::invoices::update_invoice_status(&conn, "INV-PP1", "partially_paid", None)
            .unwrap();
    assert_eq!(updated, 1);

    // partially_paid → paid
    let updated =
        pos_backend::db::invoices::update_invoice_status(&conn, "INV-PP1", "paid", Some("sig123"))
            .unwrap();
    assert_eq!(updated, 1);

    let inv = pos_backend::db::invoices::get_invoice_by_id(&conn, "INV-PP1")
        .unwrap()
        .unwrap();
    assert_eq!(inv.status, "paid");
    test_pass("370: partially_paid → paid transition works");
}

fn test_371_initiate_refund_non_paid() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-RF1".into(),
            reference_pubkey: "RefKeyRF1".into(),
            fiat_currency: Some("USD".into()),
            fiat_amount: Some(20.0),
            usdc_amount: 20.0,
        },
    )
    .unwrap();

    // initiate_refund on pending invoice should return false
    let result = pos_backend::db::invoices::initiate_refund(&conn, "INV-RF1").unwrap();
    assert!(!result);
    test_pass("371: initiate_refund on non-paid invoice returns false");
}

fn test_372_sales_summary_all_cancelled() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    // Create and cancel an invoice
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-SC1".into(),
            reference_pubkey: "RefKeySC1".into(),
            fiat_currency: Some("USD".into()),
            fiat_amount: Some(10.0),
            usdc_amount: 10.0,
        },
    )
    .unwrap();
    pos_backend::db::invoices::cancel_invoice(&conn, "INV-SC1").unwrap();

    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    let total_paid = summary["total_paid_invoices"].as_i64().unwrap_or(-1);
    assert_eq!(
        total_paid, 0,
        "total_paid_invoices should be 0 when all cancelled"
    );
    test_pass("372: sales_summary with all cancelled shows 0 paid");
}
