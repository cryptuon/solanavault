//! # Network Module
//!
//! This module handles node communication and data transfer.

/// Peer-to-peer networking implementation
pub mod p2p;

/// Message protocols for node communication
pub mod protocol;

/// Network discovery and bootstrap
pub mod discovery;