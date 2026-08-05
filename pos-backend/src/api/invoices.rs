use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use std::collections::HashMap;

use crate::db;
use crate::error::AppError;

/// GET /api/v1/invoices - List all invoices or filter by ID/status
pub async fn handle_get_invoices(
    State(state): State<crate::api::AppState>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    let invoice_id = params.get("id").map(|s| s.as_str());
    let status = params.get("status").map(|s| s.as_str());
    let invoices = db::invoices::get_invoices_list(&conn, invoice_id, status)?;

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

/// POST /api/v1/invoices/cancel - Cancel/void a pending invoice (Idempotent)
pub async fn handle_cancel_invoice(
    State(state): State<crate::api::AppState>,
    Json(data): Json<db::invoices::CancelInvoiceRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    let existing = db::invoices::get_invoice_by_id(&conn, &data.invoice_id)?;
    let inv = match existing {
        Some(i) => i,
        None => {
            return Ok((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "success": false,
                    "error": "Invoice not found"
                })),
            ));
        }
    };

    if inv.status == "cancelled" {
        return Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "cancelled_id": data.invoice_id,
                "status": "cancelled",
                "already_cancelled": true
            })),
        ));
    }

    if inv.status == "paid" {
        return Ok((
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "success": false,
                "error": "Conflict: Cannot cancel a paid invoice. Initiate refund instead."
            })),
        ));
    }

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
            "status": "cancelled",
            "already_cancelled": false
        })),
    ))
}

/// POST /api/v1/refund/approve - Manager approves refund (Squads v4 proposal)
#[derive(serde::Deserialize)]
pub struct RefundRequest {
    pub invoice_id: String,
}

pub async fn handle_refund_approve(
    State(state): State<crate::api::AppState>,
    Json(data): Json<RefundRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    let initiated = db::invoices::initiate_refund(&conn, &data.invoice_id)?;
    if !initiated {
        return Err(AppError::Conflict(format!(
            "Invoice '{}' is not in 'paid' status, cannot initiate refund",
            data.invoice_id
        )));
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "invoice_id": data.invoice_id,
            "status": "refund_proposed_squads_v4"
        })),
    ))
}

/// POST /api/v1/refund/reject - Manager rejects refund
pub async fn handle_refund_reject(
    State(state): State<crate::api::AppState>,
    Json(data): Json<RefundRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    let reverted = db::invoices::revert_refund_to_paid(&conn, &data.invoice_id)?;
    if !reverted {
        return Err(AppError::Conflict(format!(
            "Invoice '{}' is not in 'refunding' status, cannot reject refund",
            data.invoice_id
        )));
    }

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "invoice_id": data.invoice_id,
            "status": "paid"
        })),
    ))
}

/// Request payload for verify-transaction endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct VerifyTxRequest {
    pub invoice_id: String,
    pub tx_json: serde_json::Value,
    pub merchant_ata: Option<String>,
}

/// POST /api/v1/invoices/verify-transaction - Verifies transaction via Triple Verification and updates invoice status
pub async fn handle_verify_transaction(
    State(state): State<crate::api::AppState>,
    Json(data): Json<VerifyTxRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    let invoice = db::invoices::get_invoice_by_id(&conn, &data.invoice_id)?
        .ok_or_else(|| AppError::NotFound(format!("Invoice '{}' not found", data.invoice_id)))?;

    let expected_usdc_atomic =
        pos_core_logic::safe_f64_to_u64_atomic(invoice.usdc_amount, 6) as i64;
    let target_merchant_ata = data
        .merchant_ata
        .as_deref()
        .unwrap_or(&state.config.merchant_wallet_pubkey);

    let verification_result = crate::domain::verification::verify_solana_transaction(
        &data.tx_json,
        target_merchant_ata,
        expected_usdc_atomic,
        &state.config.usdc_mint_address,
    );

    let is_valid = verification_result
        .get("is_valid")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if is_valid {
        let tx_sig = data
            .tx_json
            .get("transaction")
            .and_then(|t| t.get("signatures"))
            .and_then(|s| s.as_array())
            .and_then(|arr| arr.first())
            .and_then(|val| val.as_str())
            .or_else(|| data.tx_json.get("signature").and_then(|s| s.as_str()));

        db::invoices::update_invoice_status(&conn, &data.invoice_id, "paid", tx_sig)?;
    }

    Ok((StatusCode::OK, Json(verification_result)))
}

/// Request payload for settings update.
#[derive(Debug, serde::Deserialize)]
pub struct UpdateSettingsRequest {
    pub quick_receipt_amount: Option<f64>,
    pub quick_receipt_currency: Option<String>,
}

/// POST /api/v1/settings/update - Update system settings (Manager only)
pub async fn handle_update_settings(
    State(state): State<crate::api::AppState>,
    Json(data): Json<UpdateSettingsRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    if let Some(amt) = data.quick_receipt_amount {
        if amt <= 0.0 || !amt.is_finite() {
            return Err(AppError::BadRequest("Amount must be positive".to_string()));
        }
        db::settings::set_setting(&conn, "quick_receipt_amount", &amt.to_string())?;
    }

    if let Some(ref curr) = data.quick_receipt_currency {
        let clean_curr = curr.trim().to_uppercase();
        if clean_curr.is_empty() || clean_curr.len() > 10 {
            return Err(AppError::BadRequest("Invalid currency code".to_string()));
        }
        db::settings::set_setting(&conn, "quick_receipt_currency", &clean_curr)?;
    }

    let (current_amount, current_currency) = db::settings::get_quick_receipt_config(&conn);

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "quick_receipt_amount": current_amount,
            "quick_receipt_currency": current_currency
        })),
    ))
}

/// GET /api/v1/settings - Get current system settings
pub async fn handle_get_settings(
    State(state): State<crate::api::AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;
    let (amount, currency) = db::settings::get_quick_receipt_config(&conn);

    Ok(Json(serde_json::json!({
        "quick_receipt_amount": amount,
        "quick_receipt_currency": currency
    })))
}
