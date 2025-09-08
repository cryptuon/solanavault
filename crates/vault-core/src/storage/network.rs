//! # Storage Network
//!
//! Management of the distributed storage network.

use super::node::{StorageNode, StorageError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the storage network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageNetwork {
    /// Collection of storage nodes
    nodes: HashMap<String, StorageNode>,
}

impl StorageNetwork {
    /// Create a new storage network
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }
    
    /// Add a storage node to the network
    pub fn add_node(&mut self, node: StorageNode) {
        self.nodes.insert(node.id.clone(), node);
    }
    
    /// Get a node by ID
    pub fn get_node(&self, id: &str) -> Option<&StorageNode> {
        self.nodes.get(id)
    }
    
    /// Get a mutable reference to a node by ID
    pub fn get_node_mut(&mut self, id: &str) -> Option<&mut StorageNode> {
        self.nodes.get_mut(id)
    }
    
    /// Store data across the network with replication
    pub fn store_data(&mut self, data_size: u64, replication_factor: usize) -> Result<Vec<String>, StorageError> {
        let mut stored_nodes = Vec::new();
        
        // Collect node IDs and sort by available capacity
        let mut nodes_by_capacity: Vec<(String, u64)> = self.nodes
            .iter()
            .map(|(id, node)| (id.clone(), node.available_capacity()))
            .collect();
        
        nodes_by_capacity.sort_by(|a, b| b.1.cmp(&a.1));
        
        // Store data on nodes with most available capacity
        for (node_id, _) in nodes_by_capacity.iter().take(replication_factor) {
            if let Some(node) = self.get_node_mut(node_id) {
                node.store_data(data_size)?;
                stored_nodes.push(node_id.clone());
            }
        }
        
        if stored_nodes.len() < replication_factor {
            return Err(StorageError::InsufficientCapacity);
        }
        
        Ok(stored_nodes)
    }
    
    /// Get network statistics
    pub fn stats(&self) -> NetworkStats {
        let total_nodes = self.nodes.len();
        let total_capacity: u64 = self.nodes.values().map(|n| n.capacity).sum();
        let used_capacity: u64 = self.nodes.values().map(|n| n.used).sum();
        
        NetworkStats {
            total_nodes,
            total_capacity,
            used_capacity,
            available_capacity: total_capacity - used_capacity,
        }
    }
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_nodes: usize,
    pub total_capacity: u64,
    pub used_capacity: u64,
    pub available_capacity: u64,
}

impl Default for StorageNetwork {
    fn default() -> Self {
        Self::new()
    }
}