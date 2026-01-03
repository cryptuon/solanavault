//! # Dashboard Metrics
//!
//! Data structures for dashboard metrics aggregated from all subsystems.

use serde::{Deserialize, Serialize};

/// Combined dashboard snapshot of all node metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    /// Timestamp of this snapshot (Unix seconds)
    pub timestamp: u64,
    /// Node information
    pub node_info: NodeInfo,
    /// Storage metrics
    pub storage: StorageMetrics,
    /// Network metrics
    pub network: NetworkMetrics,
    /// Economics metrics
    pub economics: EconomicsMetrics,
    /// Consensus metrics
    pub consensus: ConsensusMetrics,
}

/// Node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node identifier
    pub node_id: String,
    /// Network address
    pub address: String,
    /// Software version
    pub version: String,
    /// Uptime in seconds
    pub uptime_seconds: u64,
    /// Current node status
    pub status: NodeStatus,
}

/// Node operational status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeStatus {
    Starting,
    Running,
    Syncing,
    Degraded,
    Stopped,
}

impl Default for NodeStatus {
    fn default() -> Self {
        Self::Starting
    }
}

/// Storage metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StorageMetrics {
    /// Total storage capacity in bytes
    pub total_capacity: u64,
    /// Used storage in bytes
    pub used_capacity: u64,
    /// Available storage in bytes
    pub available_capacity: u64,
    /// Number of blocks stored
    pub blocks_stored: u64,
    /// Average compression ratio
    pub compression_ratio: f64,
    /// Total original bytes before compression
    pub total_original_bytes: u64,
    /// Total compressed bytes after compression
    pub total_compressed_bytes: u64,
    /// Cache hits count
    pub cache_hits: u64,
    /// Cache misses count
    pub cache_misses: u64,
    /// Cache hit rate (0.0 - 1.0)
    pub cache_hit_rate: f64,
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NetworkMetrics {
    /// Total known peers
    pub total_peers: usize,
    /// Currently connected peers
    pub connected_peers: usize,
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Bandwidth in (bytes)
    pub bandwidth_in_bytes: u64,
    /// Bandwidth out (bytes)
    pub bandwidth_out_bytes: u64,
    /// Average latency in milliseconds
    pub average_latency_ms: f64,
}

/// Economics metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EconomicsMetrics {
    /// Staking summary
    pub staking: StakingMetricsSummary,
    /// Rewards summary
    pub rewards: RewardMetricsSummary,
    /// Gateway summary (if running as gateway)
    pub gateway: Option<GatewayMetricsSummary>,
}

/// Staking metrics summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StakingMetricsSummary {
    /// Total staked tokens in network
    pub total_staked: u64,
    /// Own stake amount
    pub own_stake: u64,
    /// Pending rewards
    pub pending_rewards: u64,
    /// Performance score (0.0 - 2.0)
    pub performance_score: f64,
    /// Base APY percentage
    pub base_apy: f64,
}

/// Reward metrics summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RewardMetricsSummary {
    /// Total rewards earned
    pub total_earned: u64,
    /// Rewards distributed this epoch
    pub distributed_this_epoch: u64,
    /// Number of epochs completed
    pub epochs_completed: usize,
}

/// Gateway metrics summary
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GatewayMetricsSummary {
    /// Total revenue in micro-tokens
    pub total_revenue: u64,
    /// Number of active clients
    pub active_clients: u32,
    /// Total requests served
    pub requests_served: u64,
    /// Current load (0.0 - 1.0)
    pub current_load: f64,
}

/// Consensus metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConsensusMetrics {
    /// Number of active proposals
    pub active_proposals: usize,
    /// Total votes cast
    pub votes_cast: u64,
    /// Proposals accepted
    pub proposals_accepted: u64,
    /// Proposals rejected
    pub proposals_rejected: u64,
    /// Node reputation score (0.0 - 1.0+)
    pub reputation_score: f64,
}

impl Default for DashboardSnapshot {
    fn default() -> Self {
        Self {
            timestamp: 0,
            node_info: NodeInfo {
                node_id: String::new(),
                address: String::new(),
                version: String::new(),
                uptime_seconds: 0,
                status: NodeStatus::default(),
            },
            storage: StorageMetrics::default(),
            network: NetworkMetrics::default(),
            economics: EconomicsMetrics::default(),
            consensus: ConsensusMetrics::default(),
        }
    }
}
