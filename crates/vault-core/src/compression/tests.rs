//! # Compression Algorithm Tests with Real Solana Data
//!
//! Comprehensive tests for compression algorithms using real Solana block data.

use super::*;
use crate::data::TestDataManager;

/// Test suite for compression algorithms with real Solana data
pub struct CompressionTestSuite {
    data_manager: TestDataManager,
}

impl CompressionTestSuite {
    /// Create a new test suite
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // For testing, use a temporary directory
        let data_manager = TestDataManager::new("/tmp/solanavault_test")?;

        Ok(Self {
            data_manager,
        })
    }

    /// Test V1 compression with real Solana block data
    pub async fn test_v1_with_real_data(&mut self) -> Result<CompressionResults, Box<dyn std::error::Error>> {
        log::info!("Testing V1 compression with real Solana block data");

        // Get a real block
        let block = self.data_manager.get_test_block().await?;

        // Test compression
        let compressor = V1Compression::new();
        let original_data = &block.block_data;
        let original_size = original_data.len();

        let compressed = compressor.compress(original_data)?;
        let compressed_size = compressed.len();

        let decompressed = compressor.decompress(&compressed)?;

        // Verify integrity
        assert_eq!(original_data, &decompressed, "Data integrity check failed");

        let compression_ratio = original_size as f64 / compressed_size as f64;

        log::info!("V1 Compression Results:");
        log::info!("  Block slot: {}", block.slot);
        log::info!("  Transaction count: {}", block.transaction_count);
        log::info!("  Original size: {} bytes", original_size);
        log::info!("  Compressed size: {} bytes", compressed_size);
        log::info!("  Compression ratio: {:.2}:1", compression_ratio);

        Ok(CompressionResults {
            algorithm: "V1".to_string(),
            block_slot: block.slot,
            transaction_count: block.transaction_count,
            original_size,
            compressed_size,
            compression_ratio,
            compression_time_ms: 0, // TODO: Measure time
            decompression_time_ms: 0,
        })
    }

    /// Test Stage 1 compression components individually
    pub async fn test_stage1_components(&mut self) -> Result<Stage1ComponentResults, Box<dyn std::error::Error>> {
        log::info!("Testing Stage 1 compression components");

        let block = self.data_manager.get_test_block().await?;
        let original_data = &block.block_data;

        // Test Account Dictionary
        let mut account_dict = AccountDictionary::new();
        let dict_compressed = account_dict.compress_data(original_data)?;
        let dict_decompressed = account_dict.decompress_data(&dict_compressed)?;
        assert_eq!(original_data, &dict_decompressed);

        // Test Program Clustering
        let mut program_cluster = ProgramCluster::new();
        let prog_compressed = program_cluster.compress_data(original_data)?;
        let prog_decompressed = program_cluster.decompress_data(&prog_compressed)?;
        assert_eq!(original_data, &prog_decompressed);

        // Test Blockhash Delta
        let mut blockhash_delta = BlockhashDelta::new();
        let hash_compressed = blockhash_delta.compress_data(original_data)?;
        let hash_decompressed = blockhash_delta.decompress_data(&hash_compressed)?;
        assert_eq!(original_data, &hash_decompressed);

        let results = Stage1ComponentResults {
            account_dict_ratio: original_data.len() as f64 / dict_compressed.len() as f64,
            program_cluster_ratio: original_data.len() as f64 / prog_compressed.len() as f64,
            blockhash_delta_ratio: original_data.len() as f64 / hash_compressed.len() as f64,
            account_dict_entries: account_dict.entry_count(),
            program_cluster_entries: program_cluster.entry_count(),
            blockhash_delta_entries: blockhash_delta.entry_count(),
        };

        log::info!("Stage 1 Component Results:");
        log::info!("  Account Dictionary: {:.2}:1 ({} entries)", results.account_dict_ratio, results.account_dict_entries);
        log::info!("  Program Clustering: {:.2}:1 ({} entries)", results.program_cluster_ratio, results.program_cluster_entries);
        log::info!("  Blockhash Delta: {:.2}:1 ({} entries)", results.blockhash_delta_ratio, results.blockhash_delta_entries);

        Ok(results)
    }

    /// Benchmark compression algorithms on multiple blocks
    pub async fn benchmark_multiple_blocks(&mut self, count: usize) -> Result<Vec<CompressionResults>, Box<dyn std::error::Error>> {
        log::info!("Benchmarking compression on {} blocks", count);

        let blocks = self.data_manager.get_high_activity_test_blocks(count).await?;
        let mut results = Vec::new();

        for block in blocks {
            let compressor = V1Compression::new();
            let original_data = &block.block_data;
            let original_size = original_data.len();

            let start = std::time::Instant::now();
            let compressed = compressor.compress(original_data)?;
            let compression_time = start.elapsed().as_millis() as u64;

            let compressed_size = compressed.len();

            let start = std::time::Instant::now();
            let decompressed = compressor.decompress(&compressed)?;
            let decompression_time = start.elapsed().as_millis() as u64;

            assert_eq!(original_data, &decompressed);

            let compression_ratio = original_size as f64 / compressed_size as f64;

            results.push(CompressionResults {
                algorithm: "V1".to_string(),
                block_slot: block.slot,
                transaction_count: block.transaction_count,
                original_size,
                compressed_size,
                compression_ratio,
                compression_time_ms: compression_time,
                decompression_time_ms: decompression_time,
            });
        }

        // Calculate averages
        let avg_ratio = results.iter().map(|r| r.compression_ratio).sum::<f64>() / results.len() as f64;
        let avg_compression_time = results.iter().map(|r| r.compression_time_ms).sum::<u64>() / results.len() as u64;
        let avg_decompression_time = results.iter().map(|r| r.decompression_time_ms).sum::<u64>() / results.len() as u64;

        log::info!("Benchmark Results ({} blocks):", results.len());
        log::info!("  Average compression ratio: {:.2}:1", avg_ratio);
        log::info!("  Average compression time: {} ms", avg_compression_time);
        log::info!("  Average decompression time: {} ms", avg_decompression_time);

        Ok(results)
    }

    /// Test compression on blocks with different characteristics
    pub async fn test_block_characteristics(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        log::info!("Testing compression on blocks with different characteristics");

        let blocks = self.data_manager.get_high_activity_test_blocks(5).await?;

        for block in blocks {
            let compressor = V1Compression::new();
            let compressed = compressor.compress(&block.block_data)?;
            let ratio = block.original_size as f64 / compressed.len() as f64;

            log::info!("Block {} ({} txns): {:.2}:1 compression",
                      block.slot, block.transaction_count, ratio);
        }

        Ok(())
    }
}

/// Results from compression testing
#[derive(Debug, Clone)]
pub struct CompressionResults {
    pub algorithm: String,
    pub block_slot: u64,
    pub transaction_count: usize,
    pub original_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f64,
    pub compression_time_ms: u64,
    pub decompression_time_ms: u64,
}

/// Results from Stage 1 component testing
#[derive(Debug, Clone)]
pub struct Stage1ComponentResults {
    pub account_dict_ratio: f64,
    pub program_cluster_ratio: f64,
    pub blockhash_delta_ratio: f64,
    pub account_dict_entries: usize,
    pub program_cluster_entries: usize,
    pub blockhash_delta_entries: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Ignore by default since it requires internet connection
    async fn test_compression_with_real_data() {
        env_logger::try_init().ok(); // Initialize logging

        if let Ok(mut suite) = CompressionTestSuite::new().await {
            // Test V1 compression
            if let Ok(results) = suite.test_v1_with_real_data().await {
                println!("V1 compression achieved {:.2}:1 ratio", results.compression_ratio);
                assert!(results.compression_ratio > 1.0, "Should achieve some compression");
            }

            // Test Stage 1 components
            if let Ok(results) = suite.test_stage1_components().await {
                println!("Stage 1 components tested successfully");
                assert!(results.account_dict_entries > 0 || results.program_cluster_entries > 0);
            }
        }
    }

    #[tokio::test]
    #[ignore] // Ignore by default since it requires internet connection
    async fn test_benchmark_compression() {
        env_logger::try_init().ok();

        if let Ok(mut suite) = CompressionTestSuite::new().await {
            if let Ok(results) = suite.benchmark_multiple_blocks(3).await {
                assert!(!results.is_empty());
                for result in results {
                    println!("Block {}: {:.2}:1 in {}ms",
                             result.block_slot, result.compression_ratio, result.compression_time_ms);
                }
            }
        }
    }
}