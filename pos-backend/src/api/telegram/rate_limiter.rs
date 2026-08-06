use governor::{Quota, RateLimiter};
use once_cell::sync::Lazy;
use std::collections::HashMap;
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
static PER_CHAT_PAUSE_MAP: Lazy<Mutex<HashMap<i64, Instant>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));
static RECENT_429_CHATS: Lazy<Mutex<Vec<(i64, Instant)>>> = Lazy::new(|| Mutex::new(Vec::new()));

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

/// Records a 429 event for a specific chat, with adaptive escalation to global pause if >=3 distinct chats hit 429 within 1 second.
pub fn record_chat_429(chat_id: Option<i64>, retry_after_secs: u64) {
    let now = Instant::now();
    let pause_deadline = now + Duration::from_secs(retry_after_secs + 1);

    if let Some(cid) = chat_id {
        if let Ok(mut map) = PER_CHAT_PAUSE_MAP.lock() {
            map.insert(cid, pause_deadline);
        }

        if let Ok(mut recent) = RECENT_429_CHATS.lock() {
            recent.retain(|(_, ts)| now.duration_since(*ts) <= Duration::from_secs(1));
            recent.push((cid, now));

            let distinct_chats: std::collections::HashSet<i64> =
                recent.iter().map(|(id, _)| *id).collect();
            if distinct_chats.len() >= 3 {
                tracing::warn!(
                    distinct_count = distinct_chats.len(),
                    "Adaptive 429 escalation triggered: 3+ distinct chats hit 429 within 1s. Escalating to global bot pause."
                );
                set_global_429_pause(retry_after_secs);
            }
        }
    } else {
        set_global_429_pause(retry_after_secs);
    }
}

/// Enforces global rate limits, per-chat rate limits, and per-chat/global 429 pause signals using monotonic Instant.
pub async fn enforce_rate_limit(chat_id: Option<i64>) {
    let mut wait_duration = None;
    if let Ok(mut guard) = GLOBAL_PAUSE_UNTIL_INSTANT.lock() {
        if let Some(deadline) = *guard {
            let now = Instant::now();
            if deadline > now {
                wait_duration = Some(deadline - now);
            } else {
                *guard = None;
            }
        }
    }

    if let Some(dur) = wait_duration {
        sleep(dur).await;
    }

    if let Some(cid) = chat_id {
        let mut chat_wait = None;
        if let Ok(mut map) = PER_CHAT_PAUSE_MAP.lock() {
            if let Some(deadline) = map.get(&cid) {
                let now = Instant::now();
                if *deadline > now {
                    chat_wait = Some(*deadline - now);
                } else {
                    map.remove(&cid);
                }
            }
        }
        if let Some(dur) = chat_wait {
            sleep(dur).await;
        }
    }

    GLOBAL_TELEGRAM_LIMITER.until_ready().await;
    if let Some(cid) = chat_id {
        PER_CHAT_TELEGRAM_LIMITER.until_key_ready(&cid).await;
    }
}

/// Periodic GC pass for Keyed Rate Limiter & per-chat pause map to prevent memory growth.
pub fn retain_recent_keys() {
    PER_CHAT_TELEGRAM_LIMITER.retain_recent();
    let now = Instant::now();
    if let Ok(mut map) = PER_CHAT_PAUSE_MAP.lock() {
        map.retain(|_, deadline| *deadline > now);
    }
    if let Ok(mut recent) = RECENT_429_CHATS.lock() {
        recent.retain(|(_, ts)| now.duration_since(*ts) <= Duration::from_secs(1));
    }
}

/// Starts background worker that periodically purges stale rate limiter keys every 10 minutes.
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
