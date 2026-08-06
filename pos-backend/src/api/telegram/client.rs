use crate::api::telegram::client_queue::{OutboundQueueManager, Priority, QueueTask, TaskPayload};
use once_cell::sync::Lazy;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

pub type TelegramOutboundQueue = OutboundQueueManager;
pub use super::chat_action::{start_chat_action_loop, ChatActionGuard};
pub use super::qr::generate_qr_code_png_bytes;
pub use crate::api::telegram::client_queue::enforce_rate_limit as enforce_telegram_rate_limit;

static GLOBAL_QUEUE_MANAGER: Lazy<OutboundQueueManager> = Lazy::new(|| {
    let client = Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap_or_default();
    OutboundQueueManager::new(client)
});

pub async fn send_telegram_request(
    _client: &Client,
    url: &str,
    payload: &Value,
) -> Result<Value, String> {
    send_telegram_request_with_priority(_client, url, payload, Priority::Normal).await
}

pub async fn send_telegram_request_with_priority(
    _client: &Client,
    url: &str,
    payload: &Value,
    priority: Priority,
) -> Result<Value, String> {
    let chat_id = payload.get("chat_id").and_then(|v| v.as_i64());
    let task = QueueTask {
        payload: TaskPayload::Json {
            url: url.to_string(),
            payload: payload.clone(),
        },
        priority,
        responder: None,
    };
    GLOBAL_QUEUE_MANAGER.enqueue(chat_id, task).await
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
    let task = QueueTask {
        payload: TaskPayload::Photo {
            base_url: base_url.to_string(),
            chat_id,
            photo_bytes,
            filename: filename.to_string(),
            mime_type: mime_type.to_string(),
            caption: caption.to_string(),
            reply_markup: reply_markup.cloned(),
        },
        priority: Priority::Normal,
        responder: None,
    };
    GLOBAL_QUEUE_MANAGER.enqueue(Some(chat_id), task).await
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
