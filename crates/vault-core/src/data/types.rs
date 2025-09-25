//! # Data Types
//!
//! Common types for Solana block data handling.

use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

/// Cached block data with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBlock {
    pub slot: u64,
    pub raw_data: Vec<u8>, // Store as raw bytes for compression testing
    pub transaction_count: usize,
    pub block_time: u64,
    pub block_hash: String,
    pub parent_slot: u64,
}

/// Block statistics for compression analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStats {
    pub slot: u64,
    pub transaction_count: usize,
    pub account_count: usize,
    pub program_count: usize,
    pub instruction_count: usize,
    pub signature_count: usize,
    pub zero_value_transfers: usize,
    pub failed_transactions: usize,
    pub unique_accounts: Vec<Pubkey>,
    pub unique_programs: Vec<Pubkey>,
    pub instruction_types: HashMap<String, usize>,
}

impl BlockStats {
    /// Create placeholder block stats for now
    pub fn new(slot: u64, transaction_count: usize) -> Self {
        Self {
            slot,
            transaction_count,
            account_count: 0,
            program_count: 0,
            instruction_count: 0,
            signature_count: transaction_count,
            zero_value_transfers: 0,
            failed_transactions: 0,
            unique_accounts: Vec::new(),
            unique_programs: Vec::new(),
            instruction_types: HashMap::new(),
        }
    }

    /// Get compression potential score (0.0 to 1.0)
    pub fn compression_potential(&self) -> f64 {
        let mut score: f64 = 0.0;

        // High transaction count = better compression
        if self.transaction_count > 1000 {
            score += 0.2;
        }

        // High instruction repetition = better compression
        let total_instructions = self.instruction_count as f64;
        if total_instructions > 0.0 {
            let unique_instruction_types = self.instruction_types.len() as f64;
            let repetition_ratio = total_instructions / unique_instruction_types;
            if repetition_ratio > 10.0 {
                score += 0.3;
            }
        }

        // High failure rate = good for failure pattern compression
        let failure_rate = self.failed_transactions as f64 / self.transaction_count as f64;
        if failure_rate > 0.15 {
            score += 0.2;
        }

        // Account/program reuse
        let account_reuse = self.transaction_count as f64 / self.account_count as f64;
        if account_reuse > 3.0 {
            score += 0.3;
        }

        score.min(1.0)
    }
}

/// Data fetching error types
#[derive(Debug, thiserror::Error)]
pub enum DataError {
    #[error("RPC client error: {0}")]
    RpcClient(String),

    #[error("Block not found: slot {0}")]
    BlockNotFound(u64),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid block data: {0}")]
    InvalidBlock(String),
}