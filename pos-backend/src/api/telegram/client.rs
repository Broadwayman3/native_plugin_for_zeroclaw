use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

/// Sends a request to Telegram Bot API with precise rate limit (HTTP 429) backoff and retries.
pub async fn send_telegram_request(
    client: &Client,
    url: &str,
    payload: &Value,
) -> Result<Value, String> {
    let mut attempts = 0;
    loop {
        attempts += 1;
        match client.post(url).json(payload).send().await {
            Ok(resp) => {
                let status = resp.status();
                if status.is_success() {
                    return resp.json::<Value>().await.map_err(|e| e.to_string());
                }

                if status.as_u16() == 429 {
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
