//! # Block Cache
//!
//! Local filesystem cache for Solana block data to avoid repeated RPC calls.

use super::types::{CachedBlock, DataError};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Local filesystem cache for block data
pub struct BlockCache {
    cache_dir: PathBuf,
    cache_ttl: u64, // Time-to-live in seconds
    memory_cache: HashMap<u64, CachedBlock>,
    max_memory_entries: usize,
}

impl BlockCache {
    /// Create a new block cache
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self, DataError> {
        let cache_dir = cache_dir.as_ref().to_path_buf();

        // Create cache directory if it doesn't exist
        if !cache_dir.exists() {
            fs::create_dir_all(&cache_dir)
                .map_err(|e| DataError::Cache(format!("Failed to create cache dir: {}", e)))?;
        }

        Ok(Self {
            cache_dir,
            cache_ttl: 24 * 60 * 60, // 24 hours default TTL
            memory_cache: HashMap::new(),
            max_memory_entries: 100, // Keep last 100 blocks in memory
        })
    }

    /// Get a block from cache (memory first, then disk)
    pub fn get(&mut self, slot: u64) -> Option<CachedBlock> {
        // Check memory cache first
        if let Some(cached_block) = self.memory_cache.get(&slot) {
            if self.is_cache_valid(cached_block.cached_at) {
                log::debug!("Block {} found in memory cache", slot);
                return Some(cached_block.clone());
            } else {
                // Remove expired entry
                self.memory_cache.remove(&slot);
            }
        }

        // Check disk cache
        if let Ok(cached_block) = self.load_from_disk(slot) {
            if self.is_cache_valid(cached_block.cached_at) {
                log::debug!("Block {} found in disk cache", slot);
                // Add to memory cache
                self.add_to_memory_cache(cached_block.clone());
                return Some(cached_block);
            } else {
                // Remove expired file
                let _ = self.remove_from_disk(slot);
            }
        }

        None
    }

    /// Store a block in cache (both memory and disk)
    pub fn put(&mut self, cached_block: CachedBlock) -> Result<(), DataError> {
        let slot = cached_block.slot;

        // Save to disk
        self.save_to_disk(&cached_block)?;

        // Add to memory cache
        self.add_to_memory_cache(cached_block);

        log::debug!("Cached block {} to memory and disk", slot);
        Ok(())
    }

    /// Check if a block is cached and valid
    pub fn contains(&self, slot: u64) -> bool {
        // Check memory first
        if let Some(cached_block) = self.memory_cache.get(&slot) {
            return self.is_cache_valid(cached_block.cached_at);
        }

        // Check disk
        if let Ok(cached_block) = self.load_from_disk(slot) {
            return self.is_cache_valid(cached_block.cached_at);
        }

        false
    }

    /// Clear expired entries from cache
    pub fn cleanup(&mut self) -> Result<(), DataError> {
        let _current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Clean memory cache
        let cache_ttl = self.cache_ttl;
        self.memory_cache.retain(|_, cached_block| {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            current_time - cached_block.cached_at < cache_ttl
        });

        // Clean disk cache
        let entries = fs::read_dir(&self.cache_dir)
            .map_err(|e| DataError::Cache(format!("Failed to read cache dir: {}", e)))?;

        for entry in entries {
            let entry = entry
                .map_err(|e| DataError::Cache(format!("Failed to read dir entry: {}", e)))?;

            if let Some(file_name) = entry.file_name().to_str() {
                if let Some(slot_str) = file_name.strip_suffix(".json") {
                    if let Ok(slot) = slot_str.parse::<u64>() {
                        if let Ok(cached_block) = self.load_from_disk(slot) {
                            if !self.is_cache_valid(cached_block.cached_at) {
                                let _ = fs::remove_file(entry.path());
                                log::debug!("Removed expired cache file for slot {}", slot);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let disk_entries = self.count_disk_entries();
        let memory_size = self.memory_cache.len();

        CacheStats {
            memory_entries: memory_size,
            disk_entries,
            cache_dir: self.cache_dir.clone(),
            ttl_seconds: self.cache_ttl,
        }
    }

    /// Set cache TTL
    pub fn set_ttl(&mut self, ttl_seconds: u64) {
        self.cache_ttl = ttl_seconds;
    }

    // Private helper methods

    fn is_cache_valid(&self, cached_at: u64) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        current_time - cached_at < self.cache_ttl
    }

    fn cache_file_path(&self, slot: u64) -> PathBuf {
        self.cache_dir.join(format!("{}.json", slot))
    }

    fn save_to_disk(&self, cached_block: &CachedBlock) -> Result<(), DataError> {
        let file_path = self.cache_file_path(cached_block.slot);
        let json_data = serde_json::to_string_pretty(cached_block)?;
        fs::write(file_path, json_data)?;
        Ok(())
    }

    fn load_from_disk(&self, slot: u64) -> Result<CachedBlock, DataError> {
        let file_path = self.cache_file_path(slot);
        if !file_path.exists() {
            return Err(DataError::Cache(format!("Cache file not found for slot {}", slot)));
        }

        let json_data = fs::read_to_string(file_path)?;
        let cached_block: CachedBlock = serde_json::from_str(&json_data)?;
        Ok(cached_block)
    }

    fn remove_from_disk(&self, slot: u64) -> Result<(), DataError> {
        let file_path = self.cache_file_path(slot);
        if file_path.exists() {
            fs::remove_file(file_path)?;
        }
        Ok(())
    }

    fn add_to_memory_cache(&mut self, cached_block: CachedBlock) {
        let slot = cached_block.slot;

        // Remove oldest entries if cache is full
        if self.memory_cache.len() >= self.max_memory_entries {
            let oldest_slot = self.memory_cache
                .iter()
                .min_by_key(|(_, block)| block.cached_at)
                .map(|(slot, _)| *slot);

            if let Some(oldest_slot) = oldest_slot {
                self.memory_cache.remove(&oldest_slot);
            }
        }

        self.memory_cache.insert(slot, cached_block);
    }

    fn count_disk_entries(&self) -> usize {
        fs::read_dir(&self.cache_dir)
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry.ok().and_then(|e| {
                            e.file_name()
                                .to_str()
                                .and_then(|name| name.strip_suffix(".json"))
                                .and_then(|slot_str| slot_str.parse::<u64>().ok())
                        })
                    })
                    .count()
            })
            .unwrap_or(0)
    }
}

/// Cache statistics
#[derive(Debug)]
pub struct CacheStats {
    pub memory_entries: usize,
    pub disk_entries: usize,
    pub cache_dir: PathBuf,
    pub ttl_seconds: u64,
}