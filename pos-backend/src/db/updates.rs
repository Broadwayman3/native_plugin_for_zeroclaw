use rusqlite::{params, Connection};

/// Checks and registers a Telegram update ID for deduplication.
/// Returns true if the update is new, false if already processed.
pub fn check_and_register(conn: &Connection, update_id: i64) -> Result<bool, rusqlite::Error> {
    // Cleanup old updates (>24h)
    conn.execute(
        "DELETE FROM processed_updates WHERE processed_at < datetime('now', '-1 day')",
        [],
    )?;

    match conn.execute(
        "INSERT INTO processed_updates (update_id) VALUES (?1)",
        params![update_id],
    ) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false), // UNIQUE constraint violation = already processed
    }
}
