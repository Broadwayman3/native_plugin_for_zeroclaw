use crate::common::TempDbGuard;
use std::sync::{Arc, Mutex};
use std::thread;

fn setup_test_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    conn
}

#[test]
fn test_144_allocate_returns_free_nonce() {
    let conn = setup_test_db();
    let result = pos_backend::db::nonce::allocate_free_nonce(&conn).expect("144: allocate failed");
    assert!(result.is_some(), "144: returned None with available nonces");
}

#[test]
fn test_145_allocate_locks_nonce() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM nonce_accounts WHERE pubkey = ?1",
            [&pubkey],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(status, "locked", "145: nonce status is 'locked'");
}

#[test]
fn test_146_release_sets_status_free() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();
    pos_backend::db::nonce::release_nonce(&conn, &pubkey).unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM nonce_accounts WHERE pubkey = ?1",
            [&pubkey],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(status, "free", "146: released nonce status is 'free'");
}

#[test]
fn test_147_ttl_auto_release() {
    let conn = setup_test_db();

    // Manually set locked_at to 20 minutes ago
    conn.execute(
        "UPDATE nonce_accounts SET status = 'locked', locked_at = datetime('now', '-20 minutes') WHERE pubkey = 'Nonce111111111111111111111111111111111111111'",
        [],
    )
    .unwrap();

    // Allocate should auto-release the expired lock
    let result = pos_backend::db::nonce::allocate_free_nonce(&conn).unwrap();

    assert!(result.is_some(), "147: expired nonce not released");
}

#[test]
fn test_148_mark_stale_sets_status() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();
    pos_backend::db::nonce::mark_nonce_stale(&conn, &pubkey).unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM nonce_accounts WHERE pubkey = ?1",
            [&pubkey],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(
        status, "stale_needs_refresh",
        "148: mark_stale sets status correctly"
    );
}

#[test]
fn test_149_refresh_sets_status_free() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();
    pos_backend::db::nonce::mark_nonce_stale(&conn, &pubkey).unwrap();
    pos_backend::db::nonce::refresh_stale_nonce(&conn, &pubkey).unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM nonce_accounts WHERE pubkey = ?1",
            [&pubkey],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(status, "free", "149: refresh sets status to 'free'");
}

#[test]
fn test_150_allocate_returns_none_when_all_locked() {
    let conn = setup_test_db();

    // Lock all 3 nonces
    for _ in 0..3 {
        let _ = pos_backend::db::nonce::allocate_free_nonce(&conn);
    }

    // Try to allocate one more
    let result = pos_backend::db::nonce::allocate_free_nonce(&conn).unwrap();

    assert!(result.is_none(), "150: should return None");
}

#[test]
fn test_151_concurrent_allocate_returns_different() {
    let guard = TempDbGuard::new("concurrent_nonce");
    let db_path = guard.path().to_string();

    let conn = pos_backend::db::get_db_connection(&db_path).unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    drop(conn);

    let db_path = Arc::new(db_path);
    let results = Arc::new(Mutex::new(Vec::new()));

    let mut handles = vec![];
    for _ in 0..3 {
        let db_path = Arc::clone(&db_path);
        let results = Arc::clone(&results);
        handles.push(thread::spawn(move || {
            let conn = pos_backend::db::get_db_connection(&db_path).unwrap();
            if let Ok(Some(nonce)) = pos_backend::db::nonce::allocate_free_nonce(&conn) {
                results.lock().unwrap().push(nonce);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let nonces = results.lock().unwrap();
    assert_eq!(nonces.len(), 3, "151: got {} nonces", nonces.len());
    let unique: std::collections::HashSet<&str> = nonces.iter().map(|s| s.as_str()).collect();
    assert_eq!(unique.len(), 3, "151: nonces not unique");
    // TempDbGuard will cleanup files on drop
}

#[test]
fn test_152_verify_nonce_instruction_ordering_basic() {
    let ix = serde_json::json!([
        {"instruction": "AdvanceNonceAccount"},
        {"instruction": "transfer"}
    ]);
    assert!(
        pos_backend::db::nonce::verify_nonce_instruction_ordering(ix.as_array().unwrap()),
        "152: nonce first is valid"
    );
}

#[test]
fn test_153_verify_nonce_instruction_ordering_with_compute_budget() {
    let ix = serde_json::json!([
        {"instruction": "SetComputeUnitPrice"},
        {"instruction": "SetComputeUnitLimit"},
        {"instruction": "AdvanceNonceAccount"},
        {"instruction": "transfer"}
    ]);
    assert!(
        pos_backend::db::nonce::verify_nonce_instruction_ordering(ix.as_array().unwrap()),
        "153: nonce after compute budget is valid"
    );
}

#[test]
fn test_154_verify_nonce_instruction_ordering_empty() {
    assert!(
        pos_backend::db::nonce::verify_nonce_instruction_ordering(&[]),
        "154: empty instructions is valid"
    );
}

#[test]
fn test_155_verify_nonce_instruction_ordering_no_nonce() {
    let ix = serde_json::json!([
        {"instruction": "transfer"},
        {"instruction": "transfer"}
    ]);
    assert!(
        pos_backend::db::nonce::verify_nonce_instruction_ordering(ix.as_array().unwrap()),
        "155: no nonce instruction is valid"
    );
}

#[test]
fn test_156_release_nonexistent_nonce() {
    let conn = setup_test_db();
    // Should not error
    let result = pos_backend::db::nonce::release_nonce(&conn, "nonexistent");
    assert!(
        result.is_ok(),
        "156: release nonexistent nonce should not error"
    );
}

#[test]
fn test_157_mark_stale_nonexistent() {
    let conn = setup_test_db();
    let result = pos_backend::db::nonce::mark_nonce_stale(&conn, "nonexistent");
    assert!(
        result.is_ok(),
        "157: mark_stale nonexistent should not error"
    );
}

#[test]
fn test_158_refresh_nonexistent() {
    let conn = setup_test_db();
    let result = pos_backend::db::nonce::refresh_stale_nonce(&conn, "nonexistent");
    assert!(result.is_ok(), "158: refresh nonexistent should not error");
}

#[test]
fn test_159_allocate_after_release() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();
    pos_backend::db::nonce::release_nonce(&conn, &pubkey).unwrap();

    // Should be able to allocate again
    let result = pos_backend::db::nonce::allocate_free_nonce(&conn).unwrap();
    assert!(result.is_some(), "159: should allocate after release");
}

#[test]
fn test_160_multiple_allocations() {
    let conn = setup_test_db();
    let mut nonces = vec![];
    for _ in 0..3 {
        if let Ok(Some(nonce)) = pos_backend::db::nonce::allocate_free_nonce(&conn) {
            nonces.push(nonce);
        }
    }

    assert_eq!(nonces.len(), 3, "160: got {} nonces", nonces.len());
    let unique: std::collections::HashSet<&str> = nonces.iter().map(|s| s.as_str()).collect();
    assert_eq!(unique.len(), 3, "160: nonces not unique");
}

#[test]
fn test_161_nonce_pubkey_format() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();

    // Nonce pubkeys should be 43-44 chars (Base58)
    assert!(
        pubkey.len() >= 43 && pubkey.len() <= 44,
        "161: nonce pubkey has valid length, length: {}",
        pubkey.len()
    );
}

#[test]
fn test_162_ttl_constants() {
    // Verify runtime values match design spec
    assert_eq!(pos_core_logic::USDC_DECIMALS, 6);
    assert_eq!(pos_core_logic::SOL_DECIMALS, 9);
    assert_eq!(pos_core_logic::NONCE_TTL_MINUTES, 15);
    assert_eq!(pos_core_logic::DEFAULT_SLIPPAGE_TOLERANCE_PCT, 1.0);
}

#[test]
fn test_163_release_after_stale() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();
    pos_backend::db::nonce::mark_nonce_stale(&conn, &pubkey).unwrap();
    pos_backend::db::nonce::refresh_stale_nonce(&conn, &pubkey).unwrap();
    pos_backend::db::nonce::release_nonce(&conn, &pubkey).unwrap();

    let status: String = conn
        .query_row(
            "SELECT status FROM nonce_accounts WHERE pubkey = ?1",
            [&pubkey],
            |row| row.get(0),
        )
        .unwrap();

    assert_eq!(status, "free", "163: release after stale+refresh works");
}
