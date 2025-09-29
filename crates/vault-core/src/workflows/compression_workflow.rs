//! End-to-end compression workflow for SolanaVault
//!
//! This demonstrates the complete pipeline from raw Solana block data
//! to compressed storage and retrieval with perfect data integrity.

use crate::compression::{BlockchainCompressionAdapter, CompressionStrategy};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Complete end-to-end compression workflow for SolanaVault
#[derive(Debug)]
pub struct CompressionWorkflow {
    /// Compression adapter for blockchain data
    compressor: BlockchainCompressionAdapter,
    /// Simple in-memory cache for frequently accessed blocks
    cache: HashMap<u64, Vec<u8>>,
    /// Maximum cache size
    max_cache_size: usize,
    /// Storage for compressed blocks
    compressed_storage: HashMap<u64, CompressedBlock>,
    /// Performance metrics
    metrics: WorkflowMetrics,
}

/// Compressed block with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedBlock {
    /// Original slot number
    pub slot: u64,
    /// Compressed block data
    pub compressed_data: Vec<u8>,
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Compression time in microseconds
    pub compression_time_us: u64,
    /// Block hash for integrity verification
    pub block_hash: String,
    /// Timestamp when compressed
    pub compressed_at: u64,
}

/// Performance metrics for the workflow
#[derive(Debug, Default)]
pub struct WorkflowMetrics {
    /// Total blocks processed
    pub blocks_processed: u64,
    /// Total original bytes
    pub total_original_bytes: u64,
    /// Total compressed bytes
    pub total_compressed_bytes: u64,
    /// Average compression ratio
    pub average_compression_ratio: f64,
    /// Total compression time
    pub total_compression_time_us: u64,
    /// Cache hits
    pub cache_hits: u64,
    /// Cache misses
    pub cache_misses: u64,
}

impl CompressionWorkflow {
    /// Create a new compression workflow
    pub fn new() -> Self {
        Self {
            compressor: BlockchainCompressionAdapter::for_transactions(),
            cache: HashMap::new(),
            max_cache_size: 100, // Cache last 100 blocks
            compressed_storage: HashMap::new(),
            metrics: WorkflowMetrics::default(),
        }
    }

    /// Process a block through the complete compression workflow
    pub async fn process_block(&mut self, slot: u64, block_data: &[u8]) -> Result<CompressedBlock, WorkflowError> {
        println!("🚀 Processing block {} ({} bytes)", slot, block_data.len());

        // Step 1: Validate block data
        self.validate_block_data(block_data)?;

        // Step 2: Compress the block
        let start_time = std::time::Instant::now();
        let compressed_data = self.compressor.compress(block_data)
            .map_err(|e| WorkflowError::CompressionFailed(format!("Compression failed: {}", e)))?;
        let compression_time = start_time.elapsed();

        // Step 3: Create compressed block metadata
        let compressed_size = compressed_data.len();
        let compression_ratio = block_data.len() as f64 / compressed_size as f64;
        let compressed_block = CompressedBlock {
            slot,
            compressed_data,
            original_size: block_data.len(),
            compressed_size,
            compression_ratio,
            compression_time_us: compression_time.as_micros() as u64,
            block_hash: self.calculate_block_hash(block_data),
            compressed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Step 4: Store compressed block
        self.store_compressed_block(compressed_block.clone()).await?;

        // Step 5: Update metrics
        self.update_metrics(&compressed_block);

        println!("✅ Block {} compressed: {:.2}:1 ratio ({} -> {} bytes)",
                 slot, compression_ratio, block_data.len(), compressed_block.compressed_size);

        Ok(compressed_block)
    }

    /// Retrieve and decompress a block
    pub async fn retrieve_block(&mut self, slot: u64) -> Result<Vec<u8>, WorkflowError> {
        println!("🔍 Retrieving block {}", slot);

        // Step 1: Check cache first
        if let Some(cached_data) = self.cache.get(&slot) {
            self.metrics.cache_hits += 1;
            println!("📦 Cache hit for block {}", slot);
            return Ok(cached_data.clone());
        }

        self.metrics.cache_misses += 1;

        // Step 2: Retrieve from compressed storage
        let compressed_block = self.compressed_storage.get(&slot)
            .ok_or_else(|| WorkflowError::BlockNotFound(slot))?;

        println!("💾 Retrieved compressed block {} ({} bytes)", slot, compressed_block.compressed_size);

        // Step 3: Decompress the block
        let start_time = std::time::Instant::now();
        let decompressed_data = self.compressor.decompress(&compressed_block.compressed_data)
            .map_err(|e| WorkflowError::DecompressionFailed(format!("Decompression failed: {}", e)))?;
        let decompression_time = start_time.elapsed();

        // Step 4: Verify integrity
        let calculated_hash = self.calculate_block_hash(&decompressed_data);
        if calculated_hash != compressed_block.block_hash {
            return Err(WorkflowError::IntegrityCheckFailed {
                slot,
                expected: compressed_block.block_hash.clone(),
                actual: calculated_hash,
            });
        }

        // Step 5: Cache the decompressed data
        self.cache_data(slot, decompressed_data.clone());

        println!("✅ Block {} decompressed and verified in {:?} (integrity: ✓)",
                 slot, decompression_time);

        Ok(decompressed_data)
    }

    /// Demonstrate the complete workflow with a realistic scenario
    pub async fn demonstrate_workflow(&mut self) -> Result<(), WorkflowError> {
        println!("🎬 SolanaVault End-to-End Compression Workflow Demo");
        println!("==================================================\n");

        // Step 1: Create realistic test blocks
        let test_blocks = self.create_test_blocks();
        println!("📝 Created {} test blocks with realistic Solana data\n", test_blocks.len());

        // Step 2: Process all blocks
        for (slot, block_data) in &test_blocks {
            self.process_block(*slot, block_data).await?;
        }

        println!("\n📊 Compression Summary:");
        self.print_metrics();

        // Step 3: Retrieve and verify all blocks
        println!("\n🔍 Verification Phase:");
        for (slot, original_data) in &test_blocks {
            let retrieved_data = self.retrieve_block(*slot).await?;

            if retrieved_data != *original_data {
                return Err(WorkflowError::IntegrityCheckFailed {
                    slot: *slot,
                    expected: "original data".to_string(),
                    actual: "retrieved data".to_string(),
                });
            }

            println!("✅ Block {} verified: perfect data integrity", slot);
        }

        // Step 4: Performance analysis
        println!("\n🎯 Performance Analysis:");
        let total_savings = self.metrics.total_original_bytes - self.metrics.total_compressed_bytes;
        let savings_percentage = (total_savings as f64 / self.metrics.total_original_bytes as f64) * 100.0;

        println!("💾 Storage savings: {} bytes ({:.1}%)", total_savings, savings_percentage);
        println!("⚡ Average compression time: {:.2} ms per block",
                 self.metrics.total_compression_time_us as f64 / self.metrics.blocks_processed as f64 / 1000.0);
        println!("📈 Cache efficiency: {:.1}% hit rate",
                 self.metrics.cache_hits as f64 / (self.metrics.cache_hits + self.metrics.cache_misses) as f64 * 100.0);

        println!("\n🎉 Workflow completed successfully! All data integrity checks passed.");

        Ok(())
    }

    /// Get workflow performance metrics
    pub fn get_metrics(&self) -> &WorkflowMetrics {
        &self.metrics
    }

    /// Get access to the compression adapter
    pub fn get_compressor(&self) -> &BlockchainCompressionAdapter {
        &self.compressor
    }

    /// Get compression statistics from the underlying compressor
    pub fn get_compression_stats(&self) -> Result<blockchain_compression::core::traits::CompressionStats, WorkflowError> {
        self.compressor.get_stats()
            .map_err(|e| WorkflowError::CompressionFailed(format!("Failed to get stats: {}", e)))
    }

    // Private helper methods

    fn validate_block_data(&self, data: &[u8]) -> Result<(), WorkflowError> {
        if data.is_empty() {
            return Err(WorkflowError::InvalidBlockData("Block data is empty".to_string()));
        }

        if data.len() > 50_000_000 { // 50MB limit
            return Err(WorkflowError::InvalidBlockData("Block data too large".to_string()));
        }

        Ok(())
    }

    async fn store_compressed_block(&mut self, block: CompressedBlock) -> Result<(), WorkflowError> {
        // In a real implementation, this would store to disk or distributed storage
        self.compressed_storage.insert(block.slot, block);
        Ok(())
    }

    fn calculate_block_hash(&self, data: &[u8]) -> String {
        // Simple hash for demonstration (in production, use a proper cryptographic hash)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        data.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }

    fn update_metrics(&mut self, block: &CompressedBlock) {
        self.metrics.blocks_processed += 1;
        self.metrics.total_original_bytes += block.original_size as u64;
        self.metrics.total_compressed_bytes += block.compressed_size as u64;
        self.metrics.total_compression_time_us += block.compression_time_us;

        // Update average compression ratio
        self.metrics.average_compression_ratio =
            self.metrics.total_original_bytes as f64 / self.metrics.total_compressed_bytes as f64;
    }

    fn cache_data(&mut self, slot: u64, data: Vec<u8>) {
        // Simple cache implementation - just clear if full
        if self.cache.len() >= self.max_cache_size {
            self.cache.clear();
        }
        self.cache.insert(slot, data);
    }

    fn print_metrics(&self) {
        println!("   Blocks processed: {}", self.metrics.blocks_processed);
        println!("   Original size: {} bytes", self.metrics.total_original_bytes);
        println!("   Compressed size: {} bytes", self.metrics.total_compressed_bytes);
        println!("   Average ratio: {:.2}:1", self.metrics.average_compression_ratio);
        println!("   Total compression time: {:.2} ms",
                 self.metrics.total_compression_time_us as f64 / 1000.0);
    }

    fn create_test_blocks(&self) -> Vec<(u64, Vec<u8>)> {
        vec![
            (245000500, self.create_solana_block_data(245000500, 50)),   // Medium block
            (245000501, self.create_solana_block_data(245000501, 100)),  // Large block
            (245000502, self.create_solana_block_data(245000502, 25)),   // Small block
            (245000503, self.create_solana_block_data(245000503, 75)),   // Medium block
            (245000504, self.create_solana_block_data(245000504, 150)),  // Extra large block
        ]
    }

    fn create_solana_block_data(&self, slot: u64, tx_count: usize) -> Vec<u8> {
        let mut block_data = Vec::new();

        // Block header
        block_data.extend_from_slice(b"SOLANA_BLOCK_V1");
        block_data.extend_from_slice(&slot.to_le_bytes());
        block_data.extend_from_slice(&[0; 32]); // Previous blockhash
        block_data.extend_from_slice(&1234567890u64.to_le_bytes()); // Timestamp

        // Transactions (repetitive patterns for good compression)
        for i in 0..tx_count {
            // Transaction signature (64 bytes, but we'll use patterns)
            block_data.extend_from_slice(&[0x01; 64]);

            // Common Solana program IDs (compress very well due to dictionary)
            if i % 3 == 0 {
                block_data.extend_from_slice("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".as_bytes());
            } else if i % 3 == 1 {
                block_data.extend_from_slice("11111111111111111111111111111112".as_bytes());
            } else {
                block_data.extend_from_slice("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".as_bytes());
            }

            // Common transaction amounts
            if i % 4 == 0 {
                block_data.extend_from_slice(&1_000_000_000u64.to_le_bytes()); // 1 SOL
            } else if i % 4 == 1 {
                block_data.extend_from_slice(&100_000_000u64.to_le_bytes());   // 0.1 SOL
            } else {
                block_data.extend_from_slice(&10_000_000u64.to_le_bytes());    // 0.01 SOL
            }

            // Instruction data (repetitive patterns)
            block_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Transfer instruction
            block_data.extend_from_slice(&(i as u32).to_le_bytes());  // Variable data
        }

        // Block metadata
        block_data.extend_from_slice(&tx_count.to_le_bytes());
        block_data.extend_from_slice(&[0xFF; 8]); // End marker

        block_data
    }
}

impl Default for CompressionWorkflow {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow-specific errors
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Invalid block data: {0}")]
    InvalidBlockData(String),

    #[error("Block {0} not found")]
    BlockNotFound(u64),

    #[error("Integrity check failed for block {slot}: expected {expected}, got {actual}")]
    IntegrityCheckFailed {
        slot: u64,
        expected: String,
        actual: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_compression_workflow() {
        let mut workflow = CompressionWorkflow::new();

        // Test block processing
        let test_data = workflow.create_solana_block_data(123456, 10);
        let compressed_block = workflow.process_block(123456, &test_data).await.unwrap();

        assert_eq!(compressed_block.slot, 123456);
        assert_eq!(compressed_block.original_size, test_data.len());
        assert!(compressed_block.compression_ratio > 1.0);

        // Test block retrieval
        let retrieved_data = workflow.retrieve_block(123456).await.unwrap();
        assert_eq!(retrieved_data, test_data);

        // Verify metrics
        let metrics = workflow.get_metrics();
        assert_eq!(metrics.blocks_processed, 1);
        assert_eq!(metrics.total_original_bytes, test_data.len() as u64);
    }

    #[tokio::test]
    async fn test_full_workflow_demo() {
        let mut workflow = CompressionWorkflow::new();

        // Run the complete workflow demonstration
        workflow.demonstrate_workflow().await.unwrap();

        // Verify all blocks were processed
        let metrics = workflow.get_metrics();
        assert_eq!(metrics.blocks_processed, 5);
        assert!(metrics.average_compression_ratio > 10.0); // Should achieve good compression
    }
}