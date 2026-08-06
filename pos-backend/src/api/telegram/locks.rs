use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, Mutex as AsyncMutex};

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

/// Track pending in-flight update IDs and compute the Low Watermark monotonically.
#[derive(Clone, Default)]
pub struct WatermarkTracker {
    pending: Arc<Mutex<BTreeSet<i64>>>,
}

impl WatermarkTracker {
    pub fn new() -> Self {
        Self {
            pending: Arc::new(Mutex::new(BTreeSet::new())),
        }
    }

    pub fn register(&self, update_id: i64) -> WatermarkGuard {
        let mut set = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        set.insert(update_id);
        WatermarkGuard {
            update_id,
            tracker: self.clone(),
        }
    }

    pub fn complete(&self, update_id: i64) -> Option<i64> {
        let mut set = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        set.remove(&update_id);
        set.iter().next().copied()
    }

    pub fn low_watermark(&self) -> Option<i64> {
        let set = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        set.iter().next().copied()
    }

    pub fn is_empty(&self) -> bool {
        let set = self.pending.lock().unwrap_or_else(|e| e.into_inner());
        set.is_empty()
    }
}

/// Guard for completing an update in WatermarkTracker ONLY on verified success or DLQ commit.
#[derive(Clone)]
pub struct WatermarkGuard {
    pub update_id: i64,
    pub tracker: WatermarkTracker,
}

impl WatermarkGuard {
    /// Mark completed ONLY when processing succeeded or reached DLQ in SQLite.
    pub fn complete(&self) -> Option<i64> {
        self.tracker.complete(self.update_id)
    }
}

/// Task closure type for chat queue dispatcher.
pub type DispatchTask = Box<dyn FnOnce() -> tokio::task::JoinHandle<()> + Send + 'static>;

struct QueueEntry {
    sender: mpsc::Sender<DispatchTask>,
    last_active: Instant,
}

/// Per-session FIFO queue dispatcher with bounded capacity (64 items) per chat and idle TTL cleanup.
#[derive(Clone, Default)]
pub struct ChatQueueDispatcher {
    senders: Arc<Mutex<HashMap<(i64, i64), QueueEntry>>>,
}

impl ChatQueueDispatcher {
    pub fn new() -> Self {
        Self {
            senders: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Enqueues a task for session `(chat_id, user_id)`.
    /// Returns Ok(()) if enqueued, or Err(mpsc::error::TrySendError) if bounded capacity (64) is full.
    pub fn try_enqueue(
        &self,
        chat_id: i64,
        user_id: i64,
        task: DispatchTask,
    ) -> Result<(), mpsc::error::TrySendError<DispatchTask>> {
        let key = (chat_id, user_id);
        let mut map = self.senders.lock().unwrap_or_else(|e| e.into_inner());
        let now = Instant::now();

        // Safe TTL cleanup: remove queues idle for over 60 seconds
        if map.len() > 64 {
            map.retain(|_, entry| now.duration_since(entry.last_active) < Duration::from_secs(60));
        }

        let entry = map.entry(key).or_insert_with(|| {
            let (tx, mut rx) = mpsc::channel::<DispatchTask>(64);
            tokio::spawn(async move {
                while let Some(task_fn) = rx.recv().await {
                    let handle = task_fn();
                    let _ = handle.await;
                }
            });
            QueueEntry {
                sender: tx,
                last_active: now,
            }
        });

        entry.last_active = now;
        entry.sender.try_send(task)
    }
}
