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

/// API layer for external interfaces (placeholder)
// pub mod api;

/// Re-exports for convenient access to core types
pub use compression::CompressionStrategy;
pub use storage::StorageNode;
pub use data::{SolanaBlockClient, BlockCache};