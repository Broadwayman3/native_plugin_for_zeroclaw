use governor::{Quota, RateLimiter};
use once_cell::sync::Lazy;
use std::num::NonZeroU32;
use std::sync::Mutex;
use tokio::time::{sleep, Duration, Instant};

static GLOBAL_TELEGRAM_LIMITER: Lazy<governor::DefaultDirectRateLimiter> = Lazy::new(|| {
    let quota = Quota::per_second(NonZeroU32::new(25).unwrap());
    RateLimiter::direct(quota)
});

static PER_CHAT_TELEGRAM_LIMITER: Lazy<governor::DefaultKeyedRateLimiter<i64>> = Lazy::new(|| {
    let quota = Quota::per_second(NonZeroU32::new(1).unwrap());
    RateLimiter::keyed(quota)
});

static GLOBAL_PAUSE_UNTIL_INSTANT: Lazy<Mutex<Option<Instant>>> = Lazy::new(|| Mutex::new(None));

/// Triggers a global outbound queue pause when Telegram returns HTTP 429 (Too Many Requests).
pub fn set_global_429_pause(retry_after_secs: u64) {
    let pause_deadline = Instant::now() + Duration::from_secs(retry_after_secs + 1);
    if let Ok(mut guard) = GLOBAL_PAUSE_UNTIL_INSTANT.lock() {
        *guard = Some(pause_deadline);
    }
    tracing::warn!(
        retry_after_secs = retry_after_secs,
        "Telegram HTTP 429 received! Global rate limiter paused."
    );
}

/// Enforces global rate limits, chat rate limits, and checks for active HTTP 429 pause signals using monotonic Instant.
pub async fn enforce_rate_limit(chat_id: Option<i64>) {
    let mut wait_duration = None;
    if let Ok(mut guard) = GLOBAL_PAUSE_UNTIL_INSTANT.lock() {
        if let Some(deadline) = *guard {
            let now = Instant::now();
            if deadline > now {
                wait_duration = Some(deadline - now);
            } else {
                *guard = None; // Reset expired deadline to zero out Mutex lock overhead
            }
        }
    }

    if let Some(dur) = wait_duration {
        sleep(dur).await;
    }

    GLOBAL_TELEGRAM_LIMITER.until_ready().await;
    if let Some(cid) = chat_id {
        PER_CHAT_TELEGRAM_LIMITER.until_key_ready(&cid).await;
    }
}

/// Periodic GC pass for Keyed Rate Limiter to remove stale chat entries and prevent memory growth.
pub fn retain_recent_keys() {
    PER_CHAT_TELEGRAM_LIMITER.retain_recent();
}

/// Starts background worker that periodically purges stale chat keys from the rate limiter map every 10 minutes.
pub fn start_rate_limiter_gc_worker(
    cancel_token: tokio_util::sync::CancellationToken,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let interval = Duration::from_secs(600);
        loop {
            tokio::select! {
                _ = cancel_token.cancelled() => break,
                _ = sleep(interval) => {
                    retain_recent_keys();
                }
            }
        }
    })
}
