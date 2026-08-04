use std::path::PathBuf;

/// File-based test database guard. Creates a temp DB in system temp dir.
/// Automatically removes main + -wal + -shm files on drop.
pub struct TempDbGuard {
    path: PathBuf,
}

impl TempDbGuard {
    pub fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("zeroclaw_test_{}.db", name));
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(format!("{}-wal", path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", path.display()));
        Self { path }
    }

    pub fn path(&self) -> &str {
        self.path.to_str().unwrap()
    }

    pub fn conn(&self) -> rusqlite::Connection {
        pos_backend::db::get_db_connection(self.path.to_str().unwrap()).unwrap()
    }
}

impl Drop for TempDbGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
        let _ = std::fs::remove_file(format!("{}-wal", self.path.display()));
        let _ = std::fs::remove_file(format!("{}-shm", self.path.display()));
    }
}

/// Saves env vars and clears them, returns old values for restoration.
pub fn save_and_clear_env(keys: &[&str]) -> Vec<(String, Result<String, std::env::VarError>)> {
    let saved: Vec<_> = keys
        .iter()
        .map(|k| (k.to_string(), std::env::var(k)))
        .collect();
    for k in keys {
        std::env::remove_var(k);
    }
    saved
}

/// Restores env vars from save_and_clear_env result.
pub fn restore_env(saved: &[(String, Result<String, std::env::VarError>)]) {
    for (k, v) in saved {
        match v {
            Ok(val) => std::env::set_var(k, val),
            Err(_) => std::env::remove_var(k),
        }
    }
}

/// Creates an in-memory DB with full schema (no seed data).
pub fn setup_memory_db() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, false).unwrap();
    conn
}

/// Creates an in-memory DB with full schema and sample data.
pub fn setup_memory_db_with_seed() -> rusqlite::Connection {
    let conn = pos_backend::db::get_db_connection(":memory:").unwrap();
    pos_backend::db::schema::init_db(&conn, true).unwrap();
    conn
}
