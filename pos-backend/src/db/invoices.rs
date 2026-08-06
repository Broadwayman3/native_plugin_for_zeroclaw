use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub reference_pubkey: String,
    pub fiat_currency: String,
    pub fiat_amount: f64,
    pub usdc_amount: f64,
    pub status: String,
    pub tx_signature: Option<String>,
    pub customer_address: Option<String>,
    pub pix_id: Option<String>,
    pub pix_payload: Option<String>,
    pub tax_rate_pct: Option<f64>,
    pub items_breakdown: Option<String>,
    pub telegram_chat_id: Option<i64>,
    pub telegram_msg_id: Option<i64>,
    pub telegram_expired_notified: Option<i64>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct CreateInvoiceRequest {
    pub id: String,
    pub reference_pubkey: String,
    pub fiat_currency: Option<String>,
    pub fiat_amount: Option<f64>,
    pub usdc_amount: f64,
    pub telegram_chat_id: Option<i64>,
    pub telegram_msg_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateInvoiceStatusRequest {
    pub invoice_id: String,
    pub status: String,
    pub tx_signature: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CancelInvoiceRequest {
    pub invoice_id: String,
}

pub const ALLOWED_INVOICE_STATUSES: &[&str] = &[
    "pending",
    "paid",
    "partially_paid",
    "cancelled",
    "refunding",
    "refund_proposed_squads_v4",
    "expired",
    "failed",
];

/// Cleans up expired pending invoices (older than 15 minutes).
pub fn cleanup_expired_pending_invoices(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute(
        "UPDATE invoices SET status = 'expired', updated_at = CURRENT_TIMESTAMP
         WHERE status = 'pending' AND created_at < datetime('now', '-15 minutes')",
        [],
    )?;
    Ok(())
}

/// Creates a new pending invoice.
pub fn create_invoice(
    conn: &Connection,
    req: &CreateInvoiceRequest,
) -> Result<String, rusqlite::Error> {
    let fiat_currency = req.fiat_currency.as_deref().unwrap_or("USD");
    let fiat_amount = req.fiat_amount.unwrap_or(req.usdc_amount);

    conn.execute(
        "INSERT INTO invoices (id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount, status, telegram_chat_id, telegram_msg_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)",
        params![req.id, req.reference_pubkey, fiat_currency, fiat_amount, req.usdc_amount, req.telegram_chat_id, req.telegram_msg_id],
    )?;
    Ok(req.id.clone())
}

/// Updates invoice's Telegram chat and message IDs after sendPhoto.
pub fn update_invoice_telegram_context(
    conn: &Connection,
    invoice_id: &str,
    chat_id: i64,
    msg_id: i64,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "UPDATE invoices SET telegram_chat_id = ?1, telegram_msg_id = ?2, updated_at = CURRENT_TIMESTAMP WHERE id = ?3",
        params![chat_id, msg_id, invoice_id],
    )
}

/// Updates invoice status atomically if transition is valid.
pub fn update_invoice_status(
    conn: &Connection,
    invoice_id: &str,
    status: &str,
    tx_signature: Option<&str>,
) -> Result<usize, rusqlite::Error> {
    if !ALLOWED_INVOICE_STATUSES.contains(&status) {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "Invalid status: {}",
            status
        )));
    }

    let updated = conn.execute(
        "UPDATE invoices
         SET status = ?1, tx_signature = ?2, updated_at = CURRENT_TIMESTAMP
         WHERE id = ?3 AND (status = 'pending' OR status = 'partially_paid' OR status = 'expired' OR status = ?1)",
        params![status, tx_signature, invoice_id],
    )?;
    Ok(updated)
}

/// Atomically cancels/voids a pending invoice.
pub fn cancel_invoice(conn: &Connection, invoice_id: &str) -> Result<usize, rusqlite::Error> {
    let cancelled = conn.execute(
        "UPDATE invoices
         SET status = 'cancelled', updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1 AND status = 'pending'",
        params![invoice_id],
    )?;
    Ok(cancelled)
}

/// Marks expired invoice notification as sent in SQLite to prevent infinite notification loops.
pub fn mark_invoice_expired_notified(
    conn: &Connection,
    invoice_id: &str,
) -> Result<usize, rusqlite::Error> {
    conn.execute(
        "UPDATE invoices SET telegram_expired_notified = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        params![invoice_id],
    )
}

/// Maps a rusqlite Row to an Invoice struct.
fn row_to_invoice(row: &rusqlite::Row) -> Result<Invoice, rusqlite::Error> {
    Ok(Invoice {
        id: row.get(0)?,
        reference_pubkey: row.get(1)?,
        fiat_currency: row.get(2)?,
        fiat_amount: row.get(3)?,
        usdc_amount: row.get(4)?,
        status: row.get(5)?,
        tx_signature: row.get(6)?,
        customer_address: row.get(7)?,
        pix_id: row.get(8)?,
        pix_payload: row.get(9)?,
        tax_rate_pct: row.get(10)?,
        items_breakdown: row.get(11)?,
        telegram_chat_id: row.get(12)?,
        telegram_msg_id: row.get(13)?,
        telegram_expired_notified: row.get(14)?,
        created_at: row.get(15)?,
        updated_at: row.get(16)?,
    })
}

/// Fetches a single invoice by ID.
pub fn get_invoice_by_id(
    conn: &Connection,
    invoice_id: &str,
) -> Result<Option<Invoice>, rusqlite::Error> {
    conn.query_row(
        "SELECT id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount,
                status, tx_signature, customer_address, pix_id, pix_payload,
                tax_rate_pct, items_breakdown, telegram_chat_id, telegram_msg_id,
                telegram_expired_notified, created_at, updated_at
         FROM invoices WHERE id = ?1",
        params![invoice_id],
        row_to_invoice,
    )
    .optional()
}

/// Fetches all invoices or filters by ID.
pub fn get_invoices_list(
    conn: &Connection,
    invoice_id: Option<&str>,
    status: Option<&str>,
) -> Result<Vec<Invoice>, rusqlite::Error> {
    cleanup_expired_pending_invoices(conn)?;

    let stmt = if let Some(id) = invoice_id {
        let mut s = conn.prepare(
            "SELECT id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount,
                    status, tx_signature, customer_address, pix_id, pix_payload,
                    tax_rate_pct, items_breakdown, telegram_chat_id, telegram_msg_id,
                    telegram_expired_notified, created_at, updated_at
             FROM invoices WHERE id = ?1 ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = s.query_map(params![id], row_to_invoice)?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    } else if let Some(st) = status {
        let mut s = conn.prepare(
            "SELECT id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount,
                    status, tx_signature, customer_address, pix_id, pix_payload,
                    tax_rate_pct, items_breakdown, telegram_chat_id, telegram_msg_id,
                    telegram_expired_notified, created_at, updated_at
             FROM invoices WHERE status = ?1 ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = s.query_map(params![st], row_to_invoice)?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    } else {
        let mut s = conn.prepare(
            "SELECT id, reference_pubkey, fiat_currency, fiat_amount, usdc_amount,
                    status, tx_signature, customer_address, pix_id, pix_payload,
                    tax_rate_pct, items_breakdown, telegram_chat_id, telegram_msg_id,
                    telegram_expired_notified, created_at, updated_at
             FROM invoices ORDER BY updated_at DESC, created_at DESC",
        )?;
        let rows = s.query_map([], row_to_invoice)?;
        rows.filter_map(|r| r.ok()).collect::<Vec<_>>()
    };

    Ok(stmt)
}

/// Gets aggregated sales summary statistics.
pub fn get_sales_summary(conn: &Connection) -> Result<serde_json::Value, rusqlite::Error> {
    let total_paid: (i64, Option<f64>) = conn.query_row(
        "SELECT COUNT(*), SUM(usdc_amount) FROM invoices WHERE status = 'paid'",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let pending_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM invoices WHERE status = 'pending'",
        [],
        |row| row.get(0),
    )?;

    // Sales by currency
    let mut stmt = conn.prepare(
        "SELECT fiat_currency, COUNT(*) as count, SUM(fiat_amount) as total_fiat, SUM(usdc_amount) as total_usdc
         FROM invoices WHERE status = 'paid' GROUP BY fiat_currency",
    )?;

    let mut by_currency = serde_json::Map::new();
    let rows = stmt.query_map([], |row| {
        let currency: String = row.get(0)?;
        let count: i64 = row.get(1)?;
        let total_fiat: Option<f64> = row.get(2)?;
        let total_usdc: Option<f64> = row.get(3)?;
        Ok((currency, count, total_fiat, total_usdc))
    })?;

    for row in rows.flatten() {
        let mut entry = serde_json::Map::new();
        entry.insert("count".to_string(), serde_json::json!(row.1));
        entry.insert(
            "total_fiat".to_string(),
            serde_json::json!((row.2.unwrap_or(0.0) * 100.0).round() / 100.0),
        );
        entry.insert(
            "total_usdc".to_string(),
            serde_json::json!((row.3.unwrap_or(0.0) * 100.0).round() / 100.0),
        );
        by_currency.insert(row.0, serde_json::Value::Object(entry));
    }

    // Generate proper ISO timestamp
    let now_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let timestamp = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}+00:00",
        (now_secs / 31536000) + 1970,
        ((now_secs % 31536000) / 2592000) + 1,
        ((now_secs % 2592000) / 86400) + 1,
        (now_secs % 86400) / 3600,
        (now_secs % 3600) / 60,
        now_secs % 60
    );

    Ok(serde_json::json!({
        "business_name": "ZeroClaw Coffee POS",
        "currency": "USDC",
        "total_paid_invoices": total_paid.0,
        "total_sales_usdc": (total_paid.1.unwrap_or(0.0) * 100.0).round() / 100.0,
        "total_pending_invoices": pending_count,
        "sales_by_currency": by_currency,
        "journal_mode": "WAL",
        "timestamp": timestamp
    }))
}

/// Initiates refund request (re-entrancy guard).
pub fn initiate_refund(conn: &Connection, invoice_id: &str) -> Result<bool, rusqlite::Error> {
    let updated = conn.execute(
        "UPDATE invoices SET status = 'refunding', updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1 AND status = 'paid'",
        params![invoice_id],
    )?;
    Ok(updated > 0)
}

/// Proposes refund via Squads v4 (refunding → refund_proposed_squads_v4).
/// NOTE: Python original had no status check (fail-open). Rust adds this check
/// for safety — prevents invalid state transitions from cancelled/paid/etc.
pub fn propose_refund(conn: &Connection, invoice_id: &str) -> Result<bool, rusqlite::Error> {
    let updated = conn.execute(
        "UPDATE invoices SET status = 'refund_proposed_squads_v4', updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1 AND status = 'refunding'",
        params![invoice_id],
    )?;
    Ok(updated > 0)
}

/// Reverts a refund request back to paid status (refunding → paid).
pub fn revert_refund_to_paid(conn: &Connection, invoice_id: &str) -> Result<bool, rusqlite::Error> {
    let updated = conn.execute(
        "UPDATE invoices SET status = 'paid', updated_at = CURRENT_TIMESTAMP
         WHERE id = ?1 AND status = 'refunding'",
        params![invoice_id],
    )?;
    Ok(updated > 0)
}

/// Retrieves a value from system_settings table by key.
pub fn get_system_setting(conn: &Connection, key: &str) -> Result<Option<String>, rusqlite::Error> {
    let mut stmt = conn.prepare("SELECT value FROM system_settings WHERE key = ?1")?;
    let mut rows = stmt.query(params![key])?;
    if let Some(row) = rows.next()? {
        let val: String = row.get(0)?;
        Ok(Some(val))
    } else {
        Ok(None)
    }
}

/// Sets or updates a key-value pair in system_settings table.
pub fn set_system_setting(
    conn: &Connection,
    key: &str,
    value: &str,
) -> Result<(), rusqlite::Error> {
    conn.execute(
        "INSERT INTO system_settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}
