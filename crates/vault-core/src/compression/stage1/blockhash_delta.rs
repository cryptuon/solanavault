//! # Blockhash Delta Compression
//!
//! Compresses Solana blockhashes using delta-of-delta encoding to exploit
//! the predictable evolution patterns in blockhash sequences.

use super::super::traits::CompressionError;
use serde::{Serialize, Deserialize};
use std::collections::VecDeque;

/// Blockhash delta compressor using delta-of-delta encoding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockhashDelta {
    /// Recent blockhashes for delta calculation
    recent_blockhashes: VecDeque<[u8; 32]>,
    /// Maximum number of recent blockhashes to keep
    max_history: usize,
    /// Statistics for compression efficiency
    stats: DeltaStats,
}

impl BlockhashDelta {
    /// Create a new blockhash delta compressor
    pub fn new() -> Self {
        Self {
            recent_blockhashes: VecDeque::new(),
            max_history: 100, // Keep last 100 blockhashes for better delta calculation
            stats: DeltaStats::default(),
        }
    }

    /// Add a blockhash to the history
    pub fn add_blockhash(&mut self, blockhash: [u8; 32]) {
        if self.recent_blockhashes.len() >= self.max_history {
            self.recent_blockhashes.pop_front();
        }
        self.recent_blockhashes.push_back(blockhash);
    }

    /// Compress data by finding and delta-compressing blockhashes
    pub fn compress_data(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.len() < 32 {
            return Ok(data.to_vec());
        }

        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            // Look for 32-byte sequences that could be blockhashes
            if i + 32 <= data.len() {
                if let Ok(bytes) = <[u8; 32]>::try_from(&data[i..i + 32]) {
                    // Check if this looks like a blockhash
                    if self.is_likely_blockhash(&bytes) {
                        // Try to find a delta match
                        if let Some(delta_compressed) = self.try_delta_compress(&bytes) {
                            // Write delta compressed marker (0xFD) followed by delta data
                            compressed.push(0xFD);
                            compressed.extend_from_slice(&delta_compressed);
                            self.add_blockhash(bytes);
                            self.stats.successful_compressions += 1;
                            i += 32;
                            continue;
                        }
                    }
                }
            }

            // If not a blockhash or couldn't compress, copy the byte as-is
            compressed.push(data[i]);
            i += 1;
        }

        Ok(compressed)
    }

    /// Decompress data by reconstructing blockhashes from deltas
    pub fn decompress_data(&mut self, compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < compressed_data.len() {
            if compressed_data[i] == 0xFD && i + 1 < compressed_data.len() {
                // Read delta compression format
                let format = compressed_data[i + 1];
                let mut consumed = 2;

                let reconstructed_hash = match format {
                    0x00 => {
                        // XOR delta format
                        if i + 2 + 32 <= compressed_data.len() {
                            let delta = &compressed_data[i + 2..i + 2 + 32];
                            consumed += 32;
                            self.reconstruct_from_xor_delta(delta)?
                        } else {
                            return Err(CompressionError::InvalidFormat);
                        }
                    }
                    0x01 => {
                        // Incremental delta format (not implemented in this example)
                        return Err(CompressionError::InvalidFormat);
                    }
                    0x02 => {
                        // Reference to recent blockhash
                        if i + 3 <= compressed_data.len() {
                            let reference_index = compressed_data[i + 2];
                            consumed += 1;
                            self.get_reference_blockhash(reference_index)?
                        } else {
                            return Err(CompressionError::InvalidFormat);
                        }
                    }
                    _ => {
                        return Err(CompressionError::InvalidFormat);
                    }
                };

                decompressed.extend_from_slice(&reconstructed_hash);
                self.add_blockhash(reconstructed_hash);
                i += consumed;
            } else {
                decompressed.push(compressed_data[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    /// Try to compress a blockhash using delta encoding
    fn try_delta_compress(&self, blockhash: &[u8; 32]) -> Option<Vec<u8>> {
        if self.recent_blockhashes.is_empty() {
            return None;
        }

        // Try XOR delta with the most recent blockhash
        let most_recent = self.recent_blockhashes.back().unwrap();
        let xor_delta = self.compute_xor_delta(blockhash, most_recent);

        // Check if XOR delta is beneficial (has many zeros)
        let zero_bytes = xor_delta.iter().filter(|&&b| b == 0).count();
        if zero_bytes >= 16 { // If at least half the bytes are zero
            let mut result = vec![0x00]; // XOR delta format
            result.extend_from_slice(&xor_delta);
            return Some(result);
        }

        // Try reference compression - check if this blockhash is in recent history
        for (index, recent_hash) in self.recent_blockhashes.iter().enumerate() {
            if recent_hash == blockhash {
                return Some(vec![0x02, index as u8]); // Reference format
            }
        }

        // Could add more sophisticated delta algorithms here
        None
    }

    /// Compute XOR delta between two blockhashes
    fn compute_xor_delta(&self, hash1: &[u8; 32], hash2: &[u8; 32]) -> [u8; 32] {
        let mut delta = [0u8; 32];
        for i in 0..32 {
            delta[i] = hash1[i] ^ hash2[i];
        }
        delta
    }

    /// Reconstruct blockhash from XOR delta
    fn reconstruct_from_xor_delta(&self, delta: &[u8]) -> Result<[u8; 32], CompressionError> {
        if delta.len() != 32 {
            return Err(CompressionError::InvalidFormat);
        }

        if let Some(most_recent) = self.recent_blockhashes.back() {
            let mut reconstructed = [0u8; 32];
            for i in 0..32 {
                reconstructed[i] = most_recent[i] ^ delta[i];
            }
            Ok(reconstructed)
        } else {
            Err(CompressionError::InvalidFormat)
        }
    }

    /// Get a blockhash by reference index
    fn get_reference_blockhash(&self, index: u8) -> Result<[u8; 32], CompressionError> {
        if let Some(blockhash) = self.recent_blockhashes.get(index as usize) {
            Ok(*blockhash)
        } else {
            Err(CompressionError::InvalidFormat)
        }
    }

    /// Heuristic to determine if bytes look like a blockhash
    fn is_likely_blockhash(&self, bytes: &[u8; 32]) -> bool {
        // Blockhashes are SHA-256 hashes, so they should have good entropy

        // 1. Not all zeros or all 0xFF
        if bytes.iter().all(|&b| b == 0) || bytes.iter().all(|&b| b == 0xFF) {
            return false;
        }

        // 2. Check entropy - count unique bytes
        let unique_bytes = bytes.iter().collect::<std::collections::HashSet<_>>().len();
        if unique_bytes < 8 {
            return false;
        }

        // 3. If we have recent blockhashes, check if this is similar to them
        if let Some(most_recent) = self.recent_blockhashes.back() {
            let xor_delta = self.compute_xor_delta(bytes, most_recent);
            let zero_bytes = xor_delta.iter().filter(|&&b| b == 0).count();

            // If there's significant similarity, likely a blockhash
            if zero_bytes >= 8 {
                return true;
            }
        }

        // 4. For standalone detection, assume good entropy means possible blockhash
        unique_bytes >= 16
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> &DeltaStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = DeltaStats::default();
    }

    /// Get the number of blockhashes in history
    pub fn entry_count(&self) -> usize {
        self.recent_blockhashes.len()
    }

    /// Calculate compression efficiency
    pub fn compression_efficiency(&self) -> f64 {
        if self.stats.successful_compressions == 0 {
            return 1.0;
        }

        // Estimate average compression ratio based on delta patterns
        // XOR deltas with many zeros compress well
        // References compress to ~2 bytes
        let avg_compressed_size = 10.0; // Conservative estimate
        let original_size = 32.0;

        (original_size - avg_compressed_size) / original_size
    }

    /// Clear blockhash history
    pub fn clear_history(&mut self) {
        self.recent_blockhashes.clear();
    }
}

impl Default for BlockhashDelta {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for delta compression
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeltaStats {
    pub successful_compressions: u32,
    pub failed_compressions: u32,
    pub total_bytes_saved: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockhash_delta_basic() {
        let mut compressor = BlockhashDelta::new();

        // Create two similar blockhashes (simulating sequential blocks)
        let mut hash1 = [0u8; 32];
        let mut hash2 = [0u8; 32];

        // Fill with some pattern
        for i in 0..32 {
            hash1[i] = i as u8;
            hash2[i] = i as u8;
        }

        // Make hash2 slightly different
        hash2[31] = 255;

        compressor.add_blockhash(hash1);

        // Test XOR delta calculation
        let delta = compressor.compute_xor_delta(&hash2, &hash1);
        assert_eq!(delta[31], 255); // Only the last byte should be different
        assert_eq!(delta[30], 0);   // Other bytes should be zero
    }

    #[test]
    fn test_compression_decompression() {
        let mut compressor = BlockhashDelta::new();

        // Add a blockhash to history
        let hash1 = [1u8; 32];
        compressor.add_blockhash(hash1);

        // Create test data with a similar blockhash
        let mut hash2 = [1u8; 32];
        hash2[0] = 2; // Slight difference

        let mut test_data = vec![0xFF, 0xFE]; // Some prefix data
        test_data.extend_from_slice(&hash2);
        test_data.push(0xFD); // Some suffix data

        let compressed = compressor.compress_data(&test_data).unwrap();

        // Reset compressor state for decompression test
        let mut decompressor = BlockhashDelta::new();
        decompressor.add_blockhash(hash1); // Add the same history

        let decompressed = decompressor.decompress_data(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_reference_compression() {
        let mut compressor = BlockhashDelta::new();

        let hash1 = [42u8; 32];
        compressor.add_blockhash(hash1);

        // Create test data with the same blockhash (should compress to reference)
        let mut test_data = vec![1, 2, 3];
        test_data.extend_from_slice(&hash1);
        test_data.extend_from_slice(&[4, 5, 6]);

        let compressed = compressor.compress_data(&test_data).unwrap();

        // Should be smaller due to reference compression
        assert!(compressed.len() < test_data.len());

        // Test decompression
        let mut decompressor = BlockhashDelta::new();
        decompressor.add_blockhash(hash1);
        let decompressed = decompressor.decompress_data(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_entropy_detection() {
        let compressor = BlockhashDelta::new();

        // High entropy (good blockhash candidate)
        let high_entropy = [
            0x12, 0x34, 0x56, 0x78, 0x9A, 0xBC, 0xDE, 0xF0,
            0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88,
            0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x00, 0x11,
            0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99,
        ];

        // Low entropy (poor blockhash candidate)
        let low_entropy = [0x00; 32];

        assert!(compressor.is_likely_blockhash(&high_entropy));
        assert!(!compressor.is_likely_blockhash(&low_entropy));
    }
}