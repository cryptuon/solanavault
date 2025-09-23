//! # Test Utilities
//!
//! Utilities for testing with real Solana block data.

use super::{SolanaBlockClient, BlockCache, CachedBlock, DataError};
use std::path::Path;

/// Test utilities for working with Solana block data
pub struct TestDataManager {
    client: SolanaBlockClient,
    cache: BlockCache,
}

impl TestDataManager {
    /// Create a new test data manager
    pub fn new<P: AsRef<Path>>(cache_dir: P) -> Result<Self, DataError> {
        let client = SolanaBlockClient::new();
        let cache = BlockCache::new(cache_dir)?;

        Ok(Self { client, cache })
    }

    /// Get or fetch a recent block for testing
    pub async fn get_test_block(&mut self) -> Result<CachedBlock, DataError> {
        // Try to get a cached block first
        let recent_slots = vec![290000000, 289000000, 288000000]; // Some recent mainnet slots

        for slot in recent_slots {
            if let Some(cached_block) = self.cache.get(slot) {
                log::info!("Using cached test block from slot {}", slot);
                return Ok(cached_block);
            }
        }

        // If no cached blocks, fetch some recent ones
        log::info!("Fetching recent blocks for testing...");
        let recent_blocks = self.client.fetch_recent_blocks(5).await?;

        if let Some(block) = recent_blocks.into_iter().next() {
            self.cache.put(block.clone())?;
            log::info!("Fetched and cached test block from slot {}", block.slot);
            Ok(block)
        } else {
            Err(DataError::BlockNotFound(0))
        }
    }

    /// Get multiple test blocks with high transaction counts
    pub async fn get_high_activity_test_blocks(&mut self, count: usize) -> Result<Vec<CachedBlock>, DataError> {
        let slots = self.client.find_high_activity_blocks(1000, 100).await?;
        let mut blocks = Vec::new();

        for slot in slots.into_iter().take(count) {
            if let Some(cached) = self.cache.get(slot) {
                blocks.push(cached);
            } else {
                let block = self.client.fetch_block(slot).await?;
                self.cache.put(block.clone())?;
                blocks.push(block);
            }
        }

        log::info!("Collected {} high-activity test blocks", blocks.len());
        Ok(blocks)
    }

    /// Cleanup cache
    pub fn cleanup(&mut self) -> Result<(), DataError> {
        self.cache.cleanup()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_fetch_block_data() {
        env_logger::init();

        let temp_dir = TempDir::new().unwrap();
        let mut manager = TestDataManager::new(temp_dir.path()).unwrap();

        // This test requires internet connection to Solana RPC
        if let Ok(block) = manager.get_test_block().await {
            assert!(block.original_size > 0);
            assert!(block.transaction_count >= 0);
            println!("Successfully fetched block {} with {} transactions",
                     block.slot, block.transaction_count);
        }
    }
}