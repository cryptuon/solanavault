//! # Memory Management Module
//!
//! Provides efficient memory management using RocksDB for persistent storage
//! and optimized caching strategies for the SolanaVault network.

/// RocksDB-based storage engine for persistent data
pub mod storage_engine;

/// Advanced caching system with LRU and compression-aware policies
pub mod cache_manager;

/// Memory pool management for large block processing
pub mod memory_pool;

/// Performance monitoring and memory metrics
pub mod metrics;

// Re-export key components
pub use storage_engine::{VaultStorageEngine, StorageEngineConfig, StorageError};
pub use cache_manager::{VaultCacheManager, CacheConfig, CachePolicy};
pub use memory_pool::{VaultMemoryPool, MemoryPoolConfig};
pub use metrics::{MemoryMetrics, PerformanceTracker};