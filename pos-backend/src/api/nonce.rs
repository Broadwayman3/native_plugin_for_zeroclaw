use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;

use crate::db;
use crate::error::AppError;

/// POST /api/v1/nonce/allocate - Allocate a free nonce account
pub async fn handle_nonce_allocate(
    State(state): State<crate::api::AppState>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let conn = db::get_db_connection(&state.config.db_path)?;

    match db::nonce::allocate_free_nonce(&conn)? {
        Some(pubkey) => Ok((StatusCode::OK, Json(serde_json::json!({
            "success": true,
            "nonce_pubkey": pubkey
        })))),
        None => Ok((StatusCode::SERVICE_UNAVAILABLE, Json(serde_json::json!({
            "success": false,
            "error": "No free durable nonce account available in pool"
        })))),
    }
}

/// POST /api/v1/nonce/release - Release a locked nonce account
pub async fn handle_nonce_release(
    State(state): State<crate::api::AppState>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pubkey = data
        .get("nonce_pubkey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing nonce_pubkey".to_string()))?;

    let conn = db::get_db_connection(&state.config.db_path)?;
    db::nonce::release_nonce(&conn, pubkey)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "released": pubkey
    })))
}
