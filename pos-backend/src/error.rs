use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),

    #[error("{0}")]
    BadRequest(String),

    #[error("{0}")]
    Conflict(String),

    #[error("{0}")]
    NotFound(String),

    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
            AppError::Io(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("IO error: {}", e),
            ),
            AppError::Json(e) => (
                StatusCode::BAD_REQUEST,
                format!("JSON error: {}", e),
            ),
            AppError::Parse(e) => (
                StatusCode::BAD_REQUEST,
                format!("Parse error: {}", e),
            ),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, msg.clone()),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                msg.clone(),
            ),
        };

        let body = json!({
            "error": redact_api_key(&message),
        });

        (status, axum::Json(body)).into_response()
    }
}

/// Redacts API keys, tokens, and secrets from error messages.
pub fn redact_api_key(msg: &str) -> String {
    let re_api_key = regex::Regex::new(r"(?i)(api[_-]?key|token|secret)=[^&\s]+").unwrap();
    let re_byte_array =
        regex::Regex::new(r"\[\s*\d{1,3}\s*(?:,\s*\d{1,3}\s*){31,}\]").unwrap();

    let masked = re_api_key.replace_all(msg, "$1=REDACTED");
    let masked = re_byte_array.replace_all(&masked, "[REDACTED_BYTE_KEYPAIR]");
    masked.to_string()
}
