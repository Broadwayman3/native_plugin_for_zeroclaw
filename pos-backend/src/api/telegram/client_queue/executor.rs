use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

use super::TaskPayload;

pub async fn execute_task_direct(client: &Client, payload: TaskPayload) -> Result<Value, String> {
    match payload {
        TaskPayload::Json { url, payload } => {
            let chat_id = payload.get("chat_id").and_then(|v| v.as_i64());
            super::enforce_rate_limit(chat_id).await;
            send_json_direct(client, &url, &payload).await
        }
        TaskPayload::Photo {
            base_url,
            chat_id,
            photo_bytes,
            filename,
            mime_type,
            caption,
            reply_markup,
        } => {
            super::enforce_rate_limit(Some(chat_id)).await;
            send_photo_direct(
                client,
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
    }
}

pub async fn send_json_direct(
    client: &Client,
    url: &str,
    payload: &Value,
) -> Result<Value, String> {
    let chat_id = payload.get("chat_id").and_then(|v| v.as_i64());
    let mut attempts = 0;
    loop {
        attempts += 1;
        match client.post(url).json(payload).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return resp.json::<Value>().await.map_err(|e| e.to_string());
                }
                if status.as_u16() == 400 {
                    let err_body = resp.text().await.unwrap_or_default();
                    if err_body.contains("can't parse entities")
                        || err_body.contains("cant parse entities")
                    {
                        tracing::error!(
                            url = %url,
                            error = %err_body,
                            payload = %payload.to_string(),
                            "Telegram API HTTP 400 Bad Request: MarkdownV2 entity parse error! Falling back to raw unformatted text..."
                        );
                        if let Some(obj) = payload.as_object() {
                            let mut fallback = obj.clone();
                            fallback.remove("parse_mode");
                            let fallback_val = Value::Object(fallback);
                            if let Ok(fb_resp) = client.post(url).json(&fallback_val).send().await {
                                if fb_resp.status().is_success() {
                                    return fb_resp
                                        .json::<Value>()
                                        .await
                                        .map_err(|e| e.to_string());
                                }
                            }
                        }
                    }
                    return Err(format!("Telegram API HTTP 400 Error: {}", err_body));
                }
                if status.as_u16() == 409 {
                    sleep(Duration::from_secs(10)).await;
                    if attempts < 3 {
                        continue;
                    }
                } else if status.as_u16() == 429 {
                    let retry_secs = if let Ok(j) = resp.json::<Value>().await {
                        j.get("parameters")
                            .and_then(|p| p.get("retry_after"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(2)
                    } else {
                        2
                    };
                    crate::api::telegram::rate_limiter::record_chat_429(chat_id, retry_secs);
                    if attempts < 3 {
                        sleep(Duration::from_secs(retry_secs + 1)).await;
                        continue;
                    }
                } else if status.is_server_error() && attempts < 3 {
                    sleep(Duration::from_millis(500 * (1 << attempts))).await;
                    continue;
                }
                return Err(format!("Telegram API HTTP Error: {}", status));
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

#[allow(clippy::too_many_arguments)]
pub async fn send_photo_direct(
    client: &Client,
    base_url: &str,
    chat_id: i64,
    photo_bytes: Vec<u8>,
    filename: &str,
    mime_type: &str,
    caption: &str,
    reply_markup: Option<&Value>,
) -> Result<Value, String> {
    let url = format!("{}/sendPhoto", base_url);
    let mut attempts = 0;
    loop {
        attempts += 1;
        let mut form = Form::new()
            .text("chat_id", chat_id.to_string())
            .text("caption", caption.to_string())
            .text("parse_mode", "MarkdownV2");
        if let Some(m) = reply_markup {
            form = form.text("reply_markup", m.to_string());
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
                if status.as_u16() == 400 {
                    let err_body = resp.text().await.unwrap_or_default();
                    if err_body.contains("can't parse entities")
                        || err_body.contains("cant parse entities")
                    {
                        tracing::error!(
                            chat_id = chat_id,
                            error = %err_body,
                            caption = %caption,
                            "Telegram sendPhoto HTTP 400 Bad Request: MarkdownV2 entity parse error! Falling back to unformatted caption..."
                        );
                        let mut fb_form = Form::new()
                            .text("chat_id", chat_id.to_string())
                            .text("caption", caption.to_string());
                        if let Some(m) = reply_markup {
                            fb_form = fb_form.text("reply_markup", m.to_string());
                        }
                        if let Ok(part) = Part::bytes(photo_bytes.clone())
                            .file_name(filename.to_string())
                            .mime_str(mime_type)
                        {
                            fb_form = fb_form.part("photo", part);
                            if let Ok(fb_resp) = client.post(&url).multipart(fb_form).send().await {
                                if fb_resp.status().is_success() {
                                    return fb_resp
                                        .json::<Value>()
                                        .await
                                        .map_err(|e| e.to_string());
                                }
                            }
                        }
                    }
                    return Err(format!("Telegram sendPhoto HTTP 400 Error: {}", err_body));
                }
                if status.as_u16() == 429 && attempts < 3 {
                    let retry_secs = if let Ok(j) = resp.json::<Value>().await {
                        j.get("parameters")
                            .and_then(|p| p.get("retry_after"))
                            .and_then(|v| v.as_u64())
                            .unwrap_or(3)
                    } else {
                        3
                    };
                    crate::api::telegram::rate_limiter::record_chat_429(Some(chat_id), retry_secs);
                    sleep(Duration::from_secs(retry_secs + 1)).await;
                    continue;
                } else if status.is_server_error() && attempts < 3 {
                    sleep(Duration::from_millis(500 * (1 << attempts))).await;
                    continue;
                }
                return Err(format!("Telegram sendPhoto HTTP Error: {}", status));
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
