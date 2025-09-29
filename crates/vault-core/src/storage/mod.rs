//! # Storage Module
//!
//! This module manages the distributed storage network and data availability.

/// Storage node implementation
pub mod node;

/// Storage network management
pub mod network;

/// Data replication and erasure coding
pub mod replication;

// Re-export key types
pub use node::{StorageNode, StorageError, BlockMetadata, StorageStats};
pub use network::{StorageNetwork, NetworkStats};
pub use replication::{ReplicationStrategy, ErasureCoding};