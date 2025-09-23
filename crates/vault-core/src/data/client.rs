//! # Solana Block Client
//!
//! Client for fetching real Solana block data from RPC endpoints.

use super::types::{CachedBlock, DataError};
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_transaction_status::UiTransactionEncoding;
use std::time::{SystemTime, UNIX_EPOCH};

/// Client for fetching Solana block data
pub struct SolanaBlockClient {
    rpc_client: RpcClient,
    commitment: CommitmentConfig,
}

impl SolanaBlockClient {
    /// Create a new client with the default mainnet RPC endpoint
    pub fn new() -> Self {
        Self::with_endpoint("https://api.mainnet-beta.solana.com")
    }

    /// Create a new client with a custom RPC endpoint
    pub fn with_endpoint(endpoint: &str) -> Self {
        let rpc_client = RpcClient::new_with_commitment(
            endpoint.to_string(),
            CommitmentConfig::confirmed(),
        );

        Self {
            rpc_client,
            commitment: CommitmentConfig::confirmed(),
        }
    }

    /// Fetch a single block by slot
    pub async fn fetch_block(&self, slot: u64) -> Result<CachedBlock, DataError> {
        log::info!("Fetching block at slot {}", slot);

        let block = self
            .rpc_client
            .get_block_with_config(
                slot,
                solana_client::rpc_config::RpcBlockConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    transaction_details: Some(
                        solana_transaction_status::TransactionDetails::Full,
                    ),
                    rewards: Some(true),
                    commitment: Some(self.commitment),
                    max_supported_transaction_version: Some(0),
                },
            )
            .map_err(|e| DataError::RpcClient(e.to_string()))?;

        // Serialize block to bytes for storage
        let block_data = bincode::serialize(&block)
            .map_err(|e| DataError::Cache(format!("Serialization failed: {}", e)))?;

        let transaction_count = block.transactions.as_ref().map(|t| t.len()).unwrap_or(0);
        let original_size = block_data.len();
        let cached_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Ok(CachedBlock {
            slot,
            block_data,
            cached_at,
            compressed_size: None,
            original_size,
            transaction_count,
        })
    }

    /// Fetch a range of blocks
    pub async fn fetch_block_range(
        &self,
        start_slot: u64,
        end_slot: u64,
    ) -> Result<Vec<CachedBlock>, DataError> {
        let mut blocks = Vec::new();

        for slot in start_slot..=end_slot {
            match self.fetch_block(slot).await {
                Ok(block) => {
                    log::debug!("Successfully fetched block {}", slot);
                    blocks.push(block);
                }
                Err(DataError::BlockNotFound(_)) => {
                    log::warn!("Block {} not found, skipping", slot);
                    continue;
                }
                Err(e) => {
                    log::error!("Failed to fetch block {}: {}", slot, e);
                    return Err(e);
                }
            }

            // Add small delay to avoid rate limiting
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }

        log::info!(
            "Successfully fetched {} blocks from range {}-{}",
            blocks.len(),
            start_slot,
            end_slot
        );

        Ok(blocks)
    }

    /// Get the latest slot
    pub async fn get_latest_slot(&self) -> Result<u64, DataError> {
        self.rpc_client
            .get_slot()
            .map_err(|e| DataError::RpcClient(e.to_string()))
    }

    /// Get a range of recent blocks (last N blocks)
    pub async fn fetch_recent_blocks(&self, count: usize) -> Result<Vec<CachedBlock>, DataError> {
        let latest_slot = self.get_latest_slot().await?;
        let start_slot = latest_slot.saturating_sub(count as u64);

        self.fetch_block_range(start_slot, latest_slot).await
    }

    /// Estimate the serialized size of a block
    fn estimate_block_size(&self, block_data: &[u8]) -> usize {
        block_data.len()
    }

    /// Check if a block exists at the given slot
    pub async fn block_exists(&self, slot: u64) -> bool {
        self.rpc_client
            .get_block_with_config(
                slot,
                solana_client::rpc_config::RpcBlockConfig {
                    encoding: Some(UiTransactionEncoding::Json),
                    transaction_details: Some(
                        solana_transaction_status::TransactionDetails::None,
                    ),
                    rewards: Some(false),
                    commitment: Some(self.commitment),
                    max_supported_transaction_version: Some(0),
                },
            )
            .is_ok()
    }

    /// Find a good range of blocks for testing (blocks with high transaction count)
    pub async fn find_high_activity_blocks(
        &self,
        search_range: u64,
        min_transactions: usize,
    ) -> Result<Vec<u64>, DataError> {
        let latest_slot = self.get_latest_slot().await?;
        let start_slot = latest_slot.saturating_sub(search_range);
        let mut high_activity_slots = Vec::new();

        log::info!(
            "Searching for high-activity blocks in range {}-{} (min {} transactions)",
            start_slot,
            latest_slot,
            min_transactions
        );

        for slot in start_slot..=latest_slot {
            // Fetch actual block for analysis
            match self.fetch_block(slot).await {
                Ok(cached_block) => {
                    if cached_block.transaction_count >= min_transactions {
                        high_activity_slots.push(slot);
                        log::debug!(
                            "Found high-activity block {} with {} transactions",
                            slot,
                            cached_block.transaction_count
                        );
                    }
                }
                Err(_) => {
                    // Skip blocks that don't exist or can't be fetched
                    continue;
                }
            }

            // Limit search to avoid too many API calls
            if high_activity_slots.len() >= 10 {
                break;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(high_activity_slots)
    }
}

impl Default for SolanaBlockClient {
    fn default() -> Self {
        Self::new()
    }
}