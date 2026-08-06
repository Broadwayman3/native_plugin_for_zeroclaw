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

/// Task payload for JSON requests or Photo multipart uploads.
pub enum TelegramTaskPayload {
    Json {
        url: String,
        payload: Value,
    },
    Photo {
        base_url: String,
        chat_id: i64,
        photo_bytes: Vec<u8>,
        filename: String,
        mime_type: String,
        caption: String,
        reply_markup: Option<Value>,
    },
}

/// Message payload passed over the outbound mpsc channel.
pub struct TelegramRequestTask {
    pub task_payload: TelegramTaskPayload,
    pub responder: Option<tokio::sync::oneshot::Sender<Result<Value, String>>>,
}

/// Async outbound queue for Telegram API requests to decouple HTTP callers from rate limiting backoff delays.
#[derive(Clone)]
pub struct TelegramOutboundQueue {
    sender: tokio::sync::mpsc::Sender<TelegramRequestTask>,
}

impl TelegramOutboundQueue {
    pub fn new(client: Client) -> Self {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<TelegramRequestTask>(1000);
        tokio::spawn(async move {
            while let Some(task) = rx.recv().await {
                let res = match task.task_payload {
                    TelegramTaskPayload::Json { url, payload } => {
                        send_telegram_request_direct(&client, &url, &payload).await
                    }
                    TelegramTaskPayload::Photo {
                        base_url,
                        chat_id,
                        photo_bytes,
                        filename,
                        mime_type,
                        caption,
                        reply_markup,
                    } => {
                        send_telegram_photo_bytes_direct(
                            &client,
                            &base_url,
                            chat_id,
                            photo_bytes,
                            &filename,
                            &mime_type,
                            &caption,
                            reply_markup.as_ref(),
                        )
                        .await
                    }
                };
                if let Some(responder) = task.responder {
                    let _ = responder.send(res);
                }
            }
        });
        Self { sender: tx }
    }

    pub async fn send_request(
        &self,
        url: impl Into<String>,
        payload: Value,
    ) -> Result<Value, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let task = TelegramRequestTask {
            task_payload: TelegramTaskPayload::Json {
                url: url.into(),
                payload,
            },
            responder: Some(tx),
        };
        self.sender
            .send(task)
            .await
            .map_err(|e| format!("Queue send error: {}", e))?;
        rx.await
            .map_err(|_| "Response channel dropped".to_string())?
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn send_photo(
        &self,
        base_url: impl Into<String>,
        chat_id: i64,
        photo_bytes: Vec<u8>,
        filename: impl Into<String>,
        mime_type: impl Into<String>,
        caption: impl Into<String>,
        reply_markup: Option<Value>,
    ) -> Result<Value, String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        let task = TelegramRequestTask {
            task_payload: TelegramTaskPayload::Photo {
                base_url: base_url.into(),
                chat_id,
                photo_bytes,
                filename: filename.into(),
                mime_type: mime_type.into(),
                caption: caption.into(),
                reply_markup,
            },
            responder: Some(tx),
        };
        self.sender
            .send(task)
            .await
            .map_err(|e| format!("Queue send photo error: {}", e))?;
        rx.await
            .map_err(|_| "Photo response channel dropped".to_string())?
    }
}

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

/// RAII guard for background chat action typing indicator loop.
/// Automatically aborts the background task when dropped (on scope exit or panic).
pub struct ChatActionGuard {
    handle: tokio::task::JoinHandle<()>,
}

impl ChatActionGuard {
    pub fn new(handle: tokio::task::JoinHandle<()>) -> Self {
        Self { handle }
    }
}

impl Drop for ChatActionGuard {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

/// Starts a background periodic `sendChatAction` loop (e.g. "typing" or "upload_photo").
/// Returns a `ChatActionGuard` that automatically aborts the background task when dropped.
pub fn start_chat_action_loop(
    client: Client,
    base_url: String,
    chat_id: i64,
    action: &'static str,
) -> ChatActionGuard {
    let handle = tokio::spawn(async move {
        let action_url = format!("{}/sendChatAction", base_url);
        let payload = serde_json::json!({
            "chat_id": chat_id,
            "action": action,
        });
        loop {
            let _ = client.post(&action_url).json(&payload).send().await;
            sleep(Duration::from_secs(4)).await;
        }
    });
    ChatActionGuard::new(handle)
}

// Global outbound mpsc queue for rate-limited Telegram API requests
static GLOBAL_OUTBOUND_QUEUE: Lazy<TelegramOutboundQueue> = Lazy::new(|| {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    TelegramOutboundQueue::new(client)
});

/// Sends a request to Telegram Bot API via the background mpsc outbound queue with rate limiting and retry handling.
pub async fn send_telegram_request(
    _client: &Client,
    url: &str,
    payload: &Value,
) -> Result<Value, String> {
    GLOBAL_OUTBOUND_QUEUE
        .send_request(url, payload.clone())
        .await
}

/// Internal direct execution of Telegram API HTTP POST.
pub async fn send_telegram_request_direct(
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

/// Sends a photo as raw PNG bytes via multipart/form-data to Telegram Bot API via outbound queue.
#[allow(clippy::too_many_arguments)]
pub async fn send_telegram_photo_bytes(
    _client: &Client,
    base_url: &str,
    chat_id: i64,
    photo_bytes: Vec<u8>,
    filename: &str,
    mime_type: &str,
    caption: &str,
    reply_markup: Option<&Value>,
) -> Result<Value, String> {
    GLOBAL_OUTBOUND_QUEUE
        .send_photo(
            base_url,
            chat_id,
            photo_bytes,
            filename,
            mime_type,
            caption,
            reply_markup.cloned(),
        )
        .await
}

/// Direct execution of sendPhoto multipart request.
#[allow(clippy::too_many_arguments)]
pub async fn send_telegram_photo_bytes_direct(
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
                    let retry_secs = if let Ok(json) = resp.json::<Value>().await {
                        json.get("parameters")
                            .and_then(|p| p.get("retry_after"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(3)
                    } else {
                        3
                    };
                    if attempts < 4 {
                        tracing::warn!(
                            retry_secs = retry_secs,
                            "sendPhoto HTTP 429 rate limit hit. Backing off..."
                        );
                        sleep(Duration::from_secs(retry_secs + 1)).await;
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
