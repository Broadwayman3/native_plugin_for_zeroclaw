use rusqlite::{params, Connection};

/// Cleans up old update IDs older than 24 hours and failed DLQ entries older than 7 days.
pub fn cleanup_old_updates(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let deleted_processed = conn.execute(
        "DELETE FROM processed_updates WHERE processed_at < datetime('now', '-1 day')",
        [],
    )?;
    let deleted_failed = conn.execute(
        "DELETE FROM failed_updates WHERE failed_at < datetime('now', '-7 days')",
        [],
    )?;
    Ok(deleted_processed + deleted_failed)
}

/// Enqueues an incoming webhook update into pending_webhook_updates atomically using INSERT OR IGNORE.
pub fn enqueue_webhook_update(
    conn: &Connection,
    update_id: i64,
    chat_id: Option<i64>,
    payload: &str,
) -> Result<bool, rusqlite::Error> {
    let count = conn.execute(
        "INSERT OR IGNORE INTO pending_webhook_updates (update_id, chat_id, payload, status) VALUES (?1, ?2, ?3, 'pending')",
        params![update_id, chat_id, payload],
    )?;
    Ok(count > 0)
}

/// Fetches a batch of pending/unlocked webhook updates and atomically locks them in SQLite transaction.
pub fn fetch_pending_batch(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<(i64, Option<i64>, String)>, rusqlite::Error> {
    let tx = conn.unchecked_transaction()?;
    let rows: Vec<(i64, Option<i64>, String)> = {
        let mut stmt = tx.prepare(
            "SELECT update_id, chat_id, payload FROM pending_webhook_updates 
             WHERE status = 'pending' OR (status = 'processing' AND locked_at < datetime('now', '-5 minutes'))
             ORDER BY created_at ASC LIMIT ?1",
        )?;
        let mapped = stmt.query_map(params![limit as i64], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        mapped.collect::<Result<Vec<_>, _>>()?
    };

    for (update_id, _, _) in &rows {
        tx.execute(
            "UPDATE pending_webhook_updates SET status = 'processing', locked_at = datetime('now') WHERE update_id = ?1",
            params![update_id],
        )?;
    }
    tx.commit()?;
    Ok(rows)
}

/// Removes a completed update from pending_webhook_updates.
pub fn mark_webhook_update_done(conn: &Connection, update_id: i64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM pending_webhook_updates WHERE update_id = ?1",
        params![update_id],
    )?;
    Ok(())
}

/// Atomically moves an update to failed_updates (DLQ) after max retries and removes from pending queue.
pub fn move_to_dlq(
    conn: &Connection,
    update_id: i64,
    chat_id: Option<i64>,
    payload: &str,
    error_msg: &str,
    retry_count: i32,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT OR REPLACE INTO failed_updates (update_id, chat_id, payload, error_message, retry_count) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![update_id, chat_id, payload, error_msg, retry_count],
    )?;
    conn.execute(
        "DELETE FROM pending_webhook_updates WHERE update_id = ?1",
        params![update_id],
    )?;
    Ok(())
}

/// Checks if a Telegram update ID has already been completely processed or DLQ'd in SQLite.
/// Returns false if status is 'retry_pending' so that retries can proceed.
pub fn is_processed(conn: &Connection, update_id: i64) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM processed_updates WHERE update_id = ?1 AND status IN ('processed', 'failed')",
        params![update_id],
        |row| row.get(0),
    )?;
    Ok(count > 0)
}

/// Registers a Telegram update ID for deduplication.
/// Returns true if the update is new or upgraded from retry_pending, false if already processed.
pub fn check_and_register(conn: &Connection, update_id: i64) -> Result<bool, rusqlite::Error> {
    match conn.execute(
        "INSERT INTO processed_updates (update_id, status) VALUES (?1, 'processed')",
        params![update_id],
    ) {
        Ok(_) => Ok(true),
        Err(rusqlite::Error::SqliteFailure(err, _))
            if err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
                || err.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
                || err.extended_code == 1555
                || err.extended_code == 2067 =>
        {
            let updated = conn.execute(
                "UPDATE processed_updates SET status = 'processed' WHERE update_id = ?1 AND status = 'retry_pending'",
                params![update_id],
            )?;
            if updated > 0 {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(e) => Err(e),
    }
}

/// Records a failure for an update_id, incrementing retry_count.
/// If retry_count reaches max_retries (default 3), sets status to 'failed' and returns Ok(true)
/// indicating that this update should be marked as DLQ'd to allow offset advancement.
pub fn record_failure_and_check_max_retries(
    conn: &Connection,
    update_id: i64,
    max_retries: i32,
) -> Result<bool, rusqlite::Error> {
    let current_retry: i32 = conn
        .query_row(
            "SELECT retry_count FROM processed_updates WHERE update_id = ?1",
            params![update_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let new_retry = current_retry + 1;
    if new_retry >= max_retries {
        conn.execute(
            "INSERT INTO processed_updates (update_id, retry_count, status) VALUES (?1, ?2, 'failed')
             ON CONFLICT(update_id) DO UPDATE SET retry_count = ?2, status = 'failed'",
            params![update_id, new_retry],
        )?;
        Ok(true) // Reached max retries -> DLQ'd -> allow offset advancement
    } else {
        conn.execute(
            "INSERT INTO processed_updates (update_id, retry_count, status) VALUES (?1, ?2, 'retry_pending')
             ON CONFLICT(update_id) DO UPDATE SET retry_count = ?2, status = 'retry_pending'",
            params![update_id, new_retry],
        )?;
        Ok(false) // Retries remaining -> keep retrying
    }
}
