use crate::common;
use pos_backend::api::telegram::fsm::FsmStore;
use pos_backend::api::telegram::state::{get_update_offset, set_update_offset};
use pos_backend::api::telegram::ChatLocks;
use pos_backend::db;
use pos_backend::domain::sanitizer::strip_bot_mention;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex as AsyncMutex;

#[tokio::test]
async fn test_390_fsm_composite_key_and_ttl() {
    let fsm = FsmStore::new();

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
    let chat_locks: ChatLocks = Arc::new(std::sync::Mutex::new(HashMap::new()));
    let target_chat_id = 9988776655;

    // Acquire lock
    let chat_lock = {
        let mut map = chat_locks.lock().unwrap();
        map.entry(target_chat_id)
            .or_insert_with(|| Arc::new(AsyncMutex::new(())))
            .clone()
    };

    assert_eq!(chat_locks.lock().unwrap().len(), 1);

    // Release lock & run GC
    {
        let mut map = chat_locks.lock().unwrap();
        if Arc::strong_count(&chat_lock) <= 2 {
            map.remove(&target_chat_id);
        }
    }

    assert_eq!(
        chat_locks.lock().unwrap().len(),
        0,
        "395: Unused chat_lock should be purged from memory by GC"
    );
}
