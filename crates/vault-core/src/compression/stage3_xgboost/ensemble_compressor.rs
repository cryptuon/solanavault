//! # Ensemble Compression Strategies
//!
//! Multiple specialized compression algorithms working together based on XGBoost predictions.

use super::*;
use std::collections::HashMap;

/// Ensemble of compression strategies optimized by XGBoost
#[derive(Debug, Clone)]
pub struct EnsembleCompressor {
    /// Dictionary-based compressor
    dictionary_compressor: DictionaryCompressor,

    /// Pattern-based compressor
    pattern_compressor: PatternCompressor,

    /// Tree-based compressor
    tree_compressor: TreeBasedCompressor,

    /// Hybrid compressor
    hybrid_compressor: HybridCompressor,

    /// Token transfer specialized compressor
    token_transfer_compressor: TokenTransferCompressor,

    /// Repetitive transaction compressor
    repetitive_compressor: RepetitiveCompressor,

    /// Strategy performance tracking
    strategy_performance: HashMap<CompressionStrategy, StrategyMetrics>,

    /// Configuration
    config: XGBoostConfig,
}

impl EnsembleCompressor {
    /// Creates a new ensemble compressor
    pub fn new() -> Self {
        Self {
            dictionary_compressor: DictionaryCompressor::new(),
            pattern_compressor: PatternCompressor::new(),
            tree_compressor: TreeBasedCompressor::new(),
            hybrid_compressor: HybridCompressor::new(),
            token_transfer_compressor: TokenTransferCompressor::new(),
            repetitive_compressor: RepetitiveCompressor::new(),
            strategy_performance: HashMap::new(),
            config: XGBoostConfig::default(),
        }
    }

    /// Creates ensemble with custom configuration
    pub fn with_config(config: &XGBoostConfig) -> Self {
        Self {
            dictionary_compressor: DictionaryCompressor::new(),
            pattern_compressor: PatternCompressor::new(),
            tree_compressor: TreeBasedCompressor::new(),
            hybrid_compressor: HybridCompressor::new(),
            token_transfer_compressor: TokenTransferCompressor::new(),
            repetitive_compressor: RepetitiveCompressor::new(),
            strategy_performance: HashMap::new(),
            config: config.clone(),
        }
    }

    /// Compress data using specified strategy
    pub fn compress_with_strategy(&mut self, data: &[u8], strategy: &CompressionStrategy) -> Result<Vec<u8>, CompressionError> {
        let start_time = std::time::Instant::now();

        let result = match strategy {
            CompressionStrategy::DictionaryBased => {
                self.dictionary_compressor.compress(data)
            }
            CompressionStrategy::PatternBased => {
                self.pattern_compressor.compress(data)
            }
            CompressionStrategy::TreeBased => {
                self.tree_compressor.compress(data)
            }
            CompressionStrategy::Hybrid => {
                self.hybrid_compressor.compress(data)
            }
            CompressionStrategy::TokenTransfer => {
                self.token_transfer_compressor.compress(data)
            }
            CompressionStrategy::Repetitive => {
                self.repetitive_compressor.compress(data)
            }
        };

        // Track performance
        let compression_time = start_time.elapsed();
        self.update_strategy_performance(strategy, data.len(), result.as_ref().map(|r| r.len()).unwrap_or(data.len()), compression_time);

        result
    }

    /// Decompress data using specified strategy
    pub fn decompress_with_strategy(&self, data: &[u8], strategy: &CompressionStrategy) -> Result<Vec<u8>, CompressionError> {
        match strategy {
            CompressionStrategy::DictionaryBased => {
                self.dictionary_compressor.decompress(data)
            }
            CompressionStrategy::PatternBased => {
                self.pattern_compressor.decompress(data)
            }
            CompressionStrategy::TreeBased => {
                self.tree_compressor.decompress(data)
            }
            CompressionStrategy::Hybrid => {
                self.hybrid_compressor.decompress(data)
            }
            CompressionStrategy::TokenTransfer => {
                self.token_transfer_compressor.decompress(data)
            }
            CompressionStrategy::Repetitive => {
                self.repetitive_compressor.decompress(data)
            }
        }
    }

    /// Train ensemble on dataset
    pub fn train_on_dataset(&mut self, training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        println!("Training ensemble compressors on {} samples", training_data.len());

        // Train each compressor
        self.dictionary_compressor.train(training_data)?;
        self.pattern_compressor.train(training_data)?;
        self.tree_compressor.train(training_data)?;
        self.hybrid_compressor.train(training_data)?;

        Ok(())
    }

    /// Update strategy performance metrics
    fn update_strategy_performance(&mut self, strategy: &CompressionStrategy, original_size: usize, compressed_size: usize, time: std::time::Duration) {
        let metrics = self.strategy_performance.entry(strategy.clone()).or_insert_with(|| StrategyMetrics::default());

        metrics.usage_count += 1;
        metrics.total_original_bytes += original_size;
        metrics.total_compressed_bytes += compressed_size;
        metrics.total_time += time;

        // Update rolling averages
        let ratio = original_size as f32 / compressed_size as f32;
        metrics.average_ratio = (metrics.average_ratio * (metrics.usage_count - 1) as f32 + ratio) / metrics.usage_count as f32;
    }

    /// Get strategy performance metrics
    pub fn get_strategy_performance(&self) -> Vec<StrategyPerformance> {
        self.strategy_performance.iter().map(|(strategy, metrics)| {
            StrategyPerformance {
                strategy: strategy.clone(),
                average_ratio: metrics.average_ratio,
                success_rate: 1.0, // Simplified - could track failures
                usage_count: metrics.usage_count,
            }
        }).collect()
    }
}

impl Default for EnsembleCompressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Strategy performance metrics
#[derive(Debug, Clone, Default)]
struct StrategyMetrics {
    usage_count: usize,
    total_original_bytes: usize,
    total_compressed_bytes: usize,
    total_time: std::time::Duration,
    average_ratio: f32,
}

/// Dictionary-based compression
#[derive(Debug, Clone)]
struct DictionaryCompressor {
    dictionary: Vec<Vec<u8>>,
}

impl DictionaryCompressor {
    fn new() -> Self {
        Self { dictionary: Vec::new() }
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Simple dictionary compression
        let mut compressed = Vec::new();
        compressed.push(0u8); // Strategy marker

        if self.dictionary.is_empty() {
            // No dictionary, use LZ4
            let lz4_compressed = lz4::block::compress(data, None, false)
                .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
            compressed.extend_from_slice(&lz4_compressed);
        } else {
            // Use dictionary for common patterns
            compressed.extend_from_slice(data); // Simplified - would implement actual dictionary compression
        }

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.is_empty() {
            return Err(CompressionError::InvalidFormat);
        }

        match data[0] {
            0 => {
                // Dictionary compression
                if self.dictionary.is_empty() {
                    // LZ4 decompression
                    lz4::block::decompress(&data[1..], Some(1024 * 1024))
                        .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
                } else {
                    // Dictionary decompression
                    Ok(data[1..].to_vec()) // Simplified
                }
            }
            _ => Err(CompressionError::InvalidFormat),
        }
    }

    fn train(&mut self, _training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        // Build dictionary from training data
        // Simplified implementation
        Ok(())
    }
}

/// Pattern-based compression
#[derive(Debug, Clone)]
struct PatternCompressor {
    patterns: Vec<Pattern>,
}

impl PatternCompressor {
    fn new() -> Self {
        Self { patterns: Vec::new() }
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut compressed = Vec::new();
        compressed.push(1u8); // Strategy marker

        // Simple run-length encoding for patterns
        if data.is_empty() {
            return Ok(compressed);
        }

        let mut i = 0;
        while i < data.len() {
            let current_byte = data[i];
            let mut count = 1;

            // Count consecutive bytes
            while i + count < data.len() && data[i + count] == current_byte && count < 255 {
                count += 1;
            }

            if count > 3 {
                // Use RLE for repetitive patterns
                compressed.push(255); // RLE marker
                compressed.push(count as u8);
                compressed.push(current_byte);
                i += count;
            } else {
                // Copy literal bytes
                compressed.push(current_byte);
                i += 1;
            }
        }

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.len() < 2 || data[0] != 1 {
            return Err(CompressionError::InvalidFormat);
        }

        let mut decompressed = Vec::new();
        let mut i = 1;

        while i < data.len() {
            if data[i] == 255 && i + 2 < data.len() {
                // RLE pattern
                let count = data[i + 1] as usize;
                let byte_value = data[i + 2];
                decompressed.extend(vec![byte_value; count]);
                i += 3;
            } else {
                // Literal byte
                decompressed.push(data[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    fn train(&mut self, _training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        // Learn common patterns
        Ok(())
    }
}

/// Tree-based compression
#[derive(Debug, Clone)]
struct TreeBasedCompressor {
    tree_model: Option<CompressionTree>,
}

impl TreeBasedCompressor {
    fn new() -> Self {
        Self { tree_model: None }
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut compressed = Vec::new();
        compressed.push(2u8); // Strategy marker

        // Use context-based compression with tree structure
        if let Some(ref tree) = self.tree_model {
            tree.compress_data(data, &mut compressed)?;
        } else {
            // Fallback to simple compression
            compressed.extend_from_slice(data);
        }

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.is_empty() || data[0] != 2 {
            return Err(CompressionError::InvalidFormat);
        }

        if let Some(ref tree) = self.tree_model {
            tree.decompress_data(&data[1..])
        } else {
            Ok(data[1..].to_vec())
        }
    }

    fn train(&mut self, _training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        // Build compression tree
        Ok(())
    }
}

/// Hybrid compression combining multiple strategies
#[derive(Debug, Clone)]
struct HybridCompressor {}

impl HybridCompressor {
    fn new() -> Self {
        Self {}
    }

    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut compressed = Vec::new();
        compressed.push(3u8); // Strategy marker

        // Try multiple compression methods and use the best
        let lz4_compressed = lz4::block::compress(data, None, false)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // For now, just use LZ4
        compressed.extend_from_slice(&lz4_compressed);

        Ok(compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.is_empty() || data[0] != 3 {
            return Err(CompressionError::InvalidFormat);
        }

        lz4::block::decompress(&data[1..], Some(1024 * 1024))
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    fn train(&mut self, _training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        Ok(())
    }
}

/// Simple pattern representation
#[derive(Debug, Clone)]
struct Pattern {
    bytes: Vec<u8>,
    frequency: usize,
}

/// Compression tree for hierarchical compression
#[derive(Debug, Clone)]
struct CompressionTree {}

impl CompressionTree {
    fn compress_data(&self, _data: &[u8], _output: &mut Vec<u8>) -> Result<(), CompressionError> {
        Ok(())
    }

    fn decompress_data(&self, _data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        Ok(Vec::new())
    }
}

/// Token Transfer specialized compressor for Solana token transactions
#[derive(Debug, Clone)]
struct TokenTransferCompressor {
    /// Common token program accounts
    token_programs: Vec<Vec<u8>>,
    /// Token account patterns
    token_account_patterns: Vec<Vec<u8>>,
    /// Transfer instruction templates
    transfer_templates: Vec<Vec<u8>>,
}

impl TokenTransferCompressor {
    fn new() -> Self {
        Self {
            token_programs: vec![
                vec![0x06; 32], // Standard SPL token program
                vec![0x0B; 32], // Token-2022 program
            ],
            token_account_patterns: Vec::new(),
            transfer_templates: Vec::new(),
        }
    }

    fn count_token_patterns(&self, data: &[u8]) -> usize {
        let mut count = 0;
        let mut position = 0;

        while position + 32 <= data.len() {
            let slice = &data[position..position + 32];

            // Check for known token programs
            if self.token_programs.iter().any(|p| p == slice) {
                count += 1;
            }

            // Check for transfer instructions
            if position + 8 <= data.len() && data[position] == 0x03 {
                count += 1;
            }

            position += 1;
        }

        count
    }

    fn compress(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Always use the 0x10 marker for consistency - no separate pass-through path

        let mut compressed = Vec::new();
        compressed.push(0x10); // Token transfer marker

        // Only use static patterns - no dynamic pattern discovery
        let mut position = 0;
        while position < data.len() {
            // Look for token program IDs (32 bytes)
            if position + 32 <= data.len() {
                let slice = &data[position..position + 32];

                // Check if this is a known token program
                if let Some(index) = self.token_programs.iter().position(|p| p == slice) {
                    compressed.push(0x80 | index as u8); // Program reference
                    position += 32;
                    continue;
                }
            }

            // Look for transfer instruction patterns (8 bytes)
            if position + 8 <= data.len() {
                let slice = &data[position..position + 8];

                // Check for transfer instruction pattern (0x03 followed by amount)
                if slice[0] == 0x03 {
                    compressed.push(0x20); // Transfer instruction marker
                    compressed.extend_from_slice(&slice[1..8]); // Amount (7 bytes)
                    position += 8;
                    continue;
                }
            }

            // Default: copy byte as-is
            compressed.push(data[position]);
            position += 1;
        }

        // Apply final LZ4 compression
        let final_compressed = lz4::block::compress(&compressed, None, false)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(final_compressed)
    }

    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.is_empty() {
            return Err(CompressionError::InvalidFormat);
        }

        // Decompress LZ4 first
        let lz4_decompressed = lz4::block::decompress(data, Some(10 * 1024 * 1024))
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        if lz4_decompressed.is_empty() {
            return Err(CompressionError::InvalidFormat);
        }

        // Only handle token compression (0x10) - no pass-through path

        // Handle token compression (0x10)
        if lz4_decompressed[0] != 0x10 {
            return Err(CompressionError::InvalidFormat);
        }

        let mut decompressed = Vec::new();
        let mut position = 1; // Skip marker

        while position < lz4_decompressed.len() {
            let byte = lz4_decompressed[position];

            if byte & 0x80 != 0 {
                // Program reference
                let index = (byte & 0x7F) as usize;
                if index < self.token_programs.len() {
                    decompressed.extend_from_slice(&self.token_programs[index]);
                } else {
                    // Index out of bounds - this should not happen in valid compressed data
                    return Err(CompressionError::InvalidFormat);
                }
                position += 1;
            } else if byte & 0x40 != 0 {
                // Account reference (currently disabled - treat as regular byte)
                decompressed.push(byte);
                position += 1;
            } else if byte == 0x20 {
                // Transfer instruction
                decompressed.push(0x03); // Transfer opcode
                if position + 7 < lz4_decompressed.len() {
                    decompressed.extend_from_slice(&lz4_decompressed[position + 1..position + 8]);
                    position += 8;
                } else {
                    position += 1;
                }
            } else {
                // Regular byte
                decompressed.push(byte);
                position += 1;
            }
        }

        Ok(decompressed)
    }

    fn train(&mut self, _training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        // Token transfer compressor is mostly rule-based
        Ok(())
    }
}

/// Repetitive transaction compressor for high-frequency patterns
#[derive(Debug, Clone)]
pub struct RepetitiveCompressor {
    /// Detected repetitive patterns
    patterns: Vec<RepetitivePattern>,
    /// Pattern detection threshold
    min_frequency: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RepetitivePattern {
    pattern: Vec<u8>,
    frequency: usize,
    size_bytes: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct RepetitivePackage {
    patterns: Vec<RepetitivePattern>,
    compressed_data: Vec<u8>,
}

impl RepetitiveCompressor {
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            min_frequency: 3, // Minimum 3 occurrences to be considered repetitive
        }
    }

    pub fn compress(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Detect patterns of various sizes (4, 8, 16, 32, 64 bytes)
        let pattern_sizes = [64, 32, 16, 8, 4]; // Try larger patterns first
        self.detect_patterns(data, &pattern_sizes);

        // Create compressed package that includes pattern dictionary
        let package = RepetitivePackage {
            patterns: self.patterns.clone(),
            compressed_data: self.compress_with_patterns(data)?,
        };

        // Serialize the package
        let serialized = bincode::serialize(&package)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Apply LZ4 compression for final output
        let final_compressed = lz4::block::compress(&serialized, None, false)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok(final_compressed)
    }

    fn compress_with_patterns(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut compressed = Vec::new();
        let mut position = 0;

        while position < data.len() {
            let mut matched = false;

            // Try to match against known patterns (largest first)
            for (index, pattern) in self.patterns.iter().enumerate() {
                if position + pattern.pattern.len() <= data.len() {
                    let slice = &data[position..position + pattern.pattern.len()];
                    if slice == pattern.pattern {
                        // Encode pattern reference
                        if index < 127 {
                            compressed.push(0x80 | (index as u8));
                        } else {
                            compressed.push(0xFF); // Extended index marker
                            compressed.extend_from_slice(&(index as u16).to_le_bytes());
                        }
                        position += pattern.pattern.len();
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                // Ensure raw bytes don't conflict with pattern markers
                let byte = data[position];
                if byte >= 0x7F {  // Escape both 0x7F and 0x80+
                    compressed.push(0x7E); // Escape marker (changed from 0x7F)
                    compressed.push(byte);
                } else {
                    compressed.push(byte);
                }
                position += 1;
            }
        }

        Ok(compressed)
    }

    pub fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.is_empty() {
            return Err(CompressionError::InvalidFormat);
        }

        // Decompress LZ4 first
        let lz4_decompressed = lz4::block::decompress(data, Some(10 * 1024 * 1024))
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        // Deserialize the package
        let package: RepetitivePackage = bincode::deserialize(&lz4_decompressed)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        self.decompress_with_patterns(&package.compressed_data, &package.patterns)
    }

    fn decompress_with_patterns(&self, data: &[u8], patterns: &[RepetitivePattern]) -> Result<Vec<u8>, CompressionError> {
        let mut decompressed = Vec::new();
        let mut position = 0;

        while position < data.len() {
            let byte = data[position];

            if byte == 0x7E {
                // Escaped byte
                position += 1;
                if position < data.len() {
                    decompressed.push(data[position]);
                    position += 1;
                }
            } else if byte == 0xFF {
                // Extended index marker
                position += 1;
                if position + 1 < data.len() {
                    let index = u16::from_le_bytes([data[position], data[position + 1]]) as usize;
                    position += 2;
                    if index < patterns.len() {
                        decompressed.extend_from_slice(&patterns[index].pattern);
                    }
                }
            } else if byte & 0x80 != 0 {
                // Pattern reference (short index)
                let index = (byte & 0x7F) as usize;
                position += 1;
                if index < patterns.len() {
                    decompressed.extend_from_slice(&patterns[index].pattern);
                }
            } else {
                // Regular byte
                decompressed.push(byte);
                position += 1;
            }
        }

        Ok(decompressed)
    }

    fn detect_patterns(&mut self, data: &[u8], pattern_sizes: &[usize]) {
        self.patterns.clear();

        for &size in pattern_sizes {
            if size > data.len() { continue; }

            let mut pattern_counts: std::collections::HashMap<Vec<u8>, usize> = std::collections::HashMap::new();

            // Count occurrences of each pattern
            for window in data.windows(size) {
                *pattern_counts.entry(window.to_vec()).or_insert(0) += 1;
            }

            // Collect patterns that meet the frequency threshold
            for (pattern, frequency) in pattern_counts {
                if frequency >= self.min_frequency {
                    self.patterns.push(RepetitivePattern {
                        pattern: pattern.clone(),
                        frequency,
                        size_bytes: pattern.len(),
                    });
                }
            }
        }

        // Sort by potential compression benefit (frequency * size)
        self.patterns.sort_by(|a, b| {
            let benefit_a = a.frequency * a.size_bytes;
            let benefit_b = b.frequency * b.size_bytes;
            benefit_b.cmp(&benefit_a)
        });

        // Keep only the most beneficial patterns (to avoid explosion)
        self.patterns.truncate(100);
    }

    fn train(&mut self, training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        // Analyze training data to build better pattern recognition
        let pattern_sizes = [64, 32, 16, 8, 4];

        for data in training_data {
            self.detect_patterns(data, &pattern_sizes);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ensemble_compressor_creation() {
        let compressor = EnsembleCompressor::new();
        assert_eq!(compressor.strategy_performance.len(), 0);
    }

    #[test]
    fn test_pattern_compression() {
        let compressor = PatternCompressor::new();

        // Test repetitive data
        let repetitive_data = vec![42u8; 100];
        let compressed = compressor.compress(&repetitive_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(repetitive_data, decompressed);
        assert!(compressed.len() < repetitive_data.len());
    }

    #[test]
    fn test_dictionary_compression() {
        let compressor = DictionaryCompressor::new();

        let test_data = b"Hello, World!".to_vec();
        let compressed = compressor.compress(&test_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_hybrid_compression() {
        let compressor = HybridCompressor::new();

        let test_data = b"This is a test of hybrid compression".to_vec();
        let compressed = compressor.compress(&test_data).unwrap();
        let decompressed = compressor.decompress(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }
}