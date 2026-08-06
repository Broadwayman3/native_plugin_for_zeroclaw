use governor::{Quota, RateLimiter};
use image::{ImageBuffer, Luma};
use once_cell::sync::Lazy;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::Value;
use std::num::NonZeroU32;
use tokio::time::{sleep, Duration};

// Global rate limiter: max 25 requests per second to Telegram API
static GLOBAL_TELEGRAM_LIMITER: Lazy<governor::DefaultDirectRateLimiter> = Lazy::new(|| {
    let quota = Quota::per_second(NonZeroU32::new(25).unwrap());
    RateLimiter::direct(quota)
});

// Per-chat rate limiter: max 1 message per second per chat_id
static PER_CHAT_TELEGRAM_LIMITER: Lazy<governor::DefaultKeyedRateLimiter<i64>> = Lazy::new(|| {
    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
    RateLimiter::keyed(quota)
});

// Background GC worker for rate limiter keys (runs every 10 minutes)
static RATE_LIMITER_GC_WORKER: Lazy<()> = Lazy::new(|| {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(600));
        loop {
            interval.tick().await;
            PER_CHAT_TELEGRAM_LIMITER.retain_recent();
        }
    });
});

/// Enforces global (25 req/s) and per-chat (1 req/s) rate limits before sending Telegram API requests.
/// Uses governor's native async `.until_ready()`.
pub async fn enforce_telegram_rate_limit(chat_id: Option<i64>) {
    Lazy::force(&RATE_LIMITER_GC_WORKER);
    GLOBAL_TELEGRAM_LIMITER.until_ready().await;
    if let Some(cid) = chat_id {
        PER_CHAT_TELEGRAM_LIMITER.until_key_ready(&cid).await;
    }
}

/// Generates PNG image bytes for a QR code from a given string payload.
/// Telegram sendPhoto method requires raster/vector formats (PNG/JPG/WEBP) and rejects SVG.
pub fn generate_qr_code_png_bytes(payload: &str) -> Result<Vec<u8>, String> {
    let code = qrcode::QrCode::new(payload).map_err(|e| format!("QR Code Error: {}", e))?;
    let image: ImageBuffer<Luma<u8>, Vec<u8>> =
        code.render::<Luma<u8>>().min_dimensions(300, 300).build();

    let mut png_bytes = Vec::new();
    let mut cursor = std::io::Cursor::new(&mut png_bytes);
    image
        .write_to(&mut cursor, image::ImageFormat::Png)
        .map_err(|e| format!("PNG Encoding Error: {}", e))?;

    Ok(png_bytes)
}

/// Starts a background periodic `sendChatAction` loop (e.g. "typing" or "upload_photo").
/// Returns a `JoinHandle` that MUST be aborted via `handle.abort()` after request processing.
pub fn start_chat_action_loop(
    client: Client,
    base_url: String,
    chat_id: i64,
    action: &'static str,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let action_url = format!("{}/sendChatAction", base_url);
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "action": action,
        });
        loop {
            let _ = client.post(&action_url).json(&payload).send().await;
            sleep(Duration::from_secs(4)).await;
        }
    })
}

/// Sends a request to Telegram Bot API with rate limiting and retry handling.
pub async fn send_telegram_request(
    client: &Client,
    url: &str,
    payload: &Value,
) -> Result<Value, String> {
    let chat_id = payload.get("chat_id").and_then(|v| v.as_i64());
    enforce_telegram_rate_limit(chat_id).await;

    let mut attempts = 0;
    loop {
        attempts += 1;
        match client.post(url).json(payload).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return resp.json::<Value>().await.map_err(|e| e.to_string());
                }

                if status.as_u16() == 409 {
                    tracing::error!("Telegram HTTP 409 Conflict: duplicate bot instance running. Retrying in 10s...");
                    sleep(Duration::from_secs(10)).await;
                    if attempts < 4 {
                        continue;
                    }
                } else if status.as_u16() == 429 {
                    let retry_secs = if let Ok(json) = resp.json::<Value>().await {
                        json.get("parameters")
                            .and_then(|p| p.get("retry_after"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(2)
                    } else {
                        2
                    };

                    if attempts < 4 {
                        tracing::warn!(
                            retry_secs = retry_secs,
                            "Telegram HTTP 429 rate limit hit. Backing off..."
                        );
                        sleep(Duration::from_secs(retry_secs + 1)).await;
                        continue;
                    }
                } else if status.is_server_error() && attempts < 3 {
                    sleep(Duration::from_millis(500 * (1 << attempts))).await;
                    continue;
                }

                let err_text = format!("Telegram API HTTP Error: {}", status);
                tracing::error!(url = %url, error = %err_text, "Telegram request failed");
                return Err(err_text);
            }
            Err(e) => {
                if attempts < 3 {
                    sleep(Duration::from_millis(500 * (1 << attempts))).await;
                    continue;
                }
                tracing::error!(url = %url, error = %e, "Telegram network error");
                return Err(e.to_string());
            }
        }
    }
}

/// Sends a photo as raw PNG bytes via multipart/form-data to Telegram Bot API.
#[allow(clippy::too_many_arguments)]
pub async fn send_telegram_photo_bytes(
    client: &Client,
    base_url: &str,
    chat_id: i64,
    photo_bytes: Vec<u8>,
    filename: &str,
    mime_type: &str,
    caption: &str,
    reply_markup: Option<&Value>,
) -> Result<Value, String> {
    enforce_telegram_rate_limit(Some(chat_id)).await;

    let url = format!("{}/sendPhoto", base_url);
    let mut attempts = 0;

    loop {
        attempts += 1;
        let mut form = Form::new()
            .text("chat_id", chat_id.to_string())
            .text("caption", caption.to_string())
            .text("parse_mode", "MarkdownV2");

        if let Some(markup) = reply_markup {
            form = form.text("reply_markup", markup.to_string());
        }

        let part = match Part::bytes(photo_bytes.clone())
            .file_name(filename.to_string())
            .mime_str(mime_type)
        {
            Ok(p) => p,
            Err(e) => return Err(e.to_string()),
        };

        form = form.part("photo", part);

        match client.post(&url).multipart(form).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return resp.json::<Value>().await.map_err(|e| e.to_string());
                }

                if status.as_u16() == 409 {
                    tracing::error!("Telegram HTTP 409 Conflict in sendPhoto: Retrying in 10s...");
                    sleep(Duration::from_secs(10)).await;
                    if attempts < 4 {
                        continue;
                    }
                } else if status.as_u16() == 429 {
                    if attempts < 4 {
                        sleep(Duration::from_secs(3)).await;
                        continue;
                    }
                } else if status.is_server_error() && attempts < 3 {
                    sleep(Duration::from_millis(500 * (1 << attempts))).await;
                    continue;
                }

                let err_msg = format!("Telegram sendPhoto HTTP Error: {}", status);
                tracing::error!(error = %err_msg, "Failed to send photo bytes");
                return Err(err_msg);
            }
            Err(e) => {
                if attempts < 3 {
                    sleep(Duration::from_millis(500 * (1 << attempts))).await;
                    continue;
                }
                return Err(e.to_string());
            }
        }
    }
}
