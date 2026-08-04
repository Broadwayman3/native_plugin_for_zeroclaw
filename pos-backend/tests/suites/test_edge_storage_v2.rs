use crate::{test_fail, test_pass};

pub fn run_suite() {
    println!("\n📦 Edge Storage Tests V2 (227-244)");
    test_227_sales_summary_empty();
    test_228_sales_summary_with_data();
    test_229_sales_summary_by_currency();
    test_230_sales_summary_rounding();
    test_231_sales_summary_timestamp();
    test_232_get_invoices_list_empty();
    test_233_get_invoices_list_with_data();
    test_234_get_invoices_by_id();
    test_235_get_invoices_by_id_not_found();
    test_236_db_connection_wal();
    test_237_db_connection_busy_timeout();
    test_238_db_connection_cache_size();
    test_239_invoice_status_all_allowed();
    test_240_squads_proposal_create();
    test_241_squads_proposal_update();
    test_242_processed_updates_dedup();
    test_243_propose_refund_rejects_non_refunding();
    test_244_db_cleanup_files();
}

fn setup_test_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    conn
}

fn test_227_sales_summary_empty() {
    let conn = setup_test_db();
    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    if summary["total_paid_invoices"] == 0 {
        test_pass("227: empty sales summary");
    } else {
        test_fail("227", &format!("summary: {}", summary));
    }
}

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
        },
    )
    .unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-SUM", "paid", None).unwrap();

    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    if summary["total_paid_invoices"] == 1 {
        test_pass("228: sales summary with data");
    } else {
        test_fail("228", &format!("summary: {}", summary));
    }
}

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
        },
    )
    .unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-UAH", "paid", None).unwrap();

    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    let by_currency = summary["sales_by_currency"].as_object().unwrap();
    if by_currency.contains_key("UAH") {
        test_pass("229: sales summary by currency");
    } else {
        test_fail("229", "UAH not in breakdown");
    }
}

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
        },
    )
    .unwrap();
    pos_backend::db::invoices::update_invoice_status(&conn, "INV-ROUND", "paid", None).unwrap();

    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    let total = summary["total_sales_usdc"].as_f64().unwrap();
    // Should be rounded to 2 decimal places
    let rounded = (total * 100.0).round() / 100.0;
    if (total - rounded).abs() < f64::EPSILON {
        test_pass("230: sales summary rounded to 2 decimals");
    } else {
        test_fail("230", &format!("total: {}", total));
    }
}

fn test_231_sales_summary_timestamp() {
    let conn = setup_test_db();
    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    let timestamp = summary["timestamp"].as_str().unwrap();
    // Should be ISO format, not midnight
    if timestamp.contains("T") && !timestamp.contains("T00:00:00") {
        test_pass("231: timestamp is proper ISO format");
    } else {
        test_fail("231", &format!("timestamp: {}", timestamp));
    }
}

fn test_232_get_invoices_list_empty() {
    let conn = setup_test_db();
    let invoices = pos_backend::db::invoices::get_invoices_list(&conn, None, None).unwrap();
    if invoices.is_empty() {
        test_pass("232: empty invoices list");
    } else {
        test_fail("232", &format!("count: {}", invoices.len()));
    }
}

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
        },
    )
    .unwrap();

    let invoices = pos_backend::db::invoices::get_invoices_list(&conn, None, None).unwrap();
    if invoices.len() == 1 {
        test_pass("233: invoices list with data");
    } else {
        test_fail("233", &format!("count: {}", invoices.len()));
    }
}

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
        },
    )
    .unwrap();

    let invoices =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("INV-FIND"), None).unwrap();
    if invoices.len() == 1 && invoices[0].id == "INV-FIND" {
        test_pass("234: get invoice by ID");
    } else {
        test_fail("234", &format!("count: {}", invoices.len()));
    }
}

fn test_235_get_invoices_by_id_not_found() {
    let conn = setup_test_db();
    let invoices =
        pos_backend::db::invoices::get_invoices_list(&conn, Some("NONEXISTENT"), None).unwrap();
    if invoices.is_empty() {
        test_pass("235: nonexistent invoice returns empty");
    } else {
        test_fail("235", "should return empty");
    }
}

fn test_236_db_connection_wal() {
    // In-memory databases don't support WAL mode
    // Just verify the connection function works
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))
        .unwrap();
    if count == 0 {
        test_pass("236: connection works (in-memory, no WAL)");
    } else {
        test_fail("236", &format!("count: {}", count));
    }
}

fn test_237_db_connection_busy_timeout() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    let timeout: i64 = conn
        .query_row("PRAGMA busy_timeout", [], |row| row.get(0))
        .unwrap();
    if timeout == 5000 {
        test_pass("237: busy_timeout set to 5000");
    } else {
        test_fail("237", &format!("timeout: {}", timeout));
    }
}

fn test_238_db_connection_cache_size() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    let cache_size: i64 = conn
        .query_row("PRAGMA cache_size", [], |row| row.get(0))
        .unwrap();
    if cache_size == -64000 {
        test_pass("238: cache_size set to -64000");
    } else {
        test_fail("238", &format!("cache_size: {}", cache_size));
    }
}

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
    if allowed.len() == expected.len() && allowed.iter().all(|s| expected.contains(s)) {
        test_pass("239: all allowed statuses present");
    } else {
        test_fail("239", &format!("allowed: {:?}", allowed));
    }
}

fn test_240_squads_proposal_create() {
    let conn = setup_test_db();
    // Create invoice first (required for foreign key)
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-240".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-240", "recipient1", 10.0).unwrap();
    if idx > 0 {
        test_pass("240: squads proposal created");
    } else {
        test_fail("240", &format!("idx: {}", idx));
    }
}

fn test_241_squads_proposal_update() {
    let conn = setup_test_db();
    // Create invoice first (required for foreign key)
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-241".to_string(),
            reference_pubkey: "ref1".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();
    let idx =
        pos_backend::db::squads::create_proposal(&conn, "INV-241", "recipient1", 10.0).unwrap();
    let updated = pos_backend::db::squads::update_proposal_status(&conn, idx, "approved").unwrap();
    if updated {
        test_pass("241: squads proposal updated");
    } else {
        test_fail("241", "update failed");
    }
}

fn test_242_processed_updates_dedup() {
    let conn = setup_test_db();
    let first = pos_backend::db::updates::check_and_register(&conn, 99999).unwrap();
    let second = pos_backend::db::updates::check_and_register(&conn, 99999).unwrap();
    if first && !second {
        test_pass("242: processed_updates dedup works");
    } else {
        test_fail("242", &format!("first={}, second={}", first, second));
    }
}

fn test_243_propose_refund_rejects_non_refunding() {
    let conn = setup_test_db();

    // Create invoice with status 'cancelled'
    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-243".to_string(),
            reference_pubkey: "ref243".to_string(),
            fiat_currency: Some("UAH".to_string()),
            fiat_amount: Some(100.0),
            usdc_amount: 2.41,
        },
    )
    .unwrap();
    pos_backend::db::invoices::cancel_invoice(&conn, "INV-243").unwrap();

    // Try to propose refund on cancelled invoice (should fail)
    let result = pos_backend::db::invoices::propose_refund(&conn, "INV-243").unwrap();

    if !result {
        test_pass("243: propose_refund rejects non-refunding status");
    } else {
        test_fail("243", "propose_refund should have returned false");
    }
}

fn test_244_db_cleanup_files() {
    let db_path = "data/test_cleanup_243.db";
    // Create DB
    let _conn = pos_backend::db::get_db_connection(db_path).unwrap();
    pos_backend::db::schema::init_db(&_conn, false).unwrap();
    drop(_conn);
    assert!(std::path::Path::new(db_path).exists());

    // Cleanup
    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path));
    let _ = std::fs::remove_file(format!("{}-shm", db_path));

    // Verify cleanup
    assert!(!std::path::Path::new(db_path).exists());
    assert!(!std::path::Path::new(&format!("{}-wal", db_path)).exists());
    assert!(!std::path::Path::new(&format!("{}-shm", db_path)).exists());
    test_pass("243: DB cleanup files verified");
}
