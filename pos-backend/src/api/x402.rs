use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;

use crate::api::AppState;

/// GET /api/v1/sales/premium_analytics - x402 Machine Commerce endpoint
pub async fn handle_premium_analytics(
    State(_state): State<AppState>,
    headers: HeaderMap,
) -> Response {
    // Check for x402 payment header
    let accept_payment = headers
        .get("X-ACCEPT-PAYMENT")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if accept_payment == "x402" {
        let mut response_headers = HeaderMap::new();
        response_headers.insert("X-PAYMENT-REQUIRED-AMOUNT", "1.00 USDC".parse().unwrap());
        response_headers.insert(
            "X-PAYMENT-RECIPIENT",
            "8xAZmQ1111111111111111111111111111111111111"
                .parse()
                .unwrap(),
        );

        let body = serde_json::json!({
            "error": "Payment Required",
            "x402_spec": "solana-pay",
            "amount_usdc": 1.00,
            "pay_url": "solana:8xAZmQ11111111111111111111111111111111111?amount=1.00&spl-token=EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
        });

        let mut response = (StatusCode::PAYMENT_REQUIRED, Json(body)).into_response();
        response.headers_mut().extend(response_headers);
        return response;
    }

    // Regular request without x402
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "error": "Payment Required",
            "x402_spec": "solana-pay",
            "amount_usdc": 1.00,
            "headers_required": ["X-ACCEPT-PAYMENT: x402"]
        })),
    )
        .into_response()
}
