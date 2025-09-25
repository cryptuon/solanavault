//! Hybrid Compression Algorithm
//!
//! Combines the best working components to achieve maximum compression efficiency:
//! - Stage1 structural compression (AccountDictionary + ProgramCluster)
//! - Intelligent strategy selection based on data characteristics
//! - Fallback mechanisms for robust performance

use super::traits::CompressionError;
use super::stage1::{AccountDictionary, ProgramCluster, BlockhashDelta};
// Simple feature analysis without complex dependencies
use serde::{Serialize, Deserialize};

/// Hybrid compression algorithm combining best working strategies
#[derive(Debug, Clone)]
pub struct HybridCompression {
    /// Stage1 structural compression components
    account_dict: AccountDictionary,
    program_cluster: ProgramCluster,
    blockhash_delta: BlockhashDelta,

    /// Simple feature tracking
    _feature_placeholder: u8,

    /// Performance tracking
    compression_stats: HybridStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HybridStats {
    total_compressions: u64,
    stage1_usage: u64,
    fallback_usage: u64,
    total_original_bytes: u64,
    total_compressed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum HybridStrategy {
    /// Use Stage1 compression (AccountDict + ProgramCluster)
    Stage1Structural,
    /// Use simple LZ4 fallback
    LZ4Fallback,
    /// Use pattern-based compression
    PatternBased,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct HybridPackage {
    strategy: HybridStrategy,
    compressed_data: Vec<u8>,
    metadata: Vec<u8>,
}

impl HybridCompression {
    /// Create a new hybrid compression instance
    pub fn new() -> Self {
        Self {
            account_dict: AccountDictionary::new(),
            program_cluster: ProgramCluster::new(),
            blockhash_delta: BlockhashDelta::new(),
            _feature_placeholder: 0,
            compression_stats: HybridStats {
                total_compressions: 0,
                stage1_usage: 0,
                fallback_usage: 0,
                total_original_bytes: 0,
                total_compressed_bytes: 0,
            },
        }
    }

    /// Compress block data using hybrid approach
    pub fn compress_block_data(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let start_time = std::time::Instant::now();

        // Analyze data characteristics to select optimal strategy
        let strategy = self.select_optimal_strategy(data)?;

        let compressed_data = match strategy {
            HybridStrategy::Stage1Structural => {
                self.compress_with_stage1(data)?
            }
            HybridStrategy::LZ4Fallback => {
                self.compress_with_lz4(data)?
            }
            HybridStrategy::PatternBased => {
                self.compress_with_patterns(data)?
            }
        };

        // Package with strategy metadata
        let package = HybridPackage {
            strategy: strategy.clone(),
            compressed_data,
            metadata: Vec::new(), // Future: store strategy-specific metadata
        };

        let serialized = bincode::serialize(&package)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Update statistics
        self.compression_stats.total_compressions += 1;
        self.compression_stats.total_original_bytes += data.len() as u64;
        self.compression_stats.total_compressed_bytes += serialized.len() as u64;

        match strategy {
            HybridStrategy::Stage1Structural => self.compression_stats.stage1_usage += 1,
            _ => self.compression_stats.fallback_usage += 1,
        }

        println!("🔧 Hybrid compression: {} -> {} bytes ({:.2}:1) using {:?} in {:?}",
                 data.len(), serialized.len(),
                 data.len() as f32 / serialized.len() as f32,
                 strategy, start_time.elapsed());

        Ok(serialized)
    }

    /// Decompress block data using hybrid approach
    pub fn decompress_block_data(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Deserialize package to determine strategy
        let package: HybridPackage = bincode::deserialize(data)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let decompressed = match package.strategy {
            HybridStrategy::Stage1Structural => {
                self.decompress_with_stage1(&package.compressed_data)?
            }
            HybridStrategy::LZ4Fallback => {
                self.decompress_with_lz4(&package.compressed_data)?
            }
            HybridStrategy::PatternBased => {
                self.decompress_with_patterns(&package.compressed_data)?
            }
        };

        Ok(decompressed)
    }

    /// Select optimal compression strategy based on data analysis
    fn select_optimal_strategy(&mut self, data: &[u8]) -> Result<HybridStrategy, CompressionError> {
        // Simple but effective strategy selection based on data characteristics
        let repetitive_score = self.analyze_repetitive_patterns(data);
        let transfer_score = self.analyze_transfer_patterns(data);
        let compression_ratio_estimate = self.estimate_compression_potential(data);

        // Strategy selection logic based on our successful testing
        if data.len() > 1000 && repetitive_score > 0.7 {
            // Use Stage1 for larger blocks with high repetitive patterns
            Ok(HybridStrategy::Stage1Structural)
        } else if transfer_score > 0.1 && compression_ratio_estimate > 2.0 {
            // Use Stage1 for transfer-heavy data with good compression potential
            Ok(HybridStrategy::Stage1Structural)
        } else if compression_ratio_estimate > 5.0 {
            // Use pattern-based for highly compressible data
            Ok(HybridStrategy::PatternBased)
        } else {
            // Use LZ4 fallback for other cases
            Ok(HybridStrategy::LZ4Fallback)
        }
    }

    /// Analyze repetitive patterns in data
    fn analyze_repetitive_patterns(&self, data: &[u8]) -> f32 {
        if data.len() < 64 {
            return 0.0;
        }

        let mut repetitive_score = 0.0;
        let sample_size = 1000.min(data.len());

        // Check for 32-byte repetitive patterns (common in Solana)
        for i in 0..sample_size.saturating_sub(64) {
            if i + 64 < data.len() {
                let pattern1 = &data[i..i + 32];
                let pattern2 = &data[i + 32..i + 64];
                if pattern1 == pattern2 {
                    repetitive_score += 1.0;
                }
            }
        }

        repetitive_score / sample_size as f32
    }

    /// Analyze transfer patterns in data
    fn analyze_transfer_patterns(&self, data: &[u8]) -> f32 {
        if data.len() < 100 {
            return 0.0;
        }

        let mut transfer_indicators = 0;
        let sample_size = 1000.min(data.len());

        // Look for transfer instruction patterns
        for i in 0..sample_size.saturating_sub(8) {
            if data[i] == 0x02 || data[i] == 0x03 {
                // Common transfer instruction opcodes
                transfer_indicators += 1;
            }
        }

        transfer_indicators as f32 / sample_size as f32
    }

    /// Estimate compression potential using quick heuristics
    fn estimate_compression_potential(&self, data: &[u8]) -> f32 {
        if data.len() < 100 {
            return 1.0;
        }

        // Count unique bytes
        let mut byte_counts = [0u32; 256];
        for &byte in data {
            byte_counts[byte as usize] += 1;
        }

        let unique_bytes = byte_counts.iter().filter(|&&count| count > 0).count();
        let entropy_ratio = unique_bytes as f32 / 256.0;

        // Lower entropy = higher compression potential
        if entropy_ratio < 0.3 {
            10.0 // High compression potential
        } else if entropy_ratio < 0.6 {
            5.0  // Medium compression potential
        } else {
            2.0  // Low compression potential
        }
    }

    /// Compress using our verified Stage1 components
    fn compress_with_stage1(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Apply Stage1 pipeline in the correct order
        let dict_compressed = self.account_dict.compress_data(data)?;
        let prog_compressed = self.program_cluster.compress_data(&dict_compressed)?;

        // Note: BlockhashDelta has issues with pre-compressed data, so we skip it for now
        // let final_compressed = self.blockhash_delta.compress_data(&prog_compressed)?;

        Ok(prog_compressed)
    }

    /// Decompress using Stage1 components
    fn decompress_with_stage1(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Reverse the Stage1 pipeline
        // let delta_decompressed = self.blockhash_delta.decompress_data(data)?;
        let prog_decompressed = self.program_cluster.decompress_data(data)?;
        let dict_decompressed = self.account_dict.decompress_data(&prog_decompressed)?;

        Ok(dict_decompressed)
    }

    /// Compress using LZ4 fallback
    fn compress_with_lz4(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        lz4::block::compress(data, None, false)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    /// Decompress using LZ4 fallback
    fn decompress_with_lz4(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        lz4::block::decompress(data, Some(100 * 1024 * 1024))
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    /// Compress using pattern-based approach
    fn compress_with_patterns(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Simple pattern compression: find repeating sequences
        let mut compressed = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            // Look for repeating patterns
            let mut best_len = 0;
            let mut best_dist = 0;

            // Look back up to 256 bytes for matches
            let lookback_start = pos.saturating_sub(256);

            for back_pos in lookback_start..pos {
                let mut match_len = 0;
                while pos + match_len < data.len()
                    && back_pos + match_len < pos
                    && data[pos + match_len] == data[back_pos + match_len]
                    && match_len < 255 {
                    match_len += 1;
                }

                if match_len > best_len && match_len >= 3 {
                    best_len = match_len;
                    best_dist = pos - back_pos;
                }
            }

            if best_len >= 3 {
                // Encode as back-reference: marker + distance + length
                compressed.push(0xFF); // Back-reference marker
                compressed.push(best_dist as u8);
                compressed.push(best_len as u8);
                pos += best_len;
            } else {
                // Literal byte
                compressed.push(data[pos]);
                pos += 1;
            }
        }

        Ok(compressed)
    }

    /// Decompress using pattern-based approach
    fn decompress_with_patterns(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut decompressed = Vec::new();
        let mut pos = 0;

        while pos < data.len() {
            if data[pos] == 0xFF && pos + 2 < data.len() {
                // Back-reference
                let distance = data[pos + 1] as usize;
                let length = data[pos + 2] as usize;

                if distance > 0 && distance <= decompressed.len() {
                    let start_pos = decompressed.len() - distance;
                    for i in 0..length {
                        let byte = decompressed[start_pos + (i % distance)];
                        decompressed.push(byte);
                    }
                }
                pos += 3;
            } else {
                // Literal byte
                decompressed.push(data[pos]);
                pos += 1;
            }
        }

        Ok(decompressed)
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> &HybridStats {
        &self.compression_stats
    }

    /// Get overall compression ratio
    pub fn get_compression_ratio(&self) -> f64 {
        if self.compression_stats.total_compressed_bytes > 0 {
            self.compression_stats.total_original_bytes as f64 / self.compression_stats.total_compressed_bytes as f64
        } else {
            1.0
        }
    }
}

impl Default for HybridCompression {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_compression_basic() {
        let mut compressor = HybridCompression::new();

        let test_data = b"Hello, Solana! This is a test of hybrid compression. ".repeat(10);

        let compressed = compressor.compress_block_data(&test_data).unwrap();
        let decompressed = compressor.decompress_block_data(&compressed).unwrap();

        assert_eq!(test_data, decompressed);

        let ratio = compressor.get_compression_ratio();
        println!("Compression ratio: {:.2}:1", ratio);

        // Should achieve some compression
        assert!(ratio > 1.0);
    }

    #[test]
    fn test_hybrid_compression_solana_like() {
        let mut compressor = HybridCompression::new();

        // Generate Solana-like block data
        let mut test_data = Vec::new();

        // Block header
        test_data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]);
        test_data.extend_from_slice(&[0xAA; 32]); // Block hash
        test_data.extend_from_slice(&[0xBB; 32]); // Parent hash
        test_data.extend_from_slice(&1699123456u64.to_le_bytes());
        test_data.extend_from_slice(&50u32.to_le_bytes());

        // Add transactions with repetitive patterns
        for i in 0..50 {
            test_data.extend_from_slice(&[i as u8; 64]); // Signature
            test_data.extend_from_slice(&[(i % 10) as u8; 32]); // Account
            test_data.extend_from_slice(&[0x00; 32]); // System program
            test_data.extend_from_slice(&[0x02, 0x00, 0x00, 0x00]); // Instruction
            test_data.extend_from_slice(&((i as u64 + 1) * 1000).to_le_bytes());
        }

        let compressed = compressor.compress_block_data(&test_data).unwrap();
        let decompressed = compressor.decompress_block_data(&compressed).unwrap();

        assert_eq!(test_data, decompressed);

        let ratio = compressor.get_compression_ratio();
        println!("Solana-like data compression ratio: {:.2}:1", ratio);

        // Should achieve significant compression on repetitive Solana data
        assert!(ratio > 3.0);
    }
}