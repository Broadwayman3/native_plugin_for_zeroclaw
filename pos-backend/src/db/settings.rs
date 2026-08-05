use rusqlite::{params, Connection, OptionalExtension};

/// Gets a setting value by key from system_settings.
pub fn get_setting(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    conn.query_row(
        "SELECT value FROM system_settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .optional()
}

/// Sets or updates a setting key-value pair in system_settings.
pub fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO system_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Gets configured quick receipt amount and currency (falling back to defaults).
pub fn get_quick_receipt_config(conn: &Connection) -> (f64, String) {
    let amount = get_setting(conn, "quick_receipt_amount")
        .ok()
        .flatten()
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(200.0);

    let currency = get_setting(conn, "quick_receipt_currency")
        .ok()
        .flatten()
        .unwrap_or_else(|| "UAH".to_string());

    (amount, currency)
}
