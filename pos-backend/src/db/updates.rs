use rusqlite::{params, Connection};

/// Cleans up old update IDs older than 24 hours, failed DLQ entries older than 7 days, and cancelled/stale pending webhook updates older than 2 days.
pub fn cleanup_old_updates(conn: &Connection) -> Result<usize, rusqlite::Error> {
    let deleted_processed = conn.execute(
        "DELETE FROM processed_updates WHERE processed_at < datetime('now', '-1 day')",
        [],
    )?;
    let deleted_failed = conn.execute(
        "DELETE FROM failed_updates WHERE failed_at < datetime('now', '-7 days')",
        [],
    )?;
    let deleted_pending = conn.execute(
        "DELETE FROM pending_webhook_updates WHERE status = 'cancelled' OR created_at < datetime('now', '-2 days')",
        [],
    )?;
    Ok(deleted_processed + deleted_failed + deleted_pending)
}

fn is_reset_event(payload: &str) -> bool {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(payload) {
        let msg = v.get("message").or_else(|| v.get("edited_message"));
        if let Some(text) = msg.and_then(|m| m.get("text")).and_then(|t| t.as_str()) {
            let trimmed = text.trim();
            if trimmed == "/cancel"
                || trimmed == "/start"
                || trimmed.starts_with("/cancel ")
                || trimmed.starts_with("/start ")
            {
                return true;
            }
        }
        if let Some(cb_data) = v
            .get("callback_query")
            .and_then(|cb| cb.get("data"))
            .and_then(|d| d.as_str())
        {
            if cb_data == "cancel" || cb_data == "menu_main" {
                return true;
            }
        }
    }
    false
}

/// Enqueues an incoming webhook update into pending_webhook_updates atomically using INSERT OR IGNORE.
/// If payload contains an exact reset command (/cancel, /start), cancels preceding pending retries for chat_id.
pub fn enqueue_webhook_update(
    conn: &Connection,
    update_id: i64,
    chat_id: Option<i64>,
    payload: &str,
) -> Result<bool, rusqlite::Error> {
    if is_reset_event(payload) {
        if let Some(cid) = chat_id {
            let _ = conn.execute(
                "UPDATE pending_webhook_updates SET status = 'cancelled' WHERE chat_id = ?1 AND update_id < ?2 AND status IN ('pending', 'retry_pending')",
                params![cid, update_id],
            );
        }
    }

    let count = conn.execute(
        "INSERT OR IGNORE INTO pending_webhook_updates (update_id, chat_id, payload, status) VALUES (?1, ?2, ?3, 'pending')",
        params![update_id, chat_id, payload],
    )?;
    Ok(count > 0)
}

struct TransactionRollbackGuard<'a>(&'a Connection, bool);

impl<'a> Drop for TransactionRollbackGuard<'a> {
    fn drop(&mut self) {
        if !self.1 {
            let _ = self.0.execute("ROLLBACK", []);
        }
    }
}

/// Fetches a batch of pending/unlocked webhook updates atomically using UPDATE...RETURNING with 30-second lease expiration.
pub fn fetch_pending_batch(
    conn: &Connection,
    limit: usize,
) -> Result<Vec<(i64, Option<i64>, String)>, rusqlite::Error> {
    conn.execute("BEGIN IMMEDIATE", [])?;
    let mut guard = TransactionRollbackGuard(conn, false);

    let mut stmt = conn.prepare(
        "UPDATE pending_webhook_updates
         SET status = 'processing', locked_at = CURRENT_TIMESTAMP
         WHERE update_id IN (
             SELECT p1.update_id FROM pending_webhook_updates p1
             WHERE (p1.status = 'pending'
                    OR (p1.status = 'retry_pending' AND (p1.next_retry_at IS NULL OR p1.next_retry_at <= datetime('now')))
                    OR (p1.status = 'processing' AND p1.locked_at < datetime('now', '-30 seconds')))
               AND (p1.chat_id IS NULL OR NOT EXISTS (
                   SELECT 1 FROM pending_webhook_updates p2
                   WHERE p2.chat_id = p1.chat_id
                     AND p2.update_id < p1.update_id
                     AND (p2.status = 'pending'
                          OR (p2.status = 'processing' AND p2.locked_at >= datetime('now', '-30 seconds'))
                          OR (p2.status = 'retry_pending' AND (p2.next_retry_at IS NULL OR p2.next_retry_at <= datetime('now'))))
               ))
             ORDER BY p1.update_id ASC LIMIT ?1
         ) RETURNING update_id, chat_id, payload",
    )?;

    let mapped = stmt.query_map(params![limit as i64], |row| {
        Ok((row.get(0)?, row.get(1)?, row.get(2)?))
    })?;
    let rows: Vec<(i64, Option<i64>, String)> = mapped.collect::<Result<Vec<_>, _>>()?;
    drop(stmt);

    conn.execute("COMMIT", [])?;
    guard.1 = true; // Successfully committed, disable rollback guard

    Ok(rows)
}

/// Reverts status from 'processing' back to 'pending' if in-flight claim failed.
pub fn revert_processing_status(conn: &Connection, update_id: i64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE pending_webhook_updates SET status = 'pending', locked_at = NULL WHERE update_id = ?1 AND status = 'processing'",
        params![update_id],
    )?;
    Ok(())
}

/// Removes a completed update from pending_webhook_updates.
pub fn mark_webhook_update_done(conn: &Connection, update_id: i64) -> Result<(), rusqlite::Error> {
    conn.execute(
        "DELETE FROM pending_webhook_updates WHERE update_id = ?1",
        params![update_id],
    )?;
    Ok(())
}

/// Marks a failed webhook update as 'retry_pending' with exponential backoff (5 * 2^(attempts-1) seconds).
pub fn mark_webhook_update_retry(
    conn: &Connection,
    update_id: i64,
    attempts: i32,
) -> Result<(), rusqlite::Error> {
    let backoff_secs = 5 * (1 << (attempts.saturating_sub(1).min(6)));
    conn.execute(
        "UPDATE pending_webhook_updates SET status = 'retry_pending', attempts = attempts + 1, next_retry_at = datetime('now', '+' || ?2 || ' seconds') WHERE update_id = ?1",
        params![update_id, backoff_secs],
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
        "INSERT INTO processed_updates (update_id, retry_count, status) VALUES (?1, ?2, 'failed')
         ON CONFLICT(update_id) DO UPDATE SET retry_count = ?2, status = 'failed'",
        params![update_id, retry_count],
    )?;
    conn.execute(
        "DELETE FROM pending_webhook_updates WHERE update_id = ?1",
        params![update_id],
    )?;
    Ok(())
}

/// Atomically increments attempt counter and either schedules exponential backoff retry or moves to DLQ.
/// Returns Ok(true) if update reached DLQ, Ok(false) if retry scheduled.
pub fn record_webhook_failure(
    conn: &Connection,
    update_id: i64,
    chat_id: Option<i64>,
    payload: &str,
    error_msg: &str,
    max_retries: i32,
) -> Result<bool, rusqlite::Error> {
    let current_attempts: i32 = conn
        .query_row(
            "SELECT attempts FROM pending_webhook_updates WHERE update_id = ?1",
            params![update_id],
            |row| row.get(0),
        )
        .unwrap_or(0);

    let new_attempts = current_attempts + 1;
    if new_attempts >= max_retries {
        move_to_dlq(conn, update_id, chat_id, payload, error_msg, new_attempts)?;
        Ok(true)
    } else {
        let backoff_secs = 5 * (1 << (new_attempts.saturating_sub(1).min(6)));
        conn.execute(
            "UPDATE pending_webhook_updates SET status = 'retry_pending', attempts = ?2, next_retry_at = datetime('now', '+' || ?3 || ' seconds') WHERE update_id = ?1",
            params![update_id, new_attempts, backoff_secs],
        )?;
        conn.execute(
            "INSERT INTO processed_updates (update_id, retry_count, status) VALUES (?1, ?2, 'retry_pending')
             ON CONFLICT(update_id) DO UPDATE SET retry_count = ?2, status = 'retry_pending'",
            params![update_id, new_attempts],
        )?;
        Ok(false)
    }
}

/// Checks if a Telegram update ID has already been completely processed or DLQ'd in SQLite.
/// Returns false if status is 'retry_pending' and backoff timer has expired, allowing retry.
pub fn is_processed(conn: &Connection, update_id: i64) -> Result<bool, rusqlite::Error> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM processed_updates WHERE update_id = ?1 AND (status IN ('processed', 'failed') OR (status = 'retry_pending' AND EXISTS (SELECT 1 FROM pending_webhook_updates WHERE update_id = ?1 AND next_retry_at > datetime('now'))))",
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
