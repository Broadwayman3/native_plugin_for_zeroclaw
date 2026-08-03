use rusqlite::Connection;

/// Populates SQLite database with default sample invoices if table is empty.
pub fn seed_sample_data(conn: &Connection) -> Result<(), rusqlite::Error> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM invoices", [], |row| row.get(0))?;

    if count == 0 {
        conn.execute_batch(
            "INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, tx_signature, customer_address, tax_rate_pct, items_breakdown, created_at, updated_at)
             VALUES
                ('INV-101', '7xRefKey11111111111111111111111111111111111', 'UAH', 200.0, 4.82, 'paid', '5k9X...Signature1', '9xK2...Customer1', 0.0, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('INV-102', '8xRefKey22222222222222222222222222222222222', 'UAH', 150.0, 3.61, 'paid', '5k9X...Signature2', '9xK2...Customer2', 0.0, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
                ('INV-103', '9xRefKey33333333333333333333333333333333333', 'USD', 10.0, 10.00, 'pending', NULL, NULL, 0.0, NULL, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP);",
        )?;
    }

    Ok(())
}
