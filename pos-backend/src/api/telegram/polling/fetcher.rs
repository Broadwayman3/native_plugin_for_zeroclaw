use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;

/// Resets/deletes Webhook on Telegram API to prepare for Long Polling.
pub async fn delete_webhook_with_retry(
    client: &Client,
    base_url: &str,
    cancel_token: &CancellationToken,
) -> bool {
    let delete_url = format!("{}/deleteWebhook?drop_pending_updates=false", base_url);
    let mut del_attempts = 0;
    loop {
        if cancel_token.is_cancelled() {
            tracing::info!("Polling worker cancelled during webhook reset. Exiting.");
            return false;
        }
        del_attempts += 1;
        match client.get(&delete_url).send().await {
            Ok(resp) if resp.status().is_success() => {
                tracing::info!("Successfully reset Telegram webhook for long polling");
                return true;
            }
            Ok(resp) => {
                tracing::warn!(status = %resp.status(), "deleteWebhook returned non-success status");
            }
            Err(e) => {
                tracing::warn!(error = %e, "Failed to call deleteWebhook");
            }
        }
        if del_attempts >= 3 {
            return false;
        }
        sleep(Duration::from_secs(2)).await;
    }
}

pub enum FetchResult {
    Success(Vec<Value>),
    Conflict,
    RateLimited(u64),
    Error,
}

/// Fetches a batch of updates from Telegram API via long polling getUpdates endpoint.
pub async fn fetch_updates_batch(
    client: &Client,
    base_url: &str,
    offset: i64,
    cancel_token: &CancellationToken,
) -> Option<FetchResult> {
    let poll_url = format!(
        "{}/getUpdates?offset={}&timeout=20&allowed_updates=%5B%22message%22%2C%22edited_message%22%2C%22callback_query%22%2C%22my_chat_member%22%5D",
        base_url, offset
    );

    let resp_res = tokio::select! {
        _ = cancel_token.cancelled() => return None,
        res = client.get(&poll_url).send() => res,
    };

    match resp_res {
        Ok(resp) => {
            let status = resp.status();
            if status.as_u16() == 409 {
                Some(FetchResult::Conflict)
            } else if status.as_u16() == 429 {
                let retry_secs = if let Ok(json) = resp.json::<Value>().await {
                    json.get("parameters")
                        .and_then(|p| p.get("retry_after"))
                        .and_then(|v| v.as_u64())
                        .unwrap_or(3)
                } else {
                    3
                };
                Some(FetchResult::RateLimited(retry_secs))
            } else if let Ok(json) = resp.json::<Value>().await {
                if let Some(result) = json.get("result").and_then(|v| v.as_array()) {
                    Some(FetchResult::Success(result.clone()))
                } else {
                    Some(FetchResult::Success(Vec::new()))
                }
            } else {
                Some(FetchResult::Error)
            }
        }
        Err(_) => Some(FetchResult::Error),
    }
}
