//! # Stage 1 Structural Compression
//!
//! The first stage of the compression pipeline targeting 85% compression through
//! Solana-specific optimizations.

pub mod account_dictionary;
pub mod program_clustering;
pub mod blockhash_delta;

pub use account_dictionary::AccountDictionary;
pub use program_clustering::ProgramCluster;
pub use blockhash_delta::BlockhashDelta;

use super::traits::{CompressionStrategy, CompressionVersion, CompressionError};
use crate::data::CachedBlock;
use serde::{Serialize, Deserialize};

/// Stage 1 structural compressor
#[derive(Debug, Clone)]
pub struct Stage1Compressor {
    account_dict: AccountDictionary,
    program_cluster: ProgramCluster,
    blockhash_delta: BlockhashDelta,
}

impl Stage1Compressor {
    /// Create a new Stage 1 compressor
    pub fn new() -> Self {
        Self {
            account_dict: AccountDictionary::new(),
            program_cluster: ProgramCluster::new(),
            blockhash_delta: BlockhashDelta::new(),
        }
    }

    /// Compress raw block data using Stage 1 algorithms
    pub fn compress_block_data(&mut self, block_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // For now, apply basic compression to the raw block data
        // This will be expanded to parse and compress specific fields

        // 1. Apply account dictionary compression (simulate for now)
        let dict_compressed = self.account_dict.compress_data(block_data)?;

        // 2. Apply program clustering
        let prog_compressed = self.program_cluster.compress_data(&dict_compressed)?;

        // 3. Apply blockhash delta compression
        let final_compressed = self.blockhash_delta.compress_data(&prog_compressed)?;

        Ok(final_compressed)
    }

    /// Decompress Stage 1 compressed data
    pub fn decompress_block_data(&mut self, compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Reverse the compression stages
        let blockhash_decompressed = self.blockhash_delta.decompress_data(compressed_data)?;
        let prog_decompressed = self.program_cluster.decompress_data(&blockhash_decompressed)?;
        let final_decompressed = self.account_dict.decompress_data(&prog_decompressed)?;

        Ok(final_decompressed)
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> Stage1Stats {
        Stage1Stats {
            account_dict_entries: self.account_dict.entry_count(),
            program_cluster_entries: self.program_cluster.entry_count(),
            compression_ratio: 0.0, // Calculate from actual compression
        }
    }
}

impl Default for Stage1Compressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for Stage 1 compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage1Stats {
    pub account_dict_entries: usize,
    pub program_cluster_entries: usize,
    pub compression_ratio: f64,
}