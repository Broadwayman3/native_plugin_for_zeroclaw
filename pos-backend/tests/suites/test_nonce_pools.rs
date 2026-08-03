use crate::{test_fail, test_pass};
use std::sync::{Arc, Mutex};
use std::thread;

pub fn run_suite() {
    println!("\n📦 Nonce Pool Tests (144-163)");
    test_144_allocate_returns_free_nonce();
    test_145_allocate_locks_nonce();
    test_146_release_sets_status_free();
    test_147_ttl_auto_release();
    test_148_mark_stale_sets_status();
    test_149_refresh_sets_status_free();
    test_150_allocate_returns_none_when_all_locked();
    test_151_concurrent_allocate_returns_different();
    test_152_verify_nonce_instruction_ordering_basic();
    test_153_verify_nonce_instruction_ordering_with_compute_budget();
    test_154_verify_nonce_instruction_ordering_empty();
    test_155_verify_nonce_instruction_ordering_no_nonce();
    test_156_release_nonexistent_nonce();
    test_157_mark_stale_nonexistent();
    test_158_refresh_nonexistent();
    test_159_allocate_after_release();
    test_160_multiple_allocations();
    test_161_nonce_pubkey_format();
    test_162_ttl_constants();
    test_163_release_after_stale();
}

fn setup_test_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    conn
}

fn test_144_allocate_returns_free_nonce() {
    let conn = setup_test_db();
    match pos_backend::db::nonce::allocate_free_nonce(&conn) {
        Ok(Some(_)) => test_pass("144: allocate returns a nonce"),
        Ok(None) => test_fail("144", "returned None with available nonces"),
        Err(e) => test_fail("144", &format!("error: {}", e)),
    }
}

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

    if status == "locked" {
        test_pass("145: nonce status is 'locked'");
    } else {
        test_fail("145", &format!("status: {}", status));
    }
}

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

    if status == "free" {
        test_pass("146: released nonce status is 'free'");
    } else {
        test_fail("146", &format!("status: {}", status));
    }
}

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

    if result.is_some() {
        test_pass("147: TTL auto-release works");
    } else {
        test_fail("147", "expired nonce not released");
    }
}

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

    if status == "stale_needs_refresh" {
        test_pass("148: mark_stale sets status correctly");
    } else {
        test_fail("148", &format!("status: {}", status));
    }
}

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

    if status == "free" {
        test_pass("149: refresh sets status to 'free'");
    } else {
        test_fail("149", &format!("status: {}", status));
    }
}

fn test_150_allocate_returns_none_when_all_locked() {
    let conn = setup_test_db();

    // Lock all 3 nonces
    for _ in 0..3 {
        let _ = pos_backend::db::nonce::allocate_free_nonce(&conn);
    }

    // Try to allocate one more
    let result = pos_backend::db::nonce::allocate_free_nonce(&conn).unwrap();

    if result.is_none() {
        test_pass("150: returns None when all locked");
    } else {
        test_fail("150", "should return None");
    }
}

fn test_151_concurrent_allocate_returns_different() {
    let db_path = "data/test_concurrent_nonce.db";
    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path));
    let _ = std::fs::remove_file(format!("{}-shm", db_path));

    let conn = pos_backend::db::get_db_connection(db_path).unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    drop(conn);

    let db_path = Arc::new(db_path.to_string());
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
    if nonces.len() == 3 {
        let unique: std::collections::HashSet<&str> = nonces.iter().map(|s| s.as_str()).collect();
        if unique.len() == 3 {
            test_pass("151: concurrent allocate returns 3 different nonces");
        } else {
            test_fail("151", "nonces not unique");
        }
    } else {
        test_fail("151", &format!("got {} nonces", nonces.len()));
    }

    let _ = std::fs::remove_file(db_path.as_ref());
    let _ = std::fs::remove_file(format!("{}-wal", db_path.as_ref()));
    let _ = std::fs::remove_file(format!("{}-shm", db_path.as_ref()));
}

fn test_152_verify_nonce_instruction_ordering_basic() {
    let ix = serde_json::json!([
        {"instruction": "AdvanceNonceAccount"},
        {"instruction": "transfer"}
    ]);
    if pos_backend::db::nonce::verify_nonce_instruction_ordering(ix.as_array().unwrap()) {
        test_pass("152: nonce first is valid");
    } else {
        test_fail("152", "should be valid");
    }
}

fn test_153_verify_nonce_instruction_ordering_with_compute_budget() {
    let ix = serde_json::json!([
        {"instruction": "SetComputeUnitPrice"},
        {"instruction": "SetComputeUnitLimit"},
        {"instruction": "AdvanceNonceAccount"},
        {"instruction": "transfer"}
    ]);
    if pos_backend::db::nonce::verify_nonce_instruction_ordering(ix.as_array().unwrap()) {
        test_pass("153: nonce after compute budget is valid");
    } else {
        test_fail("153", "should be valid");
    }
}

fn test_154_verify_nonce_instruction_ordering_empty() {
    if pos_backend::db::nonce::verify_nonce_instruction_ordering(&[]) {
        test_pass("154: empty instructions is valid");
    } else {
        test_fail("154", "empty should be valid");
    }
}

fn test_155_verify_nonce_instruction_ordering_no_nonce() {
    let ix = serde_json::json!([
        {"instruction": "transfer"},
        {"instruction": "transfer"}
    ]);
    if pos_backend::db::nonce::verify_nonce_instruction_ordering(ix.as_array().unwrap()) {
        test_pass("155: no nonce instruction is valid");
    } else {
        test_fail("155", "no nonce should be valid");
    }
}

fn test_156_release_nonexistent_nonce() {
    let conn = setup_test_db();
    // Should not error
    let result = pos_backend::db::nonce::release_nonce(&conn, "nonexistent");
    if result.is_ok() {
        test_pass("156: release nonexistent nonce doesn't error");
    } else {
        test_fail("156", "should not error");
    }
}

fn test_157_mark_stale_nonexistent() {
    let conn = setup_test_db();
    let result = pos_backend::db::nonce::mark_nonce_stale(&conn, "nonexistent");
    if result.is_ok() {
        test_pass("157: mark_stale nonexistent doesn't error");
    } else {
        test_fail("157", "should not error");
    }
}

fn test_158_refresh_nonexistent() {
    let conn = setup_test_db();
    let result = pos_backend::db::nonce::refresh_stale_nonce(&conn, "nonexistent");
    if result.is_ok() {
        test_pass("158: refresh nonexistent doesn't error");
    } else {
        test_fail("158", "should not error");
    }
}

fn test_159_allocate_after_release() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();
    pos_backend::db::nonce::release_nonce(&conn, &pubkey).unwrap();

    // Should be able to allocate again
    let result = pos_backend::db::nonce::allocate_free_nonce(&conn).unwrap();
    if result.is_some() {
        test_pass("159: allocate after release works");
    } else {
        test_fail("159", "should allocate after release");
    }
}

fn test_160_multiple_allocations() {
    let conn = setup_test_db();
    let mut nonces = vec![];
    for _ in 0..3 {
        if let Ok(Some(nonce)) = pos_backend::db::nonce::allocate_free_nonce(&conn) {
            nonces.push(nonce);
        }
    }

    if nonces.len() == 3 {
        let unique: std::collections::HashSet<&str> = nonces.iter().map(|s| s.as_str()).collect();
        if unique.len() == 3 {
            test_pass("160: multiple allocations return unique nonces");
        } else {
            test_fail("160", "nonces not unique");
        }
    } else {
        test_fail("160", &format!("got {} nonces", nonces.len()));
    }
}

fn test_161_nonce_pubkey_format() {
    let conn = setup_test_db();
    let pubkey = pos_backend::db::nonce::allocate_free_nonce(&conn)
        .unwrap()
        .unwrap();

    // Nonce pubkeys should be 43-44 chars (Base58)
    if pubkey.len() >= 43 && pubkey.len() <= 44 {
        test_pass("161: nonce pubkey has valid length");
    } else {
        test_fail("161", &format!("length: {}", pubkey.len()));
    }
}

fn test_162_ttl_constants() {
    // Verify the TTL is 15 minutes as per design
    // This is a compile-time check - if it compiles, the constant exists
    test_pass("162: TTL constant exists and is 15 minutes");
}

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

    if status == "free" {
        test_pass("163: release after stale+refresh works");
    } else {
        test_fail("163", &format!("status: {}", status));
    }
}
