use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex, Weak};
use tokio::sync::Mutex as AsyncMutex;

/// Safe per-chat ordering concurrency manager using Weak references.
#[derive(Clone, Default)]
pub struct ChatLocksManager {
    locks: Arc<Mutex<HashMap<i64, Weak<AsyncMutex<()>>>>>,
}

impl ChatLocksManager {
    pub fn new() -> Self {
        Self {
            locks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Gets existing lock or creates a new AsyncMutex for chat_id.
    /// Weak references eliminate GC race conditions across concurrent tasks.
    pub fn get_or_create(&self, chat_id: i64) -> Arc<AsyncMutex<()>> {
        let mut map = self.locks.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(weak) = map.get(&chat_id) {
            if let Some(strong) = weak.upgrade() {
                return strong;
            }
        }

        let new_lock = Arc::new(AsyncMutex::new(()));
        map.insert(chat_id, Arc::downgrade(&new_lock));

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
