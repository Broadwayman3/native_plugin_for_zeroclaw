use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, Weak};
use tokio::sync::Mutex as AsyncMutex;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum LockKey {
    UserSession(i64, i64),
    Invoice(String),
}

/// Safe per-session or per-invoice concurrency manager using Weak references.
#[derive(Clone, Default)]
pub struct ChatLocksManager {
    locks: Arc<Mutex<HashMap<LockKey, Weak<AsyncMutex<()>>>>>,
}

impl ChatLocksManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Gets existing lock or creates a new AsyncMutex for (chat_id, user_id).
    /// For group chats (chat_id < 0), key is (chat_id, user_id). For private chats, user_id == chat_id.
    pub fn get_or_create(&self, chat_id: i64, user_id: i64) -> Arc<AsyncMutex<()>> {
        let key = if chat_id < 0 && user_id != 0 {
            LockKey::UserSession(chat_id, user_id)
        } else {
            LockKey::UserSession(chat_id, chat_id)
        };
        self.get_or_create_key(key)
    }

    /// Gets or creates an AsyncMutex lock by invoice_id.
    pub fn get_or_create_by_invoice(&self, invoice_id: &str) -> Arc<AsyncMutex<()>> {
        self.get_or_create_key(LockKey::Invoice(invoice_id.to_string()))
    }

    fn get_or_create_key(&self, key: LockKey) -> Arc<AsyncMutex<()>> {
        let mut map = self.locks.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(weak) = map.get(&key) {
            if let Some(strong) = weak.upgrade() {
                return strong;
            }
        }

        let new_lock = Arc::new(AsyncMutex::new(()));
        map.insert(key, Arc::downgrade(&new_lock));

        // Periodic light GC if map gets large
        if map.len() > 128 {
            map.retain(|_, v| v.strong_count() > 0);
        }

        new_lock
    }
}

/// In-flight update tracker to eliminate concurrent update_id execution race conditions.
#[derive(Clone, Default)]
pub struct InFlightTracker {
    in_flight: Arc<Mutex<HashSet<i64>>>,
}

/// RAII guard that automatically releases update_id from InFlightTracker when dropped.
pub struct InFlightGuard {
    update_id: i64,
    tracker: InFlightTracker,
}

impl Drop for InFlightGuard {
    fn drop(&mut self) {
        self.tracker.remove(self.update_id);
    }
}

impl InFlightTracker {
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    /// Attempts to claim update_id for in-flight processing.
    /// Returns Some(InFlightGuard) if claimed successfully, or None if update_id is already in-flight.
    pub fn try_claim(&self, update_id: i64) -> Option<InFlightGuard> {
        let mut set = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
        if set.insert(update_id) {
            Some(InFlightGuard {
                update_id,
                tracker: self.clone(),
            })
        } else {
            None
        }
    }

    pub fn remove(&self, update_id: i64) {
        let mut set = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
        set.remove(&update_id);
    }
}
