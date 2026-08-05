use rusqlite::{params, Connection, OptionalExtension};

/// Nonce account TTL in minutes for stale lock auto-healing
const NONCE_TTL_MINUTES: i64 = 2;

/// Atomically allocates a free Nonce account with TTL auto-release.
/// Supports SQLite >= 3.35.0 (RETURNING) with fallback for older versions.
pub fn allocate_free_nonce(conn: &Connection) -> Result<Option<String>, rusqlite::Error> {
    // 1. Auto-release locks hanging for >2 minutes or stale_needs_refresh >2 minutes
    let interval = format!("-{} minutes", NONCE_TTL_MINUTES);
    conn.execute(
        "UPDATE nonce_accounts SET status = 'free', locked_at = NULL
         WHERE (status = 'locked' OR status = 'stale_needs_refresh')
           AND locked_at < datetime('now', ?1)",
        params![interval],
    )?;

    // 2. Try RETURNING (SQLite >= 3.35.0)
    let result = conn.query_row(
        "UPDATE nonce_accounts SET status = 'locked', locked_at = CURRENT_TIMESTAMP
         WHERE pubkey = (SELECT pubkey FROM nonce_accounts WHERE status = 'free' LIMIT 1)
         RETURNING pubkey",
        [],
        |row| row.get::<_, String>(0),
    );

    match result {
        Ok(pubkey) => Ok(Some(pubkey)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(_) => {
            // Fallback: SELECT + UPDATE (SQLite < 3.35.0)
            let pubkey: Option<String> = conn
                .query_row(
                    "SELECT pubkey FROM nonce_accounts WHERE status = 'free' LIMIT 1",
                    [],
                    |row| row.get(0),
                )
                .optional()?;

            if let Some(pk) = pubkey {
                conn.execute_batch("BEGIN IMMEDIATE")?;
                let updated = conn.execute(
                    "UPDATE nonce_accounts SET status = 'locked', locked_at = CURRENT_TIMESTAMP
                 WHERE pubkey = ?1 AND status = 'free'",
                    params![pk],
                )?;
                if updated > 0 {
                    conn.execute_batch("COMMIT")?;
                    return Ok(Some(pk));
                }
                conn.execute_batch("ROLLBACK")?;
            }
            Ok(None)
        }
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

/// Updates the stored nonce_blockhash for a nonce account after RPC synchronization.
pub fn update_nonce_blockhash(
    conn: &Connection,
    pubkey: &str,
    new_blockhash: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE nonce_accounts SET nonce_blockhash = ?1, status = 'free', locked_at = NULL WHERE pubkey = ?2",
        params![new_blockhash, pubkey],
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
