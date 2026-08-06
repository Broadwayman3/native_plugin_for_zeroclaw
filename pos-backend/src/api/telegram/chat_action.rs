use reqwest::Client;
use tokio::time::{sleep, Duration};

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
