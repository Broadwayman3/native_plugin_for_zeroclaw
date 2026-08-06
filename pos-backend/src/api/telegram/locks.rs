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
///
/// LOCK HIERARCHY RULES (DEADLOCK PREVENTION):
/// 1. Top-level dispatcher acquires `chat_lock` per `(chat_id, user_id)` first.
/// 2. Specific invoice handlers (e.g., `orders.rs`) acquire `invoice_lock` per `invoice_id` second as a child lock.
/// 3. Background services (`verifier.rs`, Squads watcher) may acquire `invoice_lock`, but MUST NEVER attempt
///    to acquire a `chat_lock` afterwards.
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
        let key = if chat_id < 0 {
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

    /// Explicitly prunes unused/idle locks.
    pub fn prune_idle(&self) {
        let mut map = self.locks.lock().unwrap_or_else(|e| e.into_inner());
        map.retain(|_, v| v.strong_count() > 0);
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

    pub fn try_claim_guard(&self, update_id: i64) -> Option<IdempotencyClaimGuard> {
        self.try_claim(update_id)
            .map(|g| IdempotencyClaimGuard::new(update_id, g))
    }

    pub fn remove(&self, update_id: i64) {
        let mut set = self.in_flight.lock().unwrap_or_else(|e| e.into_inner());
        set.remove(&update_id);
    }
}

/// 3-Phase Idempotency Claim Guard to prevent race conditions during update execution.
pub struct IdempotencyClaimGuard {
    pub update_id: i64,
    flight_guard: Option<InFlightGuard>,
    committed: bool,
}

impl IdempotencyClaimGuard {
    pub fn new(update_id: i64, flight_guard: InFlightGuard) -> Self {
        Self {
            update_id,
            flight_guard: Some(flight_guard),
            committed: false,
        }
    }

    /// Phase 3: Mark as successfully committed (processed in SQLite & LRU cache).
    pub fn commit(mut self) {
        self.committed = true;
    }

    /// Phase 3 DLQ: Mark as isolated in DLQ (terminal failure, do not retry).
    pub fn commit_dlq(mut self) {
        self.committed = true;
    }

    /// Phase 3 Release: Explicitly release claim on transient error to allow retry.
    pub fn release(mut self) {
        self.committed = false;
        self.flight_guard.take();
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

    /// Enqueues a task for session `(chat_id, user_id)` with backpressure timeout.
    /// Returns Ok(()) if enqueued within timeout, or Err if bounded capacity (64) remained full.
    pub async fn enqueue_timeout(
        &self,
        chat_id: i64,
        user_id: i64,
        task: DispatchTask,
        timeout: Duration,
    ) -> Result<(), mpsc::error::SendTimeoutError<DispatchTask>> {
        let key = (chat_id, user_id);
        let sender = {
            let mut map = self.senders.lock().unwrap_or_else(|e| e.into_inner());
            let now = Instant::now();

            if map.len() > 64 {
                map.retain(|_, entry| {
                    now.duration_since(entry.last_active) < Duration::from_secs(60)
                });
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
            entry.sender.clone()
        };

        sender.send_timeout(task, timeout).await
    }
}
