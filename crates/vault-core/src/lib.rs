//! # Vault Core
//!
//! The core library for SolanaVault, providing abstractions for all major components.

/// Data layer for Solana block data access and caching
pub mod data;

/// Compression layer with versioned compression algorithms
pub mod compression;

/// Storage layer for managing the distributed storage network
pub mod storage;

/// Network layer for P2P communication
pub mod network;

/// Economics layer for staking and incentives
pub mod economics;

/// Workflows for end-to-end operations
pub mod workflows;

/// Memory management with RocksDB and optimized caching
pub mod memory;

/// Dashboard API for TUI and Web interfaces
pub mod dashboard;

/// Re-exports for convenient access to core types
pub use compression::{CompressionStrategy, BlockchainCompressionAdapter};
pub use storage::StorageNode;
pub use data::{SolanaBlockClient, BlockCache};
pub use workflows::CompressionWorkflow;
pub use memory::{VaultStorageEngine, VaultCacheManager, VaultMemoryPool, MemoryMetrics, PerformanceTracker};
pub use dashboard::{NodeDashboardApi, DashboardSnapshot, MetricsHistory};