//! # Network Module
//!
//! This module handles node communication and data transfer.

/// Peer-to-peer networking implementation
pub mod p2p;

// Re-export key types
pub use p2p::{P2PNetwork, PeerInfo, NetworkStats, P2PError};