//! # Stage 3: Machine Learning Compression
//!
//! Advanced ML-based compression algorithms that learn patterns and predict future data
//! to achieve maximum compression ratios.

use super::traits::CompressionError;
use serde::{Serialize, Deserialize};

/// Neural network-based pattern prediction
pub mod neural_predictor;

/// Adaptive learning compression system
pub mod adaptive_learner;

/// Feature extraction for ML models
pub mod feature_extractor;

/// Model training and optimization
pub mod model_trainer;

pub use neural_predictor::NeuralPredictor;
pub use adaptive_learner::AdaptiveLearner;
pub use feature_extractor::FeatureExtractor;
pub use model_trainer::ModelTrainer;

/// Stage 3 ML compressor that uses neural networks for pattern prediction
#[derive(Debug, Clone)]
pub struct Stage3Compressor {
    neural_predictor: NeuralPredictor,
    adaptive_learner: AdaptiveLearner,
    feature_extractor: FeatureExtractor,
    model_trainer: ModelTrainer,
    stats: Stage3Stats,
    config: Stage3Config,
}

impl Stage3Compressor {
    /// Creates a new Stage 3 compressor with default ML models
    pub fn new() -> Self {
        Self {
            neural_predictor: NeuralPredictor::new(),
            adaptive_learner: AdaptiveLearner::new(),
            feature_extractor: FeatureExtractor::new(),
            model_trainer: ModelTrainer::new(),
            stats: Stage3Stats::default(),
            config: Stage3Config::default(),
        }
    }

    /// Creates a new Stage 3 compressor with custom configuration
    pub fn with_config(config: Stage3Config) -> Self {
        Self {
            neural_predictor: NeuralPredictor::with_config(&config),
            adaptive_learner: AdaptiveLearner::with_config(&config),
            feature_extractor: FeatureExtractor::with_config(&config),
            model_trainer: ModelTrainer::with_config(&config),
            stats: Stage3Stats::default(),
            config,
        }
    }

    /// Compress block data using machine learning
    pub fn compress_block_data(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let start_time = std::time::Instant::now();

        // Step 1: Extract features from the data
        let features = self.feature_extractor.extract_features(data)?;
        self.stats.features_extracted += features.len();

        // Step 2: Use neural predictor to predict patterns
        let predictions = self.neural_predictor.predict_patterns(&features)?;
        self.stats.predictions_made += predictions.len();

        // Step 3: Apply adaptive learning compression
        let ml_compressed = self.adaptive_learner.compress_with_predictions(data, &predictions)?;

        // Step 4: Update models with new data
        if self.config.online_learning {
            self.model_trainer.update_models(&features, data)?;
            self.stats.model_updates += 1;
        }

        // Step 5: Serialize the complete package
        let package = Stage3Package {
            features,
            predictions,
            compressed_data: ml_compressed,
        };

        let serialized = bincode::serialize(&package)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        self.stats.compression_time_ms += start_time.elapsed().as_millis() as u64;
        self.stats.original_bytes += data.len();
        self.stats.compressed_bytes += serialized.len();

        Ok(serialized)
    }

    /// Decompress block data using machine learning
    pub fn decompress_block_data(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // Extract compressed features and predictions
        let (features, predictions, compressed_data) = self.parse_compressed_data(data)?;

        // Use adaptive learner to decompress
        let decompressed = self.adaptive_learner.decompress_with_predictions(&compressed_data, &predictions)?;

        Ok(decompressed)
    }

    /// Train the ML models on a dataset
    pub fn train_on_dataset(&mut self, training_data: &[Vec<u8>]) -> Result<(), CompressionError> {
        println!("Training Stage 3 ML models on {} samples", training_data.len());

        for (i, data) in training_data.iter().enumerate() {
            // Extract features
            let features = self.feature_extractor.extract_features(data)?;

            // Train models
            self.model_trainer.train_batch(&features, data)?;

            if i % 100 == 0 {
                println!("Trained on {} samples", i + 1);
            }
        }

        // Finalize training
        self.model_trainer.finalize_training()?;

        // Update neural predictor with trained models
        self.neural_predictor.update_weights(self.model_trainer.get_weights())?;

        println!("✅ Stage 3 training completed");
        Ok(())
    }

    /// Parse compressed data format
    fn parse_compressed_data(&self, data: &[u8]) -> Result<(Vec<f32>, Vec<Prediction>, Vec<u8>), CompressionError> {
        // Deserialize the complete package
        let package: Stage3Package = bincode::deserialize(data)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        Ok((package.features, package.predictions, package.compressed_data))
    }

    /// Get compression statistics
    pub fn get_stats(&self) -> &Stage3Stats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = Stage3Stats::default();
    }

    /// Get model performance metrics
    pub fn get_model_metrics(&self) -> ModelMetrics {
        ModelMetrics {
            prediction_accuracy: self.neural_predictor.get_accuracy(),
            learning_rate: self.adaptive_learner.get_learning_rate(),
            feature_importance: self.feature_extractor.get_importance_scores(),
            model_complexity: self.model_trainer.get_complexity_score(),
        }
    }
}

impl Default for Stage3Compressor {
    fn default() -> Self {
        Self::new()
    }
}

/// Machine learning prediction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prediction {
    pub pattern_type: PatternType,
    pub confidence: f32,
    pub predicted_bytes: Vec<u8>,
    pub context_length: usize,
}

/// Types of patterns that can be predicted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    SequentialBytes,
    RepeatedPattern,
    AddressReference,
    InstructionSequence,
    TimestampProgression,
    SignaturePattern,
}

/// Statistics for Stage 3 compression
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Stage3Stats {
    pub features_extracted: usize,
    pub predictions_made: usize,
    pub model_updates: usize,
    pub compression_time_ms: u64,
    pub original_bytes: usize,
    pub compressed_bytes: usize,
}

/// Stage 3 compression package format
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Stage3Package {
    features: Vec<f32>,
    predictions: Vec<Prediction>,
    compressed_data: Vec<u8>,
}

impl Stage3Stats {
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

/// Configuration for Stage 3 compression
#[derive(Debug, Clone)]
pub struct Stage3Config {
    pub neural_network_layers: usize,
    pub learning_rate: f32,
    pub prediction_window: usize,
    pub feature_dimensions: usize,
    pub online_learning: bool,
    pub max_model_size: usize,
}

impl Default for Stage3Config {
    fn default() -> Self {
        Self {
            neural_network_layers: 3,
            learning_rate: 0.001,
            prediction_window: 32,
            feature_dimensions: 128,
            online_learning: true,
            max_model_size: 10 * 1024 * 1024, // 10MB
        }
    }
}

/// Model performance metrics
#[derive(Debug, Clone)]
pub struct ModelMetrics {
    pub prediction_accuracy: f64,
    pub learning_rate: f32,
    pub feature_importance: Vec<f32>,
    pub model_complexity: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage3_compressor_creation() {
        let compressor = Stage3Compressor::new();
        assert_eq!(compressor.get_stats().features_extracted, 0);
    }

    #[test]
    fn test_stage3_with_config() {
        let config = Stage3Config {
            neural_network_layers: 5,
            learning_rate: 0.01,
            prediction_window: 64,
            feature_dimensions: 256,
            online_learning: false,
            max_model_size: 50 * 1024 * 1024,
        };

        let compressor = Stage3Compressor::with_config(config.clone());
        assert_eq!(compressor.config.neural_network_layers, 5);
        assert_eq!(compressor.config.learning_rate, 0.01);
    }

    #[test]
    fn test_stage3_stats() {
        let mut stats = Stage3Stats::default();
        stats.original_bytes = 1000;
        stats.compressed_bytes = 50;

        assert_eq!(stats.compression_ratio(), 20.0);
        assert_eq!(stats.compression_percentage(), 95.0);
    }

    #[test]
    fn test_pattern_types() {
        let prediction = Prediction {
            pattern_type: PatternType::RepeatedPattern,
            confidence: 0.95,
            predicted_bytes: vec![1, 2, 3, 4],
            context_length: 16,
        };

        assert!(prediction.confidence > 0.9);
        assert_eq!(prediction.predicted_bytes.len(), 4);
    }
}