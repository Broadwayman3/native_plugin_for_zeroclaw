use rusqlite::Connection;

/// Creates all database tables, configures WAL mode, and runs schema migrations.
pub fn init_db(conn: &Connection, seed_sample_data: bool) -> Result<(), rusqlite::Error> {
    // 1. Enforce SQLite WAL mode and busy timeout pragmas for high-concurrency safety
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA busy_timeout=5000;
         PRAGMA synchronous=NORMAL;",
    )?;

    // 2. Create base tables
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
        );

        CREATE TABLE IF NOT EXISTS system_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS telegram_fsm_sessions (
            chat_id INTEGER NOT NULL,
            user_id INTEGER NOT NULL,
            state TEXT NOT NULL,
            payload_json TEXT NOT NULL,
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (chat_id, user_id)
        );",
    )?;

    // 3. Safe Schema Migrations (Check existing columns before ALTER TABLE)
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
    if !columns.contains(&"telegram_chat_id".to_string()) {
        conn.execute_batch("ALTER TABLE invoices ADD COLUMN telegram_chat_id INTEGER")?;
    }
    if !columns.contains(&"telegram_msg_id".to_string()) {
        conn.execute_batch("ALTER TABLE invoices ADD COLUMN telegram_msg_id INTEGER")?;
    }
    if !columns.contains(&"telegram_expired_notified".to_string()) {
        conn.execute_batch(
            "ALTER TABLE invoices ADD COLUMN telegram_expired_notified INTEGER DEFAULT 0",
        )?;
    }

    let update_columns: Vec<String> = {
        let mut stmt = conn.prepare("PRAGMA table_info(processed_updates)")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
        rows.filter_map(|r| r.ok()).collect()
    };

    if !update_columns.contains(&"retry_count".to_string()) {
        conn.execute_batch(
            "ALTER TABLE processed_updates ADD COLUMN retry_count INTEGER DEFAULT 0",
        )?;
    }
    if !update_columns.contains(&"status".to_string()) {
        conn.execute_batch(
            "ALTER TABLE processed_updates ADD COLUMN status TEXT DEFAULT 'processed'",
        )?;
    }

    // Seed nonce accounts if empty
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM nonce_accounts", [], |row| row.get(0))?;
    if count == 0 {
        conn.execute_batch(
            "INSERT INTO nonce_accounts (pubkey, status) VALUES
                ('Nonce111111111111111111111111111111111111111', 'free'),
                ('Nonce222222222222222222222222222222222222222', 'free'),
                ('Nonce333333333333333333333333333333333333333', 'free');",
        )?;
    }

    // Seed system settings if empty
    let settings_count: i64 =
        conn.query_row("SELECT COUNT(*) FROM system_settings", [], |row| row.get(0))?;
    if settings_count == 0 {
        conn.execute_batch(
            "INSERT INTO system_settings (key, value) VALUES
                ('quick_receipt_amount', '200.0'),
                ('quick_receipt_currency', 'UAH');",
        )?;
    }

    // Seed sample data if requested
    if seed_sample_data {
        super::seed::seed_sample_data(conn)?;
    }

    // Cleanup expired pending invoices & old update IDs (>24h)
    super::invoices::cleanup_expired_pending_invoices(conn)?;
    let _ = super::updates::cleanup_old_updates(conn);

    Ok(())
}
