use crate::common;

#[test]
fn test_370_partially_paid_transition() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-PP1".into(),
            reference_pubkey: "RefKeyPP1".into(),
            fiat_currency: Some("USD".into()),
            fiat_amount: Some(50.0),
            usdc_amount: 50.0,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();

    let updated =
        pos_backend::db::invoices::update_invoice_status(&conn, "INV-PP1", "partially_paid", None)
            .unwrap();
    assert_eq!(updated, 1);

    let updated =
        pos_backend::db::invoices::update_invoice_status(&conn, "INV-PP1", "paid", Some("sig123"))
            .unwrap();
    assert_eq!(updated, 1);

    let inv = pos_backend::db::invoices::get_invoice_by_id(&conn, "INV-PP1")
        .unwrap()
        .unwrap();
    assert_eq!(inv.status, "paid");
}

#[test]
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
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();

    let result = pos_backend::db::invoices::initiate_refund(&conn, "INV-RF1").unwrap();
    assert!(
        !result,
        "371: initiate_refund on non-paid should return false"
    );
}

#[test]
fn test_372_sales_summary_all_cancelled() {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();

    pos_backend::db::invoices::create_invoice(
        &conn,
        &pos_backend::db::invoices::CreateInvoiceRequest {
            id: "INV-SC1".into(),
            reference_pubkey: "RefKeySC1".into(),
            fiat_currency: Some("USD".into()),
            fiat_amount: Some(10.0),
            usdc_amount: 10.0,
            telegram_chat_id: None,
            telegram_msg_id: None,
        },
    )
    .unwrap();
    pos_backend::db::invoices::cancel_invoice(&conn, "INV-SC1").unwrap();

    let summary = pos_backend::db::invoices::get_sales_summary(&conn).unwrap();
    let total_paid = summary["total_paid_invoices"].as_i64().unwrap_or(-1);
    assert_eq!(
        total_paid, 0,
        "372: total_paid should be 0 when all cancelled"
    );
}

#[test]
fn test_381_checkpoint_update_success() {
    let conn = common::setup_memory_db();
    pos_backend::db::sop_checkpoints::create_checkpoint(
        &conn,
        "cp-381",
        "sop-1",
        "step-1",
        Some("data"),
    )
    .unwrap();

    let updated =
        pos_backend::db::sop_checkpoints::update_checkpoint_status(&conn, "cp-381", "running")
            .unwrap();
    assert!(updated, "381: should return true for existing checkpoint");

    let status: String = conn
        .query_row(
            "SELECT status FROM sop_checkpoints WHERE id = 'cp-381'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(status, "running", "381: status should be 'running'");
}

#[test]
fn test_382_checkpoint_update_nonexistent() {
    let conn = common::setup_memory_db();
    let updated = pos_backend::db::sop_checkpoints::update_checkpoint_status(
        &conn,
        "nonexistent-id",
        "completed",
    )
    .unwrap();
    assert!(!updated, "382: should return false for nonexistent id");
}

#[test]
fn test_383_checkpoint_update_preserves_fields() {
    let conn = common::setup_memory_db();
    pos_backend::db::sop_checkpoints::create_checkpoint(
        &conn,
        "cp-383",
        "sop-xyz",
        "step-42",
        Some("important state"),
    )
    .unwrap();

    pos_backend::db::sop_checkpoints::update_checkpoint_status(&conn, "cp-383", "completed")
        .unwrap();

    let row: (String, String, Option<String>) = conn
        .query_row(
            "SELECT sop_id, step_id, state_data FROM sop_checkpoints WHERE id = 'cp-383'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();

    assert_eq!(row.0, "sop-xyz", "383: sop_id should be preserved");
    assert_eq!(row.1, "step-42", "383: step_id should be preserved");
    assert_eq!(
        row.2,
        Some("important state".to_string()),
        "383: state_data should be preserved"
    );
}

#[test]
fn test_384_checkpoint_update_transitions() {
    let conn = common::setup_memory_db();
    pos_backend::db::sop_checkpoints::create_checkpoint(&conn, "cp-384", "sop-1", "step-1", None)
        .unwrap();

    pos_backend::db::sop_checkpoints::update_checkpoint_status(&conn, "cp-384", "running").unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM sop_checkpoints WHERE id = 'cp-384'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        status, "running",
        "384: should be 'running' after first transition"
    );

    pos_backend::db::sop_checkpoints::update_checkpoint_status(&conn, "cp-384", "completed")
        .unwrap();
    let status: String = conn
        .query_row(
            "SELECT status FROM sop_checkpoints WHERE id = 'cp-384'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(
        status, "completed",
        "384: should be 'completed' after second transition"
    );
}

#[test]
fn test_385_checkpoint_update_empty_id() {
    let conn = common::setup_memory_db();
    let updated =
        pos_backend::db::sop_checkpoints::update_checkpoint_status(&conn, "", "completed").unwrap();
    assert!(!updated, "385: should return false for empty id");
}
