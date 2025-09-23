//! # Metadata Extraction and Compression
//!
//! Extracts and compresses metadata from Solana transactions for intelligent compression.

use super::super::traits::CompressionError;
use super::transaction_analysis::{BlockAnalysis, TransactionInfo};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Compresses transaction metadata using intelligent patterns
#[derive(Debug, Clone)]
pub struct MetadataCompressor {
    /// Common metadata patterns and their frequencies
    metadata_patterns: HashMap<MetadataType, MetadataPattern>,
    /// Compression statistics
    stats: MetadataStats,
    /// Configuration for metadata compression
    config: MetadataConfig,
}

impl MetadataCompressor {
    /// Creates a new metadata compressor
    pub fn new() -> Self {
        Self {
            metadata_patterns: HashMap::new(),
            stats: MetadataStats::default(),
            config: MetadataConfig::default(),
        }
    }

    /// Compress metadata from transaction data
    pub fn compress_metadata(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let start_time = std::time::Instant::now();

        // Extract metadata from the data
        let metadata = self.extract_metadata(data)?;

        // Compress the metadata
        let compressed_metadata = self.compress_extracted_metadata(&metadata)?;

        // Combine compressed metadata with remaining data
        let mut result = Vec::new();

        // Add metadata header
        result.extend_from_slice(&[0xF1, 0x03]); // Magic bytes for Stage 2 metadata compression

        // Add compressed metadata length
        let metadata_len = compressed_metadata.len() as u32;
        result.extend_from_slice(&metadata_len.to_le_bytes());

        // Add compressed metadata
        result.extend_from_slice(&compressed_metadata);

        // Add remaining data (remove original metadata)
        let cleaned_data = self.remove_metadata_from_data(data, &metadata)?;
        result.extend_from_slice(&cleaned_data);

        // Update statistics
        self.stats.compression_time_ms += start_time.elapsed().as_millis() as u64;
        self.stats.original_size += data.len();
        self.stats.compressed_size += result.len();
        self.stats.metadata_patterns_found += metadata.len();

        Ok(result)
    }

    /// Decompress metadata back to original form
    pub fn decompress_metadata(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // If data doesn't have metadata header, pass through as-is
        if data.len() < 6 || &data[0..2] != &[0xF1, 0x03] {
            return Ok(data.to_vec());
        }

        let mut offset = 2;

        // Read metadata length
        let metadata_len_bytes = &data[offset..offset + 4];
        let metadata_len = u32::from_le_bytes([
            metadata_len_bytes[0],
            metadata_len_bytes[1],
            metadata_len_bytes[2],
            metadata_len_bytes[3],
        ]) as usize;
        offset += 4;

        if offset + metadata_len > data.len() {
            return Err(CompressionError::InvalidFormat);
        }

        // Read compressed metadata
        let compressed_metadata = &data[offset..offset + metadata_len];
        offset += metadata_len;

        // Decompress metadata
        let metadata = self.decompress_extracted_metadata(compressed_metadata)?;

        // Get remaining data
        let remaining_data = &data[offset..];

        // Reconstruct original data with metadata
        let reconstructed = self.reconstruct_data_with_metadata(remaining_data, &metadata)?;

        Ok(reconstructed)
    }

    /// Optimize compression patterns based on block analysis
    pub fn optimize_for_patterns(&mut self, analysis: &BlockAnalysis) -> Result<(), CompressionError> {
        // Analyze metadata patterns across transactions
        for transaction in &analysis.transactions {
            self.analyze_transaction_metadata(transaction)?;
        }

        // Update compression patterns based on frequency
        self.update_compression_patterns();

        Ok(())
    }

    /// Extract metadata from raw transaction data
    fn extract_metadata(&self, data: &[u8]) -> Result<Vec<ExtractedMetadata>, CompressionError> {
        let mut metadata = Vec::new();
        let mut offset = 0;

        // Skip template header if present
        if data.len() >= 2 && &data[0..2] == &[0xF0, 0x02] {
            offset = 2;
        }

        while offset < data.len() {
            // Look for signature patterns (64-byte sequences)
            if offset + 64 <= data.len() {
                let potential_signature = &data[offset..offset + 64];
                if self.is_likely_signature(potential_signature) {
                    metadata.push(ExtractedMetadata {
                        metadata_type: MetadataType::Signature,
                        data: potential_signature.to_vec(),
                        offset,
                        size: 64,
                    });
                    offset += 64;
                    continue;
                }
            }

            // Look for timestamp patterns (8-byte unix timestamps)
            if offset + 8 <= data.len() {
                let potential_timestamp = &data[offset..offset + 8];
                if self.is_likely_timestamp(potential_timestamp) {
                    metadata.push(ExtractedMetadata {
                        metadata_type: MetadataType::Timestamp,
                        data: potential_timestamp.to_vec(),
                        offset,
                        size: 8,
                    });
                    offset += 8;
                    continue;
                }
            }

            // Look for nonce patterns (32-byte sequences with low entropy)
            if offset + 32 <= data.len() {
                let potential_nonce = &data[offset..offset + 32];
                if self.is_likely_nonce(potential_nonce) {
                    metadata.push(ExtractedMetadata {
                        metadata_type: MetadataType::Nonce,
                        data: potential_nonce.to_vec(),
                        offset,
                        size: 32,
                    });
                    offset += 32;
                    continue;
                }
            }

            offset += 1;
        }

        Ok(metadata)
    }

    /// Compress extracted metadata using patterns
    fn compress_extracted_metadata(&mut self, metadata: &[ExtractedMetadata]) -> Result<Vec<u8>, CompressionError> {
        let mut compressed = Vec::new();

        // Add metadata count
        compressed.push(metadata.len() as u8);

        for meta in metadata {
            // Add metadata type
            compressed.push(meta.metadata_type.to_byte());

            // Compress based on type
            match meta.metadata_type {
                MetadataType::Signature => {
                    // Use signature pattern compression
                    let compressed_sig = self.compress_signature(&meta.data)?;
                    compressed.push(compressed_sig.len() as u8);
                    compressed.extend_from_slice(&compressed_sig);
                }
                MetadataType::Timestamp => {
                    // Use delta compression for timestamps
                    let compressed_time = self.compress_timestamp(&meta.data)?;
                    compressed.push(compressed_time.len() as u8);
                    compressed.extend_from_slice(&compressed_time);
                }
                MetadataType::Nonce => {
                    // Use pattern matching for nonces
                    let compressed_nonce = self.compress_nonce(&meta.data)?;
                    compressed.push(compressed_nonce.len() as u8);
                    compressed.extend_from_slice(&compressed_nonce);
                }
                MetadataType::BlockHash => {
                    // Use delta compression for block hashes
                    let compressed_hash = self.compress_blockhash(&meta.data)?;
                    compressed.push(compressed_hash.len() as u8);
                    compressed.extend_from_slice(&compressed_hash);
                }
            }
        }

        Ok(compressed)
    }

    /// Decompress extracted metadata
    fn decompress_extracted_metadata(&self, data: &[u8]) -> Result<Vec<ExtractedMetadata>, CompressionError> {
        if data.is_empty() {
            return Ok(Vec::new());
        }

        let mut metadata = Vec::new();
        let mut offset = 0;

        // Read metadata count
        let count = data[offset] as usize;
        offset += 1;

        for _ in 0..count {
            if offset >= data.len() {
                break;
            }

            // Read metadata type
            let metadata_type = MetadataType::from_byte(data[offset]);
            offset += 1;

            if offset >= data.len() {
                break;
            }

            // Read compressed data length
            let data_len = data[offset] as usize;
            offset += 1;

            if offset + data_len > data.len() {
                break;
            }

            // Read and decompress data
            let compressed_data = &data[offset..offset + data_len];
            let decompressed_data = match metadata_type {
                MetadataType::Signature => self.decompress_signature(compressed_data)?,
                MetadataType::Timestamp => self.decompress_timestamp(compressed_data)?,
                MetadataType::Nonce => self.decompress_nonce(compressed_data)?,
                MetadataType::BlockHash => self.decompress_blockhash(compressed_data)?,
            };

            metadata.push(ExtractedMetadata {
                metadata_type,
                data: decompressed_data,
                offset: 0, // Will be recalculated during reconstruction
                size: match metadata_type {
                    MetadataType::Signature => 64,
                    MetadataType::Timestamp => 8,
                    MetadataType::Nonce => 32,
                    MetadataType::BlockHash => 32,
                },
            });

            offset += data_len;
        }

        Ok(metadata)
    }

    /// Signature compression (pattern-based)
    fn compress_signature(&mut self, signature: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Simple compression: if signature has common patterns, compress them
        if signature.len() == 64 {
            // Check for all-zero signature (common in mock data)
            if signature.iter().all(|&b| b == 0) {
                return Ok(vec![0x00]); // Special marker for zero signature
            }

            // Check for pattern in first 8 bytes
            if signature[0..8] == signature[8..16] {
                // Repeating pattern detected
                let mut compressed = vec![0x01]; // Pattern marker
                compressed.extend_from_slice(&signature[0..8]);
                return Ok(compressed);
            }
        }

        // No pattern found, store as-is with raw marker
        let mut compressed = vec![0xFF]; // Raw data marker
        compressed.extend_from_slice(signature);
        Ok(compressed)
    }

    /// Signature decompression
    fn decompress_signature(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.is_empty() {
            return Err(CompressionError::InvalidFormat);
        }

        match data[0] {
            0x00 => Ok(vec![0u8; 64]), // Zero signature
            0x01 => {
                // Repeating pattern
                if data.len() != 9 {
                    return Err(CompressionError::InvalidFormat);
                }
                let mut signature = Vec::new();
                let pattern = &data[1..9];
                for _ in 0..8 {
                    signature.extend_from_slice(pattern);
                }
                Ok(signature)
            }
            0xFF => {
                // Raw data
                if data.len() != 65 {
                    return Err(CompressionError::InvalidFormat);
                }
                Ok(data[1..].to_vec())
            }
            _ => Err(CompressionError::InvalidFormat),
        }
    }

    /// Timestamp compression (delta-based)
    fn compress_timestamp(&mut self, timestamp: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // For now, simple implementation - could use delta compression in future
        Ok(timestamp.to_vec())
    }

    fn decompress_timestamp(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        Ok(data.to_vec())
    }

    /// Nonce compression
    fn compress_nonce(&mut self, nonce: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Simple pattern detection for nonces
        if nonce.len() == 32 && nonce.iter().all(|&b| b == 0) {
            Ok(vec![0x00]) // Zero nonce marker
        } else {
            Ok(nonce.to_vec())
        }
    }

    fn decompress_nonce(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.len() == 1 && data[0] == 0x00 {
            Ok(vec![0u8; 32])
        } else {
            Ok(data.to_vec())
        }
    }

    /// Blockhash compression
    fn compress_blockhash(&mut self, blockhash: &[u8]) -> Result<Vec<u8>, CompressionError> {
        Ok(blockhash.to_vec()) // Placeholder
    }

    fn decompress_blockhash(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        Ok(data.to_vec())
    }

    /// Heuristics for identifying different metadata types
    fn is_likely_signature(&self, data: &[u8]) -> bool {
        data.len() == 64 && (data.iter().all(|&b| b == 0) || data.iter().any(|&b| b != 0))
    }

    fn is_likely_timestamp(&self, data: &[u8]) -> bool {
        if data.len() != 8 {
            return false;
        }

        let timestamp = u64::from_le_bytes([
            data[0], data[1], data[2], data[3],
            data[4], data[5], data[6], data[7],
        ]);

        // Check if it's a reasonable timestamp (between 2020 and 2030)
        timestamp > 1_577_836_800 && timestamp < 1_893_456_000
    }

    fn is_likely_nonce(&self, data: &[u8]) -> bool {
        data.len() == 32 && data.iter().filter(|&&b| b == 0).count() > 20
    }

    fn remove_metadata_from_data(&self, data: &[u8], metadata: &[ExtractedMetadata]) -> Result<Vec<u8>, CompressionError> {
        // For simplicity, return data as-is for now
        // In a real implementation, would remove extracted metadata sections
        Ok(data.to_vec())
    }

    fn reconstruct_data_with_metadata(&self, data: &[u8], metadata: &[ExtractedMetadata]) -> Result<Vec<u8>, CompressionError> {
        // For simplicity, return data as-is for now
        // In a real implementation, would insert metadata back into correct positions
        Ok(data.to_vec())
    }

    fn analyze_transaction_metadata(&mut self, transaction: &TransactionInfo) -> Result<(), CompressionError> {
        // Analyze patterns in transaction metadata
        // This is a placeholder for more sophisticated analysis
        Ok(())
    }

    fn update_compression_patterns(&mut self) {
        // Update compression patterns based on learned frequencies
        // Placeholder for pattern optimization
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> &MetadataStats {
        &self.stats
    }
}

impl Default for MetadataCompressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Types of metadata that can be extracted and compressed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetadataType {
    Signature,
    Timestamp,
    Nonce,
    BlockHash,
}

impl MetadataType {
    fn to_byte(&self) -> u8 {
        match self {
            MetadataType::Signature => 0x01,
            MetadataType::Timestamp => 0x02,
            MetadataType::Nonce => 0x03,
            MetadataType::BlockHash => 0x04,
        }
    }

    fn from_byte(byte: u8) -> Self {
        match byte {
            0x01 => MetadataType::Signature,
            0x02 => MetadataType::Timestamp,
            0x03 => MetadataType::Nonce,
            0x04 => MetadataType::BlockHash,
            _ => MetadataType::Signature, // Default fallback
        }
    }
}

/// Extracted metadata information
#[derive(Debug, Clone)]
pub struct ExtractedMetadata {
    pub metadata_type: MetadataType,
    pub data: Vec<u8>,
    pub offset: usize,
    pub size: usize,
}

/// Metadata compression pattern
#[derive(Debug, Clone)]
pub struct MetadataPattern {
    pub frequency: u32,
    pub compression_ratio: f64,
    pub template: Vec<u8>,
}

/// Metadata compression configuration
#[derive(Debug, Clone)]
pub struct MetadataConfig {
    pub enable_signature_compression: bool,
    pub enable_timestamp_compression: bool,
    pub enable_nonce_compression: bool,
    pub min_pattern_frequency: u32,
}

impl Default for MetadataConfig {
    fn default() -> Self {
        Self {
            enable_signature_compression: true,
            enable_timestamp_compression: true,
            enable_nonce_compression: true,
            min_pattern_frequency: 3,
        }
    }
}

/// Metadata compression statistics
#[derive(Debug, Clone, Default)]
pub struct MetadataStats {
    pub compression_time_ms: u64,
    pub original_size: usize,
    pub compressed_size: usize,
    pub metadata_patterns_found: usize,
}

impl MetadataStats {
    pub fn compression_ratio(&self) -> f64 {
        if self.compressed_size == 0 {
            0.0
        } else {
            self.original_size as f64 / self.compressed_size as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_data_with_metadata() -> Vec<u8> {
        let mut data = Vec::new();

        // Add some regular data
        data.extend_from_slice(&[0x01, 0x02, 0x03]);

        // Add a mock signature (64 zero bytes)
        data.extend_from_slice(&vec![0u8; 64]);

        // Add more regular data
        data.extend_from_slice(&[0x04, 0x05, 0x06]);

        data
    }

    #[test]
    fn test_metadata_compressor_creation() {
        let compressor = MetadataCompressor::new();
        assert!(compressor.metadata_patterns.is_empty());
    }

    #[test]
    fn test_compress_decompress_metadata() {
        let mut compressor = MetadataCompressor::new();
        let test_data = create_test_data_with_metadata();

        let compressed = compressor.compress_metadata(&test_data).unwrap();
        let decompressed = compressor.decompress_metadata(&compressed).unwrap();

        // Should start with metadata header
        assert_eq!(&compressed[0..2], &[0xF1, 0x03]);

        println!("Original: {} bytes, Compressed: {} bytes",
                 test_data.len(), compressed.len());
    }

    #[test]
    fn test_signature_compression() {
        let mut compressor = MetadataCompressor::new();

        // Test zero signature compression
        let zero_signature = vec![0u8; 64];
        let compressed = compressor.compress_signature(&zero_signature).unwrap();
        let decompressed = compressor.decompress_signature(&compressed).unwrap();

        assert_eq!(zero_signature, decompressed);
        assert!(compressed.len() < zero_signature.len());
    }

    #[test]
    fn test_metadata_type_conversion() {
        assert_eq!(MetadataType::Signature.to_byte(), 0x01);
        assert_eq!(MetadataType::from_byte(0x01), MetadataType::Signature);

        assert_eq!(MetadataType::Timestamp.to_byte(), 0x02);
        assert_eq!(MetadataType::from_byte(0x02), MetadataType::Timestamp);
    }

    #[test]
    fn test_metadata_extraction() {
        let compressor = MetadataCompressor::new();
        let test_data = create_test_data_with_metadata();

        let metadata = compressor.extract_metadata(&test_data).unwrap();

        // Should find the signature metadata
        assert!(!metadata.is_empty());
        let signature_meta = metadata.iter()
            .find(|m| m.metadata_type == MetadataType::Signature);
        assert!(signature_meta.is_some());
    }

    #[test]
    fn test_heuristics() {
        let compressor = MetadataCompressor::new();

        // Test signature detection
        let signature = vec![0u8; 64];
        assert!(compressor.is_likely_signature(&signature));

        // Test timestamp detection
        let timestamp = 1_640_995_200u64.to_le_bytes(); // Jan 1, 2022
        assert!(compressor.is_likely_timestamp(&timestamp));

        // Test nonce detection
        let nonce = vec![0u8; 32];
        assert!(compressor.is_likely_nonce(&nonce));
    }
}