use super::lang_cache::LangCache;
use crate::db;
use once_cell::sync::Lazy;
use std::sync::atomic::{AtomicI64, Ordering};

static IN_MEMORY_OFFSET: AtomicI64 = AtomicI64::new(0);
static GLOBAL_LANG_CACHE: Lazy<LangCache> = Lazy::new(LangCache::default);

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

/// Updates update offset in memory monotonically without disk fsync.
pub fn set_update_offset_memory(offset: i64) {
    if offset > 0 {
        IN_MEMORY_OFFSET.fetch_max(offset, Ordering::SeqCst);
    }
}

/// Updates offset in memory AND persists to SQLite system_settings.
pub fn set_update_offset(db_path: &str, offset: i64) {
    set_update_offset_memory(offset);
    let current = IN_MEMORY_OFFSET.load(Ordering::SeqCst);
    if current > 0 {
        if let Ok(conn) = db::get_db_connection(db_path) {
            let _ =
                db::settings::set_setting(&conn, "telegram_update_offset", &current.to_string());
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

fn safe_blocking_db_op<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
            return tokio::task::block_in_place(f);
        }
    }
    f()
}

/// Gets user language preference for chat_id from memory cache or fallback to SQLite DB.
pub fn get_user_lang(db_path: &str, chat_id: i64) -> String {
    if let Some(cached) = GLOBAL_LANG_CACHE.get(chat_id) {
        return cached;
    }

    let db_path_buf = db_path.to_string();
    let db_res = safe_blocking_db_op(move || {
        if let Ok(conn) = db::get_db_connection(&db_path_buf) {
            db::settings::get_setting(&conn, &format!("lang_{}", chat_id))
                .ok()
                .flatten()
        } else {
            None
        }
    });

    if let Some(lang) = db_res {
        GLOBAL_LANG_CACHE.put(chat_id, lang.clone());
        return lang;
    }

    let default_lang = "en".to_string();
    GLOBAL_LANG_CACHE.put(chat_id, default_lang.clone());
    default_lang
}

/// Sets user language preference for chat_id in memory cache AND SQLite DB.
pub fn set_user_lang(db_path: &str, chat_id: i64, lang_code: &str) {
    GLOBAL_LANG_CACHE.put(chat_id, lang_code.to_string());
    let db_path_buf = db_path.to_string();
    let lang_code_buf = lang_code.to_string();
    safe_blocking_db_op(move || {
        if let Ok(conn) = db::get_db_connection(&db_path_buf) {
            let _ = db::settings::set_setting(&conn, &format!("lang_{}", chat_id), &lang_code_buf);
        }
    });
}
