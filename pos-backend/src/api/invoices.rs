use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashMap;

use crate::db;
use crate::error::AppError;

/// GET /api/v1/invoices - List all invoices or filter by ID
pub async fn handle_get_invoices(
    State(state): State<crate::api::AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    let invoice_id = params.get("id").map(|s| s.as_str());
    let invoices = db::invoices::get_invoices_list(&conn, invoice_id)?;

    Ok(Json(serde_json::json!(invoices)))
}

/// POST /api/v1/invoices/create - Create a new pending invoice
pub async fn handle_create_invoice(
    State(state): State<crate::api::AppState>,
    Json(data): Json<db::invoices::CreateInvoiceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    let inv_id = db::invoices::create_invoice(&conn, &data)?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "success": true,
            "invoice_id": inv_id
        })),
    ))
}

/// POST /api/v1/invoices/update_status - Update invoice status atomically
pub async fn handle_update_invoice_status(
    State(state): State<crate::api::AppState>,
    Json(data): Json<db::invoices::UpdateInvoiceStatusRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    if !db::invoices::ALLOWED_INVOICE_STATUSES.contains(&data.status.as_str()) {
        return Err(AppError::BadRequest(format!(
            "Invalid status '{}'. Must be one of {:?}",
            data.status,
            db::invoices::ALLOWED_INVOICE_STATUSES
        )));
    }

    let updated = db::invoices::update_invoice_status(
        &conn,
        &data.invoice_id,
        &data.status,
        data.tx_signature.as_deref(),
    )?;

    if updated == 0 {
        return Ok((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "success": false,
                "error": "Conflict: Invoice state already finalized or invalid transition",
                "updated": 0
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "updated": updated
        })),
    ))
}

/// POST /api/v1/invoices/cancel - Cancel/void a pending invoice
pub async fn handle_cancel_invoice(
    State(state): State<crate::api::AppState>,
    Json(data): Json<db::invoices::CancelInvoiceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    let cancelled = db::invoices::cancel_invoice(&conn, &data.invoice_id)?;

    if cancelled == 0 {
        return Ok((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "success": false,
                "error": "Conflict: Invoice not found or already finalized"
            })),
        ));
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "cancelled_id": data.invoice_id,
            "status": "cancelled"
        })),
    ))
}
