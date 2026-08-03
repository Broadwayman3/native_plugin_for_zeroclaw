use rusqlite::{params, Connection};

/// Nonce account TTL in minutes
const NONCE_TTL_MINUTES: i64 = 15;

/// Atomically allocates a free Nonce account with TTL auto-release.
pub fn allocate_free_nonce(conn: &Connection) -> Result<Option<String>, rusqlite::Error> {
    // 1. Auto-release locks hanging for >15 minutes
    conn.execute(
        "UPDATE nonce_accounts SET status = 'free', locked_at = NULL
         WHERE status = 'locked' AND locked_at < datetime('now', '-' || ?1 || ' minutes')",
        params![NONCE_TTL_MINUTES.to_string()],
    )?;

    // 2. Try to atomically allocate a free nonce using subquery
    let updated = conn.execute(
        "UPDATE nonce_accounts
         SET status = 'locked', locked_at = CURRENT_TIMESTAMP
         WHERE pubkey = (SELECT pubkey FROM nonce_accounts WHERE status = 'free' LIMIT 1)",
        [],
    )?;

    if updated > 0 {
        // Get the pubkey we just locked
        let pubkey: String = conn.query_row(
            "SELECT pubkey FROM nonce_accounts WHERE status = 'locked' ORDER BY locked_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )?;
        Ok(Some(pubkey))
    } else {
        Ok(None)
    }
}

/// Releases a locked Nonce account back to the free pool.
pub fn release_nonce(conn: &Connection, pubkey: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE pubkey = ?1",
        params![pubkey],
    )?;
    Ok(())
}

/// Marks a nonce account as stale_needs_refresh when a transaction reverts.
pub fn mark_nonce_stale(conn: &Connection, pubkey: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE nonce_accounts
         SET status = 'stale_needs_refresh', locked_at = CURRENT_TIMESTAMP
         WHERE pubkey = ?1",
        params![pubkey],
    )?;
    Ok(())
}

/// Refreshes a stale nonce account after on-chain state fetch.
pub fn refresh_stale_nonce(conn: &Connection, pubkey: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE nonce_accounts SET status = 'free', locked_at = NULL WHERE pubkey = ?1",
        params![pubkey],
    )?;
    Ok(())
}

/// Validates Solana Runtime rule: AdvanceNonceAccount must be the first non-budget execution instruction.
pub fn verify_nonce_instruction_ordering(instructions: &[serde_json::Value]) -> bool {
    if instructions.is_empty() {
        return true;
    }

    let has_nonce = instructions
        .iter()
        .any(|ix| ix.get("instruction").and_then(|v| v.as_str()) == Some("AdvanceNonceAccount"));

    if !has_nonce {
        return true;
    }

    // Filter out Compute Budget instructions
    let exec_instructions: Vec<&serde_json::Value> = instructions
        .iter()
        .filter(|ix| {
            let instr = ix.get("instruction").and_then(|v| v.as_str()).unwrap_or("");
            instr != "SetComputeUnitPrice" && instr != "SetComputeUnitLimit"
        })
        .collect();

    if exec_instructions.is_empty() {
        return true;
    }

    exec_instructions[0]
        .get("instruction")
        .and_then(|v| v.as_str())
        == Some("AdvanceNonceAccount")
}
