//! # Feature Importance Analysis
//!
//! Analyzes which features are most important for compression decisions.

use super::*;
use std::collections::HashMap;

/// Feature importance analyzer for compression
#[derive(Debug, Clone)]
pub struct FeatureImportanceAnalyzer {
    /// Feature importance scores
    importance_scores: HashMap<String, f32>,

    /// Feature usage tracking
    feature_usage: HashMap<String, usize>,

    /// Configuration
    config: XGBoostConfig,
}

impl FeatureImportanceAnalyzer {
    /// Creates a new feature importance analyzer
    pub fn new() -> Self {
        Self {
            importance_scores: HashMap::new(),
            feature_usage: HashMap::new(),
            config: XGBoostConfig::default(),
        }
    }

    /// Creates analyzer with custom configuration
    pub fn with_config(config: &XGBoostConfig) -> Self {
        Self {
            importance_scores: HashMap::new(),
            feature_usage: HashMap::new(),
            config: config.clone(),
        }
    }

    /// Analyze features for given data
    pub fn analyze_features(&mut self, data: &[u8]) -> Result<Vec<FeatureImportanceScore>, CompressionError> {
        let mut features = Vec::new();

        // Extract and analyze various features
        features.extend(self.analyze_statistical_features(data)?);
        features.extend(self.analyze_pattern_features(data)?);
        features.extend(self.analyze_structural_features(data)?);
        features.extend(self.analyze_blockchain_features(data)?);
        features.extend(self.analyze_token_transfer_features(data)?);
        features.extend(self.analyze_repetitive_features(data)?);

        // Update usage tracking
        for feature in &features {
            *self.feature_usage.entry(feature.feature_name.clone()).or_insert(0) += 1;
        }

        Ok(features)
    }

    /// Analyze statistical features
    fn analyze_statistical_features(&self, data: &[u8]) -> Result<Vec<FeatureImportanceScore>, CompressionError> {
        let mut features = Vec::new();

        // Entropy
        let entropy = self.calculate_entropy(data);
        features.push(FeatureImportanceScore {
            feature_name: "entropy".to_string(),
            importance: entropy / 8.0, // Normalize
            gain: entropy * 0.1,
            cover: data.len() as f32,
        });

        // Mean
        let mean = data.iter().map(|&b| b as f32).sum::<f32>() / data.len() as f32;
        features.push(FeatureImportanceScore {
            feature_name: "mean_value".to_string(),
            importance: mean / 255.0,
            gain: 0.05,
            cover: data.len() as f32,
        });

        // Variance
        let variance = data.iter().map(|&b| (b as f32 - mean).powi(2)).sum::<f32>() / data.len() as f32;
        features.push(FeatureImportanceScore {
            feature_name: "variance".to_string(),
            importance: (variance.sqrt() / 127.5).min(1.0),
            gain: 0.08,
            cover: data.len() as f32,
        });

        Ok(features)
    }

    /// Analyze pattern features
    fn analyze_pattern_features(&self, data: &[u8]) -> Result<Vec<FeatureImportanceScore>, CompressionError> {
        let mut features = Vec::new();

        // Repetition ratio
        let repetition_ratio = self.calculate_repetition_ratio(data);
        features.push(FeatureImportanceScore {
            feature_name: "repetition_ratio".to_string(),
            importance: repetition_ratio,
            gain: repetition_ratio * 0.2,
            cover: data.len() as f32,
        });

        // Unique byte count
        let unique_bytes = data.iter().collect::<std::collections::HashSet<_>>().len();
        features.push(FeatureImportanceScore {
            feature_name: "unique_bytes".to_string(),
            importance: (unique_bytes as f32 / 256.0),
            gain: 0.1,
            cover: data.len() as f32,
        });

        // Pattern complexity
        let complexity = self.calculate_pattern_complexity(data);
        features.push(FeatureImportanceScore {
            feature_name: "pattern_complexity".to_string(),
            importance: complexity,
            gain: complexity * 0.15,
            cover: data.len() as f32,
        });

        Ok(features)
    }

    /// Analyze structural features
    fn analyze_structural_features(&self, data: &[u8]) -> Result<Vec<FeatureImportanceScore>, CompressionError> {
        let mut features = Vec::new();

        // Data size
        features.push(FeatureImportanceScore {
            feature_name: "data_size".to_string(),
            importance: (data.len() as f32 / 10000.0).min(1.0),
            gain: 0.05,
            cover: data.len() as f32,
        });

        // Zero byte ratio
        let zero_count = data.iter().filter(|&&b| b == 0).count();
        let zero_ratio = zero_count as f32 / data.len() as f32;
        features.push(FeatureImportanceScore {
            feature_name: "zero_ratio".to_string(),
            importance: zero_ratio,
            gain: zero_ratio * 0.3,
            cover: data.len() as f32,
        });

        // High value byte ratio (potential addresses/hashes)
        let high_value_count = data.iter().filter(|&&b| b > 200).count();
        let high_value_ratio = high_value_count as f32 / data.len() as f32;
        features.push(FeatureImportanceScore {
            feature_name: "high_value_ratio".to_string(),
            importance: high_value_ratio,
            gain: high_value_ratio * 0.1,
            cover: data.len() as f32,
        });

        Ok(features)
    }

    /// Calculate data entropy
    fn calculate_entropy(&self, data: &[u8]) -> f32 {
        let mut counts = [0u32; 256];
        for &byte in data {
            counts[byte as usize] += 1;
        }

        let len = data.len() as f32;
        let mut entropy = 0.0;

        for &count in &counts {
            if count > 0 {
                let p = count as f32 / len;
                entropy -= p * p.log2();
            }
        }

        entropy
    }

    /// Calculate repetition ratio
    fn calculate_repetition_ratio(&self, data: &[u8]) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let mut repetitions = 0;
        for i in 1..data.len() {
            if data[i] == data[i-1] {
                repetitions += 1;
            }
        }

        repetitions as f32 / (data.len() - 1) as f32
    }

    /// Calculate pattern complexity
    fn calculate_pattern_complexity(&self, data: &[u8]) -> f32 {
        if data.len() < 3 {
            return 0.0;
        }

        let mut pattern_changes = 0;
        for i in 2..data.len() {
            let diff1 = data[i-1] as i32 - data[i-2] as i32;
            let diff2 = data[i] as i32 - data[i-1] as i32;

            if diff1 != diff2 {
                pattern_changes += 1;
            }
        }

        pattern_changes as f32 / (data.len() - 2) as f32
    }

    /// Get importance rankings
    pub fn get_importance_rankings(&self) -> Vec<FeatureImportanceScore> {
        let mut rankings: Vec<_> = self.importance_scores.iter().map(|(name, &importance)| {
            FeatureImportanceScore {
                feature_name: name.clone(),
                importance,
                gain: importance * 0.1,
                cover: *self.feature_usage.get(name).unwrap_or(&0) as f32,
            }
        }).collect();

        rankings.sort_by(|a, b| b.importance.partial_cmp(&a.importance).unwrap());
        rankings
    }

    /// Update importance scores
    pub fn update_importance(&mut self, feature_name: String, importance: f32) {
        self.importance_scores.insert(feature_name, importance);
    }
}

impl Default for FeatureImportanceAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

impl FeatureImportanceAnalyzer {
    /// Analyze blockchain-specific features
    fn analyze_blockchain_features(&self, data: &[u8]) -> Result<Vec<FeatureImportanceScore>, CompressionError> {
        let mut features = Vec::new();

        // Solana account size pattern detection (32-byte accounts)
        let account_pattern_density = self.calculate_account_pattern_density(data);
        features.push(FeatureImportanceScore {
            feature_name: "solana_account_density".to_string(),
            importance: account_pattern_density,
            gain: account_pattern_density * 0.3, // High gain for blockchain patterns
            cover: data.len() as f32,
        });

        // Program ID repetition (common programs like SPL Token)
        let program_repetition = self.calculate_program_repetition(data);
        features.push(FeatureImportanceScore {
            feature_name: "program_id_repetition".to_string(),
            importance: program_repetition,
            gain: program_repetition * 0.25,
            cover: data.len() as f32,
        });

        // Instruction data patterns
        let instruction_pattern_score = self.calculate_instruction_patterns(data);
        features.push(FeatureImportanceScore {
            feature_name: "instruction_patterns".to_string(),
            importance: instruction_pattern_score,
            gain: instruction_pattern_score * 0.2,
            cover: data.len() as f32,
        });

        Ok(features)
    }

    /// Analyze token transfer specific features
    fn analyze_token_transfer_features(&self, data: &[u8]) -> Result<Vec<FeatureImportanceScore>, CompressionError> {
        let mut features = Vec::new();

        // SPL Token program detection (TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA)
        let spl_token_density = self.calculate_spl_token_density(data);
        features.push(FeatureImportanceScore {
            feature_name: "spl_token_density".to_string(),
            importance: spl_token_density,
            gain: spl_token_density * 0.4, // Very high gain for token transfers
            cover: data.len() as f32,
        });

        // Transfer instruction frequency (instruction discriminator 0x03)
        let transfer_instruction_freq = self.calculate_transfer_instruction_frequency(data);
        features.push(FeatureImportanceScore {
            feature_name: "transfer_instruction_frequency".to_string(),
            importance: transfer_instruction_freq,
            gain: transfer_instruction_freq * 0.35,
            cover: data.len() as f32,
        });

        // Token account pattern similarity
        let token_account_similarity = self.calculate_token_account_similarity(data);
        features.push(FeatureImportanceScore {
            feature_name: "token_account_similarity".to_string(),
            importance: token_account_similarity,
            gain: token_account_similarity * 0.3,
            cover: data.len() as f32,
        });

        // Amount field patterns (8-byte amounts)
        let amount_pattern_score = self.calculate_amount_patterns(data);
        features.push(FeatureImportanceScore {
            feature_name: "amount_patterns".to_string(),
            importance: amount_pattern_score,
            gain: amount_pattern_score * 0.25,
            cover: data.len() as f32,
        });

        Ok(features)
    }

    /// Analyze repetitive pattern features
    fn analyze_repetitive_features(&self, data: &[u8]) -> Result<Vec<FeatureImportanceScore>, CompressionError> {
        let mut features = Vec::new();

        // Multi-scale repetition analysis
        let repetition_scales = [4, 8, 16, 32, 64];
        for &scale in &repetition_scales {
            let repetition_score = self.calculate_multi_scale_repetition(data, scale);
            features.push(FeatureImportanceScore {
                feature_name: format!("repetition_scale_{}", scale),
                importance: repetition_score,
                gain: repetition_score * 0.3,
                cover: data.len() as f32,
            });
        }

        // Sequence periodicity detection
        let periodicity_score = self.calculate_sequence_periodicity(data);
        features.push(FeatureImportanceScore {
            feature_name: "sequence_periodicity".to_string(),
            importance: periodicity_score,
            gain: periodicity_score * 0.35,
            cover: data.len() as f32,
        });

        // High-frequency trading pattern detection
        let hft_pattern_score = self.calculate_hft_patterns(data);
        features.push(FeatureImportanceScore {
            feature_name: "hft_patterns".to_string(),
            importance: hft_pattern_score,
            gain: hft_pattern_score * 0.4,
            cover: data.len() as f32,
        });

        Ok(features)
    }

    // Helper methods for blockchain-specific feature calculation

    fn calculate_account_pattern_density(&self, data: &[u8]) -> f32 {
        if data.len() < 32 { return 0.0; }

        let mut account_like_patterns = 0;
        let max_samples = 1000.min(data.len() / 32); // Limit sampling for performance
        let step_size = (data.len() / 32).max(1) / max_samples.max(1);

        // Sample every step_size positions instead of checking all
        for i in (0..=data.len().saturating_sub(32)).step_by(step_size) {
            let slice = &data[i..i + 32];

            // Optimized non-zero count
            let non_zero_bytes = slice.iter().take_while(|&&b| b != 0).count() +
                                  slice.iter().rev().take_while(|&&b| b != 0).count();

            if non_zero_bytes > 4 && non_zero_bytes < 30 {
                account_like_patterns += 1;
            }
        }

        (account_like_patterns as f32) / (max_samples as f32)
    }

    fn calculate_program_repetition(&self, data: &[u8]) -> f32 {
        if data.len() < 64 { return 0.0; }

        let mut program_id_counts = std::collections::HashMap::new();
        let max_chunks = 50.min(data.len() / 32); // Limit to first 50 chunks for performance

        // Count first 50 32-byte patterns that could be program IDs
        for i in (0..data.len().saturating_sub(31)).step_by(32).take(max_chunks) {
            let slice = &data[i..i + 32];

            // Use first 8 bytes as hash key for efficiency (instead of full 32 bytes)
            if slice.len() >= 8 {
                let key = u64::from_le_bytes([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7]]);
                *program_id_counts.entry(key).or_insert(0) += 1;
            }
        }

        // Calculate repetition score
        let max_repetition = program_id_counts.values().max().unwrap_or(&1);
        (*max_repetition as f32) / (max_chunks as f32)
    }

    fn calculate_instruction_patterns(&self, data: &[u8]) -> f32 {
        if data.is_empty() { return 0.0; }

        let common_instructions = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05]; // Common Solana instruction discriminators
        let mut instruction_matches = 0;

        for &byte in data {
            if common_instructions.contains(&byte) {
                instruction_matches += 1;
            }
        }

        (instruction_matches as f32) / (data.len() as f32)
    }

    fn calculate_spl_token_density(&self, data: &[u8]) -> f32 {
        if data.len() < 32 { return 0.0; }

        // SPL Token program ID pattern
        let spl_token_pattern = [0x06, 0xdd, 0xf6, 0xe1, 0xd7, 0x65, 0xa1, 0x93];
        let mut spl_matches = 0;

        for window in data.windows(8) {
            if window == &spl_token_pattern {
                spl_matches += 1;
            }
        }

        (spl_matches as f32) / ((data.len() / 8) as f32).max(1.0)
    }

    fn calculate_transfer_instruction_frequency(&self, data: &[u8]) -> f32 {
        if data.is_empty() { return 0.0; }

        let transfer_discriminator = 0x03; // Transfer instruction discriminator
        let transfer_count = data.iter().filter(|&&b| b == transfer_discriminator).count();

        (transfer_count as f32) / (data.len() as f32)
    }

    fn calculate_token_account_similarity(&self, data: &[u8]) -> f32 {
        if data.len() < 64 { return 0.0; }

        let mut similarities = Vec::new();
        let max_chunks = 10.min(data.len() / 32); // Only compare first 10 chunks

        // Compare only a few 32-byte chunks for similarity (optimized)
        for i in (0..data.len().saturating_sub(63)).step_by(32).take(max_chunks) {
            for j in ((i + 32)..data.len().saturating_sub(31)).step_by(32).take(max_chunks) {
                if j - i > 320 { break; } // Don't compare very distant chunks

                let chunk1 = &data[i..i + 32];
                let chunk2 = &data[j..j + 32];

                // Fast similarity check using first 8 bytes only
                let similarity = self.calculate_fast_similarity(&chunk1[..8], &chunk2[..8]);
                similarities.push(similarity);

                if similarities.len() >= 25 { break; } // Limit comparisons
            }
            if similarities.len() >= 25 { break; }
        }

        if similarities.is_empty() { 0.0 } else { similarities.iter().sum::<f32>() / similarities.len() as f32 }
    }

    fn calculate_amount_patterns(&self, data: &[u8]) -> f32 {
        if data.len() < 8 { return 0.0; }

        let mut amount_like_patterns = 0;

        // Look for 8-byte patterns that could be token amounts
        for window in data.windows(8) {
            // Check if this looks like a token amount (little-endian u64)
            let value = u64::from_le_bytes(window.try_into().unwrap_or([0; 8]));

            // Token amounts often have specific patterns
            if value > 0 && value < u64::MAX / 2 {
                amount_like_patterns += 1;
            }
        }

        (amount_like_patterns as f32) / ((data.len() - 7) as f32)
    }

    fn calculate_multi_scale_repetition(&self, data: &[u8], scale: usize) -> f32 {
        if data.len() < scale * 2 { return 0.0; }

        let max_windows = 1000.min(data.len() - scale + 1); // Limit sampling for performance
        let step_size = (data.len() - scale + 1).max(1) / max_windows.max(1);

        // Use different counting strategies based on scale size
        if scale > 8 {
            // For large patterns, use hash of first 8 bytes
            let mut pattern_counts: std::collections::HashMap<u64, usize> = std::collections::HashMap::new();

            for i in (0..data.len() - scale + 1).step_by(step_size).take(max_windows) {
                let window = &data[i..i + scale];
                let key = if window.len() >= 8 {
                    u64::from_le_bytes([window[0], window[1], window[2], window[3],
                                      window[4], window[5], window[6], window[7]])
                } else {
                    0
                };
                *pattern_counts.entry(key).or_insert(0) += 1;
            }

            let repeated_patterns: usize = pattern_counts.values().filter(|&&count| count > 1).sum();
            (repeated_patterns as f32) / (max_windows as f32)
        } else {
            // For small patterns, use full bytes
            let mut pattern_counts: std::collections::HashMap<Vec<u8>, usize> = std::collections::HashMap::new();

            for i in (0..data.len() - scale + 1).step_by(step_size).take(max_windows) {
                let window = &data[i..i + scale];
                *pattern_counts.entry(window.to_vec()).or_insert(0) += 1;
            }

            let repeated_patterns: usize = pattern_counts.values().filter(|&&count| count > 1).sum();
            (repeated_patterns as f32) / (max_windows as f32)
        }
    }

    fn calculate_sequence_periodicity(&self, data: &[u8]) -> f32 {
        if data.len() < 16 { return 0.0; }

        let mut max_periodicity: f32 = 0.0;

        // Check for periodic patterns of different lengths
        for period in 2..=16 {
            if data.len() < period * 3 { continue; }

            let mut matches = 0;
            let max_checks = (data.len() / period).min(100); // Limit checks

            for i in 0..max_checks - 2 {
                let start1 = i * period;
                let start2 = (i + 1) * period;

                if start2 + period <= data.len() {
                    let chunk1 = &data[start1..start1 + period];
                    let chunk2 = &data[start2..start2 + period];

                    if chunk1 == chunk2 {
                        matches += 1;
                    }
                }
            }

            let periodicity = (matches as f32) / ((max_checks - 2) as f32).max(1.0);
            max_periodicity = max_periodicity.max(periodicity);
        }

        max_periodicity
    }

    fn calculate_hft_patterns(&self, data: &[u8]) -> f32 {
        if data.len() < 100 { return 0.0; }

        // HFT patterns: small variations in similar structures
        let mut hft_score = 0.0;
        let chunk_size = 32;
        let max_chunks = (data.len() / chunk_size).min(20); // Reduce from 50 to 20
        let max_comparisons = 50; // Limit total comparisons
        let mut comparison_count = 0;

        for i in 0..max_chunks - 1 {
            for j in i + 1..max_chunks {
                if comparison_count >= max_comparisons { break; }

                let start1 = i * chunk_size;
                let start2 = j * chunk_size;

                if start2 + chunk_size <= data.len() {
                    let chunk1 = &data[start1..start1 + chunk_size];
                    let chunk2 = &data[start2..start2 + chunk_size];

                    // Fast similarity check using first 8 bytes
                    let quick_similarity = self.calculate_fast_similarity(chunk1, chunk2);

                    // Only do full similarity check if quick check passes
                    if quick_similarity > 0.6 {
                        let similarity = self.calculate_hamming_similarity(chunk1, chunk2);

                        // HFT patterns have high similarity but not identical
                        if similarity > 0.7 && similarity < 0.95 {
                            hft_score += similarity;
                        }
                    }

                    comparison_count += 1;
                }
            }
            if comparison_count >= max_comparisons { break; }
        }

        if comparison_count > 0 {
            hft_score / comparison_count as f32
        } else {
            0.0
        }
    }

    fn calculate_fast_similarity(&self, a: &[u8], b: &[u8]) -> f32 {
        // Quick similarity check using first 8 bytes
        let sample_size = 8.min(a.len()).min(b.len());
        if sample_size == 0 { return 0.0; }

        let matches = a.iter().take(sample_size).zip(b.iter().take(sample_size))
            .filter(|(x, y)| x == y).count();
        (matches as f32) / (sample_size as f32)
    }

    fn calculate_hamming_similarity(&self, a: &[u8], b: &[u8]) -> f32 {
        if a.len() != b.len() { return 0.0; }

        let matches = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();
        (matches as f32) / (a.len() as f32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_importance_analyzer() {
        let mut analyzer = FeatureImportanceAnalyzer::new();

        let test_data = vec![1, 2, 3, 4, 5, 5, 5, 5, 6, 7, 8];
        let features = analyzer.analyze_features(&test_data).unwrap();

        assert!(!features.is_empty());

        // Check that we have expected feature types
        let feature_names: Vec<_> = features.iter().map(|f| &f.feature_name).collect();
        assert!(feature_names.contains(&&"entropy".to_string()));
        assert!(feature_names.contains(&&"repetition_ratio".to_string()));
    }

    #[test]
    fn test_entropy_calculation() {
        let analyzer = FeatureImportanceAnalyzer::new();

        // High entropy data
        let random_data = (0..=255u8).collect::<Vec<_>>();
        let entropy = analyzer.calculate_entropy(&random_data);
        assert!(entropy > 7.0);

        // Low entropy data
        let repetitive_data = vec![42u8; 100];
        let entropy = analyzer.calculate_entropy(&repetitive_data);
        assert!(entropy < 1.0);
    }

    #[test]
    fn test_repetition_ratio() {
        let analyzer = FeatureImportanceAnalyzer::new();

        // High repetition
        let repetitive = vec![1, 1, 1, 1, 1];
        let ratio = analyzer.calculate_repetition_ratio(&repetitive);
        assert!(ratio > 0.9);

        // Low repetition
        let varied = vec![1, 2, 3, 4, 5];
        let ratio = analyzer.calculate_repetition_ratio(&varied);
        assert!(ratio < 0.1);
    }

    #[test]
    fn test_pattern_complexity() {
        let analyzer = FeatureImportanceAnalyzer::new();

        // Simple pattern
        let simple = vec![1, 2, 3, 4, 5, 6];
        let complexity = analyzer.calculate_pattern_complexity(&simple);
        assert!(complexity < 0.1);

        // Complex pattern
        let complex = vec![1, 5, 2, 8, 3, 1, 9, 4];
        let complexity = analyzer.calculate_pattern_complexity(&complex);
        assert!(complexity > 0.5);
    }
}