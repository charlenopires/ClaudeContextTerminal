//! Intelligent caching system for completion performance optimization

use super::CompletionItem;
use std::collections::HashMap;
use std::time::{Instant, Duration};
use tracing::{debug, trace};

/// Cache entry with expiration time
#[derive(Debug, Clone)]
struct CacheEntry {
    items: Vec<CompletionItem>,
    created_at: Instant,
    access_count: u64,
    last_accessed: Instant,
}

impl CacheEntry {
    fn new(items: Vec<CompletionItem>) -> Self {
        let now = Instant::now();
        Self {
            items,
            created_at: now,
            access_count: 0,
            last_accessed: now,
        }
    }

    fn is_expired(&self, ttl: Duration) -> bool {
        self.created_at.elapsed() > ttl
    }

    fn access(&mut self) -> &Vec<CompletionItem> {
        self.access_count += 1;
        self.last_accessed = Instant::now();
        &self.items
    }
}

/// Intelligent completion cache with LRU eviction and TTL support
#[derive(Debug)]
pub struct CompletionCache {
    cache: HashMap<String, CacheEntry>,
    max_size: usize,
    default_ttl: Duration,
    hits: u64,
    misses: u64,
}

impl CompletionCache {
    /// Create a new completion cache
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            max_size: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            hits: 0,
            misses: 0,
        }
    }

    /// Create cache with custom settings
    pub fn with_settings(max_size: usize, default_ttl: Duration) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
            default_ttl,
            hits: 0,
            misses: 0,
        }
    }

    /// Insert completion items into cache
    pub fn insert(&mut self, key: String, items: Vec<CompletionItem>) {
        trace!("Caching {} items for key: {}", items.len(), key);

        // Clean expired entries before inserting
        self.clean_expired();

        // Check if we need to evict entries to make room
        if self.cache.len() >= self.max_size {
            self.evict_lru();
        }

        let entry = CacheEntry::new(items);
        self.cache.insert(key, entry);
    }

    /// Get completion items from cache
    pub fn get(&mut self, key: &str) -> Option<Vec<CompletionItem>> {
        if let Some(entry) = self.cache.get_mut(key) {
            if entry.is_expired(self.default_ttl) {
                trace!("Cache entry expired for key: {}", key);
                self.cache.remove(key);
                self.misses += 1;
                return None;
            }

            trace!("Cache hit for key: {} (access count: {})", key, entry.access_count + 1);
            self.hits += 1;
            Some(entry.access().clone())
        } else {
            trace!("Cache miss for key: {}", key);
            self.misses += 1;
            None
        }
    }

    /// Remove entry from cache
    pub fn remove(&mut self, key: &str) -> Option<Vec<CompletionItem>> {
        self.cache.remove(key).map(|entry| entry.items)
    }

    /// Clear all cache entries
    pub fn clear(&mut self) {
        debug!("Clearing completion cache ({} entries)", self.cache.len());
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }

    /// Get number of cached entries
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }

    /// Get cache hit rate
    pub fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            self.hits as f64 / total as f64
        }
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        CacheStats {
            size: self.cache.len(),
            max_size: self.max_size,
            hits: self.hits,
            misses: self.misses,
            hit_rate: self.hit_rate(),
            total_access_count: self.cache.values().map(|e| e.access_count).sum(),
        }
    }

    /// Set maximum cache size
    pub fn set_max_size(&mut self, max_size: usize) {
        self.max_size = max_size;
        while self.cache.len() > max_size {
            self.evict_lru();
        }
    }

    /// Set default TTL
    pub fn set_default_ttl(&mut self, ttl: Duration) {
        self.default_ttl = ttl;
    }

    /// Invalidate entries matching a pattern
    pub fn invalidate_pattern(&mut self, pattern: &str) {
        let keys_to_remove: Vec<String> = self.cache
            .keys()
            .filter(|key| key.contains(pattern))
            .cloned()
            .collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
            debug!("Invalidated cache entry: {}", key);
        }
    }

    /// Clean expired entries
    fn clean_expired(&mut self) {
        let now = Instant::now();
        let keys_to_remove: Vec<String> = self.cache
            .iter()
            .filter(|(_, entry)| entry.is_expired(self.default_ttl))
            .map(|(key, _)| key.clone())
            .collect();

        for key in keys_to_remove {
            self.cache.remove(&key);
            trace!("Removed expired cache entry: {}", key);
        }
    }

    /// Evict least recently used entry
    fn evict_lru(&mut self) {
        if let Some((lru_key, _)) = self.cache
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, e)| (k.clone(), e.last_accessed))
        {
            self.cache.remove(&lru_key);
            trace!("Evicted LRU cache entry: {}", lru_key);
        }
    }
}

impl Default for CompletionCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub size: usize,
    pub max_size: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub total_access_count: u64,
}

/// Smart cache that adapts based on usage patterns
#[derive(Debug)]
pub struct AdaptiveCache {
    cache: CompletionCache,
    query_patterns: HashMap<String, u32>,
    adaptive_ttl: bool,
}

impl AdaptiveCache {
    /// Create a new adaptive cache
    pub fn new() -> Self {
        Self {
            cache: CompletionCache::new(),
            query_patterns: HashMap::new(),
            adaptive_ttl: true,
        }
    }

    /// Insert with adaptive TTL based on query patterns
    pub fn insert_adaptive(&mut self, key: String, items: Vec<CompletionItem>) {
        // Track query pattern frequency
        let pattern = self.extract_pattern(&key);
        *self.query_patterns.entry(pattern).or_insert(0) += 1;

        // Adjust cache behavior based on patterns
        if self.adaptive_ttl {
            self.adjust_cache_settings();
        }

        self.cache.insert(key, items);
    }

    /// Get with pattern tracking
    pub fn get_adaptive(&mut self, key: &str) -> Option<Vec<CompletionItem>> {
        let pattern = self.extract_pattern(key);
        *self.query_patterns.entry(pattern).or_insert(0) += 1;
        self.cache.get(key)
    }

    /// Extract query pattern for adaptive behavior
    fn extract_pattern(&self, key: &str) -> String {
        // Simple pattern extraction - could be more sophisticated
        let parts: Vec<&str> = key.split(':').collect();
        if parts.len() >= 2 {
            format!("{}:*", parts[0])
        } else {
            "*".to_string()
        }
    }

    /// Adjust cache settings based on usage patterns
    fn adjust_cache_settings(&mut self) {
        let total_queries: u32 = self.query_patterns.values().sum();
        if total_queries < 10 {
            return; // Not enough data
        }

        // Find most common patterns
        let mut patterns: Vec<_> = self.query_patterns.iter().collect();
        patterns.sort_by(|a, b| b.1.cmp(a.1));

        // Adjust TTL based on pattern frequency
        if let Some((_, count)) = patterns.first() {
            let frequency_ratio = **count as f64 / total_queries as f64;
            
            if frequency_ratio > 0.5 {
                // High frequency patterns get longer TTL
                self.cache.set_default_ttl(Duration::from_secs(600)); // 10 minutes
            } else if frequency_ratio < 0.1 {
                // Low frequency patterns get shorter TTL
                self.cache.set_default_ttl(Duration::from_secs(60)); // 1 minute
            }
        }
    }

    /// Delegate methods to underlying cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.query_patterns.clear();
    }

    pub fn len(&self) -> usize {
        self.cache.len()
    }

    pub fn stats(&self) -> CacheStats {
        self.cache.stats()
    }

    pub fn hit_rate(&self) -> f64 {
        self.cache.hit_rate()
    }
}

impl Default for AdaptiveCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::components::completions::CompletionItem;

    #[test]
    fn test_cache_basic_operations() {
        let mut cache = CompletionCache::new();
        
        let items = vec![
            CompletionItem::new("test1", "test1", "provider"),
            CompletionItem::new("test2", "test2", "provider"),
        ];

        // Test insert and get
        cache.insert("key1".to_string(), items.clone());
        let cached = cache.get("key1").unwrap();
        assert_eq!(cached.len(), 2);
        assert_eq!(cached[0].title, "test1");

        // Test hit rate
        assert!(cache.hit_rate() > 0.0);
    }

    #[test]
    fn test_cache_expiration() {
        let mut cache = CompletionCache::with_settings(10, Duration::from_millis(1));
        
        let items = vec![CompletionItem::new("test", "test", "provider")];
        cache.insert("key1".to_string(), items);

        // Wait for expiration
        std::thread::sleep(Duration::from_millis(2));

        // Should be expired
        assert!(cache.get("key1").is_none());
    }

    #[test]
    fn test_cache_lru_eviction() {
        let mut cache = CompletionCache::with_settings(2, Duration::from_secs(300));
        
        let items = vec![CompletionItem::new("test", "test", "provider")];
        
        // Fill cache to capacity
        cache.insert("key1".to_string(), items.clone());
        cache.insert("key2".to_string(), items.clone());
        
        // Access key1 to make key2 LRU
        cache.get("key1");
        
        // Insert key3, should evict key2
        cache.insert("key3".to_string(), items);
        
        assert!(cache.get("key1").is_some());
        assert!(cache.get("key2").is_none());
        assert!(cache.get("key3").is_some());
    }

    #[test]
    fn test_adaptive_cache() {
        let mut cache = AdaptiveCache::new();
        
        let items = vec![CompletionItem::new("test", "test", "provider")];
        
        // Test adaptive insertion and retrieval
        cache.insert_adaptive("pattern1:query1".to_string(), items.clone());
        cache.insert_adaptive("pattern1:query2".to_string(), items.clone());
        cache.insert_adaptive("pattern2:query3".to_string(), items);
        
        assert!(cache.get_adaptive("pattern1:query1").is_some());
        assert_eq!(cache.len(), 3);
        
        // Verify pattern tracking
        assert!(cache.query_patterns.len() > 0);
    }
}