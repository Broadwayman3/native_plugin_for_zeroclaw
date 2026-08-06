use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{Duration, Instant};

pub const FSM_TTL: Duration = Duration::from_secs(300); // 5 minutes

#[derive(Debug, Clone)]
pub struct PendingSession {
    pub item_name: String,
    pub currency: Option<String>,
    pub created_at: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct FsmStore {
    sessions: Arc<RwLock<HashMap<(i64, i64), PendingSession>>>,
}

impl FsmStore {
    pub fn new() -> Self {
        let store = Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        };

        // Spawn background periodic GC worker (runs every 10 minutes)
        let store_clone = store.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(600));
            loop {
                interval.tick().await;
                store_clone.cleanup_expired().await;
            }
        });

        store
    }

    /// Sets or updates pending session state for (chat_id, user_id).
    pub async fn set_pending(
        &self,
        chat_id: i64,
        user_id: i64,
        item_name: String,
        currency: Option<String>,
    ) {
        let mut map = self.sessions.write().await;
        map.insert(
            (chat_id, user_id),
            PendingSession {
                item_name,
                currency,
                created_at: Instant::now(),
            },
        );
    }

    /// Gets valid pending session for (chat_id, user_id) if within TTL.
    pub async fn get_pending(&self, chat_id: i64, user_id: i64) -> Option<PendingSession> {
        let map = self.sessions.read().await;
        if let Some(session) = map.get(&(chat_id, user_id)) {
            if session.created_at.elapsed() < FSM_TTL {
                return Some(session.clone());
            }
        }
        None
    }

    /// Removes session state for (chat_id, user_id).
    pub async fn clear(&self, chat_id: i64, user_id: i64) {
        let mut map = self.sessions.write().await;
        map.remove(&(chat_id, user_id));
    }

    /// Cleans up expired sessions from memory (GC).
    pub async fn cleanup_expired(&self) {
        let mut map = self.sessions.write().await;
        map.retain(|_, session| session.created_at.elapsed() < FSM_TTL);
    }
}
