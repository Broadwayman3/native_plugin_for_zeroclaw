use std::sync::atomic::{AtomicI64, Ordering};

static IN_MEMORY_OFFSET: AtomicI64 = AtomicI64::new(0);

/// Get current in-memory update offset.
pub fn get_offset() -> i64 {
    IN_MEMORY_OFFSET.load(Ordering::SeqCst)
}

/// Set current in-memory update offset.
pub fn set_offset(val: i64) {
    IN_MEMORY_OFFSET.store(val, Ordering::SeqCst);
}

/// Helper to compute next batch offset monotonically.
pub fn advance_offset_if_greater(current: &mut i64, candidate: i64) -> bool {
    if candidate > *current {
        *current = candidate;
        set_offset(candidate);
        true
    } else {
        false
    }
}
