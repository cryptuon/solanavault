//! # Stage 2: Bot Intelligence Compression
//!
//! Intelligent compression algorithms that understand Solana transaction patterns
//! and common operations to achieve significant additional compression.

use super::traits::CompressionError;
use serde::{Serialize, Deserialize};

/// Pattern recognition for Solana transactions
pub mod pattern_recognition;

/// Instruction template system for common operations
pub mod instruction_templates;

/// Metadata extraction and compression
pub mod metadata;

/// Transaction structure analysis
pub mod transaction_analysis;

/// Benchmark Stage 2 compression algorithms
pub mod benchmark;

pub use pattern_recognition::PatternRecognizer;
pub use instruction_templates::InstructionTemplateEngine;
pub use metadata::MetadataCompressor;
pub use transaction_analysis::TransactionAnalyzer;

/// Stage 2 Bot Intelligence compressor that understands Solana patterns
#[derive(Debug, Clone)]
pub struct Stage2Compressor {
    pattern_recognizer: PatternRecognizer,
    template_engine: InstructionTemplateEngine,
    metadata_compressor: MetadataCompressor,
    transaction_analyzer: TransactionAnalyzer,
    stats: Stage2Stats,
}

impl Stage2Compressor {
    /// Creates a new Stage 2 compressor with default intelligence
    pub fn new() -> Self {
        Self {
            pattern_recognizer: PatternRecognizer::new(),
            template_engine: InstructionTemplateEngine::new(),
            metadata_compressor: MetadataCompressor::new(),
            transaction_analyzer: TransactionAnalyzer::new(),
            stats: Stage2Stats::default(),
        }
    }

    /// Compress block data using bot intelligence
    pub fn compress_block_data(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let start_time = std::time::Instant::now();

        // Step 1: Analyze transaction structures
        let analysis = self.transaction_analyzer.analyze_block(data)?;
        self.stats.transactions_analyzed += analysis.transaction_count;

        // Step 2: Recognize common patterns
        let patterns = self.pattern_recognizer.find_patterns(&analysis)?;
        self.stats.patterns_found += patterns.len();

        // Step 3: Apply instruction templates
        let templated = self.template_engine.apply_templates(data, &patterns)?;
        self.stats.templates_applied += patterns.iter().map(|p| p.occurrences).sum::<usize>();

        // Step 4: Compress metadata
        let compressed = self.metadata_compressor.compress_metadata(&templated)?;

        self.stats.compression_time_ms += start_time.elapsed().as_millis() as u64;
        self.stats.original_bytes += data.len();
        self.stats.compressed_bytes += compressed.len();

        Ok(compressed)
    }

    /// Decompress block data using bot intelligence
    pub fn decompress_block_data(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let start_time = std::time::Instant::now();

        // Step 1: Decompress metadata
        let metadata_decompressed = self.metadata_compressor.decompress_metadata(data)?;

        // Step 2: Expand instruction templates
        let template_expanded = self.template_engine.expand_templates(&metadata_decompressed)?;

        // Step 3: Reconstruct transaction structures
        let reconstructed = self.transaction_analyzer.reconstruct_block(&template_expanded)?;

        Ok(reconstructed)
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> &Stage2Stats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = Stage2Stats::default();
    }

    /// Train the bot intelligence on new data patterns
    pub fn train_on_data(&mut self, training_data: &[u8]) -> Result<(), CompressionError> {
        let analysis = self.transaction_analyzer.analyze_block(training_data)?;

        // Train pattern recognizer
        self.pattern_recognizer.learn_patterns(&analysis)?;

        // Update instruction templates
        self.template_engine.learn_templates(&analysis)?;

        // Optimize metadata compression
        self.metadata_compressor.optimize_for_patterns(&analysis)?;

        Ok(())
    }
}

impl Default for Stage2Compressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for Stage 2 compression
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stage2Stats {
    pub transactions_analyzed: usize,
    pub patterns_found: usize,
    pub templates_applied: usize,
    pub compression_time_ms: u64,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
}

impl Stage2Stats {
    /// Calculate compression ratio
    pub fn compression_ratio(&self) -> f64 {
        if self.compressed_bytes == 0 {
            0.0
        } else {
            self.original_bytes as f64 / self.compressed_bytes as f64
        }
    }

    /// Get compression percentage
    pub fn compression_percentage(&self) -> f64 {
        if self.original_bytes == 0 {
            0.0
        } else {
            (1.0 - (self.compressed_bytes as f64 / self.original_bytes as f64)) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage2_compressor_creation() {
        let compressor = Stage2Compressor::new();
        assert_eq!(compressor.get_stats().transactions_analyzed, 0);
    }

    #[test]
    fn test_stage2_stats() {
        let mut stats = Stage2Stats::default();
        stats.original_bytes = 1000;
        stats.compressed_bytes = 250;

        assert_eq!(stats.compression_ratio(), 4.0);
        assert_eq!(stats.compression_percentage(), 75.0);
    }
}