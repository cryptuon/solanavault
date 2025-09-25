//! # Real Solana Block Data Client
//!
//! Fetches real block data from Solana mainnet/devnet for compression testing.

use crate::data::CachedBlock;
use solana_client::rpc_client::RpcClient;
use solana_sdk::clock::Slot;
use solana_transaction_status::{UiTransactionEncoding, EncodedConfirmedBlock};
use std::collections::HashMap;
use thiserror::Error;
use serde::{Serialize, Deserialize};

/// Error types for Solana data fetching
#[derive(Error, Debug)]
pub enum SolanaDataError {
    #[error("RPC client error: {0}")]
    RpcError(#[from] solana_client::client_error::ClientError),

    #[error("Block not found: slot {0}")]
    BlockNotFound(Slot),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Configuration for Solana data fetching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaClientConfig {
    /// RPC endpoint URL
    pub rpc_url: String,

    /// Maximum number of blocks to fetch in one request
    pub max_blocks_per_request: usize,

    /// Whether to include transaction details
    pub include_transactions: bool,

    /// Transaction encoding format
    pub transaction_encoding: UiTransactionEncoding,

    /// Whether to include rewards
    pub include_rewards: bool,

    /// Cache directory for storing fetched blocks
    pub cache_dir: Option<String>,
}

impl Default for SolanaClientConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            max_blocks_per_request: 100,
            include_transactions: true,
            transaction_encoding: UiTransactionEncoding::Base64,
            include_rewards: false,
            cache_dir: Some("./solana_block_cache".to_string()),
        }
    }
}

/// Real Solana block data fetcher
pub struct SolanaBlockDataClient {
    /// RPC client for connecting to Solana
    rpc_client: RpcClient,

    /// Configuration
    pub config: SolanaClientConfig,

    /// Local cache of fetched blocks
    block_cache: HashMap<Slot, CachedBlock>,

    /// Statistics
    stats: SolanaClientStats,
}

/// Statistics for the Solana client
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SolanaClientStats {
    pub blocks_fetched: usize,
    pub cache_hits: usize,
    pub rpc_requests: usize,
    pub total_bytes_fetched: usize,
    pub average_block_size: f64,
    pub fetch_errors: usize,
}

impl SolanaBlockDataClient {
    /// Creates a new Solana block data client
    pub fn new() -> Self {
        let config = SolanaClientConfig::default();
        Self::with_config(config)
    }

    /// Creates a new client with custom configuration
    pub fn with_config(config: SolanaClientConfig) -> Self {
        let rpc_client = RpcClient::new(config.rpc_url.clone());

        Self {
            rpc_client,
            config,
            block_cache: HashMap::new(),
            stats: SolanaClientStats::default(),
        }
    }

    /// Creates a client for devnet testing
    pub fn devnet() -> Self {
        let mut config = SolanaClientConfig::default();
        config.rpc_url = "https://api.devnet.solana.com".to_string();
        Self::with_config(config)
    }

    /// Fetches a single block by slot number
    pub async fn fetch_block(&mut self, slot: Slot) -> Result<CachedBlock, SolanaDataError> {
        // Check cache first
        if let Some(cached_block) = self.block_cache.get(&slot) {
            self.stats.cache_hits += 1;
            return Ok(cached_block.clone());
        }

        println!("📡 Fetching block {} from Solana RPC...", slot);

        // Fetch from RPC
        self.stats.rpc_requests += 1;

        let block_result = self.rpc_client.get_block_with_config(
            slot,
            solana_client::rpc_config::RpcBlockConfig {
                encoding: Some(self.config.transaction_encoding),
                transaction_details: Some(solana_transaction_status::TransactionDetails::Full),
                rewards: Some(self.config.include_rewards),
                commitment: Some(solana_sdk::commitment_config::CommitmentConfig::confirmed()),
                max_supported_transaction_version: Some(0),
            },
        );

        let block = match block_result {
            Ok(block) => block,
            Err(e) => {
                self.stats.fetch_errors += 1;
                return Err(SolanaDataError::RpcError(e));
            }
        };

        // Convert to our cached block format
        let cached_block = self.convert_to_cached_block(slot, block)?;

        // Update statistics
        self.stats.blocks_fetched += 1;
        self.stats.total_bytes_fetched += cached_block.raw_data.len();
        self.stats.average_block_size = self.stats.total_bytes_fetched as f64 / self.stats.blocks_fetched as f64;

        // Cache the block
        self.block_cache.insert(slot, cached_block.clone());

        // Save to disk cache if configured
        if let Some(ref cache_dir) = self.config.cache_dir {
            self.save_block_to_cache(cache_dir, slot, &cached_block).await?;
        }

        println!("✅ Fetched block {} ({} bytes, {} transactions)",
                 slot, cached_block.raw_data.len(), cached_block.transaction_count);

        Ok(cached_block)
    }

    /// Fetches multiple blocks by slot range
    pub async fn fetch_block_range(&mut self, start_slot: Slot, end_slot: Slot) -> Result<Vec<CachedBlock>, SolanaDataError> {
        println!("📡 Fetching block range {} to {} from Solana...", start_slot, end_slot);

        let mut blocks = Vec::new();

        for slot in start_slot..=end_slot {
            match self.fetch_block(slot).await {
                Ok(block) => blocks.push(block),
                Err(SolanaDataError::BlockNotFound(_)) => {
                    println!("⚠️  Block {} not found, skipping", slot);
                    continue;
                }
                Err(e) => {
                    println!("❌ Error fetching block {}: {}", slot, e);
                    return Err(e);
                }
            }

            // Rate limiting - small delay between requests
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        println!("✅ Fetched {} blocks from range {} to {}", blocks.len(), start_slot, end_slot);
        Ok(blocks)
    }

    /// Fetches recent blocks from the current slot
    pub async fn fetch_recent_blocks(&mut self, count: usize) -> Result<Vec<CachedBlock>, SolanaDataError> {
        println!("📡 Fetching {} recent blocks from Solana...", count);

        // Get current slot
        self.stats.rpc_requests += 1;
        let current_slot = self.rpc_client.get_slot()?;

        println!("Current slot: {}", current_slot);

        // Fetch the most recent blocks
        let start_slot = current_slot.saturating_sub(count as u64);
        self.fetch_block_range(start_slot, current_slot).await
    }

    /// Fetches blocks with specific characteristics for testing
    pub async fn fetch_test_blocks(&mut self) -> Result<TestBlockSet, SolanaDataError> {
        println!("📡 Fetching diverse test blocks for compression testing...");

        let mut test_set = TestBlockSet::new();

        // Get recent blocks first
        let recent_blocks = self.fetch_recent_blocks(20).await?;

        // Categorize blocks by characteristics
        for block in recent_blocks {
            let characteristics = self.analyze_block_characteristics(&block);

            match characteristics.block_type {
                BlockType::Small => {
                    if test_set.small_blocks.len() < 3 {
                        test_set.small_blocks.push(block);
                    }
                }
                BlockType::Large => {
                    if test_set.large_blocks.len() < 3 {
                        test_set.large_blocks.push(block);
                    }
                }
                BlockType::HighVolume => {
                    if test_set.high_volume_blocks.len() < 3 {
                        test_set.high_volume_blocks.push(block);
                    }
                }
                BlockType::DeFi => {
                    if test_set.defi_blocks.len() < 3 {
                        test_set.defi_blocks.push(block);
                    }
                }
                BlockType::Mixed => {
                    if test_set.mixed_blocks.len() < 3 {
                        test_set.mixed_blocks.push(block);
                    }
                }
            }
        }

        println!("✅ Collected test block set:");
        println!("   Small blocks: {}", test_set.small_blocks.len());
        println!("   Large blocks: {}", test_set.large_blocks.len());
        println!("   High volume blocks: {}", test_set.high_volume_blocks.len());
        println!("   DeFi blocks: {}", test_set.defi_blocks.len());
        println!("   Mixed blocks: {}", test_set.mixed_blocks.len());

        Ok(test_set)
    }

    /// Converts Solana RPC block to our cached block format
    fn convert_to_cached_block(&self, slot: Slot, block: solana_transaction_status::UiConfirmedBlock) -> Result<CachedBlock, SolanaDataError> {
        // Serialize the entire block as raw data
        let raw_data = serde_json::to_vec(&block)?;

        // Extract basic information
        let transaction_count = block.transactions.as_ref().map(|txs| txs.len()).unwrap_or(0);
        let block_time = block.block_time.unwrap_or(0) as u64;
        let block_hash = block.blockhash.clone();

        Ok(CachedBlock {
            slot,
            raw_data,
            transaction_count,
            block_time,
            block_hash,
            parent_slot: block.parent_slot,
        })
    }

    /// Analyzes block characteristics for categorization
    fn analyze_block_characteristics(&self, block: &CachedBlock) -> BlockCharacteristics {
        let size = block.raw_data.len();
        let tx_count = block.transaction_count;

        let block_type = if tx_count < 100 {
            BlockType::Small
        } else if tx_count > 1000 {
            BlockType::HighVolume
        } else if size > 100_000 {
            BlockType::Large
        } else if self.is_defi_heavy_block(block) {
            BlockType::DeFi
        } else {
            BlockType::Mixed
        };

        BlockCharacteristics {
            block_type,
            transaction_count: tx_count,
            size_bytes: size,
            estimated_defi_ratio: self.estimate_defi_ratio(block),
        }
    }

    /// Estimates if block is DeFi heavy (heuristic)
    fn is_defi_heavy_block(&self, block: &CachedBlock) -> bool {
        // Simple heuristic: look for patterns in the raw data that suggest DeFi activity
        let raw_str = String::from_utf8_lossy(&block.raw_data);

        // Look for common DeFi program patterns
        let defi_indicators = [
            "Swap", "swap", "DEX", "dex", "AMM", "amm",
            "liquidity", "pool", "stake", "yield"
        ];

        let indicator_count = defi_indicators.iter()
            .map(|indicator| raw_str.matches(indicator).count())
            .sum::<usize>();

        indicator_count > 10 // Arbitrary threshold
    }

    /// Estimates DeFi transaction ratio
    fn estimate_defi_ratio(&self, block: &CachedBlock) -> f64 {
        let raw_str = String::from_utf8_lossy(&block.raw_data);
        let defi_matches = raw_str.matches("swap").count() + raw_str.matches("dex").count();
        let total_tx = block.transaction_count.max(1);

        (defi_matches as f64 / total_tx as f64).min(1.0)
    }

    /// Saves block to disk cache
    async fn save_block_to_cache(&self, cache_dir: &str, slot: Slot, block: &CachedBlock) -> Result<(), SolanaDataError> {
        std::fs::create_dir_all(cache_dir)?;

        let file_path = format!("{}/block_{}.json", cache_dir, slot);
        let serialized = serde_json::to_vec_pretty(block)?;

        tokio::fs::write(file_path, serialized).await?;
        Ok(())
    }

    /// Loads block from disk cache
    pub async fn load_block_from_cache(&mut self, cache_dir: &str, slot: Slot) -> Result<Option<CachedBlock>, SolanaDataError> {
        let file_path = format!("{}/block_{}.json", cache_dir, slot);

        match tokio::fs::read(file_path).await {
            Ok(data) => {
                let block: CachedBlock = serde_json::from_slice(&data)?;
                self.block_cache.insert(slot, block.clone());
                Ok(Some(block))
            }
            Err(_) => Ok(None), // File doesn't exist
        }
    }

    /// Gets client statistics
    pub fn get_stats(&self) -> &SolanaClientStats {
        &self.stats
    }

    /// Gets cached blocks count
    pub fn cached_blocks_count(&self) -> usize {
        self.block_cache.len()
    }

    /// Clears the block cache
    pub fn clear_cache(&mut self) {
        self.block_cache.clear();
    }
}

/// Test block set with different characteristics
#[derive(Debug, Clone)]
pub struct TestBlockSet {
    pub small_blocks: Vec<CachedBlock>,
    pub large_blocks: Vec<CachedBlock>,
    pub high_volume_blocks: Vec<CachedBlock>,
    pub defi_blocks: Vec<CachedBlock>,
    pub mixed_blocks: Vec<CachedBlock>,
}

impl TestBlockSet {
    pub fn new() -> Self {
        Self {
            small_blocks: Vec::new(),
            large_blocks: Vec::new(),
            high_volume_blocks: Vec::new(),
            defi_blocks: Vec::new(),
            mixed_blocks: Vec::new(),
        }
    }

    /// Gets all blocks as a single vector
    pub fn all_blocks(&self) -> Vec<CachedBlock> {
        let mut all = Vec::new();
        all.extend(self.small_blocks.clone());
        all.extend(self.large_blocks.clone());
        all.extend(self.high_volume_blocks.clone());
        all.extend(self.defi_blocks.clone());
        all.extend(self.mixed_blocks.clone());
        all
    }

    /// Gets total block count
    pub fn total_count(&self) -> usize {
        self.small_blocks.len() +
        self.large_blocks.len() +
        self.high_volume_blocks.len() +
        self.defi_blocks.len() +
        self.mixed_blocks.len()
    }
}

/// Block type classification
#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Small,      // < 100 transactions
    Large,      // > 100KB
    HighVolume, // > 1000 transactions
    DeFi,       // High DeFi activity
    Mixed,      // General activity
}

/// Block characteristics for analysis
#[derive(Debug, Clone)]
pub struct BlockCharacteristics {
    pub block_type: BlockType,
    pub transaction_count: usize,
    pub size_bytes: usize,
    pub estimated_defi_ratio: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solana_client_creation() {
        let client = SolanaBlockDataClient::new();
        assert_eq!(client.cached_blocks_count(), 0);
        assert_eq!(client.get_stats().blocks_fetched, 0);
    }

    #[test]
    fn test_devnet_client_creation() {
        let client = SolanaBlockDataClient::devnet();
        assert!(client.config.rpc_url.contains("devnet"));
    }

    #[test]
    fn test_test_block_set_creation() {
        let test_set = TestBlockSet::new();
        assert_eq!(test_set.total_count(), 0);
        assert!(test_set.all_blocks().is_empty());
    }

    #[tokio::test]
    #[ignore] // Requires network access
    async fn test_fetch_current_slot() {
        let mut client = SolanaBlockDataClient::devnet();

        // This test is ignored by default as it requires network access
        // Run with: cargo test test_fetch_current_slot -- --ignored

        match client.rpc_client.get_slot() {
            Ok(slot) => {
                println!("Current devnet slot: {}", slot);
                assert!(slot > 0);
            }
            Err(e) => {
                println!("Network test skipped: {}", e);
            }
        }
    }
}