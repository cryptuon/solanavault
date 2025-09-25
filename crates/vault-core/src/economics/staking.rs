//! # Staking System
//!
//! Token staking and reward distribution for SolanaVault network participants.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Staking contract managing node participation and rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingContract {
    /// Minimum stake required to participate
    pub minimum_stake: u64,
    /// Total staked tokens in the network
    pub total_staked: u64,
    /// Staked amounts by node ID
    pub stakes: HashMap<String, NodeStake>,
    /// Reward pool for distribution
    pub reward_pool: u64,
    /// Annual percentage yield for staking rewards
    pub base_apy: f64,
}

/// Staking information for a specific node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStake {
    /// Node ID
    pub node_id: String,
    /// Amount of tokens staked
    pub staked_amount: u64,
    /// Timestamp when stake was created
    pub stake_timestamp: u64,
    /// Performance score affecting rewards
    pub performance_score: f64,
    /// Pending rewards to be claimed
    pub pending_rewards: u64,
    /// Total rewards earned
    pub total_rewards_earned: u64,
    /// Slashing record
    pub slashing_record: Vec<SlashingEvent>,
}

/// Record of slashing events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    /// Timestamp of slashing event
    pub timestamp: u64,
    /// Amount slashed
    pub amount: u64,
    /// Reason for slashing
    pub reason: SlashingReason,
    /// Severity of the offense
    pub severity: SlashingSeverity,
}

/// Reasons for slashing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashingReason {
    DataUnavailability,
    DataCorruption,
    ExtendedDowntime,
    FailedProofOfStorage,
    MaliciousBehavior,
}

/// Severity levels for slashing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashingSeverity {
    Minor,   // 5% slash
    Major,   // 15% slash
    Severe,  // 30% slash
    Critical, // 50% slash
}

impl StakingContract {
    /// Create a new staking contract
    pub fn new(minimum_stake: u64, base_apy: f64) -> Self {
        Self {
            minimum_stake,
            total_staked: 0,
            stakes: HashMap::new(),
            reward_pool: 0,
            base_apy,
        }
    }

    /// Stake tokens for a node
    pub fn stake_tokens(&mut self, node_id: String, amount: u64) -> Result<(), StakingError> {
        if amount < self.minimum_stake {
            return Err(StakingError::InsufficientStake);
        }

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let node_stake = NodeStake {
            node_id: node_id.clone(),
            staked_amount: amount,
            stake_timestamp: current_time,
            performance_score: 1.0,
            pending_rewards: 0,
            total_rewards_earned: 0,
            slashing_record: Vec::new(),
        };

        self.stakes.insert(node_id, node_stake);
        self.total_staked += amount;

        Ok(())
    }

    /// Unstake tokens for a node
    pub fn unstake_tokens(&mut self, node_id: &str, amount: u64) -> Result<u64, StakingError> {
        // Calculate pending rewards first (before borrowing mutably)
        let pending_rewards = self.calculate_pending_rewards(node_id)?;

        let stake = self.stakes.get_mut(node_id)
            .ok_or(StakingError::NodeNotFound)?;

        if amount > stake.staked_amount {
            return Err(StakingError::InsufficientStake);
        }

        // Add pending rewards before unstaking
        stake.pending_rewards += pending_rewards;

        stake.staked_amount -= amount;
        self.total_staked -= amount;

        // Remove stake entry if no tokens left
        if stake.staked_amount == 0 {
            let final_stake = self.stakes.remove(node_id).unwrap();
            Ok(final_stake.pending_rewards)
        } else {
            Ok(0)
        }
    }

    /// Calculate pending rewards for a node
    pub fn calculate_pending_rewards(&self, node_id: &str) -> Result<u64, StakingError> {
        let stake = self.stakes.get(node_id)
            .ok_or(StakingError::NodeNotFound)?;

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let staking_duration = current_time - stake.stake_timestamp;
        let annual_rewards = (stake.staked_amount as f64 * self.base_apy) as u64;
        let time_factor = staking_duration as f64 / (365.25 * 24.0 * 3600.0); // Convert to years

        // Apply performance multiplier
        let performance_multiplier = stake.performance_score;
        let base_rewards = (annual_rewards as f64 * time_factor * performance_multiplier) as u64;

        Ok(base_rewards)
    }

    /// Update performance score for a node
    pub fn update_performance_score(&mut self, node_id: &str, score: f64) -> Result<(), StakingError> {
        let stake = self.stakes.get_mut(node_id)
            .ok_or(StakingError::NodeNotFound)?;

        // Performance score should be between 0.0 and 2.0 (200% for exceptional performance)
        stake.performance_score = score.max(0.0).min(2.0);
        Ok(())
    }

    /// Apply slashing to a node
    pub fn apply_slashing(
        &mut self,
        node_id: &str,
        reason: SlashingReason,
        severity: SlashingSeverity,
    ) -> Result<u64, StakingError> {
        let stake = self.stakes.get_mut(node_id)
            .ok_or(StakingError::NodeNotFound)?;

        let slash_percentage = match severity {
            SlashingSeverity::Minor => 0.05,
            SlashingSeverity::Major => 0.15,
            SlashingSeverity::Severe => 0.30,
            SlashingSeverity::Critical => 0.50,
        };

        let slashed_amount = (stake.staked_amount as f64 * slash_percentage) as u64;

        // Record the slashing event
        let slashing_event = SlashingEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            amount: slashed_amount,
            reason,
            severity,
        };

        stake.slashing_record.push(slashing_event);
        stake.staked_amount = stake.staked_amount.saturating_sub(slashed_amount);
        self.total_staked = self.total_staked.saturating_sub(slashed_amount);

        Ok(slashed_amount)
    }

    /// Claim pending rewards
    pub fn claim_rewards(&mut self, node_id: &str) -> Result<u64, StakingError> {
        // Calculate pending rewards first (before borrowing mutably)
        let pending_rewards = self.calculate_pending_rewards(node_id)?;

        let stake = self.stakes.get_mut(node_id)
            .ok_or(StakingError::NodeNotFound)?;

        let total_claimable = stake.pending_rewards + pending_rewards;

        stake.pending_rewards = 0;
        stake.total_rewards_earned += total_claimable;

        // Reset reward calculation timestamp
        stake.stake_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(total_claimable)
    }

    /// Get staking information for a node
    pub fn get_node_stake(&self, node_id: &str) -> Option<&NodeStake> {
        self.stakes.get(node_id)
    }

    /// Get network staking statistics
    pub fn get_staking_stats(&self) -> StakingStats {
        StakingStats {
            total_staked: self.total_staked,
            total_nodes: self.stakes.len(),
            minimum_stake: self.minimum_stake,
            base_apy: self.base_apy,
            reward_pool: self.reward_pool,
        }
    }

    /// Add tokens to the reward pool (typically from network fees)
    pub fn add_to_reward_pool(&mut self, amount: u64) {
        self.reward_pool += amount;
    }
}

/// Network staking statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StakingStats {
    pub total_staked: u64,
    pub total_nodes: usize,
    pub minimum_stake: u64,
    pub base_apy: f64,
    pub reward_pool: u64,
}

/// Staking system errors
#[derive(Debug, thiserror::Error)]
pub enum StakingError {
    #[error("Insufficient stake amount")]
    InsufficientStake,

    #[error("Node not found")]
    NodeNotFound,

    #[error("Invalid performance score")]
    InvalidPerformanceScore,

    #[error("Reward calculation failed")]
    RewardCalculationFailed,

    #[error("Staking error: {0}")]
    StakingError(String),
}