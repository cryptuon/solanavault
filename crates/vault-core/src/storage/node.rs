//! # Storage Node
//!
//! Implementation of a storage node in the Vault network.

use serde::{Deserialize, Serialize};

/// Represents a storage node in the Vault network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNode {
    /// Unique identifier for the node
    pub id: String,
    
    /// Node's network address
    pub address: String,
    
    /// Available storage capacity in bytes
    pub capacity: u64,
    
    /// Used storage in bytes
    pub used: u64,
    
    /// Node's reputation score
    pub reputation: f64,
}

impl StorageNode {
    /// Create a new storage node
    pub fn new(id: String, address: String, capacity: u64) -> Self {
        Self {
            id,
            address,
            capacity,
            used: 0,
            reputation: 1.0,
        }
    }
    
    /// Check if the node has enough space for data of given size
    pub fn has_capacity(&self, size: u64) -> bool {
        self.used + size <= self.capacity
    }
    
    /// Store data on this node
    pub fn store_data(&mut self, size: u64) -> Result<(), StorageError> {
        if !self.has_capacity(size) {
            return Err(StorageError::InsufficientCapacity);
        }
        
        self.used += size;
        Ok(())
    }
    
    /// Get available capacity
    pub fn available_capacity(&self) -> u64 {
        self.capacity - self.used
    }
}

/// Error types for storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Insufficient capacity on storage node")]
    InsufficientCapacity,
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}