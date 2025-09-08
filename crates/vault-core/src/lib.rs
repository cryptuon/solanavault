//! # Vault Core
//!
//! The core library for SolanaVault, providing abstractions for all major components.

/// Compression layer with versioned compression algorithms
pub mod compression;

/// Storage layer for managing the distributed storage network
pub mod storage;

/// Re-exports for convenient access to core types
pub use compression::CompressionStrategy;
pub use storage::StorageNode;