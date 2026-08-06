use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::Mutex;

/// Thread-safe canonical O(1) LRU language cache for chat_ids with max capacity limit.
pub struct LangCache {
    cache: Mutex<LruCache<i64, String>>,
}

impl Default for LangCache {
    fn default() -> Self {
        Self::new(2048)
    }
}

impl LangCache {
    pub fn new(capacity: usize) -> Self {
        let cap = NonZeroUsize::new(capacity).unwrap_or_else(|| NonZeroUsize::new(2048).unwrap());
        Self {
            cache: Mutex::new(LruCache::new(cap)),
        }
    }

    /// Gets cached language code for chat_id with guaranteed O(1) time complexity.
    pub fn get(&self, chat_id: i64) -> Option<String> {
        let mut guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        guard.get(&chat_id).cloned()
    }

    /// Puts language code for chat_id into cache with guaranteed O(1) time complexity and LRU eviction.
    pub fn put(&self, chat_id: i64, lang_code: String) {
        let mut guard = self.cache.lock().unwrap_or_else(|e| e.into_inner());
        guard.put(chat_id, lang_code);
    }
}
