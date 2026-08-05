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
        Some(pubkey) => Ok((
            StatusCode::OK,
            Json(serde_json::json!({
                "success": true,
                "nonce_pubkey": pubkey,
                "fallback_to_recent_blockhash": false
            })),
        )),
        None => {
            tracing::warn!("Durable nonce pool exhausted. Falling back to standard recent blockhash (90s TTL).");
            Ok((
                StatusCode::OK,
                Json(serde_json::json!({
                    "success": true,
                    "nonce_pubkey": serde_json::Value::Null,
                    "fallback_to_recent_blockhash": true,
                    "warning": "Nonce pool exhausted. Falling back to standard recent blockhash (90s TTL)."
                })),
            ))
        }
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

/// POST /api/v1/nonce/sync - Update nonce_blockhash from RPC or client resync
pub async fn handle_nonce_sync(
    State(state): State<crate::api::AppState>,
    Json(data): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, AppError> {
    let pubkey = data
        .get("nonce_pubkey")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing nonce_pubkey".to_string()))?;

    let new_blockhash = data
        .get("nonce_blockhash")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("Missing nonce_blockhash".to_string()))?;

    let conn = db::get_db_connection(&state.config.db_path)?;
    db::nonce::update_nonce_blockhash(&conn, pubkey, new_blockhash)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "synced_pubkey": pubkey,
        "nonce_blockhash": new_blockhash
    })))
}
