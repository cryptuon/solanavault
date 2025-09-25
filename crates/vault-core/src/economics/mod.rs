//! # Economics Module
//!
//! Token economics, staking, and incentive mechanisms for the SolanaVault network.

/// Staking and reward distribution system
pub mod staking;

/// Reward distribution mechanisms
pub mod rewards;

// Re-export key types
pub use staking::{StakingContract, NodeStake, StakingStats, StakingError};
pub use rewards::{RewardDistribution, RewardCalculator, RewardStats};