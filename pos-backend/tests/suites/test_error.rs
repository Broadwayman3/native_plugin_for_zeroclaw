use axum::response::IntoResponse;
use pos_backend::error::AppError;

#[test]
fn test_336_app_error_bad_request() {
    let err = AppError::BadRequest("test".into());
    let resp = err.into_response();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::BAD_REQUEST,
        "336: expected 400"
    );
}

#[test]
fn test_337_app_error_conflict() {
    let err = AppError::Conflict("test".into());
    let resp = err.into_response();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::CONFLICT,
        "337: expected 409"
    );
}

#[test]
fn test_338_app_error_not_found() {
    let err = AppError::NotFound("test".into());
    let resp = err.into_response();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::NOT_FOUND,
        "338: expected 404"
    );
}

#[test]
fn test_339_app_error_internal() {
    let err = AppError::Internal("test".into());
    let resp = err.into_response();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "339: expected 500"
    );
}

#[test]
fn test_340_app_error_database() {
    let err = AppError::from(rusqlite::Error::ExecuteReturnedResults);
    let resp = err.into_response();
    assert_eq!(
        resp.status(),
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        "340: expected 500"
    );
}

#[test]
fn test_341_redact_api_key() {
    let r = pos_backend::domain::sanitizer::redact_api_key("api_key=secret123");
    assert!(
        r.contains("REDACTED"),
        "341: result should contain REDACTED, got: {}",
        r
    );
}

#[test]
fn test_342_redact_byte_array() {
    let kp = format!(
        "[{}]",
        (0..64)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let r = pos_backend::domain::sanitizer::redact_api_key(&format!("key: {}", kp));
    assert!(
        r.contains("REDACTED_BYTE_KEYPAIR"),
        "342: result should contain REDACTED_BYTE_KEYPAIR"
    );
}
