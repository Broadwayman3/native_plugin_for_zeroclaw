use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::Value;
use std::time::Duration;

use crate::api::AppState;
use crate::db;

/// Constant-time string comparison to prevent timing side-channel attacks on secret tokens.
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    let a_bytes = a.as_bytes();
    let b_bytes = b.as_bytes();
    if a_bytes.len() != b_bytes.len() {
        return false;
    }
    let mut result = 0u8;
    for (x, y) in a_bytes.iter().zip(b_bytes.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Registers Webhook with Telegram API if TELEGRAM_WEBHOOK_URL is configured.
pub async fn register_telegram_webhook(config: &crate::config::AppConfig) -> Result<(), String> {
    let webhook_url = match &config.telegram_webhook_url {
        Some(url) if !url.trim().is_empty() => url.trim(),
        _ => return Ok(()),
    };

    if !webhook_url.starts_with("https://") {
        return Err("TELEGRAM_WEBHOOK_URL must start with https://".to_string());
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let api_url = format!(
        "https://api.telegram.org/bot{}/setWebhook",
        config.telegram_bot_token
    );

    let mut payload = serde_json::json!({
        "url": webhook_url,
        "drop_pending_updates": false,
    });

    let secret = match &config.telegram_bot_secret_token {
        Some(s) if !s.trim().is_empty() => s.trim(),
        _ => {
            let err_msg = "TELEGRAM_BOT_SECRET_TOKEN is required for Webhook mode. Falling back to Long Polling.".to_string();
            tracing::error!(error = %err_msg);
            return Err(err_msg);
        }
    };

    if let Some(obj) = payload.as_object_mut() {
        obj.insert("secret_token".to_string(), serde_json::json!(secret));
    }

    let resp = client
        .post(&api_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| e.to_string())?;

    let status = resp.status();
    if !status.is_success() {
        let body = resp.text().await.unwrap_or_default();
        let err_msg = format!("Failed to set Telegram Webhook: HTTP {} - {}", status, body);
        tracing::error!(error = %err_msg);
        return Err(err_msg);
    }

    let json: Value = resp.json().await.map_err(|e| e.to_string())?;
    let ok = json.get("ok").and_then(|v| v.as_bool()).unwrap_or(false);

    if ok {
        tracing::info!(url = %webhook_url, "Successfully registered Telegram Webhook");
        Ok(())
    } else {
        let desc = json
            .get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown Telegram API error");
        let err_msg = format!("Telegram setWebhook API returned ok=false: {}", desc);
        tracing::error!(error = %err_msg);
        Err(err_msg)
    }
}

/// Axum POST endpoint for Telegram Webhook updates.
pub async fn handle_telegram_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(update): Json<Value>,
) -> StatusCode {
    // 1. Verify X-Telegram-Bot-Api-Secret-Token header using constant-time comparison
    let expected_secret = match &state.config.telegram_bot_secret_token {
        Some(secret) if !secret.is_empty() => secret,
        _ => {
            tracing::warn!("Webhook request rejected: no bot secret token configured on server");
            return StatusCode::UNAUTHORIZED;
        }
    };

    let actual_secret = headers
        .get("x-telegram-bot-api-secret-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if !constant_time_eq(actual_secret, expected_secret) {
        tracing::warn!("Unauthorized Webhook attempt: invalid secret token header");
        return StatusCode::UNAUTHORIZED;
    }

    let update_id = match update.get("update_id").and_then(|v| v.as_i64()) {
        Some(id) => id,
        None => return StatusCode::OK,
    };

    // 2. Claim update_id in in-flight tracker to eliminate concurrent execution race conditions
    let _flight_guard = match state.in_flight.try_claim(update_id) {
        Some(guard) => guard,
        None => {
            tracing::info!(
                update_id = update_id,
                "Update is already in-flight; returning 200 OK"
            );
            return StatusCode::OK;
        }
    };

    // 3. Deduplication pre-check using deadpool-sqlite interact
    let is_already_processed = if let Some(ref pool) = state.db_pool {
        if let Ok(conn) = pool.get().await {
            conn.interact(move |c| db::updates::is_processed(c, update_id).unwrap_or(false))
                .await
                .unwrap_or(false)
        } else {
            false
        }
    } else {
        let db_path = state.config.db_path.clone();
        tokio::task::spawn_blocking(move || {
            if let Ok(conn) = db::get_db_connection(&db_path) {
                db::updates::is_processed(&conn, update_id).unwrap_or(false)
            } else {
                false
            }
        })
        .await
        .unwrap_or(false)
    };

    if is_already_processed {
        // Already completely processed previously — return 200 OK to stop Telegram retries
        return StatusCode::OK;
    }

    // 3. Process update asynchronously with shared state & 15s timeout
    let base_url = format!(
        "https://api.telegram.org/bot{}",
        state.config.telegram_bot_token
    );

    let process_fut = super::process_single_update(
        &state.http_client,
        &base_url,
        &state.config,
        &state.fsm_store,
        &state.chat_locks,
        state.db_pool.as_ref(),
        &update,
        update_id,
    );

    match tokio::time::timeout(Duration::from_secs(15), process_fut).await {
        Ok(_) => StatusCode::OK,
        Err(_) => {
            tracing::error!(
                update_id = update_id,
                "Telegram Webhook update processing timed out after 15s"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
