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

/// GET /api/v1/actions/pay_invoice - Blink action GET (stub)
pub async fn handle_action_get(
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, HeaderMap, Json<serde_json::Value>) {
    let invoice_id = params
        .get("invoice_id")
        .map(|s| s.as_str())
        .unwrap_or("unknown");

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

use crate::db;
use axum::extract::State;

/// POST /api/v1/actions/pay_invoice - Blink action POST transaction builder
pub async fn handle_action_post(
    State(state): State<crate::api::AppState>,
    Query(params): Query<HashMap<String, String>>,
    Json(data): Json<serde_json::Value>,
) -> Result<(StatusCode, HeaderMap, Json<serde_json::Value>), AppError> {
    let invoice_id = params
        .get("invoice_id")
        .map(|s| s.as_str())
        .unwrap_or("unknown");

    let account = data
        .get("account")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            AppError::BadRequest(
                "Missing 'account' Base58 public key field in Blink POST request".to_string(),
            )
        })?;

    if !crate::domain::formatters::is_valid_base58(account) {
        return Err(AppError::BadRequest(
            "Invalid 'account' Base58 public key".to_string(),
        ));
    }

    let conn = db::get_db_connection(&state.config.db_path)?;
    let invoice = db::invoices::get_invoice_by_id(&conn, invoice_id)?
        .ok_or_else(|| AppError::NotFound(format!("Invoice '{}' not found", invoice_id)))?;

    // Fetch fresh recent blockhash from RPC (or fallback if offline/testing)
    let recent_blockhash = fetch_dynamic_blockhash(&state).await;

    let tx_base64 = pos_core_logic::solana_pay::build_actions_payment_transaction(
        account,
        &state.config.merchant_wallet_pubkey,
        invoice.usdc_amount,
        &state.config.usdc_mint_address,
        &invoice.reference_pubkey,
        &recent_blockhash,
    )
    .map_err(|e| AppError::Internal(format!("Failed to build Solana Action transaction: {}", e)))?;

    let payload = serde_json::json!({
        "transaction": tx_base64,
        "message": format!("Pay POS Invoice #{}", invoice_id)
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

/// Dynamically queries getLatestBlockhash from Solana JSON-RPC with SSRF validation.
/// Falls back to a deterministic blockhash if RPC is unreachable or in offline test environments.
async fn fetch_dynamic_blockhash(state: &crate::api::AppState) -> String {
    let rpc_url = &state.config.solana_rpc_url;

    if crate::domain::sanitizer::validate_safe_rpc_url(rpc_url) {
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getLatestBlockhash",
            "params": [{"commitment": "confirmed"}]
        });

        if let Ok(res) = state.http_client.post(rpc_url).json(&body).send().await {
            if let Ok(json) = res.json::<serde_json::Value>().await {
                if let Some(bh) = json
                    .get("result")
                    .and_then(|r| r.get("value"))
                    .and_then(|v| v.get("blockhash"))
                    .and_then(|b| b.as_str())
                {
                    return bh.to_string();
                }
            }
        }
    }

    // Fallback blockhash for offline/testing/graceful degradation
    "4vJ9JU1bJJE96FWSXTvBxF2vT7JhRReB88vC17A88vC1".to_string()
}
