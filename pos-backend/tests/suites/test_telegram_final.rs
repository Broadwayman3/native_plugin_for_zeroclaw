use crate::common;
use pos_backend::api::telegram::fsm::FsmStore;
use pos_backend::api::telegram::state::{get_update_offset, set_update_offset};
use pos_backend::api::telegram::ChatLocks;
use pos_backend::db;
use pos_backend::domain::sanitizer::strip_bot_mention;
use std::sync::Arc;

#[tokio::test]
async fn test_390_fsm_composite_key_and_ttl() {
    let guard = common::TempDbGuard::new("test_390");
    let fsm = FsmStore::new_with_db(guard.path().into());

    let group_chat_id = -100123456789;
    let user_a = 111;
    let user_b = 222;

    // User A sets pending item "Cappuccino"
    fsm.set_pending(
        group_chat_id,
        user_a,
        "Cappuccino".into(),
        Some("UAH".into()),
    )
    .await;

    // User B has no pending item
    let pending_b = fsm.get_pending(group_chat_id, user_b).await;
    assert!(
        pending_b.is_none(),
        "390: User B should have no pending item"
    );

    // User A gets their pending item
    let pending_a = fsm.get_pending(group_chat_id, user_a).await;
    assert!(pending_a.is_some(), "390: User A should have pending item");
    assert_eq!(pending_a.unwrap().item_name, "Cappuccino");

    // Clearing User A does not affect User B
    fsm.clear(group_chat_id, user_a).await;
    assert!(fsm.get_pending(group_chat_id, user_a).await.is_none());
}

#[test]
fn test_391_strip_bot_mention_normalization() {
    assert_eq!(strip_bot_mention("/start@zero_claw_pos_bot"), "/start");
    assert_eq!(
        strip_bot_mention("/refund@zero_claw_pos_bot INV-001 5.0"),
        "/refund INV-001 5.0"
    );
    assert_eq!(strip_bot_mention("hello@world"), "hello@world");
    assert_eq!(strip_bot_mention("   /start   "), "/start");
}

#[test]
fn test_392_sqlite_immediate_offset_persistence() {
    let guard = common::TempDbGuard::new("tg_offset");
    let conn = db::get_db_connection(guard.path()).unwrap();
    db::init_db(&conn, false).unwrap();

    // Initial offset is 0
    assert_eq!(get_update_offset(guard.path()), 0);

    // Update offset immediately persists to SQLite
    set_update_offset(guard.path(), 5001);
    assert_eq!(get_update_offset(guard.path()), 5001);

    // Verify reading from a new DB connection restores 5001
    let val = db::invoices::get_system_setting(&conn, "telegram_update_offset").unwrap();
    assert_eq!(val, Some("5001".to_string()));
}

#[test]
fn test_393_update_deduplication_sqlite() {
    let guard = common::TempDbGuard::new("tg_dedup");
    let conn = db::get_db_connection(guard.path()).unwrap();
    db::init_db(&conn, false).unwrap();

    let update_id = 998877;

    assert!(!db::updates::is_processed(&conn, update_id).unwrap());

    // First registration succeeds (returns true)
    let is_new_1 = db::updates::check_and_register(&conn, update_id).unwrap();
    assert!(
        is_new_1,
        "393: First update registration should return true"
    );

    assert!(db::updates::is_processed(&conn, update_id).unwrap());

    // Second registration of same update_id returns false (duplicate)
    let is_new_2 = db::updates::check_and_register(&conn, update_id).unwrap();
    assert!(
        !is_new_2,
        "393: Duplicate update registration should return false"
    );
}

#[test]
fn test_394_atomic_invoice_cancellation() {
    let guard = common::TempDbGuard::new("tg_cancel");
    let conn = db::get_db_connection(guard.path()).unwrap();
    db::init_db(&conn, false).unwrap();

    let inv_id = "INV-CANCEL-394";
    let req = db::invoices::CreateInvoiceRequest {
        id: inv_id.to_string(),
        reference_pubkey: "RefKey394111111111111111111111111111111111".to_string(),
        fiat_currency: Some("USD".to_string()),
        fiat_amount: Some(10.0),
        usdc_amount: 10.0,
        telegram_chat_id: Some(123456),
        telegram_msg_id: Some(7890),
    };
    db::invoices::create_invoice(&conn, &req).unwrap();

    // Cancel pending invoice succeeds
    let count1 = db::invoices::cancel_invoice(&conn, inv_id).unwrap();
    assert_eq!(
        count1, 1,
        "394: First cancel on pending invoice should succeed"
    );

    // Second cancel returns 0
    let count2 = db::invoices::cancel_invoice(&conn, inv_id).unwrap();
    assert_eq!(
        count2, 0,
        "394: Cancel on already cancelled invoice should return 0"
    );
}

#[test]
fn test_395_chat_locks_gc_cleanup() {
    let chat_locks = ChatLocks::new();
    let target_chat_id = 9988776655;

    let user_id = 1001;
    let lock1 = chat_locks.get_or_create(target_chat_id, user_id);
    let lock2 = chat_locks.get_or_create(target_chat_id, user_id);

    assert!(Arc::ptr_eq(&lock1, &lock2));
    let weak2 = Arc::downgrade(&lock2);
    drop(lock1);
    drop(lock2);

    assert!(weak2.upgrade().is_none());

    let lock3 = chat_locks.get_or_create(target_chat_id, user_id);
    assert_eq!(Arc::strong_count(&lock3), 1);
}

#[test]
fn test_396_composite_chat_locks_isolation() {
    let chat_locks = ChatLocks::new();
    let group_chat_id = 9988776655;

    let user_a_lock = chat_locks.get_or_create(group_chat_id, 1001);
    let user_b_lock = chat_locks.get_or_create(group_chat_id, 1002);

    // Locks for different users in the same group chat must NOT be identical
    assert!(!Arc::ptr_eq(&user_a_lock, &user_b_lock));
}
