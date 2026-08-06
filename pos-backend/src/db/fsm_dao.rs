use rusqlite::{params, Connection, Result};
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns current Unix epoch timestamp in seconds.
pub fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Sets or updates an FSM session for (chat_id, user_id).
pub fn set_session(
    conn: &Connection,
    chat_id: i64,
    user_id: i64,
    state: &str,
    payload_json: &str,
) -> Result<()> {
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS telegram_fsm_sessions (
            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            state TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (chat_id, user_id)
        );",
    );
    let now = current_unix_timestamp() as i64;
    conn.execute(
        "INSERT INTO telegram_fsm_sessions (chat_id, user_id, state, payload_json, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(chat_id, user_id) DO UPDATE SET
             state = excluded.state,
             payload_json = excluded.payload_json,
             updated_at = excluded.updated_at",
        params![chat_id, user_id, state, payload_json, now],
    )?;
    Ok(())
}

/// Gets a valid FSM session for (chat_id, user_id) if within TTL.
/// Returns Option<(state, payload_json)>.
pub fn get_session(
    conn: &Connection,
    chat_id: i64,
    user_id: i64,
    ttl_secs: u64,
) -> Result<Option<(String, String)>> {
    let min_time = (current_unix_timestamp().saturating_sub(ttl_secs)) as i64;
    let mut stmt = conn.prepare(
        "SELECT state, payload_json FROM telegram_fsm_sessions
         WHERE chat_id = ?1 AND user_id = ?2 AND updated_at >= ?3",
    )?;

    let mut rows = stmt.query(params![chat_id, user_id, min_time])?;
    if let Some(row) = rows.next()? {
        let state: String = row.get(0)?;
        let payload_json: String = row.get(1)?;
        Ok(Some((state, payload_json)))
    } else {
        Ok(None)
    }
}

/// Clears an FSM session for (chat_id, user_id).
pub fn clear_session(conn: &Connection, chat_id: i64, user_id: i64) -> Result<()> {
    conn.execute(
        "DELETE FROM telegram_fsm_sessions WHERE chat_id = ?1 AND user_id = ?2",
        params![chat_id, user_id],
    )?;
    Ok(())
}

/// Clears all FSM sessions for a given chat_id (e.g. when bot is kicked/removed from chat).
pub fn clear_all_chat_sessions(conn: &Connection, chat_id: i64) -> Result<usize> {
    let count = conn.execute(
        "DELETE FROM telegram_fsm_sessions WHERE chat_id = ?1",
        params![chat_id],
    )?;
    Ok(count)
}

/// Cleans up expired FSM sessions older than ttl_secs.
pub fn cleanup_expired_sessions(conn: &Connection, ttl_secs: u64) -> Result<usize> {
    let min_time = (current_unix_timestamp().saturating_sub(ttl_secs)) as i64;
    let count = conn.execute(
        "DELETE FROM telegram_fsm_sessions WHERE updated_at < ?1",
        params![min_time],
    )?;
    Ok(count)
}
