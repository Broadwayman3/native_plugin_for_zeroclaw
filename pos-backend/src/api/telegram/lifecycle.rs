use crate::api::telegram::fsm::FsmStore;
use crate::api::telegram::locks::{ChatLocksManager, InFlightTracker};
use crate::api::telegram::{polling, verifier, webhook, webhook_worker};
use crate::config::AppConfig;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

static FAILED_WEBHOOK_ATTEMPTS: AtomicU32 = AtomicU32::new(0);
static WEBHOOK_COOLDOWN_UNTIL: AtomicU64 = AtomicU64::new(0);
pub static IS_POLLER_ACTIVE: AtomicBool = AtomicBool::new(false);

/// RAII Guard that automatically resets IS_POLLER_ACTIVE to false when dropped (even on panic).
pub struct PollerActiveGuard;

impl Default for PollerActiveGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl PollerActiveGuard {
    pub fn new() -> Self {
        IS_POLLER_ACTIVE.store(true, Ordering::SeqCst);
        Self
    }
}

impl Drop for PollerActiveGuard {
    fn drop(&mut self) {
        IS_POLLER_ACTIVE.store(false, Ordering::SeqCst);
        tracing::info!("IS_POLLER_ACTIVE flag reset to false via RAII guard");
    }
}

pub const MAX_WEBHOOK_FAILURES: u32 = 3;
pub const WEBHOOK_COOLDOWN_SECS: u64 = 300;

pub struct TelegramServicesHandles {
    pub verifier_handle: tokio::task::JoinHandle<()>,
    pub webhook_worker_handle: tokio::task::JoinHandle<()>,
    pub listener_handle: tokio::task::JoinHandle<()>,
}

impl TelegramServicesHandles {
    pub async fn shutdown_with_timeout(self, timeout_secs: u64) {
        let _ = tokio::time::timeout(std::time::Duration::from_secs(timeout_secs), async move {
            let _ = self.verifier_handle.await;
            let _ = self.webhook_worker_handle.await;
            let _ = self.listener_handle.await;
        })
        .await;
    }
}

/// Starts background Telegram listener (Webhook or Polling) and Solana RPC payment verifier services.
pub fn start_telegram_services(
    config: Arc<AppConfig>,
    db_pool: Option<deadpool_sqlite::Pool>,
    cancel_token: CancellationToken,
) -> Option<TelegramServicesHandles> {
    let token = config.telegram_bot_token.clone();
    if token.is_empty() || token.contains("123456789:ABC") {
        tracing::warn!("Telegram Bot token not set or placeholder. Skipping Telegram services.");
        return None;
    }

    let webhook_notify = Arc::new(tokio::sync::Notify::new());
    let _gc_handle = super::rate_limiter::start_rate_limiter_gc_worker(cancel_token.clone());
    let verifier_handle = verifier::start_verifier_worker(config.clone(), cancel_token.clone());

    let fsm_store = if let Some(ref pool) = db_pool {
        FsmStore::new_with_pool(config.db_path.clone(), pool.clone())
    } else {
        FsmStore::new_with_db(config.db_path.clone())
    };
    let chat_locks = ChatLocksManager::new();
    let in_flight = InFlightTracker::new();

    let webhook_worker_handle = webhook_worker::start_webhook_queue_worker(
        config.clone(),
        fsm_store.clone(),
        chat_locks.clone(),
        in_flight.clone(),
        db_pool.clone(),
        webhook_notify.clone(),
        cancel_token.clone(),
    );

    let poller_config = config.clone();
    let poller_pool = db_pool.clone();
    let parent_cancel_token = cancel_token.clone();

    let listener_handle = tokio::spawn(async move {
        if let Some(ref webhook_url) = poller_config.telegram_webhook_url {
            if !webhook_url.trim().is_empty() {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                let cooldown_until = WEBHOOK_COOLDOWN_UNTIL.load(Ordering::SeqCst);

                if now < cooldown_until {
                    tracing::warn!(
                        cooldown_secs = cooldown_until - now,
                        "Webhook circuit breaker active (cooldown). Falling back to Long Polling directly."
                    );
                    let poller_cancel = parent_cancel_token.child_token();
                    let _active_guard = PollerActiveGuard::new();
                    let poller_h = polling::start_poller_worker(
                        poller_config,
                        fsm_store,
                        chat_locks,
                        in_flight.clone(),
                        poller_pool,
                        poller_cancel,
                    );
                    let _ = poller_h.await;
                    return;
                }

                if let Err(e) = webhook::register_telegram_webhook(&poller_config).await {
                    let failures = FAILED_WEBHOOK_ATTEMPTS.fetch_add(1, Ordering::SeqCst) + 1;
                    tracing::error!(
                        error = %e,
                        failures = failures,
                        "Failed to register Telegram Webhook"
                    );

                    let poller_cancel = parent_cancel_token.child_token();
                    if failures >= MAX_WEBHOOK_FAILURES {
                        let cooldown = now + WEBHOOK_COOLDOWN_SECS;
                        WEBHOOK_COOLDOWN_UNTIL.store(cooldown, Ordering::SeqCst);
                        tracing::error!(
                            cooldown_secs = WEBHOOK_COOLDOWN_SECS,
                            "Webhook registration failed 3 consecutive times. Circuit breaker TRIPPED! Falling back to Long Polling with 5-minute cooldown."
                        );

                        let recovery_config = poller_config.clone();
                        let recovery_fsm_store = fsm_store.clone();
                        let recovery_chat_locks = chat_locks.clone();
                        let recovery_in_flight = in_flight.clone();
                        let recovery_poller_pool = poller_pool.clone();
                        let recovery_poller_cancel = poller_cancel.clone();
                        let recovery_parent_cancel = parent_cancel_token.clone();
                        tokio::spawn(async move {
                            tokio::select! {
                                _ = recovery_parent_cancel.cancelled() => {
                                    tracing::info!("Webhook recovery task cancelled by parent token.");
                                    return;
                                }
                                _ = tokio::time::sleep(std::time::Duration::from_secs(WEBHOOK_COOLDOWN_SECS)) => {}
                            }
                            tracing::info!("Webhook circuit breaker cooldown expired. Cancelling poller worker and attempting Webhook recovery...");
                            recovery_poller_cancel.cancel();
                            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                            if let Ok(()) =
                                webhook::register_telegram_webhook(&recovery_config).await
                            {
                                FAILED_WEBHOOK_ATTEMPTS.store(0, Ordering::SeqCst);
                                WEBHOOK_COOLDOWN_UNTIL.store(0, Ordering::SeqCst);
                                tracing::info!("Webhook registration recovered successfully! Webhook mode restored.");
                            } else {
                                tracing::warn!("Webhook recovery re-attempt failed. Restarting Long Polling worker.");
                                let new_poller_cancel = recovery_parent_cancel.child_token();
                                let _active_guard = PollerActiveGuard::new();
                                let poller_h = polling::start_poller_worker(
                                    recovery_config,
                                    recovery_fsm_store,
                                    recovery_chat_locks,
                                    recovery_in_flight,
                                    recovery_poller_pool,
                                    new_poller_cancel,
                                );
                                let _ = poller_h.await;
                            }
                        });
                    }
                    // Wait with 15s safety timeout for in-flight Webhook POST requests & pending queue to drain before starting Long Polling
                    drain_pending_webhooks_with_timeout(
                        poller_pool.as_ref(),
                        &poller_config.db_path,
                    )
                    .await;
                    let _active_guard = PollerActiveGuard::new();
                    let poller_h = polling::start_poller_worker(
                        poller_config,
                        fsm_store,
                        chat_locks,
                        in_flight.clone(),
                        poller_pool,
                        poller_cancel,
                    );
                    let _ = poller_h.await;
                    return;
                }

                FAILED_WEBHOOK_ATTEMPTS.store(0, Ordering::SeqCst);
                WEBHOOK_COOLDOWN_UNTIL.store(0, Ordering::SeqCst);
                return;
            }
        }

        let poller_cancel = parent_cancel_token.child_token();
        drain_pending_webhooks_with_timeout(poller_pool.as_ref(), &poller_config.db_path).await;
        let _active_guard = PollerActiveGuard::new();
        let poller_h = polling::start_poller_worker(
            poller_config,
            fsm_store,
            chat_locks,
            in_flight.clone(),
            poller_pool,
            poller_cancel,
        );
        let _ = poller_h.await;
    });

    Some(TelegramServicesHandles {
        verifier_handle,
        webhook_worker_handle,
        listener_handle,
    })
}

/// Drains pending webhook updates from SQLite with a 15-second safety timeout.
async fn drain_pending_webhooks_with_timeout(
    db_pool: Option<&deadpool_sqlite::Pool>,
    db_path: &str,
) {
    let _ = tokio::time::timeout(std::time::Duration::from_secs(15), async {
        loop {
            let pending_count = if let Some(pool) = db_pool {
                if let Ok(conn) = pool.get().await {
                    conn.interact(|c| {
                        c.query_row(
                            "SELECT COUNT(*) FROM pending_webhook_updates WHERE status = 'pending'",
                            [],
                            |r| r.get::<_, i64>(0),
                        )
                        .unwrap_or(0)
                    })
                    .await
                    .unwrap_or(0)
                } else {
                    0
                }
            } else {
                let db_p = db_path.to_string();
                tokio::task::spawn_blocking(move || {
                    if let Ok(conn) = crate::db::get_db_connection(&db_p) {
                        conn.query_row(
                            "SELECT COUNT(*) FROM pending_webhook_updates WHERE status = 'pending'",
                            [],
                            |r| r.get::<_, i64>(0),
                        )
                        .unwrap_or(0)
                    } else {
                        0
                    }
                })
                .await
                .unwrap_or(0)
            };

            if pending_count == 0 {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    })
    .await;
}
