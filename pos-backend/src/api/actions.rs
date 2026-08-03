use axum::extract::Query;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use std::collections::HashMap;

use crate::error::AppError;

/// GET /actions.json - Solana Actions discovery
pub async fn handle_actions_spec() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "rules": [
            {"pathPattern": "/api/v1/actions/**", "apiPath": "/api/v1/actions/**"}
        ]
    }))
}

/// GET /api/v1/actions/pay_invoice - Blink action GET
pub async fn handle_action_get(
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, HeaderMap, Json<serde_json::Value>) {
    let invoice_id = params
        .get("invoice_id")
        .map(|s| s.as_str())
        .unwrap_or("INV-101");

    let payload = serde_json::json!({
        "icon": "https://raw.githubusercontent.com/solana-developers/branding/main/assets/solana-pay-logo.png",
        "label": format!("Pay Invoice #{}", invoice_id),
        "title": format!("ZeroClaw POS - Invoice #{}", invoice_id),
        "description": format!("Scan & Complete payment for POS Invoice #{} in USDC", invoice_id),
        "links": {
            "actions": [
                {"label": "Pay Now", "href": format!("/api/v1/actions/pay_invoice?invoice_id={}", invoice_id)}
            ]
        }
    });

    let mut headers = HeaderMap::new();
    headers.insert("X-Action-Version", "2.1.3".parse().unwrap());
    headers.insert(
        "X-Blockchain-Ids",
        "solana:EtWTRABZaYqXxicM2Tz2fSpo5nszvh6wT9D3gYqH1cQ"
            .parse()
            .unwrap(),
    );

    (StatusCode::OK, headers, Json(payload))
}

/// POST /api/v1/actions/pay_invoice - Blink action POST
pub async fn handle_action_post(
    Json(data): Json<serde_json::Value>,
) -> Result<(StatusCode, HeaderMap, Json<serde_json::Value>), AppError> {
    let account = data.get("account").and_then(|v| v.as_str());

    match account {
        Some(acc) if crate::domain::formatters::is_valid_base58(acc) => {
            let payload = serde_json::json!({
                "transaction": "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==",
                "message": "ZeroClaw POS Invoice Payment Transaction"
            });

            let mut headers = HeaderMap::new();
            headers.insert("X-Action-Version", "2.1.3".parse().unwrap());
            headers.insert(
                "X-Blockchain-Ids",
                "solana:EtWTRABZaYqXxicM2Tz2fSpo5nszvh6wT9D3gYqH1cQ"
                    .parse()
                    .unwrap(),
            );

            Ok((StatusCode::OK, headers, Json(payload)))
        }
        _ => Err(AppError::BadRequest(
            "Invalid or missing 'account' Base58 public key field in Blink POST request"
                .to_string(),
        )),
    }
}
