use rusqlite::{params, Connection};

/// Creates a Squads v4 multisig refund proposal record.
pub fn create_proposal(
    conn: &Connection,
    invoice_id: &str,
    recipient_pubkey: &str,
    amount_usdc: f64,
) -> Result<i64, rusqlite::Error> {
    conn.execute(
        "INSERT INTO squads_proposals (invoice_id, recipient_pubkey, amount_usdc, status)
         VALUES (?1, ?2, ?3, 'created')",
        params![invoice_id, recipient_pubkey, amount_usdc],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Updates Squads v4 proposal status (approved/rejected).
pub fn update_proposal_status(
    conn: &Connection,
    proposal_index: i64,
    status: &str,
) -> Result<bool, rusqlite::Error> {
    let updated = conn.execute(
        "UPDATE squads_proposals SET status = ?1 WHERE proposal_index = ?2",
        params![status, proposal_index],
    )?;
    Ok(updated > 0)
}
