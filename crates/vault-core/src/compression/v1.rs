//! # V1 Compression Algorithm
//!
//! Baseline compression algorithm with a target ratio of 10:1 using Stage 1 algorithms.

use super::traits::{CompressionStrategy, CompressionVersion, CompressionError};
use super::stage1::Stage1Compressor;
use serde::{Serialize, Deserialize};

/// V1 compression algorithm using Stage 1 structural compression
pub struct V1Compression {
    stage1: Stage1Compressor,
}

impl V1Compression {
    /// Creates a new V1 compression instance
    pub fn new() -> Self {
        Self {
            stage1: Stage1Compressor::new(),
        }
    }

    /// Creates a new V1 compression instance with custom Stage 1 compressor
    pub fn with_stage1(stage1: Stage1Compressor) -> Self {
        Self { stage1 }
    }

    /// Get mutable reference to Stage 1 compressor for training/optimization
    pub fn stage1_mut(&mut self) -> &mut Stage1Compressor {
        &mut self.stage1
    }

    /// Get reference to Stage 1 compressor for inspection
    pub fn stage1(&self) -> &Stage1Compressor {
        &self.stage1
    }
}

impl CompressionStrategy for V1Compression {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Apply Stage 1 structural compression
        let mut stage1 = self.stage1.clone(); // Clone for thread safety
        let stage1_compressed = stage1.compress_block_data(data)?;

        // Apply additional basic compression (LZ4 for fast compression)
        let lz4_compressed = lz4::block::compress(&stage1_compressed, None, false)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Create V1 compression package
        let package = V1CompressionPackage {
            version: 1,
            stage1_stats: stage1.get_stats(),
            lz4_data: lz4_compressed,
        };

        // Serialize the package
        let serialized = bincode::serialize(&package)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Add version tag
        let mut result = vec![0u8]; // Version tag for V1
        result.extend_from_slice(&serialized);
        Ok(result)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Check version tag
        if data.is_empty() || data[0] != 0 {
            return Err(CompressionError::InvalidFormat);
        }

        // Deserialize package
        let package: V1CompressionPackage = bincode::deserialize(&data[1..])
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if package.version != 1 {
            return Err(CompressionError::UnsupportedVersion(CompressionVersion::V1));
        }

        // Decompress LZ4 data
        let stage1_data = lz4::block::decompress(&package.lz4_data, None)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Apply Stage 1 decompression
        let mut stage1 = self.stage1.clone();
        stage1.decompress_block_data(&stage1_data)
    }

    fn version(&self) -> CompressionVersion {
        CompressionVersion::V1
    }
}

impl Default for V1Compression {
    fn default() -> Self {
        Self::new()
    }
}

/// V1 compression package format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct V1CompressionPackage {
    version: u8,
    stage1_stats: super::stage1::Stage1Stats,
    lz4_data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v1_compression_basic() {
        let compressor = V1Compression::new();
        let test_data = b"Hello, Solana! This is test data for V1 compression.".to_vec();

        let compressed = compressor.compress(&test_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_v1_compression_with_pubkeys() {
        let compressor = V1Compression::new();

        // Create test data with repeated Solana pubkeys (should compress well)
        let mut test_data = Vec::new();
        let system_program = "11111111111111111111111111111111".parse::<solana_sdk::pubkey::Pubkey>().unwrap();

        // Add the same pubkey multiple times
        for _ in 0..10 {
            test_data.extend_from_slice(system_program.as_ref());
        }

        let original_size = test_data.len();
        let compressed = compressor.compress(&test_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
        // Should achieve some compression due to repeated pubkeys
        println!("Original size: {}, Compressed size: {}, Ratio: {:.2}:1",
                 original_size, compressed.len(),
                 original_size as f64 / compressed.len() as f64);
    }

    #[test]
    fn test_version_handling() {
        let compressor = V1Compression::new();
        assert_eq!(compressor.version(), CompressionVersion::V1);

        // Test invalid version tag
        let invalid_data = vec![99, 1, 2, 3, 4]; // Wrong version tag
        assert!(compressor.decompress(&invalid_data).is_err());
    }
}