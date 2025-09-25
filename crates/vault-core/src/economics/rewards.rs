//! # Reward Distribution
//!
//! Performance-based reward calculation and distribution system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Reward distribution system for network participants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    /// Total rewards available for distribution
    pub total_reward_pool: u64,
    /// Rewards distributed this epoch
    pub distributed_this_epoch: u64,
    /// Performance metrics by node
    pub performance_metrics: HashMap<String, PerformanceMetrics>,
    /// Reward history
    pub reward_history: Vec<RewardEpoch>,
}

/// Performance metrics for reward calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    /// Node ID
    pub node_id: String,
    /// Uptime percentage (0.0 to 1.0)
    pub uptime: f64,
    /// Average response time in milliseconds
    pub avg_response_time: u64,
    /// Data retrieval success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Storage proof verification rate (0.0 to 1.0)
    pub proof_success_rate: f64,
    /// Bandwidth contributed (bytes per second)
    pub bandwidth_contributed: u64,
    /// Storage space provided (bytes)
    pub storage_provided: u64,
}

/// Reward epoch record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardEpoch {
    /// Epoch number
    pub epoch: u64,
    /// Timestamp
    pub timestamp: u64,
    /// Total rewards distributed
    pub total_distributed: u64,
    /// Rewards by node
    pub node_rewards: HashMap<String, u64>,
    /// Average performance score
    pub avg_performance_score: f64,
}

/// Reward calculation engine
pub struct RewardCalculator {
    /// Base reward rate (tokens per epoch per unit of stake)
    pub base_reward_rate: f64,
    /// Performance weight factors
    pub performance_weights: PerformanceWeights,
}

/// Weight factors for different performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceWeights {
    /// Weight for uptime (default: 0.3)
    pub uptime_weight: f64,
    /// Weight for response time (default: 0.2)
    pub response_time_weight: f64,
    /// Weight for success rate (default: 0.2)
    pub success_rate_weight: f64,
    /// Weight for storage proof (default: 0.15)
    pub proof_weight: f64,
    /// Weight for bandwidth contribution (default: 0.1)
    pub bandwidth_weight: f64,
    /// Weight for storage provision (default: 0.05)
    pub storage_weight: f64,
}

impl RewardDistribution {
    /// Create a new reward distribution system
    pub fn new() -> Self {
        Self {
            total_reward_pool: 0,
            distributed_this_epoch: 0,
            performance_metrics: HashMap::new(),
            reward_history: Vec::new(),
        }
    }

    /// Update performance metrics for a node
    pub fn update_performance_metrics(
        &mut self,
        node_id: String,
        metrics: PerformanceMetrics,
    ) {
        self.performance_metrics.insert(node_id, metrics);
    }

    /// Add rewards to the pool
    pub fn add_to_reward_pool(&mut self, amount: u64) {
        self.total_reward_pool += amount;
    }

    /// Distribute rewards for the current epoch
    pub fn distribute_epoch_rewards(
        &mut self,
        calculator: &RewardCalculator,
        staked_amounts: &HashMap<String, u64>,
    ) -> Result<HashMap<String, u64>, RewardError> {
        let mut epoch_rewards = HashMap::new();
        let mut total_performance_score = 0.0;

        // Calculate total performance score
        for (node_id, metrics) in &self.performance_metrics {
            if staked_amounts.contains_key(node_id) {
                let performance_score = calculator.calculate_performance_score(metrics);
                total_performance_score += performance_score;
            }
        }

        if total_performance_score == 0.0 {
            return Err(RewardError::NoEligibleNodes);
        }

        // Distribute rewards proportionally based on performance and stake
        let available_rewards = self.total_reward_pool.saturating_sub(self.distributed_this_epoch);

        for (node_id, metrics) in &self.performance_metrics {
            if let Some(&staked_amount) = staked_amounts.get(node_id) {
                let performance_score = calculator.calculate_performance_score(metrics);
                let stake_weight = staked_amount as f64;

                // Calculate reward based on performance and stake
                let reward_share = (performance_score * stake_weight) / total_performance_score;
                let node_reward = (available_rewards as f64 * reward_share) as u64;

                epoch_rewards.insert(node_id.clone(), node_reward);
            }
        }

        // Record the epoch
        let epoch_num = self.reward_history.len() as u64 + 1;
        let total_distributed: u64 = epoch_rewards.values().sum();

        let reward_epoch = RewardEpoch {
            epoch: epoch_num,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            total_distributed,
            node_rewards: epoch_rewards.clone(),
            avg_performance_score: total_performance_score / self.performance_metrics.len() as f64,
        };

        self.reward_history.push(reward_epoch);
        self.distributed_this_epoch += total_distributed;

        Ok(epoch_rewards)
    }

    /// Get reward statistics
    pub fn get_reward_stats(&self) -> RewardStats {
        RewardStats {
            total_reward_pool: self.total_reward_pool,
            distributed_this_epoch: self.distributed_this_epoch,
            total_nodes: self.performance_metrics.len(),
            epochs_completed: self.reward_history.len(),
            avg_rewards_per_epoch: if self.reward_history.is_empty() {
                0
            } else {
                self.reward_history.iter()
                    .map(|e| e.total_distributed)
                    .sum::<u64>() / self.reward_history.len() as u64
            },
        }
    }
}

impl RewardCalculator {
    /// Create a new reward calculator with default weights
    pub fn new() -> Self {
        Self {
            base_reward_rate: 0.1, // 10% base reward rate
            performance_weights: PerformanceWeights::default(),
        }
    }

    /// Calculate performance score for a node based on metrics
    pub fn calculate_performance_score(&self, metrics: &PerformanceMetrics) -> f64 {
        let weights = &self.performance_weights;

        // Normalize response time (lower is better, so invert)
        let response_time_score = if metrics.avg_response_time > 0 {
            1000.0 / (metrics.avg_response_time as f64).max(100.0)
        } else {
            1.0
        };

        // Calculate weighted performance score
        let score = metrics.uptime * weights.uptime_weight
            + response_time_score * weights.response_time_weight
            + metrics.success_rate * weights.success_rate_weight
            + metrics.proof_success_rate * weights.proof_weight
            + (metrics.bandwidth_contributed as f64 / 1_000_000.0).min(1.0) * weights.bandwidth_weight
            + (metrics.storage_provided as f64 / 1_000_000_000.0).min(1.0) * weights.storage_weight;

        score.max(0.0).min(2.0) // Cap between 0 and 2
    }
}

impl Default for PerformanceWeights {
    fn default() -> Self {
        Self {
            uptime_weight: 0.3,
            response_time_weight: 0.2,
            success_rate_weight: 0.2,
            proof_weight: 0.15,
            bandwidth_weight: 0.1,
            storage_weight: 0.05,
        }
    }
}

impl Default for RewardCalculator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RewardDistribution {
    fn default() -> Self {
        Self::new()
    }
}

/// Reward system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardStats {
    pub total_reward_pool: u64,
    pub distributed_this_epoch: u64,
    pub total_nodes: usize,
    pub epochs_completed: usize,
    pub avg_rewards_per_epoch: u64,
}

/// Reward system errors
#[derive(Debug, thiserror::Error)]
pub enum RewardError {
    #[error("No eligible nodes for reward distribution")]
    NoEligibleNodes,

    #[error("Insufficient reward pool")]
    InsufficientRewards,

    #[error("Invalid performance metrics")]
    InvalidMetrics,

    #[error("Reward calculation failed: {0}")]
    CalculationFailed(String),
}