//! # Dashboard API
//!
//! Unified interface for TUI and Web Dashboard to access node metrics.

use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::storage::{StorageNode, StorageStats};
use crate::economics::{StakingContract, NodeStake};

use super::metrics::*;
use super::history::MetricsHistory;

/// Trait for providing network statistics
#[async_trait]
pub trait NetworkStatsProvider: Send + Sync {
    /// Get current network statistics
    async fn get_network_stats(&self) -> NetworkMetrics;
}

/// Trait for providing gateway statistics
#[async_trait]
pub trait GatewayStatsProvider: Send + Sync {
    /// Get current gateway statistics
    async fn get_gateway_stats(&self) -> GatewayMetricsSummary;
}

/// Trait for providing consensus statistics
#[async_trait]
pub trait ConsensusStatsProvider: Send + Sync {
    /// Get current consensus statistics
    async fn get_consensus_stats(&self) -> ConsensusMetrics;
}

/// NodeDashboardApi - Unified interface for TUI and Web Dashboard
pub struct NodeDashboardApi {
    /// Node identifier
    node_id: String,
    /// Node address
    address: String,
    /// Startup time for uptime calculation
    start_time: Instant,
    /// Reference to storage node
    storage_node: Arc<RwLock<StorageNode>>,
    /// Network stats provider
    network_provider: Option<Arc<dyn NetworkStatsProvider>>,
    /// Staking contract (optional)
    staking_contract: Option<Arc<RwLock<StakingContract>>>,
    /// Gateway stats provider (optional)
    gateway_provider: Option<Arc<dyn GatewayStatsProvider>>,
    /// Consensus stats provider (optional)
    consensus_provider: Option<Arc<dyn ConsensusStatsProvider>>,
    /// Metrics history for sparklines
    history: Arc<RwLock<MetricsHistory>>,
    /// Current node status
    status: Arc<RwLock<NodeStatus>>,
}

impl NodeDashboardApi {
    /// Create a new dashboard API with required components
    pub fn new(
        node_id: String,
        address: String,
        storage_node: Arc<RwLock<StorageNode>>,
    ) -> Self {
        Self {
            node_id,
            address,
            start_time: Instant::now(),
            storage_node,
            network_provider: None,
            staking_contract: None,
            gateway_provider: None,
            consensus_provider: None,
            history: Arc::new(RwLock::new(MetricsHistory::new())),
            status: Arc::new(RwLock::new(NodeStatus::Starting)),
        }
    }

    /// Add network stats provider
    pub fn with_network_provider(mut self, provider: Arc<dyn NetworkStatsProvider>) -> Self {
        self.network_provider = Some(provider);
        self
    }

    /// Add staking contract for economics metrics
    pub fn with_staking(mut self, staking: Arc<RwLock<StakingContract>>) -> Self {
        self.staking_contract = Some(staking);
        self
    }

    /// Add gateway stats provider
    pub fn with_gateway_provider(mut self, provider: Arc<dyn GatewayStatsProvider>) -> Self {
        self.gateway_provider = Some(provider);
        self
    }

    /// Add consensus stats provider
    pub fn with_consensus_provider(mut self, provider: Arc<dyn ConsensusStatsProvider>) -> Self {
        self.consensus_provider = Some(provider);
        self
    }

    /// Set the node status
    pub async fn set_status(&self, status: NodeStatus) {
        let mut s = self.status.write().await;
        *s = status;
    }

    /// Get current snapshot of all metrics
    pub async fn get_snapshot(&self) -> DashboardSnapshot {
        let storage = self.get_storage_metrics().await;
        let network = self.get_network_metrics().await;
        let economics = self.get_economics_metrics().await;
        let consensus = self.get_consensus_metrics().await;
        let status = self.status.read().await.clone();

        let snapshot = DashboardSnapshot {
            timestamp: current_timestamp(),
            node_info: NodeInfo {
                node_id: self.node_id.clone(),
                address: self.address.clone(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                uptime_seconds: self.start_time.elapsed().as_secs(),
                status,
            },
            storage,
            network,
            economics,
            consensus,
        };

        // Update history
        let mut history = self.history.write().await;
        history.push(&snapshot);

        snapshot
    }

    /// Get metrics history for charts
    pub async fn get_history(&self) -> MetricsHistory {
        self.history.read().await.clone()
    }

    /// Get storage metrics
    async fn get_storage_metrics(&self) -> StorageMetrics {
        let storage = self.storage_node.read().await;
        let stats: StorageStats = storage.get_storage_stats();
        let workflow_metrics = storage.compression_workflow.get_metrics();

        let cache_total = workflow_metrics.cache_hits + workflow_metrics.cache_misses;
        let cache_hit_rate = if cache_total > 0 {
            workflow_metrics.cache_hits as f64 / cache_total as f64
        } else {
            0.0
        };

        StorageMetrics {
            total_capacity: stats.total_capacity,
            used_capacity: stats.used_capacity,
            available_capacity: stats.available_capacity,
            blocks_stored: stats.blocks_stored,
            compression_ratio: stats.total_compression_ratio,
            total_original_bytes: stats.total_original_bytes,
            total_compressed_bytes: stats.total_compressed_bytes,
            cache_hits: workflow_metrics.cache_hits,
            cache_misses: workflow_metrics.cache_misses,
            cache_hit_rate,
        }
    }

    /// Get network metrics
    async fn get_network_metrics(&self) -> NetworkMetrics {
        if let Some(ref provider) = self.network_provider {
            provider.get_network_stats().await
        } else {
            NetworkMetrics::default()
        }
    }

    /// Get economics metrics
    async fn get_economics_metrics(&self) -> EconomicsMetrics {
        let staking = if let Some(ref contract) = self.staking_contract {
            let c = contract.read().await;
            let stats = c.get_staking_stats();
            let own_stake = c.get_node_stake(&self.node_id);

            StakingMetricsSummary {
                total_staked: stats.total_staked,
                own_stake: own_stake.map(|s| s.staked_amount).unwrap_or(0),
                pending_rewards: own_stake.map(|s| s.pending_rewards).unwrap_or(0),
                performance_score: own_stake.map(|s| s.performance_score).unwrap_or(1.0),
                base_apy: stats.base_apy,
            }
        } else {
            StakingMetricsSummary::default()
        };

        let gateway = if let Some(ref provider) = self.gateway_provider {
            Some(provider.get_gateway_stats().await)
        } else {
            None
        };

        EconomicsMetrics {
            staking,
            rewards: RewardMetricsSummary::default(), // TODO: Add reward distribution provider
            gateway,
        }
    }

    /// Get consensus metrics
    async fn get_consensus_metrics(&self) -> ConsensusMetrics {
        if let Some(ref provider) = self.consensus_provider {
            provider.get_consensus_stats().await
        } else {
            ConsensusMetrics {
                reputation_score: 1.0,
                ..Default::default()
            }
        }
    }
}

/// Get current Unix timestamp
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Simple network stats provider for standalone node
pub struct SimpleNetworkStatsProvider {
    connected_peers: Arc<RwLock<usize>>,
    messages_sent: Arc<RwLock<u64>>,
    messages_received: Arc<RwLock<u64>>,
}

impl SimpleNetworkStatsProvider {
    /// Create a new simple network stats provider
    pub fn new() -> Self {
        Self {
            connected_peers: Arc::new(RwLock::new(0)),
            messages_sent: Arc::new(RwLock::new(0)),
            messages_received: Arc::new(RwLock::new(0)),
        }
    }

    /// Update peer count
    pub async fn set_peer_count(&self, count: usize) {
        let mut peers = self.connected_peers.write().await;
        *peers = count;
    }

    /// Increment messages sent
    pub async fn increment_sent(&self) {
        let mut sent = self.messages_sent.write().await;
        *sent += 1;
    }

    /// Increment messages received
    pub async fn increment_received(&self) {
        let mut recv = self.messages_received.write().await;
        *recv += 1;
    }
}

impl Default for SimpleNetworkStatsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NetworkStatsProvider for SimpleNetworkStatsProvider {
    async fn get_network_stats(&self) -> NetworkMetrics {
        NetworkMetrics {
            total_peers: 0,
            connected_peers: *self.connected_peers.read().await,
            messages_sent: *self.messages_sent.read().await,
            messages_received: *self.messages_received.read().await,
            bandwidth_in_bytes: 0,
            bandwidth_out_bytes: 0,
            average_latency_ms: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_dashboard_api() {
        let storage = Arc::new(RwLock::new(StorageNode::new(
            "test-node".to_string(),
            "127.0.0.1:8080".to_string(),
            1_000_000_000,
        )));

        let api = NodeDashboardApi::new(
            "test-node".to_string(),
            "127.0.0.1:8080".to_string(),
            storage,
        );

        let snapshot = api.get_snapshot().await;

        assert_eq!(snapshot.node_info.node_id, "test-node");
        assert_eq!(snapshot.storage.total_capacity, 1_000_000_000);
        assert!(snapshot.node_info.uptime_seconds < 1);
    }
}
