use rusqlite::Connection;

/// Creates all database tables and runs migrations.
pub fn init_db(conn: &Connection, seed_sample_data: bool) -> Result<(), rusqlite::Error> {
    // Enable WAL mode
    let _ = conn.execute_batch("PRAGMA journal_mode=WAL;");
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;
    conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
    conn.execute_batch("PRAGMA cache_size=-64000;")?;

    // Create tables
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS invoices (
            id TEXT PRIMARY KEY,
            reference_pubkey TEXT UNIQUE NOT NULL,
            fiat_currency TEXT NOT NULL,
            fiat_amount REAL NOT NULL,
            usdc_amount REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            tx_signature TEXT,
            customer_address TEXT,
            pix_id TEXT,
            pix_payload TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE UNIQUE INDEX IF NOT EXISTS idx_invoices_tx_sig
            ON invoices(tx_signature) WHERE tx_signature IS NOT NULL;

        CREATE TABLE IF NOT EXISTS squads_proposals (
            proposal_index INTEGER PRIMARY KEY,
            invoice_id TEXT NOT NULL,
            recipient_pubkey TEXT NOT NULL,
            amount_usdc REAL NOT NULL,
            status TEXT NOT NULL DEFAULT 'created',
            tx_base64 TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (invoice_id) REFERENCES invoices(id)
        );

        CREATE TABLE IF NOT EXISTS processed_updates (
            update_id INTEGER PRIMARY KEY,
            processed_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS nonce_accounts (
            pubkey TEXT PRIMARY KEY,
            status TEXT NOT NULL DEFAULT 'free',
            locked_at TIMESTAMP
        );

        CREATE TABLE IF NOT EXISTS sop_checkpoints (
            id TEXT PRIMARY KEY,
            sop_id TEXT NOT NULL,
            step_id TEXT NOT NULL,
            state_data TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        );",
    )?;

    // Schema migration: add tax_rate_pct and items_breakdown columns if missing
    let columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(invoices)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    if !columns.contains(&"tax_rate_pct".to_string()) {
        conn.execute_batch("ALTER TABLE invoices ADD COLUMN tax_rate_pct REAL DEFAULT 0.0")?;
    }
    if !columns.contains(&"items_breakdown".to_string()) {
        conn.execute_batch("ALTER TABLE invoices ADD COLUMN items_breakdown TEXT")?;
    }

    // Seed nonce accounts if empty
    let count: i64 =
        conn.query_row("SELECT COUNT(*) FROM nonce_accounts", [], |row| row.get(0))?;
    if count == 0 {
        conn.execute_batch(
            "INSERT INTO nonce_accounts (pubkey, status) VALUES
                ('Nonce111111111111111111111111111111111111111', 'free'),
                ('Nonce222222222222222222222222222222222222222', 'free'),
                ('Nonce333333333333333333333333333333333333333', 'free');",
        )?;
    }

    // Seed sample data if requested
    if seed_sample_data {
        super::seed::seed_sample_data(conn)?;
    }

    // Cleanup expired pending invoices
    super::invoices::cleanup_expired_pending_invoices(conn)?;

    Ok(())
}
