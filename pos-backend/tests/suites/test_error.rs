use crate::{test_fail, test_pass};
use axum::response::IntoResponse;
use pos_backend::error::AppError;

pub fn run_suite() {
    println!("\n📦 Error Tests (336-342)");
    test_336_app_error_bad_request();
    test_337_app_error_conflict();
    test_338_app_error_not_found();
    test_339_app_error_internal();
    test_340_app_error_database();
    test_341_redact_api_key();
    test_342_redact_byte_array();
}

fn test_336_app_error_bad_request() {
    let err = AppError::BadRequest("test".into());
    let resp = err.into_response();
    if resp.status() == axum::http::StatusCode::BAD_REQUEST {
        test_pass("336: BadRequest has status 400");
    } else {
        test_fail("336", &format!("status = {}", resp.status()));
    }
}

fn test_337_app_error_conflict() {
    let err = AppError::Conflict("test".into());
    let resp = err.into_response();
    if resp.status() == axum::http::StatusCode::CONFLICT {
        test_pass("337: Conflict has status 409");
    } else {
        test_fail("337", &format!("status = {}", resp.status()));
    }
}

fn test_338_app_error_not_found() {
    let err = AppError::NotFound("test".into());
    let resp = err.into_response();
    if resp.status() == axum::http::StatusCode::NOT_FOUND {
        test_pass("338: NotFound has status 404");
    } else {
        test_fail("338", &format!("status = {}", resp.status()));
    }
}

fn test_339_app_error_internal() {
    let err = AppError::Internal("test".into());
    let resp = err.into_response();
    if resp.status() == axum::http::StatusCode::INTERNAL_SERVER_ERROR {
        test_pass("339: Internal has status 500");
    } else {
        test_fail("339", &format!("status = {}", resp.status()));
    }
}

fn test_340_app_error_database() {
    let err = AppError::from(rusqlite::Error::ExecuteReturnedResults);
    let resp = err.into_response();
    if resp.status() == axum::http::StatusCode::INTERNAL_SERVER_ERROR {
        test_pass("340: Database error has status 500");
    } else {
        test_fail("340", &format!("status = {}", resp.status()));
    }
}

fn test_341_redact_api_key() {
    let r = pos_backend::domain::sanitizer::redact_api_key("api_key=secret123");
    if r.contains("REDACTED") {
        test_pass("341: redact_api_key contains REDACTED");
    } else {
        test_fail("341", &format!("result: {}", r));
    }
}

fn test_342_redact_byte_array() {
    let kp = format!(
        "[{}]",
        (0..64)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    );
    let r = pos_backend::domain::sanitizer::redact_api_key(&format!("key: {}", kp));
    if r.contains("REDACTED_BYTE_KEYPAIR") {
        test_pass("342: redact_api_key contains REDACTED_BYTE_KEYPAIR");
    } else {
        test_fail("342", &format!("result: {}", &r[..100.min(r.len())]));
    }
}
