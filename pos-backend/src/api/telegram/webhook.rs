use axum::extract::State;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use serde_json::Value;
use std::time::Duration;

use crate::api::AppState;

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
        "allowed_updates": ["message", "edited_message", "callback_query", "my_chat_member"]
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
/// Persists payload into SQLite `pending_webhook_updates` queue and returns 200 OK fast.
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

    let chat_id = update
        .get("message")
        .or_else(|| update.get("edited_message"))
        .and_then(|m| m.get("chat"))
        .and_then(|c| c.get("id"))
        .and_then(|v| v.as_i64())
        .or_else(|| {
            update
                .get("callback_query")
                .and_then(|cb| cb.get("message"))
                .and_then(|m| m.get("chat"))
                .and_then(|c| c.get("id"))
                .and_then(|v| v.as_i64())
        });

    let payload_str = update.to_string();
    if payload_str.len() > 131072 {
        tracing::error!(
            update_id = update_id,
            size = payload_str.len(),
            "Webhook update payload exceeds 128KB limit, rejecting update safely"
        );
        return StatusCode::OK;
    }

    // 2. Write update payload to SQLite pending_webhook_updates queue using 4500ms timeout
    match super::webhook_db::enqueue_update_payload(
        state.db_pool.as_ref(),
        &state.config.db_path,
        update_id,
        chat_id,
        &payload_str,
    )
    .await
    {
        Ok(true) => {
            state.webhook_notify.notify_one();
            StatusCode::OK
        }
        Ok(false) => {
            // Already queued or duplicate update_id
            StatusCode::OK
        }
        Err(e) => {
            tracing::error!(
                update_id = update_id,
                error = %e,
                "Failed to enqueue webhook update to SQLite; returning HTTP 500 for Telegram retry"
            );
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
