//! # Data Replication
//!
//! Implementation of data replication and erasure coding.

/// Replication strategy for data storage
#[derive(Debug, Clone)]
pub struct ReplicationStrategy {
    /// Number of copies to store
    pub replication_factor: usize,
    
    /// Minimum number of copies needed for retrieval
    pub min_retrieval_copies: usize,
}

impl ReplicationStrategy {
    /// Create a new replication strategy
    pub fn new(replication_factor: usize, min_retrieval_copies: usize) -> Self {
        Self {
            replication_factor,
            min_retrieval_copies,
        }
    }
    
    /// Default strategy: 3 copies, 2 needed for retrieval
    pub fn default() -> Self {
        Self {
            replication_factor: 3,
            min_retrieval_copies: 2,
        }
    }
}

/// Erasure coding configuration
#[derive(Debug, Clone)]
pub struct ErasureCoding {
    /// Number of data shards
    pub data_shards: usize,
    
    /// Number of parity shards
    pub parity_shards: usize,
}

impl ErasureCoding {
    /// Create a new erasure coding configuration
    pub fn new(data_shards: usize, parity_shards: usize) -> Self {
        Self {
            data_shards,
            parity_shards,
        }
    }
    
    /// Default configuration: 2 data shards, 1 parity shard
    pub fn default() -> Self {
        Self {
            data_shards: 2,
            parity_shards: 1,
        }
    }
    
    /// Total number of shards
    pub fn total_shards(&self) -> usize {
        self.data_shards + self.parity_shards
    }
}