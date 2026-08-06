pub mod executor;

use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, oneshot};
use tokio::time::Duration;

pub use crate::api::telegram::rate_limiter::enforce_rate_limit;
use executor::execute_task_direct;

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
        if let Err(tokio::sync::mpsc::error::SendError(failed_task)) =
            sender.send(task_with_resp).await
        {
            self.remove_chat_sender(cid);
            let fresh_sender = self.get_or_spawn_actor(cid);
            let _ = fresh_sender.send(failed_task).await;
        }

        rx.await.map_err(|_| "Response dropped".to_string())?
    }

    fn remove_chat_sender(&self, chat_id: i64) {
        let mut map = self.chat_senders.lock().unwrap_or_else(|e| e.into_inner());
        map.remove(&chat_id);
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
            let idle_duration = Duration::from_secs(900);
            loop {
                match tokio::time::timeout(idle_duration, rx.recv()).await {
                    Ok(Some(task)) => {
                        let res = execute_task_direct(&client, task.payload).await;
                        if let Some(responder) = task.responder {
                            let _ = responder.send(res);
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {
                        if let Some(map_arc) = senders_weak.upgrade() {
                            let mut map = map_arc.lock().unwrap_or_else(|e| e.into_inner());
                            map.remove(&chat_id);
                        }
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
