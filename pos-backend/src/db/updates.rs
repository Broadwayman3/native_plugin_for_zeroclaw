use rusqlite::{params, Connection};

/// Cleans up old update IDs older than 24 hours.
pub fn cleanup_old_updates(conn: &Connection) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "DELETE FROM processed_updates WHERE processed_at < datetime('now', '-1 day')",
        [],
    )
}

/// Checks if a Telegram update ID has already been processed in SQLite.
pub fn is_processed(conn: &Connection, update_id: i64) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM processed_updates WHERE update_id = ?1",
        params![update_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Registers a Telegram update ID for deduplication.
/// Returns true if the update is new, false if already processed.
pub fn check_and_register(conn: &Connection, update_id: i64) -> Result<bool, rusqlite::Error> {
    match conn.execute(
        "INSERT INTO processed_updates (update_id) VALUES (?1)",
        params![update_id],
    ) {
        Ok(_) => Ok(true),
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
                || err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
                || err.extended_code == 1555
                || err.extended_code == 2067 =>
        {
            // UNIQUE / PRIMARY KEY constraint violation = already processed
            Ok(false)
        }
        Err(e) => Err(e), // Propagate real DB errors (e.g. SQLITE_BUSY)
    }
}
