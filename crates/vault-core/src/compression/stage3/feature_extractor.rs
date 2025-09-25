//! # Feature Extraction for Machine Learning
//!
//! Extracts meaningful features from Solana blockchain data for ML models.

use super::*;
use std::collections::HashMap;

/// Extracts features from blockchain data for ML models
#[derive(Debug, Clone)]
pub struct FeatureExtractor {
    /// Feature importance scores
    importance_scores: Vec<f32>,
    /// Feature statistics
    feature_stats: FeatureStats,
    /// Configuration
    config: FeatureConfig,
    /// Cached feature calculations
    feature_cache: HashMap<u64, Vec<f32>>,
}

impl FeatureExtractor {
    /// Creates a new feature extractor
    pub fn new() -> Self {
        Self {
            importance_scores: vec![1.0; 128], // Default 128 features
            feature_stats: FeatureStats::default(),
            config: FeatureConfig::default(),
            feature_cache: HashMap::new(),
        }
    }

    /// Creates a feature extractor with custom configuration
    pub fn with_config(stage3_config: &Stage3Config) -> Self {
        Self {
            importance_scores: vec![1.0; stage3_config.feature_dimensions],
            feature_stats: FeatureStats::default(),
            config: FeatureConfig {
                feature_dimensions: stage3_config.feature_dimensions,
                window_size: stage3_config.prediction_window,
                enable_caching: true,
                normalize_features: true,
            },
            feature_cache: HashMap::new(),
        }
    }

    /// Extract features from raw blockchain data
    pub fn extract_features(&mut self, data: &[u8]) -> Result<Vec<f32>, CompressionError> {
        // Check cache first
        let data_hash = self.hash_data(data);
        if let Some(cached_features) = self.feature_cache.get(&data_hash) {
            return Ok(cached_features.clone());
        }

        let mut features = Vec::with_capacity(self.config.feature_dimensions);

        // Statistical features
        features.extend(self.extract_statistical_features(data)?);

        // Pattern features
        features.extend(self.extract_pattern_features(data)?);

        // Structural features
        features.extend(self.extract_structural_features(data)?);

        // Solana-specific features
        features.extend(self.extract_solana_features(data)?);

        // Frequency domain features
        features.extend(self.extract_frequency_features(data)?);

        // Entropy and information features
        features.extend(self.extract_entropy_features(data)?);

        // Ensure we have exactly the right number of features
        features.resize(self.config.feature_dimensions, 0.0);

        // Normalize features if enabled
        if self.config.normalize_features {
            self.normalize_features(&mut features);
        }

        // Update statistics
        self.update_feature_stats(&features);

        // Cache the result
        if self.config.enable_caching && self.feature_cache.len() < 1000 {
            self.feature_cache.insert(data_hash, features.clone());
        }

        Ok(features)
    }

    /// Extract statistical features (mean, variance, skewness, etc.)
    fn extract_statistical_features(&self, data: &[u8]) -> Result<Vec<f32>, CompressionError> {
        let mut features = Vec::new();

        if data.is_empty() {
            return Ok(vec![0.0; 20]); // 20 statistical features
        }

        // Basic statistics
        let mean = data.iter().map(|&x| x as f32).sum::<f32>() / data.len() as f32;
        features.push(mean);

        let variance = data.iter()
            .map(|&x| (x as f32 - mean).powi(2))
            .sum::<f32>() / data.len() as f32;
        features.push(variance);

        let std_dev = variance.sqrt();
        features.push(std_dev);

        // Min, max, range
        let min_val = *data.iter().min().unwrap() as f32;
        let max_val = *data.iter().max().unwrap() as f32;
        features.push(min_val);
        features.push(max_val);
        features.push(max_val - min_val);

        // Percentiles
        let mut sorted_data = data.to_vec();
        sorted_data.sort_unstable();
        let len = sorted_data.len();
        features.push(sorted_data[len / 4] as f32); // 25th percentile
        features.push(sorted_data[len / 2] as f32); // Median
        features.push(sorted_data[3 * len / 4] as f32); // 75th percentile

        // Skewness (simplified)
        let skewness = if std_dev > 0.0 {
            data.iter()
                .map(|&x| ((x as f32 - mean) / std_dev).powi(3))
                .sum::<f32>() / data.len() as f32
        } else {
            0.0
        };
        features.push(skewness);

        // Kurtosis (simplified)
        let kurtosis = if std_dev > 0.0 {
            data.iter()
                .map(|&x| ((x as f32 - mean) / std_dev).powi(4))
                .sum::<f32>() / data.len() as f32 - 3.0
        } else {
            0.0
        };
        features.push(kurtosis);

        // Byte value histogram (8 buckets)
        let mut histogram = vec![0.0; 8];
        for &byte in data {
            let bucket = (byte as usize) / 32; // 256 / 8 = 32
            histogram[bucket.min(7)] += 1.0;
        }
        // Normalize histogram
        let total = data.len() as f32;
        for count in &mut histogram {
            *count /= total;
        }
        features.extend(histogram);

        // Zero padding to reach 20 features
        while features.len() < 20 {
            features.push(0.0);
        }

        Ok(features)
    }

    /// Extract pattern-based features
    fn extract_pattern_features(&self, data: &[u8]) -> Result<Vec<f32>, CompressionError> {
        let mut features = Vec::new();

        // Repetition patterns
        features.push(self.calculate_repetition_ratio(data));
        features.push(self.calculate_sequential_ratio(data));
        features.push(self.calculate_alternating_ratio(data));

        // N-gram frequencies
        features.extend(self.calculate_ngram_features(data, 2)); // Bigrams
        features.extend(self.calculate_ngram_features(data, 3)); // Trigrams

        // Pattern lengths
        features.push(self.calculate_avg_pattern_length(data));
        features.push(self.calculate_max_pattern_length(data));

        // Compression predictability
        features.push(self.calculate_lz_complexity(data));

        // Ensure we have 20 pattern features
        features.resize(20, 0.0);
        Ok(features)
    }

    /// Extract structural features specific to blockchain data
    fn extract_structural_features(&self, data: &[u8]) -> Result<Vec<f32>, CompressionError> {
        let mut features = Vec::new();

        // Block structure indicators
        features.push(self.detect_block_header_patterns(data));
        features.push(self.detect_transaction_markers(data));
        features.push(self.detect_address_patterns(data));

        // Data alignment features
        features.push(self.calculate_alignment_score(data, 4));  // 4-byte alignment
        features.push(self.calculate_alignment_score(data, 8));  // 8-byte alignment
        features.push(self.calculate_alignment_score(data, 32)); // 32-byte alignment (addresses)

        // Length indicators
        features.push((data.len() as f32).log2()); // Log length
        features.push((data.len() % 1024) as f32 / 1024.0); // Length modulo

        // Structural entropy
        features.push(self.calculate_structural_entropy(data));

        // Padding to 15 features
        features.resize(15, 0.0);
        Ok(features)
    }

    /// Extract Solana-specific features
    fn extract_solana_features(&self, data: &[u8]) -> Result<Vec<f32>, CompressionError> {
        let mut features = Vec::new();

        // Solana address detection (32-byte sequences)
        features.push(self.count_potential_addresses(data));

        // Common program detection
        features.push(self.detect_system_program_references(data));
        features.push(self.detect_token_program_references(data));

        // Instruction pattern detection
        features.push(self.detect_instruction_patterns(data));

        // Signature detection (64-byte sequences)
        features.push(self.count_potential_signatures(data));

        // Timestamp patterns
        features.push(self.detect_timestamp_patterns(data));

        // Solana magic numbers and constants
        features.push(self.detect_solana_constants(data));

        // Transaction structure score
        features.push(self.calculate_transaction_structure_score(data));

        // Padding to 25 features
        features.resize(25, 0.0);
        Ok(features)
    }

    /// Extract frequency domain features
    fn extract_frequency_features(&self, data: &[u8]) -> Result<Vec<f32>, CompressionError> {
        let mut features = Vec::new();

        // Simple frequency analysis
        let mut freq_map = HashMap::new();
        for &byte in data {
            *freq_map.entry(byte).or_insert(0) += 1;
        }

        // Most common bytes (top 5)
        let mut freq_vec: Vec<_> = freq_map.values().cloned().collect();
        freq_vec.sort_unstable_by(|a, b| b.cmp(a));

        for i in 0..5 {
            features.push(*freq_vec.get(i).unwrap_or(&0) as f32 / data.len() as f32);
        }

        // Frequency distribution features
        features.push(freq_map.len() as f32); // Unique byte count
        features.push(self.calculate_frequency_entropy(&freq_map, data.len()));

        // Periodic patterns (simplified FFT-like analysis)
        features.extend(self.detect_periodic_patterns(data));

        // Padding to 20 features
        features.resize(20, 0.0);
        Ok(features)
    }

    /// Extract entropy and information-theoretic features
    fn extract_entropy_features(&self, data: &[u8]) -> Result<Vec<f32>, CompressionError> {
        let mut features = Vec::new();

        // Shannon entropy
        features.push(self.calculate_shannon_entropy(data));

        // Conditional entropy (given previous byte)
        features.push(self.calculate_conditional_entropy(data));

        // Compression ratio estimates
        features.push(self.estimate_gzip_ratio(data));
        features.push(self.estimate_lz4_ratio(data));

        // Information content
        features.push(self.calculate_information_content(data));

        // Mutual information (simplified)
        features.push(self.calculate_mutual_information(data));

        // Kolmogorov complexity estimate
        features.push(self.estimate_kolmogorov_complexity(data));

        // Padding to 8 features
        features.resize(8, 0.0);
        Ok(features)
    }

    /// Calculate repetition ratio in data
    fn calculate_repetition_ratio(&self, data: &[u8]) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let mut repetitions = 0;
        for i in 1..data.len() {
            if data[i] == data[i - 1] {
                repetitions += 1;
            }
        }

        repetitions as f32 / (data.len() - 1) as f32
    }

    /// Calculate sequential pattern ratio
    fn calculate_sequential_ratio(&self, data: &[u8]) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let mut sequential = 0;
        for i in 1..data.len() {
            if data[i] == data[i - 1].wrapping_add(1) {
                sequential += 1;
            }
        }

        sequential as f32 / (data.len() - 1) as f32
    }

    /// Calculate alternating pattern ratio
    fn calculate_alternating_ratio(&self, data: &[u8]) -> f32 {
        if data.len() < 3 {
            return 0.0;
        }

        let mut alternating = 0;
        for i in 2..data.len() {
            if data[i] == data[i - 2] && data[i] != data[i - 1] {
                alternating += 1;
            }
        }

        alternating as f32 / (data.len() - 2) as f32
    }

    /// Calculate n-gram frequency features
    fn calculate_ngram_features(&self, data: &[u8], n: usize) -> Vec<f32> {
        if data.len() < n {
            return vec![0.0; 5]; // Return 5 zero features
        }

        let mut ngram_counts = HashMap::new();
        for window in data.windows(n) {
            *ngram_counts.entry(window.to_vec()).or_insert(0) += 1;
        }

        // Get top 5 n-grams by frequency
        let mut sorted_ngrams: Vec<_> = ngram_counts.values().cloned().collect();
        sorted_ngrams.sort_unstable_by(|a, b| b.cmp(a));

        let total_ngrams = data.len() - n + 1;
        let mut features = Vec::new();
        for i in 0..5 {
            features.push(*sorted_ngrams.get(i).unwrap_or(&0) as f32 / total_ngrams as f32);
        }

        features
    }

    /// Calculate average pattern length
    fn calculate_avg_pattern_length(&self, data: &[u8]) -> f32 {
        // Simplified pattern detection
        if data.len() < 2 {
            return 1.0;
        }

        let mut pattern_lengths = Vec::new();
        let mut current_length = 1;

        for i in 1..data.len() {
            if data[i] == data[i - 1] {
                current_length += 1;
            } else {
                pattern_lengths.push(current_length);
                current_length = 1;
            }
        }
        pattern_lengths.push(current_length);

        pattern_lengths.iter().sum::<usize>() as f32 / pattern_lengths.len() as f32
    }

    /// Calculate maximum pattern length
    fn calculate_max_pattern_length(&self, data: &[u8]) -> f32 {
        if data.len() < 2 {
            return 1.0;
        }

        let mut max_length = 1;
        let mut current_length = 1;

        for i in 1..data.len() {
            if data[i] == data[i - 1] {
                current_length += 1;
                max_length = max_length.max(current_length);
            } else {
                current_length = 1;
            }
        }

        max_length as f32
    }

    /// Calculate LZ complexity (simplified)
    fn calculate_lz_complexity(&self, data: &[u8]) -> f32 {
        // Very simplified LZ complexity
        let unique_bytes = data.iter().collect::<std::collections::HashSet<_>>().len();
        unique_bytes as f32 / 256.0
    }

    /// Detect block header patterns
    fn detect_block_header_patterns(&self, data: &[u8]) -> f32 {
        // Look for 32-byte sequences that might be block hashes
        if data.len() < 32 {
            return 0.0;
        }

        let mut header_score = 0.0;
        for chunk in data.chunks(32) {
            if chunk.len() == 32 {
                // Block hashes typically have some randomness
                let entropy = self.calculate_shannon_entropy(chunk);
                if entropy > 4.0 { // High entropy indicates hash-like data
                    header_score += 1.0;
                }
            }
        }

        header_score / (data.len() / 32) as f32
    }

    /// Detect transaction markers
    fn detect_transaction_markers(&self, data: &[u8]) -> f32 {
        let marker_count = data.iter().filter(|&&b| b == 0x01).count();
        marker_count as f32 / data.len() as f32
    }

    /// Detect address patterns (32-byte sequences)
    fn detect_address_patterns(&self, data: &[u8]) -> f32 {
        self.count_potential_addresses(data)
    }

    /// Calculate alignment score for given byte boundary
    fn calculate_alignment_score(&self, data: &[u8], alignment: usize) -> f32 {
        if data.len() < alignment {
            return 0.0;
        }

        let aligned_positions = (0..data.len())
            .filter(|&i| i % alignment == 0)
            .count();

        aligned_positions as f32 / data.len() as f32
    }

    /// Calculate structural entropy
    fn calculate_structural_entropy(&self, data: &[u8]) -> f32 {
        // Analyze entropy at different scales
        let chunk_size = (data.len() / 16).max(1);
        let mut entropies = Vec::new();

        for chunk in data.chunks(chunk_size) {
            entropies.push(self.calculate_shannon_entropy(chunk));
        }

        entropies.iter().sum::<f32>() / entropies.len() as f32
    }

    /// Count potential Solana addresses
    fn count_potential_addresses(&self, data: &[u8]) -> f32 {
        if data.len() < 32 {
            return 0.0;
        }

        let mut address_count = 0;
        for chunk in data.chunks_exact(32) {
            // Heuristic: valid addresses have moderate entropy
            let entropy = self.calculate_shannon_entropy(chunk);
            if entropy > 2.0 && entropy < 7.0 {
                address_count += 1;
            }
        }

        address_count as f32 / (data.len() / 32) as f32
    }

    /// Detect system program references
    fn detect_system_program_references(&self, data: &[u8]) -> f32 {
        let system_program = vec![0x11; 32]; // Simplified system program pattern
        let occurrences = data.windows(32)
            .filter(|&window| window.iter().all(|&b| b == 0x11))
            .count();

        occurrences as f32 / data.len().max(32) as f32
    }

    /// Detect token program references
    fn detect_token_program_references(&self, data: &[u8]) -> f32 {
        // Look for patterns that might be token program references
        let pattern_count = data.windows(4)
            .filter(|&window| window == &[0x06, 0xdd, 0xf6, 0xe1])
            .count();

        pattern_count as f32 / data.len().max(4) as f32
    }

    /// Detect instruction patterns
    fn detect_instruction_patterns(&self, data: &[u8]) -> f32 {
        // Look for common instruction patterns
        let common_patterns = [
            &[1, 2, 3, 4][..],
            &[0, 0, 0, 1][..],
            &[2, 0, 0, 0][..],
        ];

        let mut pattern_count = 0;
        for pattern in &common_patterns {
            pattern_count += data.windows(pattern.len())
                .filter(|&window| window == *pattern)
                .count();
        }

        pattern_count as f32 / data.len() as f32
    }

    /// Count potential signatures
    fn count_potential_signatures(&self, data: &[u8]) -> f32 {
        if data.len() < 64 {
            return 0.0;
        }

        let mut signature_count = 0;
        for chunk in data.chunks_exact(64) {
            // Signatures typically have high entropy
            let entropy = self.calculate_shannon_entropy(chunk);
            if entropy > 5.0 {
                signature_count += 1;
            }
        }

        signature_count as f32 / (data.len() / 64) as f32
    }

    /// Detect timestamp patterns
    fn detect_timestamp_patterns(&self, data: &[u8]) -> f32 {
        if data.len() < 8 {
            return 0.0;
        }

        let mut timestamp_count = 0;
        for chunk in data.chunks_exact(8) {
            let timestamp = u64::from_le_bytes(chunk.try_into().unwrap());
            // Check if it's a reasonable timestamp (2020-2030)
            if timestamp > 1_577_836_800 && timestamp < 1_893_456_000 {
                timestamp_count += 1;
            }
        }

        timestamp_count as f32 / (data.len() / 8) as f32
    }

    /// Detect Solana constants
    fn detect_solana_constants(&self, data: &[u8]) -> f32 {
        let constants = [
            &[0xFF, 0xFE][..], // Common metadata markers
            &[0x01][..],       // Transaction markers
            &[0x00][..],       // Null bytes
        ];

        let mut constant_count = 0;
        for constant in &constants {
            constant_count += data.windows(constant.len())
                .filter(|&window| window == *constant)
                .count();
        }

        constant_count as f32 / data.len() as f32
    }

    /// Calculate transaction structure score
    fn calculate_transaction_structure_score(&self, data: &[u8]) -> f32 {
        // Combine multiple structural indicators
        let marker_score = self.detect_transaction_markers(data);
        let address_score = self.count_potential_addresses(data);
        let alignment_score = self.calculate_alignment_score(data, 32);

        (marker_score + address_score + alignment_score) / 3.0
    }

    /// Detect periodic patterns
    fn detect_periodic_patterns(&self, data: &[u8]) -> Vec<f32> {
        let mut features = Vec::new();

        // Check for periods of length 2, 4, 8, 16
        for period in [2, 4, 8, 16] {
            if data.len() >= period * 2 {
                let mut matches = 0;
                let cycles = data.len() / period;

                for i in 0..cycles - 1 {
                    let chunk1 = &data[i * period..(i + 1) * period];
                    let chunk2 = &data[(i + 1) * period..(i + 2) * period];
                    if chunk1 == chunk2 {
                        matches += 1;
                    }
                }

                features.push(matches as f32 / (cycles - 1).max(1) as f32);
            } else {
                features.push(0.0);
            }
        }

        // Pad to desired length
        features.resize(10, 0.0);
        features
    }

    /// Calculate frequency entropy
    fn calculate_frequency_entropy(&self, freq_map: &HashMap<u8, usize>, total: usize) -> f32 {
        let mut entropy = 0.0;
        for &count in freq_map.values() {
            let p = count as f32 / total as f32;
            if p > 0.0 {
                entropy -= p * p.log2();
            }
        }
        entropy
    }

    /// Calculate Shannon entropy
    fn calculate_shannon_entropy(&self, data: &[u8]) -> f32 {
        if data.is_empty() {
            return 0.0;
        }

        let mut freq_map = HashMap::new();
        for &byte in data {
            *freq_map.entry(byte).or_insert(0) += 1;
        }

        self.calculate_frequency_entropy(&freq_map, data.len())
    }

    /// Calculate conditional entropy
    fn calculate_conditional_entropy(&self, data: &[u8]) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let mut conditional_counts: HashMap<(u8, u8), usize> = HashMap::new();
        let mut prev_counts: HashMap<u8, usize> = HashMap::new();

        for i in 1..data.len() {
            let prev = data[i - 1];
            let curr = data[i];
            *conditional_counts.entry((prev, curr)).or_insert(0) += 1;
            *prev_counts.entry(prev).or_insert(0) += 1;
        }

        let mut entropy = 0.0;
        for (&(prev, curr), &count) in &conditional_counts {
            let p_curr_given_prev = count as f32 / prev_counts[&prev] as f32;
            let p_prev = prev_counts[&prev] as f32 / (data.len() - 1) as f32;
            if p_curr_given_prev > 0.0 {
                entropy -= p_prev * p_curr_given_prev * p_curr_given_prev.log2();
            }
        }

        entropy
    }

    /// Estimate compression ratios
    fn estimate_gzip_ratio(&self, data: &[u8]) -> f32 {
        // Simplified estimation based on entropy
        let entropy = self.calculate_shannon_entropy(data);
        (8.0 / entropy.max(1.0)).min(10.0)
    }

    fn estimate_lz4_ratio(&self, data: &[u8]) -> f32 {
        // Simplified estimation based on repetition
        let repetition = self.calculate_repetition_ratio(data);
        (1.0 + repetition * 3.0).min(5.0)
    }

    /// Calculate information content
    fn calculate_information_content(&self, data: &[u8]) -> f32 {
        let entropy = self.calculate_shannon_entropy(data);
        entropy * data.len() as f32
    }

    /// Calculate mutual information (simplified)
    fn calculate_mutual_information(&self, data: &[u8]) -> f32 {
        if data.len() < 2 {
            return 0.0;
        }

        let h_x = self.calculate_shannon_entropy(&data[..data.len() / 2]);
        let h_y = self.calculate_shannon_entropy(&data[data.len() / 2..]);
        let h_xy = self.calculate_shannon_entropy(data);

        (h_x + h_y - h_xy).max(0.0)
    }

    /// Estimate Kolmogorov complexity
    fn estimate_kolmogorov_complexity(&self, data: &[u8]) -> f32 {
        // Very simplified estimate
        let unique_patterns = data.windows(4)
            .collect::<std::collections::HashSet<_>>()
            .len();

        unique_patterns as f32 / data.len().max(4) as f32
    }

    /// Hash data for caching
    fn hash_data(&self, data: &[u8]) -> u64 {
        // Simple hash function
        let mut hash = 0u64;
        for &byte in data {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }

    /// Normalize features to [0, 1] range
    fn normalize_features(&self, features: &mut [f32]) {
        for feature in features.iter_mut() {
            *feature = feature.max(0.0).min(1.0);
        }
    }

    /// Update feature statistics
    fn update_feature_stats(&mut self, features: &[f32]) {
        self.feature_stats.samples_processed += 1;

        // Update running averages (simplified)
        for (i, &feature) in features.iter().enumerate() {
            if i < self.importance_scores.len() {
                // Simple exponential moving average for importance
                let alpha = 0.01;
                self.importance_scores[i] = alpha * feature.abs() + (1.0 - alpha) * self.importance_scores[i];
            }
        }
    }

    /// Get feature importance scores
    pub fn get_importance_scores(&self) -> Vec<f32> {
        self.importance_scores.clone()
    }
}

impl Default for FeatureExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature extraction configuration
#[derive(Debug, Clone)]
pub struct FeatureConfig {
    pub feature_dimensions: usize,
    pub window_size: usize,
    pub enable_caching: bool,
    pub normalize_features: bool,
}

impl Default for FeatureConfig {
    fn default() -> Self {
        Self {
            feature_dimensions: 128,
            window_size: 32,
            enable_caching: true,
            normalize_features: true,
        }
    }
}

/// Feature extraction statistics
#[derive(Debug, Clone, Default)]
pub struct FeatureStats {
    pub samples_processed: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_extractor_creation() {
        let extractor = FeatureExtractor::new();
        assert_eq!(extractor.importance_scores.len(), 128);
    }

    #[test]
    fn test_feature_extraction() {
        let mut extractor = FeatureExtractor::new();
        let test_data = vec![1, 2, 3, 4, 1, 2, 3, 4, 5, 6, 7, 8];

        let features = extractor.extract_features(&test_data).unwrap();
        assert_eq!(features.len(), 128);

        // All features should be valid numbers
        for &feature in &features {
            assert!(feature.is_finite());
        }
    }

    #[test]
    fn test_statistical_features() {
        let extractor = FeatureExtractor::new();
        let test_data = vec![1, 2, 3, 4, 5];

        let features = extractor.extract_statistical_features(&test_data).unwrap();
        assert_eq!(features.len(), 20);

        // Mean should be 3.0
        assert!((features[0] - 3.0).abs() < 0.1);
    }

    #[test]
    fn test_pattern_features() {
        let extractor = FeatureExtractor::new();
        let test_data = vec![1, 1, 2, 2, 3, 3]; // Repeated pattern

        let features = extractor.extract_pattern_features(&test_data).unwrap();
        assert_eq!(features.len(), 20);

        // Should detect repetition
        assert!(features[0] > 0.0); // Repetition ratio
    }

    #[test]
    fn test_solana_features() {
        let extractor = FeatureExtractor::new();

        // Create data with potential Solana addresses
        let mut test_data = Vec::new();
        test_data.extend_from_slice(&[0x11; 32]); // System program-like
        test_data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // Instruction pattern

        let features = extractor.extract_solana_features(&test_data).unwrap();
        assert_eq!(features.len(), 25);

        // Should detect system program pattern
        assert!(features[1] > 0.0);
    }

    #[test]
    fn test_entropy_calculation() {
        let extractor = FeatureExtractor::new();

        // Uniform distribution should have high entropy
        let uniform_data = (0..=255).collect::<Vec<u8>>();
        let uniform_entropy = extractor.calculate_shannon_entropy(&uniform_data);
        assert!(uniform_entropy > 7.0);

        // Single value should have zero entropy
        let constant_data = vec![42; 100];
        let constant_entropy = extractor.calculate_shannon_entropy(&constant_data);
        assert!(constant_entropy < 0.1);
    }

    #[test]
    fn test_caching() {
        let mut extractor = FeatureExtractor::new();
        let test_data = vec![1, 2, 3, 4, 5];

        // First extraction
        let features1 = extractor.extract_features(&test_data).unwrap();

        // Second extraction (should use cache)
        let features2 = extractor.extract_features(&test_data).unwrap();

        assert_eq!(features1, features2);
    }

    #[test]
    fn test_normalization() {
        let extractor = FeatureExtractor::with_config(&Stage3Config::default());
        let mut features = vec![-1.0, 0.5, 2.0, 0.0, 1.0];

        extractor.normalize_features(&mut features);

        for feature in features {
            assert!(feature >= 0.0 && feature <= 1.0);
        }
    }
}