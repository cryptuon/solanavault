//! # Data Layer
//!
//! Handles raw Solana block data access, caching, and management.

pub mod client;
pub mod cache;
pub mod types;
pub mod test_utils;

/// Real Solana RPC client for fetching live block data
pub mod solana_client;

pub use client::SolanaBlockClient;
pub use cache::BlockCache;
pub use types::*;
pub use test_utils::TestDataManager;
pub use solana_client::{SolanaBlockDataClient, TestBlockSet, BlockCharacteristics, BlockType};