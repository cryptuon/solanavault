//! # V3 Compression Algorithm
//!
//! Advanced compression algorithm with a target ratio of 47:1 using all three stages.

use super::traits::{CompressionStrategy, CompressionVersion, CompressionError};
use super::stage1::Stage1Compressor;
use super::stage2::Stage2Compressor;
use super::stage3::Stage3Compressor;
use serde::{Serialize, Deserialize};

/// V3 compression algorithm using Stage 1 + Stage 2 + Stage 3 with full ML pipeline
pub struct V3Compression {
    stage1: Stage1Compressor,
    stage2: Stage2Compressor,
    stage3: Stage3Compressor,
    trained: bool,
}

impl V3Compression {
    /// Creates a new V3 compression instance
    pub fn new() -> Self {
        Self {
            stage1: Stage1Compressor::new(),
            stage2: Stage2Compressor::new(),
            stage3: Stage3Compressor::new(),
            trained: false,
        }
    }

    /// Creates a new V3 compression instance with custom compressors
    pub fn with_compressors(
        stage1: Stage1Compressor,
        stage2: Stage2Compressor,
        stage3: Stage3Compressor,
    ) -> Self {
        Self {
            stage1,
            stage2,
            stage3,
            trained: false,
        }
    }

    /// Train the full ML pipeline on a dataset
    pub fn train_on_dataset(&mut self, training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        println!("Training V3 compression on {} samples", training_data.len());

        // Train Stage 2 (bot intelligence)
        for data in training_data {
            self.stage2.train_on_data(data)?;
        }

        // Train Stage 3 (machine learning)
        self.stage3.train_on_dataset(training_data)?;

        self.trained = true;
        println!("✅ V3 training completed - ready for maximum compression!");
        Ok(())
    }

    /// Check if the compressor has been trained
    pub fn is_trained(&self) -> bool {
        self.trained
    }

    /// Get comprehensive compression statistics
    pub fn get_comprehensive_stats(&self) -> V3CompressionStats {
        V3CompressionStats {
            stage1_stats: self.stage1.get_stats().clone(),
            stage2_stats: self.stage2.get_stats().clone(),
            stage3_stats: self.stage3.get_stats().clone(),
            trained: self.trained,
        }
    }
}

impl CompressionStrategy for V3Compression {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Full 3-stage compression pipeline targeting 47:1 ratio

        // Step 1: Apply Stage 1 structural compression (target: ~7:1)
        let mut stage1 = self.stage1.clone();
        let stage1_compressed = stage1.compress_block_data(data)?;

        // Step 2: Apply Stage 2 bot intelligence compression (target: additional ~3:1)
        let mut stage2 = self.stage2.clone();
        let stage2_compressed = stage2.compress_block_data(&stage1_compressed)?;

        // Step 3: Apply Stage 3 machine learning compression (target: additional ~2.2:1)
        let mut stage3 = self.stage3.clone();
        let stage3_compressed = stage3.compress_block_data(&stage2_compressed)?;

        // Step 4: Apply final LZ4 compression for any remaining patterns
        let lz4_compressed = lz4::block::compress(&stage3_compressed, None, false)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Create V3 compression package
        let package = V3CompressionPackage {
            version: 3,
            stage1_stats: stage1.get_stats().clone(),
            stage1_compressor: stage1, // Save the complete compressor state for Stage 1 only
            stage2_stats: stage2.get_stats().clone(),
            stage3_stats: stage3.get_stats().clone(),
            trained: self.trained,
            lz4_data: lz4_compressed,
        };

        // Serialize the package
        let serialized = bincode::serialize(&package)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Add version tag
        let mut result = vec![2u8]; // Version tag for V3
        result.extend_from_slice(&serialized);
        Ok(result)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Check version tag
        if data.is_empty() || data[0] != 2 {
            return Err(CompressionError::InvalidFormat);
        }

        // Deserialize package
        let package: V3CompressionPackage = bincode::deserialize(&data[1..])
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if package.version != 3 {
            return Err(CompressionError::UnsupportedVersion(CompressionVersion::V3));
        }

        // Step 1: Decompress LZ4 data
        let stage3_data = lz4::block::decompress(&package.lz4_data, Some(100 * 1024 * 1024))
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Step 2: Apply Stage 3 decompression
        let stage3 = self.stage3.clone();
        let stage2_data = stage3.decompress_block_data(&stage3_data)?;

        // Step 3: Apply Stage 2 decompression
        let stage2 = self.stage2.clone();
        let stage1_data = stage2.decompress_block_data(&stage2_data)?;

        // Step 4: Apply Stage 1 decompression using saved state
        let mut stage1 = package.stage1_compressor;
        stage1.decompress_block_data(&stage1_data)
    }

    fn version(&self) -> CompressionVersion {
        CompressionVersion::V3
    }
}

impl Default for V3Compression {
    fn default() -> Self {
        Self::new()
    }
}

/// V3 compression package format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct V3CompressionPackage {
    version: u8,
    stage1_stats: super::stage1::Stage1Stats,
    stage1_compressor: super::stage1::Stage1Compressor, // Save the complete state
    stage2_stats: super::stage2::Stage2Stats,
    stage3_stats: super::stage3::Stage3Stats,
    trained: bool,
    lz4_data: Vec<u8>,
}

/// Comprehensive V3 compression statistics
#[derive(Debug, Clone)]
pub struct V3CompressionStats {
    pub stage1_stats: super::stage1::Stage1Stats,
    pub stage2_stats: super::stage2::Stage2Stats,
    pub stage3_stats: super::stage3::Stage3Stats,
    pub trained: bool,
}

impl V3CompressionStats {
    /// Calculate overall compression ratio
    pub fn overall_compression_ratio(&self) -> f64 {
        if self.stage3_stats.compressed_bytes == 0 {
            0.0
        } else {
            self.stage3_stats.original_bytes as f64 / self.stage3_stats.compressed_bytes as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v3_compression_basic() {
        let compressor = V3Compression::new();
        let test_data = b"Hello, Solana! V3 compression test.".to_vec();

        let compressed = compressor.compress(&test_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_v3_version_handling() {
        let compressor = V3Compression::new();
        assert_eq!(compressor.version(), CompressionVersion::V3);
    }

    #[test]
    fn test_v3_training_status() {
        let mut compressor = V3Compression::new();
        assert!(!compressor.is_trained());

        let training_data = vec![vec![1, 2, 3, 4]; 5];
        compressor.train_on_dataset(&training_data).unwrap();
        assert!(compressor.is_trained());
    }
}