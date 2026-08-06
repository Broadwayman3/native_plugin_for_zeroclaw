use governor::{Quota, RateLimiter};
use once_cell::sync::Lazy;
use reqwest::multipart::{Form, Part};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio::time::{sleep, Duration};

static GLOBAL_TELEGRAM_LIMITER: Lazy<governor::DefaultDirectRateLimiter> = Lazy::new(|| {
    let quota = Quota::per_second(NonZeroU32::new(25).unwrap());
    RateLimiter::direct(quota)
});

static PER_CHAT_TELEGRAM_LIMITER: Lazy<governor::DefaultKeyedRateLimiter<i64>> = Lazy::new(|| {
    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
    RateLimiter::keyed(quota)
});

pub async fn enforce_rate_limit(chat_id: Option<i64>) {
    GLOBAL_TELEGRAM_LIMITER.until_ready().await;
    if let Some(cid) = chat_id {
        PER_CHAT_TELEGRAM_LIMITER.until_key_ready(&cid).await;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    High,
    Normal,
}

pub enum TaskPayload {
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

pub struct QueueTask {
    pub payload: TaskPayload,
    pub priority: Priority,
    pub responder: Option<oneshot::Sender<Result<Value, String>>>,
}

/// Actor manager storing bounded mpsc channels per chat_id with 15-minute idle TTL GC.
#[derive(Clone)]
pub struct OutboundQueueManager {
    client: Client,
    chat_senders: Arc<Mutex<HashMap<i64, mpsc::Sender<QueueTask>>>>,
    high_priority_tx: mpsc::Sender<QueueTask>,
}

impl OutboundQueueManager {
    pub fn new(client: Client) -> Self {
        let chat_senders = Arc::new(Mutex::new(HashMap::new()));
        let (high_priority_tx, mut high_priority_rx) = mpsc::channel::<QueueTask>(500);

        // High Priority direct worker spawning bounded concurrent tasks (max 50) for verifier & answerCallbackQuery
        let high_client = client.clone();
        let high_sem = Arc::new(tokio::sync::Semaphore::new(50));
        tokio::spawn(async move {
            while let Some(task) = high_priority_rx.recv().await {
                let hc = high_client.clone();
                let sem_permit = high_sem.clone().acquire_owned().await;
                tokio::spawn(async move {
                    let _permit = sem_permit;
                    let res = execute_task_direct(&hc, task.payload).await;
                    if let Some(responder) = task.responder {
                        let _ = responder.send(res);
                    }
                });
            }
        });

        Self {
            client,
            chat_senders,
            high_priority_tx,
        }
    }

    pub async fn enqueue(&self, chat_id: Option<i64>, task: QueueTask) -> Result<Value, String> {
        let (tx, rx) = oneshot::channel();
        let mut task_with_resp = task;
        task_with_resp.responder = Some(tx);

        if task_with_resp.priority == Priority::High {
            self.high_priority_tx
                .send(task_with_resp)
                .await
                .map_err(|e| format!("High priority queue send error: {}", e))?;
            return rx.await.map_err(|_| "Response dropped".to_string())?;
        }

        let cid = chat_id.unwrap_or(0);
        let sender = self.get_or_spawn_actor(cid);
        let _ = sender.send(task_with_resp).await;

        rx.await.map_err(|_| "Response dropped".to_string())?
    }

    fn get_or_spawn_actor(&self, chat_id: i64) -> mpsc::Sender<QueueTask> {
        let mut map = self.chat_senders.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(sender) = map.get(&chat_id) {
            if !sender.is_closed() {
                return sender.clone();
            } else {
                map.remove(&chat_id);
            }
        }

        let (tx, mut rx) = mpsc::channel::<QueueTask>(100);
        let senders_weak = Arc::downgrade(&self.chat_senders);
        let client = self.client.clone();

        tokio::spawn(async move {
            let idle_duration = Duration::from_secs(900); // 15-minute TTL GC
            loop {
                match tokio::time::timeout(idle_duration, rx.recv()).await {
                    Ok(Some(task)) => {
                        let res = execute_task_direct(&client, task.payload).await;
                        if let Some(responder) = task.responder {
                            let _ = responder.send(res);
                        }
                    }
                    Ok(None) => break, // Channel closed
                    Err(_) => {
                        // 15-minute idle timeout hit: remove tx from map under lock first
                        if let Some(map_arc) = senders_weak.upgrade() {
                            let mut map = map_arc.lock().unwrap_or_else(|e| e.into_inner());
                            map.remove(&chat_id);
                        }
                        // Drain any remaining messages that arrived right before removal
                        while let Ok(task) = rx.try_recv() {
                            let res = execute_task_direct(&client, task.payload).await;
                            if let Some(responder) = task.responder {
                                let _ = responder.send(res);
                            }
                        }
                        break;
                    }
                }
            }
        });

        map.insert(chat_id, tx.clone());
        tx
    }
}

async fn execute_task_direct(client: &Client, payload: TaskPayload) -> Result<Value, String> {
    match payload {
        TaskPayload::Json { url, payload } => {
            let chat_id = payload.get("chat_id").and_then(|v| v.as_i64());
            enforce_rate_limit(chat_id).await;
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
            enforce_rate_limit(Some(chat_id)).await;
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

async fn send_json_direct(client: &Client, url: &str, payload: &Value) -> Result<Value, String> {
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
async fn send_photo_direct(
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
                if status.as_u16() == 429 && attempts < 3 {
                    sleep(Duration::from_secs(3)).await;
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
