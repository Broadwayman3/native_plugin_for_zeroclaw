use rusqlite::{params, Connection};

/// Creates a new SOP checkpoint.
pub fn create_checkpoint(
    conn: &Connection,
    id: &str,
    sop_id: &str,
    step_id: &str,
    state_data: Option<&str>,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO sop_checkpoints (id, sop_id, step_id, state_data, status)
         VALUES (?1, ?2, ?3, ?4, 'pending')",
        params![id, sop_id, step_id, state_data],
    )?;
    Ok(())
}

/// Updates SOP checkpoint status.
pub fn update_checkpoint_status(
    conn: &Connection,
    id: &str,
    status: &str,
) -> Result<bool, rusqlite::Error> {
    let updated = conn.execute(
        "UPDATE sop_checkpoints SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
        params![status, id],
    )?;
    Ok(updated > 0)
}
