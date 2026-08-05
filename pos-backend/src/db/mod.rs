pub mod invoices;
pub mod nonce;
pub mod schema;
pub mod seed;
pub mod settings;
pub mod sop_checkpoints;
pub mod squads;
pub mod updates;

use rusqlite::Connection;
use std::path::Path;

/// Initializes the database schema and optional sample data.
pub fn init_db(conn: &Connection, seed_sample_data: bool) -> Result<(), rusqlite::Error> {
    schema::init_db(conn, seed_sample_data)
}

/// Creates a SQLite connection with WAL mode and optimized PRAGMAs.
pub fn get_db_connection(db_path: &str) -> Result<Connection, rusqlite::Error> {
    // Ensure parent directory exists
    if let Some(parent) = Path::new(db_path).parent() {
        if !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    let conn = Connection::open(db_path)?;

    // WAL mode with fallback
    let _ = conn.execute_batch("PRAGMA journal_mode=WAL;");
    conn.execute_batch("PRAGMA busy_timeout=5000;")?;
    conn.execute_batch("PRAGMA synchronous=NORMAL;")?;
    conn.execute_batch("PRAGMA cache_size=-64000;")?;

    Ok(conn)
}
