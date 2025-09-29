//! # File-based Storage Engine
//!
//! Provides high-performance persistent storage using file-based storage with optimized
//! configurations for blockchain data compression and retrieval.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use std::fs;
use std::io::{self, Read, Write};

/// Configuration for the storage engine
#[derive(Debug, Clone)]
pub struct StorageEngineConfig {
    /// Database path
    pub db_path: String,
    /// Maximum cache size in bytes
    pub max_cache_size: usize,
    /// Enable compression
    pub enable_compression: bool,
    /// Cache TTL in seconds
    pub default_ttl: u64,
    /// Maximum file size before rotation
    pub max_file_size: usize,
    /// Number of background sync threads
    pub sync_threads: usize,
}

impl Default for StorageEngineConfig {
    fn default() -> Self {
        Self {
            db_path: "./vault-storage".to_string(),
            max_cache_size: 256 * 1024 * 1024,  // 256MB
            enable_compression: true,
            default_ttl: 3600, // 1 hour
            max_file_size: 128 * 1024 * 1024,  // 128MB
            sync_threads: 2,
        }
    }
}

/// Storage categories for organization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StorageCategory {
    /// Compressed block data
    Blocks,
    /// Block metadata and indices
    Metadata,
    /// Cache data
    Cache,
    /// Performance statistics
    Stats,
}

impl StorageCategory {
    pub fn name(&self) -> &'static str {
        match self {
            StorageCategory::Blocks => "blocks",
            StorageCategory::Metadata => "metadata",
            StorageCategory::Cache => "cache",
            StorageCategory::Stats => "stats",
        }
    }

    pub fn all() -> Vec<StorageCategory> {
        vec![
            StorageCategory::Blocks,
            StorageCategory::Metadata,
            StorageCategory::Cache,
            StorageCategory::Stats,
        ]
    }
}

/// Serializable data entry
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct StorageEntry {
    /// Entry data
    pub data: Vec<u8>,
    /// Timestamp when stored
    pub timestamp: u64,
    /// Optional TTL in seconds
    pub ttl: Option<u64>,
    /// Compression info
    pub compression_ratio: Option<f32>,
    /// Entry size
    pub size: usize,
}

impl StorageEntry {
    pub fn new(data: Vec<u8>) -> Self {
        let size = data.len();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            data,
            timestamp,
            ttl: None,
            compression_ratio: None,
            size,
        }
    }

    pub fn with_ttl(mut self, ttl: u64) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub fn with_compression_ratio(mut self, ratio: f32) -> Self {
        self.compression_ratio = Some(ratio);
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(ttl) = self.ttl {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            return current_time > self.timestamp + ttl;
        }
        false
    }
}

/// High-performance storage engine using file-based storage
pub struct VaultStorageEngine {
    config: StorageEngineConfig,

    // In-memory indices for fast lookups
    block_index: Arc<RwLock<HashMap<u64, StorageEntry>>>,
    metadata_index: Arc<RwLock<HashMap<String, StorageEntry>>>,
    cache_index: Arc<RwLock<HashMap<String, StorageEntry>>>,
    stats_index: Arc<RwLock<HashMap<String, StorageEntry>>>,

    // File management
    storage_files: Arc<Mutex<HashMap<StorageCategory, PathBuf>>>,

    // Memory management
    memory_usage: Arc<Mutex<usize>>,
}

impl VaultStorageEngine {
    /// Create a new storage engine with the given configuration
    pub fn new(config: StorageEngineConfig) -> Result<Self, StorageError> {
        // Create storage directory
        fs::create_dir_all(&config.db_path)
            .map_err(|e| StorageError::IoError(format!("Failed to create storage directory: {}", e)))?;

        let mut storage_files = HashMap::new();

        // Initialize storage files for each category
        for category in StorageCategory::all() {
            let file_path = Path::new(&config.db_path).join(format!("{}.json", category.name()));
            storage_files.insert(category, file_path);
        }

        let engine = Self {
            config,
            block_index: Arc::new(RwLock::new(HashMap::new())),
            metadata_index: Arc::new(RwLock::new(HashMap::new())),
            cache_index: Arc::new(RwLock::new(HashMap::new())),
            stats_index: Arc::new(RwLock::new(HashMap::new())),
            storage_files: Arc::new(Mutex::new(storage_files)),
            memory_usage: Arc::new(Mutex::new(0)),
        };

        // Load existing data
        engine.load_existing_data()?;

        Ok(engine)
    }

    /// Store a compressed block
    pub fn store_block(&self, slot: u64, compressed_data: &[u8]) -> Result<(), StorageError> {
        let entry = StorageEntry::new(compressed_data.to_vec());

        {
            let mut index = self.block_index.write().unwrap();
            index.insert(slot, entry.clone());
        }

        self.update_memory_usage(entry.size as i64);
        self.maybe_sync_to_disk(StorageCategory::Blocks)?;

        Ok(())
    }

    /// Retrieve a compressed block
    pub fn get_block(&self, slot: u64) -> Result<Option<Vec<u8>>, StorageError> {
        let index = self.block_index.read().unwrap();

        if let Some(entry) = index.get(&slot) {
            if entry.is_expired() {
                drop(index);
                self.remove_block(slot)?;
                return Ok(None);
            }

            Ok(Some(entry.data.clone()))
        } else {
            Ok(None)
        }
    }

    /// Remove a block
    pub fn remove_block(&self, slot: u64) -> Result<(), StorageError> {
        let mut index = self.block_index.write().unwrap();

        if let Some(entry) = index.remove(&slot) {
            self.update_memory_usage(-(entry.size as i64));
        }

        Ok(())
    }

    /// Store metadata
    pub fn store_metadata<T>(&self, key: &str, metadata: &T) -> Result<(), StorageError>
    where
        T: Serialize,
    {
        let serialized = serde_json::to_vec(metadata)
            .map_err(|e| StorageError::SerializationError(format!("Metadata serialization failed: {}", e)))?;

        let entry = StorageEntry::new(serialized);

        {
            let mut index = self.metadata_index.write().unwrap();
            index.insert(key.to_string(), entry.clone());
        }

        self.update_memory_usage(entry.size as i64);
        self.maybe_sync_to_disk(StorageCategory::Metadata)?;

        Ok(())
    }

    /// Retrieve metadata
    pub fn get_metadata<T>(&self, key: &str) -> Result<Option<T>, StorageError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let index = self.metadata_index.read().unwrap();

        if let Some(entry) = index.get(key) {
            if entry.is_expired() {
                drop(index);
                self.remove_metadata(key)?;
                return Ok(None);
            }

            let metadata = serde_json::from_slice(&entry.data)
                .map_err(|e| StorageError::DeserializationError(format!("Metadata deserialization failed: {}", e)))?;

            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    /// Remove metadata
    pub fn remove_metadata(&self, key: &str) -> Result<(), StorageError> {
        let mut index = self.metadata_index.write().unwrap();

        if let Some(entry) = index.remove(key) {
            self.update_memory_usage(-(entry.size as i64));
        }

        Ok(())
    }

    /// Store data in cache
    pub fn cache_put(&self, key: &str, data: &[u8], ttl: Option<u64>) -> Result<(), StorageError> {
        let mut entry = StorageEntry::new(data.to_vec());
        if let Some(ttl) = ttl {
            entry = entry.with_ttl(ttl);
        }

        {
            let mut index = self.cache_index.write().unwrap();
            index.insert(key.to_string(), entry.clone());
        }

        self.update_memory_usage(entry.size as i64);
        self.check_cache_limits()?;

        Ok(())
    }

    /// Get data from cache
    pub fn cache_get(&self, key: &str) -> Result<Option<Vec<u8>>, StorageError> {
        let index = self.cache_index.read().unwrap();

        if let Some(entry) = index.get(key) {
            if entry.is_expired() {
                drop(index);
                self.cache_remove(key)?;
                return Ok(None);
            }

            Ok(Some(entry.data.clone()))
        } else {
            Ok(None)
        }
    }

    /// Remove data from cache
    pub fn cache_remove(&self, key: &str) -> Result<(), StorageError> {
        let mut index = self.cache_index.write().unwrap();

        if let Some(entry) = index.remove(key) {
            self.update_memory_usage(-(entry.size as i64));
        }

        Ok(())
    }

    /// Get database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats, StorageError> {
        let block_count = self.block_index.read().unwrap().len();
        let metadata_count = self.metadata_index.read().unwrap().len();
        let cache_count = self.cache_index.read().unwrap().len();
        let stats_count = self.stats_index.read().unwrap().len();

        let memory_usage = *self.memory_usage.lock().unwrap();

        Ok(DatabaseStats {
            blocks_count: block_count as u64,
            metadata_count: metadata_count as u64,
            cache_count: cache_count as u64,
            stats_count: stats_count as u64,
            total_memory_usage: memory_usage as u64,
            disk_usage: self.calculate_disk_usage()?,
        })
    }

    /// Compact the database by removing expired entries
    pub fn compact(&self) -> Result<u64, StorageError> {
        let mut removed_count = 0u64;

        // Clean up expired cache entries
        {
            let mut cache_index = self.cache_index.write().unwrap();
            let mut expired_keys = Vec::new();

            for (key, entry) in cache_index.iter() {
                if entry.is_expired() {
                    expired_keys.push(key.clone());
                }
            }

            for key in expired_keys {
                if let Some(entry) = cache_index.remove(&key) {
                    self.update_memory_usage(-(entry.size as i64));
                    removed_count += 1;
                }
            }
        }

        // Sync to disk after cleanup
        self.sync_all_to_disk()?;

        Ok(removed_count)
    }

    /// List all blocks in a range
    pub fn list_blocks(&self, start_slot: Option<u64>, end_slot: Option<u64>) -> Result<Vec<u64>, StorageError> {
        let index = self.block_index.read().unwrap();
        let mut blocks: Vec<u64> = index.keys().cloned().collect();
        blocks.sort_unstable();

        if let Some(start) = start_slot {
            blocks.retain(|&slot| slot >= start);
        }

        if let Some(end) = end_slot {
            blocks.retain(|&slot| slot <= end);
        }

        Ok(blocks)
    }

    /// Update memory usage tracking
    fn update_memory_usage(&self, delta: i64) {
        let mut usage = self.memory_usage.lock().unwrap();
        if delta < 0 {
            *usage = usage.saturating_sub((-delta) as usize);
        } else {
            *usage += delta as usize;
        }
    }

    /// Check cache limits and evict if necessary
    fn check_cache_limits(&self) -> Result<(), StorageError> {
        let current_usage = *self.memory_usage.lock().unwrap();

        if current_usage > self.config.max_cache_size {
            // Simple LRU eviction - remove oldest entries
            let mut cache_index = self.cache_index.write().unwrap();
            let mut entries_with_time: Vec<_> = cache_index.iter()
                .map(|(key, entry)| (key.clone(), entry.timestamp))
                .collect();

            entries_with_time.sort_by_key(|&(_, timestamp)| timestamp);

            // Remove oldest 25% of entries
            let to_remove = entries_with_time.len() / 4;
            for (key, _) in entries_with_time.into_iter().take(to_remove) {
                if let Some(entry) = cache_index.remove(&key) {
                    self.update_memory_usage(-(entry.size as i64));
                }
            }
        }

        Ok(())
    }

    /// Maybe sync to disk based on memory pressure
    fn maybe_sync_to_disk(&self, category: StorageCategory) -> Result<(), StorageError> {
        let current_usage = *self.memory_usage.lock().unwrap();

        // Sync if we're using more than 80% of cache
        if current_usage > (self.config.max_cache_size * 4) / 5 {
            self.sync_category_to_disk(category)?;
        }

        Ok(())
    }

    /// Sync specific category to disk
    fn sync_category_to_disk(&self, category: StorageCategory) -> Result<(), StorageError> {
        let storage_files = self.storage_files.lock().unwrap();
        let file_path = storage_files.get(&category)
            .ok_or_else(|| StorageError::ConfigurationError("Storage file not found".to_string()))?;

        let data = match category {
            StorageCategory::Blocks => {
                let index = self.block_index.read().unwrap();
                serde_json::to_string(&*index)
                    .map_err(|e| StorageError::SerializationError(format!("Block index serialization failed: {}", e)))?
            }
            StorageCategory::Metadata => {
                let index = self.metadata_index.read().unwrap();
                serde_json::to_string(&*index)
                    .map_err(|e| StorageError::SerializationError(format!("Metadata index serialization failed: {}", e)))?
            }
            StorageCategory::Cache => {
                let index = self.cache_index.read().unwrap();
                serde_json::to_string(&*index)
                    .map_err(|e| StorageError::SerializationError(format!("Cache index serialization failed: {}", e)))?
            }
            StorageCategory::Stats => {
                let index = self.stats_index.read().unwrap();
                serde_json::to_string(&*index)
                    .map_err(|e| StorageError::SerializationError(format!("Stats index serialization failed: {}", e)))?
            }
        };

        // Write to file
        fs::write(file_path, data)
            .map_err(|e| StorageError::IoError(format!("Failed to write file: {}", e)))?;

        Ok(())
    }

    /// Sync all categories to disk
    fn sync_all_to_disk(&self) -> Result<(), StorageError> {
        for category in StorageCategory::all() {
            self.sync_category_to_disk(category)?;
        }
        Ok(())
    }

    /// Load existing data from disk
    fn load_existing_data(&self) -> Result<(), StorageError> {
        for category in StorageCategory::all() {
            if let Err(e) = self.load_category_from_disk(category) {
                // Log error but don't fail - file might not exist yet
                log::warn!("Failed to load category {:?}: {}", category, e);
            }
        }
        Ok(())
    }

    /// Load specific category from disk
    fn load_category_from_disk(&self, category: StorageCategory) -> Result<(), StorageError> {
        let storage_files = self.storage_files.lock().unwrap();
        let file_path = storage_files.get(&category)
            .ok_or_else(|| StorageError::ConfigurationError("Storage file not found".to_string()))?;

        if !file_path.exists() {
            return Ok(()); // File doesn't exist yet
        }

        let data = fs::read_to_string(file_path)
            .map_err(|e| StorageError::IoError(format!("Failed to read file: {}", e)))?;

        if data.is_empty() {
            return Ok(());
        }

        // Load the index
        match category {
            StorageCategory::Blocks => {
                let loaded_index: HashMap<u64, StorageEntry> = serde_json::from_str(&data)
                    .map_err(|e| StorageError::DeserializationError(format!("Block index deserialization failed: {}", e)))?;
                let mut index = self.block_index.write().unwrap();
                *index = loaded_index;
            }
            StorageCategory::Metadata => {
                let loaded_index: HashMap<String, StorageEntry> = serde_json::from_str(&data)
                    .map_err(|e| StorageError::DeserializationError(format!("Metadata index deserialization failed: {}", e)))?;
                let mut index = self.metadata_index.write().unwrap();
                *index = loaded_index;
            }
            StorageCategory::Cache => {
                let loaded_index: HashMap<String, StorageEntry> = serde_json::from_str(&data)
                    .map_err(|e| StorageError::DeserializationError(format!("Cache index deserialization failed: {}", e)))?;
                let mut index = self.cache_index.write().unwrap();
                *index = loaded_index;
            }
            StorageCategory::Stats => {
                let loaded_index: HashMap<String, StorageEntry> = serde_json::from_str(&data)
                    .map_err(|e| StorageError::DeserializationError(format!("Stats index deserialization failed: {}", e)))?;
                let mut index = self.stats_index.write().unwrap();
                *index = loaded_index;
            }
        }

        Ok(())
    }

    /// Calculate disk usage
    fn calculate_disk_usage(&self) -> Result<u64, StorageError> {
        let storage_files = self.storage_files.lock().unwrap();
        let mut total_size = 0u64;

        for file_path in storage_files.values() {
            if file_path.exists() {
                let metadata = fs::metadata(file_path)
                    .map_err(|e| StorageError::IoError(format!("Failed to get file metadata: {}", e)))?;
                total_size += metadata.len();
            }
        }

        Ok(total_size)
    }

    /// Shutdown the storage engine
    pub fn shutdown(&self) -> Result<(), StorageError> {
        // Sync all data to disk before shutdown
        self.sync_all_to_disk()?;
        Ok(())
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub blocks_count: u64,
    pub metadata_count: u64,
    pub cache_count: u64,
    pub stats_count: u64,
    pub total_memory_usage: u64,
    pub disk_usage: u64,
}

impl DatabaseStats {
    pub fn total_entries(&self) -> u64 {
        self.blocks_count + self.metadata_count + self.cache_count + self.stats_count
    }
}

/// Storage engine errors
#[derive(Debug, Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Configuration error: {0}")]
    ConfigurationError(String),

    #[error("Storage full")]
    StorageFull,

    #[error("Entry not found")]
    EntryNotFound,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage_engine_basic_operations() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageEngineConfig {
            db_path: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };

        let engine = VaultStorageEngine::new(config).unwrap();

        // Test block storage and retrieval
        let slot = 123456;
        let data = b"test block data";

        engine.store_block(slot, data).unwrap();
        let retrieved = engine.get_block(slot).unwrap();

        assert_eq!(retrieved, Some(data.to_vec()));

        // Test metadata storage
        #[derive(Serialize, Deserialize, PartialEq, Debug)]
        struct TestMetadata {
            compression_ratio: f64,
            original_size: usize,
        }

        let metadata = TestMetadata {
            compression_ratio: 15.5,
            original_size: 1024,
        };

        engine.store_metadata("test_key", &metadata).unwrap();
        let retrieved_metadata: Option<TestMetadata> = engine.get_metadata("test_key").unwrap();

        assert_eq!(retrieved_metadata, Some(metadata));
    }

    #[test]
    fn test_cache_with_ttl() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageEngineConfig {
            db_path: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };

        let engine = VaultStorageEngine::new(config).unwrap();

        // Store with short TTL
        let data = b"cached data";
        engine.cache_put("test_cache", data, Some(1)).unwrap(); // 1 second TTL

        // Should be available immediately
        let retrieved = engine.cache_get("test_cache").unwrap();
        assert_eq!(retrieved, Some(data.to_vec()));

        // Wait for expiration
        std::thread::sleep(std::time::Duration::from_secs(2));

        // Should be expired
        let expired = engine.cache_get("test_cache").unwrap();
        assert_eq!(expired, None);
    }

    #[test]
    fn test_storage_persistence() {
        let temp_dir = TempDir::new().unwrap();
        let config = StorageEngineConfig {
            db_path: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };

        // Store some data
        {
            let engine = VaultStorageEngine::new(config.clone()).unwrap();
            engine.store_block(12345, b"persistent data").unwrap();
            engine.shutdown().unwrap();
        }

        // Create new engine instance and verify data persists
        {
            let engine = VaultStorageEngine::new(config).unwrap();
            let retrieved = engine.get_block(12345).unwrap();
            assert_eq!(retrieved, Some(b"persistent data".to_vec()));
        }
    }
}