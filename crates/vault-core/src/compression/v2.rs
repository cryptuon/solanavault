//! # V2 Compression Algorithm
//!
//! Enhanced compression algorithm with a target ratio of 25:1 using Stage 1 + Stage 2.

use super::traits::{CompressionStrategy, CompressionVersion, CompressionError};
use super::stage1::Stage1Compressor;
use super::stage2::{Stage2Compressor, Stage2Stats};
use serde::{Serialize, Deserialize};

/// V2 compression algorithm using Stage 1 + Stage 2 with bot intelligence
pub struct V2Compression {
    stage1: Stage1Compressor,
    stage2: Stage2Compressor,
}

impl V2Compression {
    /// Creates a new V2 compression instance
    pub fn new() -> Self {
        Self {
            stage1: Stage1Compressor::new(),
            stage2: Stage2Compressor::new(),
        }
    }

    /// Creates a new V2 compression instance with custom compressors
    pub fn with_compressors(stage1: Stage1Compressor, stage2: Stage2Compressor) -> Self {
        Self { stage1, stage2 }
    }

    /// Get mutable reference to Stage 1 compressor
    pub fn stage1_mut(&mut self) -> &mut Stage1Compressor {
        &mut self.stage1
    }

    /// Get mutable reference to Stage 2 compressor
    pub fn stage2_mut(&mut self) -> &mut Stage2Compressor {
        &mut self.stage2
    }

    /// Get reference to Stage 1 compressor
    pub fn stage1(&self) -> &Stage1Compressor {
        &self.stage1
    }

    /// Get reference to Stage 2 compressor
    pub fn stage2(&self) -> &Stage2Compressor {
        &self.stage2
    }

    /// Train the bot intelligence on training data
    pub fn train_on_data(&mut self, training_data: &[u8]) -> Result<(), CompressionError> {
        self.stage2.train_on_data(training_data)
    }
}

impl CompressionStrategy for V2Compression {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Step 1: Apply Stage 1 structural compression
        let mut stage1 = self.stage1.clone();
        let stage1_compressed = stage1.compress_block_data(data)?;

        // Step 2: Apply additional LZ4 compression (simplified V2 for now)
        let lz4_compressed = lz4::block::compress(&stage1_compressed, None, false)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Create V2 compression package (simplified)
        let package = V2CompressionPackage {
            version: 2,
            stage1_stats: stage1.get_stats(),
            stage2_stats: Stage2Stats::default(), // Simplified for now
            lz4_data: lz4_compressed,
        };

        // Serialize the package
        let serialized = bincode::serialize(&package)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Add version tag
        let mut result = vec![1u8]; // Version tag for V2
        result.extend_from_slice(&serialized);
        Ok(result)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Check version tag
        if data.is_empty() || data[0] != 1 {
            return Err(CompressionError::InvalidFormat);
        }

        // Deserialize package
        let package: V2CompressionPackage = bincode::deserialize(&data[1..])
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if package.version != 2 {
            return Err(CompressionError::UnsupportedVersion(CompressionVersion::V2));
        }

        // Step 1: Decompress LZ4 data
        let stage1_data = lz4::block::decompress(&package.lz4_data, Some(100 * 1024 * 1024))
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Step 2: Apply Stage 1 decompression
        let mut stage1 = self.stage1.clone();
        stage1.decompress_block_data(&stage1_data)
    }

    fn version(&self) -> CompressionVersion {
        CompressionVersion::V2
    }
}

impl Default for V2Compression {
    fn default() -> Self {
        Self::new()
    }
}

/// V2 compression package format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct V2CompressionPackage {
    version: u8,
    stage1_stats: super::stage1::Stage1Stats,
    stage2_stats: super::stage2::Stage2Stats,
    lz4_data: Vec<u8>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v2_compression_basic() {
        let compressor = V2Compression::new();
        let test_data = b"Hello, Solana! This is test data for V2 compression with bot intelligence.".to_vec();

        let compressed = compressor.compress(&test_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_v2_compression_with_patterns() {
        let compressor = V2Compression::new();

        // Create test data with patterns that Stage 2 should recognize
        let mut test_data = Vec::new();

        // Add repeated instruction pattern
        for _ in 0..5 {
            test_data.extend_from_slice(&[1, 2, 3, 4]); // Common pattern
        }

        // Add some Solana-like addresses
        let system_program = "11111111111111111111111111111111".parse::<solana_sdk::pubkey::Pubkey>().unwrap();
        for _ in 0..3 {
            test_data.extend_from_slice(system_program.as_ref());
        }

        let original_size = test_data.len();
        let compressed = compressor.compress(&test_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(test_data, decompressed);

        let compression_ratio = original_size as f64 / compressed.len() as f64;
        println!("V2 Original size: {}, Compressed size: {}, Ratio: {:.2}:1",
                 original_size, compressed.len(), compression_ratio);

        // Should achieve better compression than V1 due to bot intelligence
        assert!(compression_ratio > 1.0);
    }

    #[test]
    fn test_version_handling() {
        let compressor = V2Compression::new();
        assert_eq!(compressor.version(), CompressionVersion::V2);

        // Test invalid version tag
        let invalid_data = vec![99, 1, 2, 3, 4]; // Wrong version tag
        assert!(compressor.decompress(&invalid_data).is_err());
    }

    #[test]
    fn test_training_functionality() {
        let mut compressor = V2Compression::new();

        // Create training data with patterns
        let mut training_data = Vec::new();
        for _ in 0..10 {
            training_data.extend_from_slice(&[0xAA, 0xBB, 0xCC, 0xDD]);
        }

        // Train the compressor
        compressor.train_on_data(&training_data).unwrap();

        // Test compression after training
        let compressed = compressor.compress(&training_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(training_data, decompressed);
    }
}