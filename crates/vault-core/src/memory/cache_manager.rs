//! # Advanced Cache Manager
//!
//! Multi-layered caching system with LRU, compression-aware policies,
//! and intelligent prefetching for optimal memory utilization.

use std::collections::{HashMap, BTreeMap};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum memory usage in bytes
    pub max_memory: usize,
    /// L1 cache size (in-memory, fastest)
    pub l1_size: usize,
    /// L2 cache size (compressed in-memory)
    pub l2_size: usize,
    /// L3 cache size (persistent, RocksDB)
    pub l3_size: usize,
    /// Default TTL for cached items
    pub default_ttl: Duration,
    /// Enable compression for L2 cache
    pub enable_compression: bool,
    /// Prefetch strategy
    pub prefetch_strategy: PrefetchStrategy,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory: 1024 * 1024 * 1024, // 1GB
            l1_size: 128 * 1024 * 1024,     // 128MB
            l2_size: 512 * 1024 * 1024,     // 512MB
            l3_size: 2048 * 1024 * 1024,    // 2GB
            default_ttl: Duration::from_secs(3600), // 1 hour
            enable_compression: true,
            prefetch_strategy: PrefetchStrategy::Sequential,
        }
    }
}

/// Prefetch strategies
#[derive(Debug, Clone, Copy)]
pub enum PrefetchStrategy {
    None,
    Sequential,
    Predictive,
    CompressionAware,
}

/// Cache policies for eviction
#[derive(Debug, Clone, Copy)]
pub enum CachePolicy {
    LRU,
    LFU,
    TimeWeighted,
    CompressionRatio,
}

/// Cache levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CacheLevel {
    L1, // Hot data, uncompressed
    L2, // Warm data, compressed
    L3, // Cold data, persistent
}

/// Cache entry metadata
#[derive(Debug, Clone)]
struct CacheEntry {
    data: Vec<u8>,
    access_count: u64,
    last_access: Instant,
    created_at: SystemTime,
    ttl: Option<Duration>,
    size: usize,
    compression_ratio: Option<f64>,
    level: CacheLevel,
}

/// Access pattern tracking for intelligent prefetching
#[derive(Debug, Default)]
struct AccessPattern {
    sequential_hits: u64,
    random_hits: u64,
    last_accessed_keys: Vec<String>,
    access_intervals: Vec<Duration>,
}

/// Multi-layered cache manager
pub struct VaultCacheManager {
    config: CacheConfig,

    // L1 Cache: Hot data, uncompressed
    l1_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    l1_lru: Arc<Mutex<BTreeMap<Instant, String>>>,
    l1_memory_used: Arc<Mutex<usize>>,

    // L2 Cache: Warm data, compressed
    l2_cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    l2_lru: Arc<Mutex<BTreeMap<Instant, String>>>,
    l2_memory_used: Arc<Mutex<usize>>,

    // Access patterns for intelligent caching
    access_patterns: Arc<RwLock<HashMap<String, AccessPattern>>>,

    // Global statistics
    stats: Arc<RwLock<CacheStats>>,
}

impl VaultCacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            l1_cache: Arc::new(RwLock::new(HashMap::new())),
            l1_lru: Arc::new(Mutex::new(BTreeMap::new())),
            l1_memory_used: Arc::new(Mutex::new(0)),
            l2_cache: Arc::new(RwLock::new(HashMap::new())),
            l2_lru: Arc::new(Mutex::new(BTreeMap::new())),
            l2_memory_used: Arc::new(Mutex::new(0)),
            access_patterns: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get data from cache (checks all levels)
    pub fn get(&self, key: &str) -> Option<Vec<u8>> {
        let now = Instant::now();

        // Try L1 first
        if let Some(data) = self.get_from_l1(key, now) {
            self.record_hit(CacheLevel::L1);
            self.update_access_pattern(key);
            return Some(data);
        }

        // Try L2
        if let Some(data) = self.get_from_l2(key, now) {
            self.record_hit(CacheLevel::L2);
            self.update_access_pattern(key);
            // Promote to L1 if frequently accessed
            self.maybe_promote_to_l1(key, &data);
            return Some(data);
        }

        // L3 would be handled by the storage engine directly
        self.record_miss();
        None
    }

    /// Put data into cache with intelligent level selection
    pub fn put(&self, key: &str, data: Vec<u8>, ttl: Option<Duration>) -> Result<(), CacheError> {
        let size = data.len();
        let level = self.select_cache_level(key, size);

        match level {
            CacheLevel::L1 => self.put_l1(key, data, ttl),
            CacheLevel::L2 => self.put_l2(key, data, ttl),
            CacheLevel::L3 => {
                // L3 would be handled by storage engine
                Ok(())
            }
        }
    }

    /// Put data specifically in L1 cache
    fn put_l1(&self, key: &str, data: Vec<u8>, ttl: Option<Duration>) -> Result<(), CacheError> {
        let now = Instant::now();
        let size = data.len();

        // Check if we need to evict
        self.ensure_l1_capacity(size)?;

        let entry = CacheEntry {
            data,
            access_count: 1,
            last_access: now,
            created_at: SystemTime::now(),
            ttl,
            size,
            compression_ratio: None,
            level: CacheLevel::L1,
        };

        {
            let mut cache = self.l1_cache.write().unwrap();
            let mut lru = self.l1_lru.lock().unwrap();
            let mut memory_used = self.l1_memory_used.lock().unwrap();

            cache.insert(key.to_string(), entry);
            lru.insert(now, key.to_string());
            *memory_used += size;
        }

        Ok(())
    }

    /// Put data specifically in L2 cache (with compression)
    fn put_l2(&self, key: &str, data: Vec<u8>, ttl: Option<Duration>) -> Result<(), CacheError> {
        let now = Instant::now();
        let original_size = data.len();

        // Compress data if enabled
        let (compressed_data, compression_ratio) = if self.config.enable_compression {
            match self.compress_data(&data) {
                Ok(compressed) => {
                    let ratio = original_size as f64 / compressed.len() as f64;
                    (compressed, Some(ratio))
                }
                Err(_) => (data, None),
            }
        } else {
            (data, None)
        };

        let size = compressed_data.len();

        // Check if we need to evict
        self.ensure_l2_capacity(size)?;

        let entry = CacheEntry {
            data: compressed_data,
            access_count: 1,
            last_access: now,
            created_at: SystemTime::now(),
            ttl,
            size,
            compression_ratio,
            level: CacheLevel::L2,
        };

        {
            let mut cache = self.l2_cache.write().unwrap();
            let mut lru = self.l2_lru.lock().unwrap();
            let mut memory_used = self.l2_memory_used.lock().unwrap();

            cache.insert(key.to_string(), entry);
            lru.insert(now, key.to_string());
            *memory_used += size;
        }

        Ok(())
    }

    /// Get data from L1 cache
    fn get_from_l1(&self, key: &str, now: Instant) -> Option<Vec<u8>> {
        let mut cache = self.l1_cache.write().unwrap();

        if let Some(entry) = cache.get_mut(key) {
            // Check TTL
            if self.is_expired(entry) {
                cache.remove(key);
                return None;
            }

            // Update access information
            entry.access_count += 1;
            entry.last_access = now;

            // Update LRU
            drop(cache);
            let mut lru = self.l1_lru.lock().unwrap();
            lru.retain(|_, v| v != key);
            lru.insert(now, key.to_string());

            // Re-acquire lock to get data
            let cache = self.l1_cache.read().unwrap();
            cache.get(key).map(|e| e.data.clone())
        } else {
            None
        }
    }

    /// Get data from L2 cache (with decompression)
    fn get_from_l2(&self, key: &str, now: Instant) -> Option<Vec<u8>> {
        let mut cache = self.l2_cache.write().unwrap();

        if let Some(entry) = cache.get_mut(key) {
            // Check TTL
            if self.is_expired(entry) {
                cache.remove(key);
                return None;
            }

            // Update access information
            entry.access_count += 1;
            entry.last_access = now;

            let data = if entry.compression_ratio.is_some() {
                // Decompress data
                self.decompress_data(&entry.data).unwrap_or_else(|_| entry.data.clone())
            } else {
                entry.data.clone()
            };

            // Update LRU
            drop(cache);
            let mut lru = self.l2_lru.lock().unwrap();
            lru.retain(|_, v| v != key);
            lru.insert(now, key.to_string());

            Some(data)
        } else {
            None
        }
    }

    /// Select appropriate cache level based on access patterns and size
    fn select_cache_level(&self, key: &str, size: usize) -> CacheLevel {
        // Small frequently accessed items go to L1
        if size < 1024 * 1024 { // < 1MB
            if let Ok(patterns) = self.access_patterns.read() {
                if let Some(pattern) = patterns.get(key) {
                    if pattern.sequential_hits > 3 || pattern.random_hits > 5 {
                        return CacheLevel::L1;
                    }
                }
            }
        }

        // Medium sized items or moderately accessed go to L2
        if size < 10 * 1024 * 1024 { // < 10MB
            return CacheLevel::L2;
        }

        // Large items go to L3 (persistent storage)
        CacheLevel::L3
    }

    /// Ensure L1 cache has enough capacity
    fn ensure_l1_capacity(&self, needed_size: usize) -> Result<(), CacheError> {
        let mut memory_used = self.l1_memory_used.lock().unwrap();

        while *memory_used + needed_size > self.config.l1_size {
            if !self.evict_l1_entry()? {
                break; // No more entries to evict
            }
            *memory_used = self.calculate_l1_memory_usage();
        }

        Ok(())
    }

    /// Ensure L2 cache has enough capacity
    fn ensure_l2_capacity(&self, needed_size: usize) -> Result<(), CacheError> {
        let mut memory_used = self.l2_memory_used.lock().unwrap();

        while *memory_used + needed_size > self.config.l2_size {
            if !self.evict_l2_entry()? {
                break; // No more entries to evict
            }
            *memory_used = self.calculate_l2_memory_usage();
        }

        Ok(())
    }

    /// Evict least recently used entry from L1
    fn evict_l1_entry(&self) -> Result<bool, CacheError> {
        let mut lru = self.l1_lru.lock().unwrap();

        if let Some((&oldest_time, key)) = lru.iter().next() {
            let key = key.clone();
            lru.remove(&oldest_time);
            drop(lru);

            let mut cache = self.l1_cache.write().unwrap();
            if let Some(entry) = cache.remove(&key) {
                drop(cache);

                // Try to demote to L2 if it's still valuable
                if entry.access_count > 2 {
                    let _ = self.put_l2(&key, entry.data, entry.ttl);
                }

                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Evict least recently used entry from L2
    fn evict_l2_entry(&self) -> Result<bool, CacheError> {
        let mut lru = self.l2_lru.lock().unwrap();

        if let Some((&oldest_time, key)) = lru.iter().next() {
            let key = key.clone();
            lru.remove(&oldest_time);
            drop(lru);

            let mut cache = self.l2_cache.write().unwrap();
            if cache.remove(&key).is_some() {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Check if an entry is expired
    fn is_expired(&self, entry: &CacheEntry) -> bool {
        if let Some(ttl) = entry.ttl {
            if let Ok(elapsed) = entry.created_at.elapsed() {
                return elapsed > ttl;
            }
        }
        false
    }

    /// Maybe promote entry to L1 if it's hot
    fn maybe_promote_to_l1(&self, key: &str, data: &[u8]) {
        // Only promote if it's small enough and frequently accessed
        if data.len() < 1024 * 1024 { // < 1MB
            if let Ok(patterns) = self.access_patterns.read() {
                if let Some(pattern) = patterns.get(key) {
                    if pattern.sequential_hits > 2 || pattern.random_hits > 3 {
                        let _ = self.put_l1(key, data.to_vec(), None);
                    }
                }
            }
        }
    }

    /// Update access patterns for intelligent caching
    fn update_access_pattern(&self, key: &str) {
        let mut patterns = self.access_patterns.write().unwrap();
        let pattern = patterns.entry(key.to_string()).or_insert_with(AccessPattern::default);

        // Simple sequential detection
        if pattern.last_accessed_keys.len() > 0 {
            let last_key = &pattern.last_accessed_keys[pattern.last_accessed_keys.len() - 1];
            if self.is_sequential_key(last_key, key) {
                pattern.sequential_hits += 1;
            } else {
                pattern.random_hits += 1;
            }
        }

        pattern.last_accessed_keys.push(key.to_string());
        if pattern.last_accessed_keys.len() > 10 {
            pattern.last_accessed_keys.remove(0);
        }
    }

    /// Check if two keys are sequential (for block slots)
    fn is_sequential_key(&self, key1: &str, key2: &str) -> bool {
        // Simple implementation - assumes keys are slot numbers
        if let (Ok(slot1), Ok(slot2)) = (key1.parse::<u64>(), key2.parse::<u64>()) {
            slot2 == slot1 + 1
        } else {
            false
        }
    }

    /// Calculate actual L1 memory usage
    fn calculate_l1_memory_usage(&self) -> usize {
        let cache = self.l1_cache.read().unwrap();
        cache.values().map(|entry| entry.size).sum()
    }

    /// Calculate actual L2 memory usage
    fn calculate_l2_memory_usage(&self) -> usize {
        let cache = self.l2_cache.read().unwrap();
        cache.values().map(|entry| entry.size).sum()
    }

    /// Compress data using zstd
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>, CacheError> {
        zstd::bulk::compress(data, 3)
            .map_err(|e| CacheError::CompressionError(format!("Compression failed: {}", e)))
    }

    /// Decompress data using zstd
    fn decompress_data(&self, data: &[u8]) -> Result<Vec<u8>, CacheError> {
        zstd::bulk::decompress(data, 64 * 1024 * 1024) // 64MB max
            .map_err(|e| CacheError::CompressionError(format!("Decompression failed: {}", e)))
    }

    /// Record cache hit
    fn record_hit(&self, level: CacheLevel) {
        let mut stats = self.stats.write().unwrap();
        match level {
            CacheLevel::L1 => stats.l1_hits += 1,
            CacheLevel::L2 => stats.l2_hits += 1,
            CacheLevel::L3 => stats.l3_hits += 1,
        }
        stats.total_requests += 1;
    }

    /// Record cache miss
    fn record_miss(&self) {
        let mut stats = self.stats.write().unwrap();
        stats.misses += 1;
        stats.total_requests += 1;
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Clear all caches
    pub fn clear(&self) {
        {
            let mut l1 = self.l1_cache.write().unwrap();
            let mut l1_lru = self.l1_lru.lock().unwrap();
            let mut l1_memory = self.l1_memory_used.lock().unwrap();

            l1.clear();
            l1_lru.clear();
            *l1_memory = 0;
        }

        {
            let mut l2 = self.l2_cache.write().unwrap();
            let mut l2_lru = self.l2_lru.lock().unwrap();
            let mut l2_memory = self.l2_memory_used.lock().unwrap();

            l2.clear();
            l2_lru.clear();
            *l2_memory = 0;
        }

        {
            let mut patterns = self.access_patterns.write().unwrap();
            patterns.clear();
        }

        {
            let mut stats = self.stats.write().unwrap();
            *stats = CacheStats::default();
        }
    }

    /// Get memory usage breakdown
    pub fn get_memory_usage(&self) -> MemoryUsage {
        MemoryUsage {
            l1_used: self.calculate_l1_memory_usage(),
            l1_max: self.config.l1_size,
            l2_used: self.calculate_l2_memory_usage(),
            l2_max: self.config.l2_size,
            total_used: self.calculate_l1_memory_usage() + self.calculate_l2_memory_usage(),
            total_max: self.config.l1_size + self.config.l2_size,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub l1_hits: u64,
    pub l2_hits: u64,
    pub l3_hits: u64,
    pub misses: u64,
    pub total_requests: u64,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.l1_hits + self.l2_hits + self.l3_hits) as f64 / self.total_requests as f64
        }
    }
}

/// Memory usage information
#[derive(Debug, Clone)]
pub struct MemoryUsage {
    pub l1_used: usize,
    pub l1_max: usize,
    pub l2_used: usize,
    pub l2_max: usize,
    pub total_used: usize,
    pub total_max: usize,
}

impl MemoryUsage {
    pub fn l1_utilization(&self) -> f64 {
        if self.l1_max == 0 { 0.0 } else { self.l1_used as f64 / self.l1_max as f64 }
    }

    pub fn l2_utilization(&self) -> f64 {
        if self.l2_max == 0 { 0.0 } else { self.l2_used as f64 / self.l2_max as f64 }
    }

    pub fn total_utilization(&self) -> f64 {
        if self.total_max == 0 { 0.0 } else { self.total_used as f64 / self.total_max as f64 }
    }
}

/// Cache errors
#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache capacity exceeded")]
    CapacityExceeded,

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Invalid cache level")]
    InvalidLevel,

    #[error("Lock contention error")]
    LockError,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_multi_level_cache() {
        let config = CacheConfig {
            l1_size: 1024,  // 1KB
            l2_size: 2048,  // 2KB
            ..Default::default()
        };

        let cache = VaultCacheManager::new(config);

        // Test L1 caching
        let small_data = vec![1; 512]; // 512 bytes
        cache.put("small", small_data.clone(), None).unwrap();

        let retrieved = cache.get("small");
        assert_eq!(retrieved, Some(small_data));

        // Test L2 caching with larger data
        let medium_data = vec![2; 1536]; // 1.5KB
        cache.put("medium", medium_data.clone(), None).unwrap();

        let retrieved = cache.get("medium");
        assert_eq!(retrieved, Some(medium_data));

        let stats = cache.get_stats();
        assert!(stats.total_requests > 0);
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            l1_size: 1024,  // 1KB
            l2_size: 1024,  // 1KB
            ..Default::default()
        };

        let cache = VaultCacheManager::new(config);

        // Fill cache beyond capacity
        for i in 0..5 {
            let data = vec![i as u8; 512]; // 512 bytes each
            cache.put(&format!("key_{}", i), data, None).unwrap();
        }

        // Some entries should have been evicted
        let usage = cache.get_memory_usage();
        assert!(usage.total_used <= usage.total_max);
    }
}