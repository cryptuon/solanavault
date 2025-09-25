//! # Adaptive Learning Compression System
//!
//! Learns from data patterns and adapts compression strategies in real-time.

use super::*;
use std::collections::VecDeque;
use rand::Rng;

/// Adaptive learning compression system
#[derive(Debug, Clone)]
pub struct AdaptiveLearner {
    /// Learning algorithms
    algorithms: Vec<CompressionAlgorithm>,
    /// Performance history
    performance_history: VecDeque<PerformanceRecord>,
    /// Current learning rate
    learning_rate: f32,
    /// Algorithm selection strategy
    selection_strategy: SelectionStrategy,
    /// Configuration
    config: AdaptiveConfig,
    /// Statistics
    stats: AdaptiveStats,
}

impl AdaptiveLearner {
    /// Creates a new adaptive learner
    pub fn new() -> Self {
        Self {
            algorithms: Self::initialize_algorithms(),
            performance_history: VecDeque::with_capacity(1000),
            learning_rate: 0.001,
            selection_strategy: SelectionStrategy::EpsilonGreedy { epsilon: 0.1 },
            config: AdaptiveConfig::default(),
            stats: AdaptiveStats::default(),
        }
    }

    /// Creates an adaptive learner with custom configuration
    pub fn with_config(stage3_config: &Stage3Config) -> Self {
        Self {
            algorithms: Self::initialize_algorithms(),
            performance_history: VecDeque::with_capacity(1000),
            learning_rate: stage3_config.learning_rate,
            selection_strategy: SelectionStrategy::EpsilonGreedy { epsilon: 0.1 },
            config: AdaptiveConfig {
                max_algorithms: 10,
                performance_window: 100,
                adaptation_threshold: 0.05,
                exploration_rate: 0.1,
            },
            stats: AdaptiveStats::default(),
        }
    }

    /// Compress data with ML predictions
    pub fn compress_with_predictions(&mut self, data: &[u8], predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        let start_time = std::time::Instant::now();

        // Select best algorithm based on current performance
        let algorithm_idx = self.select_algorithm(data, predictions)?;
        let algorithm = &mut self.algorithms[algorithm_idx];

        // Apply compression with predictions
        let algorithm_compressed = algorithm.compress_with_predictions(data, predictions)?;

        // Add compression header
        let compressed = self.add_compression_header(algorithm_idx, &algorithm_compressed)?;

        // Record performance
        let compression_ratio = data.len() as f32 / compressed.len() as f32;
        let performance = PerformanceRecord {
            algorithm_id: algorithm_idx,
            data_size: data.len(),
            compressed_size: compressed.len(),
            compression_ratio,
            compression_time: start_time.elapsed(),
            prediction_count: predictions.len(),
            timestamp: std::time::SystemTime::now(),
        };

        self.record_performance(performance);

        // Adapt algorithms based on performance
        if self.should_adapt() {
            self.adapt_algorithms()?;
        }

        // Update statistics
        self.stats.compressions_performed += 1;
        self.stats.total_compression_time += start_time.elapsed();
        self.stats.bytes_processed += data.len();

        Ok(compressed)
    }

    /// Decompress data with ML predictions
    pub fn decompress_with_predictions(&self, data: &[u8], predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        // Parse compression header to determine algorithm used
        let (algorithm_idx, compressed_data) = self.parse_compression_header(data)?;

        if algorithm_idx >= self.algorithms.len() {
            return Err(CompressionError::InvalidFormat);
        }

        // Decompress using the specified algorithm
        let algorithm = &self.algorithms[algorithm_idx];
        algorithm.decompress_with_predictions(&compressed_data, predictions)
    }

    /// Select the best algorithm for current data
    fn select_algorithm(&mut self, data: &[u8], predictions: &[Prediction]) -> Result<usize, CompressionError> {
        match &self.selection_strategy {
            SelectionStrategy::EpsilonGreedy { epsilon } => {
                if rand::random::<f32>() < *epsilon {
                    // Explore: random selection
                    Ok(rand::random::<usize>() % self.algorithms.len())
                } else {
                    // Exploit: best performing algorithm
                    Ok(self.get_best_algorithm_idx())
                }
            }
            SelectionStrategy::UCB { confidence } => {
                // Upper Confidence Bound selection
                Ok(self.select_ucb_algorithm(*confidence))
            }
            SelectionStrategy::ThompsonSampling => {
                // Thompson sampling (simplified)
                Ok(self.select_thompson_sampling())
            }
        }
    }

    /// Get the index of the best performing algorithm
    fn get_best_algorithm_idx(&self) -> usize {
        if self.performance_history.is_empty() {
            return 0;
        }

        let mut algorithm_scores = vec![0.0; self.algorithms.len()];
        let mut algorithm_counts = vec![0; self.algorithms.len()];

        // Calculate average performance for each algorithm
        for record in &self.performance_history {
            if record.algorithm_id < algorithm_scores.len() {
                algorithm_scores[record.algorithm_id] += record.compression_ratio;
                algorithm_counts[record.algorithm_id] += 1;
            }
        }

        // Find algorithm with highest average compression ratio
        let mut best_idx = 0;
        let mut best_score = 0.0;

        for i in 0..algorithm_scores.len() {
            if algorithm_counts[i] > 0 {
                let avg_score = algorithm_scores[i] / algorithm_counts[i] as f32;
                if avg_score > best_score {
                    best_score = avg_score;
                    best_idx = i;
                }
            }
        }

        best_idx
    }

    /// UCB algorithm selection
    fn select_ucb_algorithm(&self, confidence: f32) -> usize {
        let total_plays = self.performance_history.len();
        if total_plays == 0 {
            return 0;
        }

        let mut algorithm_rewards = vec![0.0; self.algorithms.len()];
        let mut algorithm_plays = vec![0; self.algorithms.len()];

        for record in &self.performance_history {
            if record.algorithm_id < algorithm_rewards.len() {
                algorithm_rewards[record.algorithm_id] += record.compression_ratio;
                algorithm_plays[record.algorithm_id] += 1;
            }
        }

        // Calculate UCB values
        let mut best_idx = 0;
        let mut best_ucb = f32::NEG_INFINITY;

        for i in 0..self.algorithms.len() {
            let ucb = if algorithm_plays[i] == 0 {
                f32::INFINITY // Unplayed algorithms get maximum priority
            } else {
                let avg_reward = algorithm_rewards[i] / algorithm_plays[i] as f32;
                let exploration_bonus = confidence * ((total_plays as f32).ln() / algorithm_plays[i] as f32).sqrt();
                avg_reward + exploration_bonus
            };

            if ucb > best_ucb {
                best_ucb = ucb;
                best_idx = i;
            }
        }

        best_idx
    }

    /// Thompson sampling selection
    fn select_thompson_sampling(&self) -> usize {
        // Simplified Thompson sampling using beta distribution approximation
        let mut best_idx = 0;
        let mut best_sample = 0.0;

        for i in 0..self.algorithms.len() {
            // Sample from beta distribution (approximated)
            let sample = rand::random::<f32>();
            if sample > best_sample {
                best_sample = sample;
                best_idx = i;
            }
        }

        best_idx
    }

    /// Record algorithm performance
    fn record_performance(&mut self, performance: PerformanceRecord) {
        self.performance_history.push_back(performance);

        // Maintain window size
        while self.performance_history.len() > self.config.performance_window {
            self.performance_history.pop_front();
        }
    }

    /// Check if algorithms should be adapted
    fn should_adapt(&self) -> bool {
        self.performance_history.len() >= self.config.performance_window &&
        self.performance_history.len() % 50 == 0 // Adapt every 50 compressions
    }

    /// Adapt algorithms based on performance
    fn adapt_algorithms(&mut self) -> Result<(), CompressionError> {
        // Calculate performance variance to detect if adaptation is needed
        let recent_performances: Vec<f32> = self.performance_history
            .iter()
            .rev()
            .take(20)
            .map(|r| r.compression_ratio)
            .collect();

        if recent_performances.len() < 10 {
            return Ok(());
        }

        let mean = recent_performances.iter().sum::<f32>() / recent_performances.len() as f32;
        let variance = recent_performances.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / recent_performances.len() as f32;

        // If performance is highly variable, try to add new algorithms
        if variance > self.config.adaptation_threshold {
            self.evolve_algorithms()?;
        }

        // Update learning rates
        self.update_learning_rates();

        Ok(())
    }

    /// Evolve algorithms through mutation and crossover
    fn evolve_algorithms(&mut self) -> Result<(), CompressionError> {
        if self.algorithms.len() >= self.config.max_algorithms {
            return Ok(());
        }

        // Select top performing algorithms for evolution
        let mut performance_indices: Vec<_> = (0..self.algorithms.len()).collect();
        performance_indices.sort_by(|&a, &b| {
            let score_a = self.get_algorithm_score(a);
            let score_b = self.get_algorithm_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Create new algorithm by combining top performers
        if performance_indices.len() >= 2 {
            let parent1_idx = performance_indices[0];
            let parent2_idx = performance_indices[1];

            let new_algorithm = self.crossover_algorithms(parent1_idx, parent2_idx)?;
            self.algorithms.push(new_algorithm);

            self.stats.algorithms_evolved += 1;
        }

        Ok(())
    }

    /// Get performance score for an algorithm
    fn get_algorithm_score(&self, algorithm_idx: usize) -> f32 {
        let relevant_records: Vec<_> = self.performance_history
            .iter()
            .filter(|r| r.algorithm_id == algorithm_idx)
            .collect();

        if relevant_records.is_empty() {
            return 0.0;
        }

        let total_ratio: f32 = relevant_records.iter().map(|r| r.compression_ratio).sum();
        total_ratio / relevant_records.len() as f32
    }

    /// Crossover two algorithms to create a new one
    fn crossover_algorithms(&self, parent1_idx: usize, parent2_idx: usize) -> Result<CompressionAlgorithm, CompressionError> {
        let parent1 = &self.algorithms[parent1_idx];
        let parent2 = &self.algorithms[parent2_idx];

        // Simple crossover: combine parameters
        let new_algorithm = CompressionAlgorithm {
            id: self.algorithms.len(),
            name: format!("Evolved_{}", self.algorithms.len()),
            algorithm_type: parent1.algorithm_type.clone(),
            parameters: self.crossover_parameters(&parent1.parameters, &parent2.parameters),
            performance_score: 0.0,
        };

        Ok(new_algorithm)
    }

    /// Crossover algorithm parameters
    fn crossover_parameters(&self, params1: &AlgorithmParameters, params2: &AlgorithmParameters) -> AlgorithmParameters {
        AlgorithmParameters {
            dictionary_size: (params1.dictionary_size + params2.dictionary_size) / 2,
            window_size: if rand::random::<bool>() { params1.window_size } else { params2.window_size },
            compression_level: (params1.compression_level + params2.compression_level) / 2,
            prediction_weight: (params1.prediction_weight + params2.prediction_weight) / 2.0,
            entropy_threshold: (params1.entropy_threshold + params2.entropy_threshold) / 2.0,
        }
    }

    /// Update learning rates based on recent performance
    fn update_learning_rates(&mut self) {
        if self.performance_history.len() < 10 {
            return;
        }

        let recent_variance = self.calculate_recent_variance();

        // Increase learning rate if performance is stable, decrease if volatile
        if recent_variance < 0.1 {
            self.learning_rate = (self.learning_rate * 1.1).min(0.1);
        } else if recent_variance > 0.5 {
            self.learning_rate = (self.learning_rate * 0.9).max(0.0001);
        }
    }

    /// Calculate variance in recent performance
    fn calculate_recent_variance(&self) -> f32 {
        let recent_ratios: Vec<f32> = self.performance_history
            .iter()
            .rev()
            .take(10)
            .map(|r| r.compression_ratio)
            .collect();

        if recent_ratios.len() < 2 {
            return 0.0;
        }

        let mean = recent_ratios.iter().sum::<f32>() / recent_ratios.len() as f32;
        recent_ratios.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f32>() / recent_ratios.len() as f32
    }

    /// Add compression header with algorithm info
    fn add_compression_header(&self, algorithm_idx: usize, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if algorithm_idx > 255 {
            return Err(CompressionError::InvalidFormat);
        }

        let mut result = Vec::with_capacity(4 + data.len());

        // Header format: [algorithm_idx (1 byte), size (3 bytes), data...]
        result.push(algorithm_idx as u8);

        // Size as 3 bytes (little endian)
        let size_bytes = (data.len() as u32).to_le_bytes();
        result.extend_from_slice(&size_bytes[0..3]);

        // Compressed data
        result.extend_from_slice(data);

        Ok(result)
    }

    /// Parse compression header to extract algorithm info
    fn parse_compression_header(&self, data: &[u8]) -> Result<(usize, Vec<u8>), CompressionError> {
        if data.len() < 4 {
            return Err(CompressionError::InvalidFormat);
        }

        // Simple header format: [algorithm_idx (1 byte), size (3 bytes), data...]
        let algorithm_idx = data[0] as usize;
        let size = u32::from_le_bytes([data[1], data[2], data[3], 0]) as usize;

        if data.len() < 4 + size {
            return Err(CompressionError::InvalidFormat);
        }

        let compressed_data = data[4..4 + size].to_vec();
        Ok((algorithm_idx, compressed_data))
    }

    /// Initialize default compression algorithms
    fn initialize_algorithms() -> Vec<CompressionAlgorithm> {
        vec![
            CompressionAlgorithm {
                id: 0,
                name: "Dictionary".to_string(),
                algorithm_type: AlgorithmType::Dictionary,
                parameters: AlgorithmParameters {
                    dictionary_size: 1024,
                    window_size: 32,
                    compression_level: 5,
                    prediction_weight: 0.5,
                    entropy_threshold: 4.0,
                },
                performance_score: 0.0,
            },
            CompressionAlgorithm {
                id: 1,
                name: "Pattern".to_string(),
                algorithm_type: AlgorithmType::Pattern,
                parameters: AlgorithmParameters {
                    dictionary_size: 512,
                    window_size: 16,
                    compression_level: 3,
                    prediction_weight: 0.7,
                    entropy_threshold: 3.0,
                },
                performance_score: 0.0,
            },
            CompressionAlgorithm {
                id: 2,
                name: "Hybrid".to_string(),
                algorithm_type: AlgorithmType::Hybrid,
                parameters: AlgorithmParameters {
                    dictionary_size: 2048,
                    window_size: 64,
                    compression_level: 7,
                    prediction_weight: 0.3,
                    entropy_threshold: 5.0,
                },
                performance_score: 0.0,
            },
        ]
    }

    /// Get current learning rate
    pub fn get_learning_rate(&self) -> f32 {
        self.learning_rate
    }

    /// Get statistics
    pub fn get_stats(&self) -> &AdaptiveStats {
        &self.stats
    }
}

impl Default for AdaptiveLearner {
    fn default() -> Self {
        Self::new()
    }
}

/// Compression algorithm with adaptive parameters
#[derive(Debug, Clone)]
pub struct CompressionAlgorithm {
    pub id: usize,
    pub name: String,
    pub algorithm_type: AlgorithmType,
    pub parameters: AlgorithmParameters,
    pub performance_score: f32,
}

impl CompressionAlgorithm {
    /// Compress data with predictions
    pub fn compress_with_predictions(&self, data: &[u8], predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        let mut compressed = Vec::new();

        // Add algorithm header
        compressed.push(self.id as u8);

        // Apply algorithm-specific compression
        let algorithm_data = match &self.algorithm_type {
            AlgorithmType::Dictionary => self.dictionary_compress(data, predictions)?,
            AlgorithmType::Pattern => self.pattern_compress(data, predictions)?,
            AlgorithmType::Hybrid => self.hybrid_compress(data, predictions)?,
        };

        // Add size and data
        let size_bytes = (algorithm_data.len() as u32).to_le_bytes();
        compressed.extend_from_slice(&size_bytes[0..3]);
        compressed.extend_from_slice(&algorithm_data);

        Ok(compressed)
    }

    /// Decompress data with predictions
    pub fn decompress_with_predictions(&self, data: &[u8], predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        match &self.algorithm_type {
            AlgorithmType::Dictionary => self.dictionary_decompress(data, predictions),
            AlgorithmType::Pattern => self.pattern_decompress(data, predictions),
            AlgorithmType::Hybrid => self.hybrid_decompress(data, predictions),
        }
    }

    /// Dictionary-based compression
    fn dictionary_compress(&self, data: &[u8], predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        let mut compressed = Vec::new();
        let mut dictionary = std::collections::HashMap::new();
        let mut dict_id = 0u16;

        // Add predictions to dictionary
        for prediction in predictions {
            if prediction.confidence > 0.5 && !prediction.predicted_bytes.is_empty() {
                dictionary.insert(prediction.predicted_bytes.clone(), dict_id);
                dict_id += 1;
            }
        }

        // Compress using dictionary
        let mut i = 0;
        while i < data.len() {
            let mut matched = false;

            // Try to match against dictionary entries
            for (pattern, &id) in &dictionary {
                if i + pattern.len() <= data.len() && &data[i..i + pattern.len()] == pattern {
                    compressed.push(0xFF); // Dictionary marker
                    compressed.extend_from_slice(&id.to_le_bytes());
                    i += pattern.len();
                    matched = true;
                    break;
                }
            }

            if !matched {
                compressed.push(data[i]);
                i += 1;
            }
        }

        Ok(compressed)
    }

    /// Dictionary-based decompression
    fn dictionary_decompress(&self, data: &[u8], predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        let mut decompressed = Vec::new();
        let mut dictionary = std::collections::HashMap::new();

        // Rebuild dictionary from predictions
        let mut dict_id = 0u16;
        for prediction in predictions {
            if prediction.confidence > 0.5 && !prediction.predicted_bytes.is_empty() {
                dictionary.insert(dict_id, prediction.predicted_bytes.clone());
                dict_id += 1;
            }
        }

        let mut i = 0;
        while i < data.len() {
            if data[i] == 0xFF && i + 3 <= data.len() {
                // Dictionary reference
                let id = u16::from_le_bytes([data[i + 1], data[i + 2]]);
                if let Some(pattern) = dictionary.get(&id) {
                    decompressed.extend_from_slice(pattern);
                }
                i += 3;
            } else {
                decompressed.push(data[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    /// Pattern-based compression
    fn pattern_compress(&self, data: &[u8], _predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        // Simplified pattern compression
        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            // Look for repeated bytes
            let mut count = 1;
            while i + count < data.len() && data[i] == data[i + count] && count < 255 {
                count += 1;
            }

            if count > 3 {
                // Use run-length encoding
                compressed.push(0xFE); // RLE marker
                compressed.push(count as u8);
                compressed.push(data[i]);
                i += count;
            } else {
                compressed.push(data[i]);
                i += 1;
            }
        }

        Ok(compressed)
    }

    /// Pattern-based decompression
    fn pattern_decompress(&self, data: &[u8], _predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            if data[i] == 0xFE && i + 2 < data.len() {
                // RLE decompression
                let count = data[i + 1] as usize;
                let byte = data[i + 2];
                decompressed.extend_from_slice(&vec![byte; count]);
                i += 3;
            } else {
                decompressed.push(data[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    /// Hybrid compression
    fn hybrid_compress(&self, data: &[u8], predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        // Combine dictionary and pattern compression
        let dict_compressed = self.dictionary_compress(data, predictions)?;
        let pattern_compressed = self.pattern_compress(&dict_compressed, predictions)?;
        Ok(pattern_compressed)
    }

    /// Hybrid decompression
    fn hybrid_decompress(&self, data: &[u8], predictions: &[Prediction]) -> Result<Vec<u8>, CompressionError> {
        // Reverse hybrid compression
        let pattern_decompressed = self.pattern_decompress(data, predictions)?;
        let dict_decompressed = self.dictionary_decompress(&pattern_decompressed, predictions)?;
        Ok(dict_decompressed)
    }
}

/// Algorithm type enumeration
#[derive(Debug, Clone)]
pub enum AlgorithmType {
    Dictionary,
    Pattern,
    Hybrid,
}

/// Algorithm parameters
#[derive(Debug, Clone)]
pub struct AlgorithmParameters {
    pub dictionary_size: usize,
    pub window_size: usize,
    pub compression_level: usize,
    pub prediction_weight: f32,
    pub entropy_threshold: f32,
}

/// Algorithm selection strategy
#[derive(Debug, Clone)]
pub enum SelectionStrategy {
    EpsilonGreedy { epsilon: f32 },
    UCB { confidence: f32 },
    ThompsonSampling,
}

/// Performance record for algorithm evaluation
#[derive(Debug, Clone)]
pub struct PerformanceRecord {
    pub algorithm_id: usize,
    pub data_size: usize,
    pub compressed_size: usize,
    pub compression_ratio: f32,
    pub compression_time: std::time::Duration,
    pub prediction_count: usize,
    pub timestamp: std::time::SystemTime,
}

/// Adaptive learner configuration
#[derive(Debug, Clone)]
pub struct AdaptiveConfig {
    pub max_algorithms: usize,
    pub performance_window: usize,
    pub adaptation_threshold: f32,
    pub exploration_rate: f32,
}

impl Default for AdaptiveConfig {
    fn default() -> Self {
        Self {
            max_algorithms: 10,
            performance_window: 100,
            adaptation_threshold: 0.05,
            exploration_rate: 0.1,
        }
    }
}

/// Adaptive learner statistics
#[derive(Debug, Clone, Default)]
pub struct AdaptiveStats {
    pub compressions_performed: usize,
    pub algorithms_evolved: usize,
    pub total_compression_time: std::time::Duration,
    pub bytes_processed: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_learner_creation() {
        let learner = AdaptiveLearner::new();
        assert_eq!(learner.algorithms.len(), 3);
        assert_eq!(learner.learning_rate, 0.001);
    }

    #[test]
    fn test_algorithm_selection() {
        let mut learner = AdaptiveLearner::new();
        let data = vec![1, 2, 3, 4, 5];
        let predictions = vec![];

        let algorithm_idx = learner.select_algorithm(&data, &predictions).unwrap();
        assert!(algorithm_idx < learner.algorithms.len());
    }

    #[test]
    fn test_compression_with_predictions() {
        let mut learner = AdaptiveLearner::new();
        let data = vec![1, 2, 3, 4, 1, 2, 3, 4];
        let predictions = vec![
            Prediction {
                pattern_type: PatternType::RepeatedPattern,
                confidence: 0.8,
                predicted_bytes: vec![1, 2, 3, 4],
                context_length: 4,
            }
        ];

        let compressed = learner.compress_with_predictions(&data, &predictions).unwrap();
        assert!(!compressed.is_empty());

        let decompressed = learner.decompress_with_predictions(&compressed, &predictions).unwrap();
        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_performance_recording() {
        let mut learner = AdaptiveLearner::new();

        let performance = PerformanceRecord {
            algorithm_id: 0,
            data_size: 100,
            compressed_size: 50,
            compression_ratio: 2.0,
            compression_time: std::time::Duration::from_millis(10),
            prediction_count: 5,
            timestamp: std::time::SystemTime::now(),
        };

        learner.record_performance(performance);
        assert_eq!(learner.performance_history.len(), 1);
    }

    #[test]
    fn test_algorithm_evolution() {
        let mut learner = AdaptiveLearner::new();

        // Add some performance records
        for i in 0..3 {
            let performance = PerformanceRecord {
                algorithm_id: i % 2, // Favor algorithm 0 and 1
                data_size: 100,
                compressed_size: 40 + i * 5,
                compression_ratio: 100.0 / (40 + i * 5) as f32,
                compression_time: std::time::Duration::from_millis(10),
                prediction_count: 3,
                timestamp: std::time::SystemTime::now(),
            };
            learner.record_performance(performance);
        }

        let initial_count = learner.algorithms.len();
        learner.evolve_algorithms().unwrap();

        // Should potentially add new algorithms
        assert!(learner.algorithms.len() >= initial_count);
    }

    #[test]
    fn test_dictionary_compression() {
        let algorithm = CompressionAlgorithm {
            id: 0,
            name: "Test".to_string(),
            algorithm_type: AlgorithmType::Dictionary,
            parameters: AlgorithmParameters {
                dictionary_size: 100,
                window_size: 16,
                compression_level: 5,
                prediction_weight: 0.5,
                entropy_threshold: 4.0,
            },
            performance_score: 0.0,
        };

        let data = vec![1, 2, 3, 4, 1, 2, 3, 4];
        let predictions = vec![
            Prediction {
                pattern_type: PatternType::RepeatedPattern,
                confidence: 0.9,
                predicted_bytes: vec![1, 2, 3, 4],
                context_length: 4,
            }
        ];

        let compressed = algorithm.compress_with_predictions(&data, &predictions).unwrap();
        let decompressed = algorithm.decompress_with_predictions(&compressed, &predictions).unwrap();

        assert_eq!(data, decompressed);
    }

    #[test]
    fn test_pattern_compression() {
        let algorithm = CompressionAlgorithm {
            id: 1,
            name: "Pattern".to_string(),
            algorithm_type: AlgorithmType::Pattern,
            parameters: AlgorithmParameters {
                dictionary_size: 100,
                window_size: 16,
                compression_level: 5,
                prediction_weight: 0.5,
                entropy_threshold: 4.0,
            },
            performance_score: 0.0,
        };

        let data = vec![5, 5, 5, 5, 5, 6, 7, 8]; // Repeated pattern
        let predictions = vec![];

        let compressed = algorithm.compress_with_predictions(&data, &predictions).unwrap();
        let decompressed = algorithm.decompress_with_predictions(&compressed, &predictions).unwrap();

        assert_eq!(data, decompressed);
    }
}