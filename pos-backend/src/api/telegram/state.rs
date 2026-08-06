use crate::db;
use std::sync::atomic::{AtomicI64, Ordering};

static IN_MEMORY_OFFSET: AtomicI64 = AtomicI64::new(0);

/// Gets current update offset from memory or loads from SQLite system_settings.
pub fn get_update_offset(db_path: &str) -> i64 {
    let current = IN_MEMORY_OFFSET.load(Ordering::SeqCst);
    if current > 0 {
        return current;
    }

    if let Ok(conn) = db::get_db_connection(db_path) {
        if let Ok(Some(val)) = db::settings::get_setting(&conn, "telegram_update_offset") {
            if let Ok(parsed) = val.parse::<i64>() {
                IN_MEMORY_OFFSET.store(parsed, Ordering::SeqCst);
                return parsed;
            }
        }
    }
    0
}

/// Updates offset in memory AND immediately persists to SQLite system_settings.
pub fn set_update_offset(db_path: &str, offset: i64) {
    IN_MEMORY_OFFSET.store(offset, Ordering::SeqCst);
    if offset > 0 {
        if let Ok(conn) = db::get_db_connection(db_path) {
            let _ = db::settings::set_setting(&conn, "telegram_update_offset", &offset.to_string());
        }
    }
}

/// Flushes current update offset to SQLite on graceful shutdown.
pub fn flush_offset_to_db(db_path: &str) {
    let current = IN_MEMORY_OFFSET.load(Ordering::SeqCst);
    if current > 0 {
        if let Ok(conn) = db::get_db_connection(db_path) {
            let _ =
                db::settings::set_setting(&conn, "telegram_update_offset", &current.to_string());
            tracing::info!(
                offset = current,
                "Flushed Telegram update offset to database"
            );
        }
    }
}

/// Gets user language preference for chat_id from SQLite DB.
pub fn get_user_lang(db_path: &str, chat_id: i64) -> String {
    if let Ok(conn) = db::get_db_connection(db_path) {
        if let Ok(Some(lang)) = db::settings::get_setting(&conn, &format!("lang_{}", chat_id)) {
            return lang;
        }
    }
    "en".to_string()
}

/// Sets user language preference for chat_id in SQLite DB.
pub fn set_user_lang(db_path: &str, chat_id: i64, lang_code: &str) {
    if let Ok(conn) = db::get_db_connection(db_path) {
        let _ = db::settings::set_setting(&conn, &format!("lang_{}", chat_id), lang_code);
    }
}
