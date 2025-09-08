//! # Vault Core
//!
//! The core library for SolanaVault, providing abstractions for all major components.

/// Data layer for accessing and storing raw Solana block data
pub mod data;

/// Compression layer with versioned compression algorithms
pub mod compression;

/// Storage layer for managing the distributed storage network
pub mod storage;

/// Network layer for node communication and data transfer
pub mod network;

/// Economics layer for staking, rewards, and slashing mechanisms
pub mod economics;

/// API layer providing interfaces for developers and applications
pub mod api;

/// Re-exports for convenient access to core types
pub use compression::CompressionStrategy;
pub use storage::StorageNode;
pub use network::NetworkManager;
pub use economics::EconomicsEngine;
pub use api::VaultAPI;