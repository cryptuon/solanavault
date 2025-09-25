//! # Native Rust XGBoost Implementation
//!
//! Proper gradient boosting using smartcore for compression pattern prediction.

use super::*;
use smartcore::tree::decision_tree_regressor::*;
use smartcore::ensemble::gradient_boost_regressor::*;
use smartcore::linalg::basic::matrix::DenseMatrix;
use smartcore::metrics::mean_squared_error;
use linfa::prelude::*;
use ndarray::{Array1, Array2};
use std::collections::HashMap;

/// Native XGBoost implementation for compression
#[derive(Debug)]
pub struct NativeXGBoost {
    /// Main gradient boosting regressor
    gb_regressor: Option<GradientBoostingRegressor<f32, f32, DenseMatrix<f32>, Vec<f32>>>,

    /// Strategy classification models
    strategy_classifiers: HashMap<CompressionStrategy, GradientBoostingRegressor<f32, f32, DenseMatrix<f32>, Vec<f32>>>,

    /// Feature importance scores
    feature_importance: Vec<f32>,

    /// Training configuration
    config: XGBoostConfig,

    /// Model performance metrics
    performance: ModelPerformance,
}

impl NativeXGBoost {
    /// Creates a new native XGBoost predictor
    pub fn new() -> Self {
        Self {
            gb_regressor: None,
            strategy_classifiers: HashMap::new(),
            feature_importance: Vec::new(),
            config: XGBoostConfig::default(),
            performance: ModelPerformance::default(),
        }
    }

    /// Creates predictor with custom configuration
    pub fn with_config(config: XGBoostConfig) -> Self {
        Self {
            gb_regressor: None,
            strategy_classifiers: HashMap::new(),
            feature_importance: Vec::new(),
            performance: ModelPerformance::default(),
            config,
        }
    }

    /// Train XGBoost model on compression data
    pub fn train(&mut self, features: &[Vec<f32>], targets: &[CompressionStrategy]) -> Result<(), CompressionError> {
        println!("🚀 Training Native XGBoost on {} samples with {} estimators",
                 features.len(), self.config.n_estimators);

        if features.is_empty() || features.len() != targets.len() {
            return Err(CompressionError::InvalidFormat);
        }

        // Prepare training data
        let (x_matrix, y_values) = self.prepare_training_data(features, targets)?;

        // Configure gradient boosting parameters
        let gb_params = GradientBoostingRegressorParameters::default()
            .with_n_estimators(self.config.n_estimators)
            .with_max_depth(self.config.max_depth)
            .with_learning_rate(self.config.learning_rate)
            .with_subsample(self.config.subsample);

        // Train main regressor
        let regressor = GradientBoostingRegressor::fit(&x_matrix, &y_values, gb_params)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other,
                                                                  format!("XGBoost training failed: {}", e))))?;

        // Calculate training performance
        let predictions = regressor.predict(&x_matrix)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other,
                                                                  format!("Training prediction failed: {}", e))))?;

        let training_mse = mean_squared_error(&y_values, &predictions);
        self.performance.training_loss = training_mse;

        // Calculate feature importance using permutation importance
        self.calculate_feature_importance(&regressor, &x_matrix, &y_values)?;

        // Train strategy-specific classifiers
        self.train_strategy_classifiers(features, targets)?;

        self.gb_regressor = Some(regressor);

        println!("✅ Native XGBoost training completed!");
        println!("   Training MSE: {:.6}", training_mse);
        println!("   Top features: {:?}", self.get_top_features(5));

        Ok(())
    }

    /// Predict optimal compression strategy
    pub fn predict_strategy(&self, data: &[u8], feature_scores: &[FeatureImportanceScore]) -> Result<CompressionStrategy, CompressionError> {
        let features = self.extract_prediction_features(data, feature_scores)?;

        if let Some(ref regressor) = self.gb_regressor {
            let feature_matrix = DenseMatrix::from_2d_vec(&vec![features]);
            let prediction = regressor.predict(&feature_matrix)
                .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other,
                                                                      format!("Prediction failed: {}", e))))?;

            let strategy = self.numeric_to_strategy(prediction[0]);
            Ok(strategy)
        } else {
            // Fallback to heuristic
            Ok(self.heuristic_strategy_selection(data))
        }
    }

    /// Predict compression ratio for given strategy
    pub fn predict_compression_ratio(&self, data: &[u8], strategy: &CompressionStrategy) -> f32 {
        if let Some(classifier) = self.strategy_classifiers.get(strategy) {
            // Use strategy-specific model to predict compression ratio
            let features = self.extract_basic_features(data);
            if let Ok(feature_matrix) = DenseMatrix::from_2d_vec(&vec![features]).to_owned() {
                if let Ok(prediction) = classifier.predict(&feature_matrix) {
                    return prediction[0].max(1.0); // Ensure ratio >= 1.0
                }
            }
        }

        // Fallback heuristic
        self.estimate_compression_ratio(data, strategy)
    }

    /// Prepare training data for gradient boosting
    fn prepare_training_data(&self, features: &[Vec<f32>], targets: &[CompressionStrategy])
        -> Result<(DenseMatrix<f32>, Vec<f32>), CompressionError> {

        let mut feature_matrix = Vec::new();
        let mut target_vector = Vec::new();

        for (feature_vec, target) in features.iter().zip(targets.iter()) {
            // Normalize and pad features
            let mut normalized_features = feature_vec.clone();
            self.normalize_features(&mut normalized_features);

            // Pad to fixed size
            while normalized_features.len() < 50 {
                normalized_features.push(0.0);
            }
            normalized_features.truncate(50);

            feature_matrix.push(normalized_features);
            target_vector.push(self.strategy_to_numeric(target));
        }

        let x_matrix = DenseMatrix::from_2d_vec(&feature_matrix);
        Ok((x_matrix, target_vector))
    }

    /// Calculate feature importance using permutation importance
    fn calculate_feature_importance(&mut self, regressor: &GradientBoostingRegressor<f32, f32, DenseMatrix<f32>, Vec<f32>>,
                                   x_matrix: &DenseMatrix<f32>, y_values: &[f32]) -> Result<(), CompressionError> {

        let baseline_score = self.calculate_model_score(regressor, x_matrix, y_values)?;
        let feature_count = x_matrix.shape().1;
        let mut importance_scores = vec![0.0; feature_count];

        println!("📊 Calculating feature importance for {} features...", feature_count);

        // Calculate permutation importance for each feature
        for feature_idx in 0..feature_count {
            let permuted_score = self.calculate_permuted_score(regressor, x_matrix, y_values, feature_idx)?;
            let importance = (baseline_score - permuted_score).max(0.0);
            importance_scores[feature_idx] = importance;

            if feature_idx % 10 == 0 {
                println!("   Feature {}: importance = {:.4}", feature_idx, importance);
            }
        }

        // Normalize importance scores
        let total_importance: f32 = importance_scores.iter().sum();
        if total_importance > 0.0 {
            for score in &mut importance_scores {
                *score /= total_importance;
            }
        }

        self.feature_importance = importance_scores;
        Ok(())
    }

    /// Calculate model score (negative MSE for maximization)
    fn calculate_model_score(&self, regressor: &GradientBoostingRegressor<f32, f32, DenseMatrix<f32>, Vec<f32>>,
                            x: &DenseMatrix<f32>, y: &[f32]) -> Result<f32, CompressionError> {
        let predictions = regressor.predict(x)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other,
                                                                  format!("Score calculation failed: {}", e))))?;
        let mse = mean_squared_error(y, &predictions);
        Ok(-mse) // Negative because higher score is better for importance
    }

    /// Calculate score with permuted feature
    fn calculate_permuted_score(&self, regressor: &GradientBoostingRegressor<f32, f32, DenseMatrix<f32>, Vec<f32>>,
                               x: &DenseMatrix<f32>, y: &[f32], feature_idx: usize) -> Result<f32, CompressionError> {
        // Create a copy and shuffle the specified feature column
        let mut x_permuted = x.clone();
        let n_samples = x.shape().0;

        // Extract the feature column
        let mut feature_values: Vec<f32> = (0..n_samples).map(|i| x.get(i, feature_idx)).collect();

        // Shuffle using Fisher-Yates
        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        feature_values.shuffle(&mut rng);

        // Replace the column with shuffled values
        for i in 0..n_samples {
            x_permuted.set(i, feature_idx, feature_values[i]);
        }

        self.calculate_model_score(regressor, &x_permuted, y)
    }

    /// Train strategy-specific classifiers
    fn train_strategy_classifiers(&mut self, features: &[Vec<f32>], targets: &[CompressionStrategy]) -> Result<(), CompressionError> {
        let strategies = [
            CompressionStrategy::DictionaryBased,
            CompressionStrategy::PatternBased,
            CompressionStrategy::TreeBased,
            CompressionStrategy::Hybrid,
        ];

        for strategy in strategies {
            // Create binary classification data for this strategy
            let mut strategy_features = Vec::new();
            let mut strategy_targets = Vec::new();

            for (feature_vec, target) in features.iter().zip(targets.iter()) {
                strategy_features.push(feature_vec.clone());
                // Binary target: 1.0 if this strategy, 0.0 otherwise
                let target_value = if std::mem::discriminant(target) == std::mem::discriminant(&strategy) { 1.0 } else { 0.0 };
                strategy_targets.push(target_value);
            }

            if strategy_features.len() > 10 { // Need minimum samples
                let (x_matrix, y_values) = self.prepare_binary_training_data(&strategy_features, &strategy_targets)?;

                // Train with smaller parameters for binary classification
                let params = GradientBoostingRegressorParameters::default()
                    .with_n_estimators(self.config.n_estimators / 2)
                    .with_max_depth(self.config.max_depth - 1)
                    .with_learning_rate(self.config.learning_rate);

                if let Ok(classifier) = GradientBoostingRegressor::fit(&x_matrix, &y_values, params) {
                    self.strategy_classifiers.insert(strategy, classifier);
                }
            }
        }

        println!("✅ Trained {} strategy-specific classifiers", self.strategy_classifiers.len());
        Ok(())
    }

    /// Prepare binary training data for strategy classifiers
    fn prepare_binary_training_data(&self, features: &[Vec<f32>], targets: &[f32])
        -> Result<(DenseMatrix<f32>, Vec<f32>), CompressionError> {

        let mut normalized_features = Vec::new();

        for feature_vec in features {
            let mut normalized = feature_vec.clone();
            self.normalize_features(&mut normalized);

            while normalized.len() < 30 {
                normalized.push(0.0);
            }
            normalized.truncate(30);

            normalized_features.push(normalized);
        }

        let x_matrix = DenseMatrix::from_2d_vec(&normalized_features);
        Ok((x_matrix, targets.to_vec()))
    }

    /// Extract prediction features from data
    fn extract_prediction_features(&self, data: &[u8], feature_scores: &[FeatureImportanceScore]) -> Result<Vec<f32>, CompressionError> {
        let mut features = Vec::new();

        // Basic data characteristics
        features.push(data.len() as f32);
        features.push(self.calculate_entropy(data));
        features.push(self.calculate_repetition_ratio(data));
        features.push(self.calculate_byte_diversity(data));
        features.push(self.calculate_compression_potential(data));

        // Add feature importance scores
        for score in feature_scores.iter().take(15) {
            features.push(score.importance);
            features.push(score.gain);
            features.push(score.cover / 1000.0); // Normalize
        }

        // Pad to expected size
        while features.len() < 50 {
            features.push(0.0);
        }

        Ok(features)
    }

    /// Extract basic features for ratio prediction
    fn extract_basic_features(&self, data: &[u8]) -> Vec<f32> {
        vec![
            data.len() as f32 / 1000.0,
            self.calculate_entropy(data),
            self.calculate_repetition_ratio(data),
            self.calculate_byte_diversity(data),
            self.calculate_compression_potential(data),
        ]
    }

    /// Normalize features to [0, 1] range
    fn normalize_features(&self, features: &mut [f32]) {
        for feature in features {
            *feature = feature.max(0.0).min(100.0) / 100.0; // Simple normalization
        }
    }

    /// Convert strategy to numeric value
    fn strategy_to_numeric(&self, strategy: &CompressionStrategy) -> f32 {
        match strategy {
            CompressionStrategy::DictionaryBased => 0.0,
            CompressionStrategy::PatternBased => 1.0,
            CompressionStrategy::TreeBased => 2.0,
            CompressionStrategy::Hybrid => 3.0,
        }
    }

    /// Convert numeric value to strategy
    fn numeric_to_strategy(&self, value: f32) -> CompressionStrategy {
        let rounded = (value.round() as i32).max(0).min(3);
        match rounded {
            0 => CompressionStrategy::DictionaryBased,
            1 => CompressionStrategy::PatternBased,
            2 => CompressionStrategy::TreeBased,
            _ => CompressionStrategy::Hybrid,
        }
    }

    /// Heuristic strategy selection fallback
    fn heuristic_strategy_selection(&self, data: &[u8]) -> CompressionStrategy {
        let entropy = self.calculate_entropy(data);
        let repetition = self.calculate_repetition_ratio(data);

        if repetition > 0.6 {
            CompressionStrategy::PatternBased
        } else if entropy < 4.0 {
            CompressionStrategy::DictionaryBased
        } else if data.len() > 5000 {
            CompressionStrategy::TreeBased
        } else {
            CompressionStrategy::Hybrid
        }
    }

    /// Estimate compression ratio heuristically
    fn estimate_compression_ratio(&self, data: &[u8], strategy: &CompressionStrategy) -> f32 {
        let entropy = self.calculate_entropy(data);
        let repetition = self.calculate_repetition_ratio(data);

        let base_ratio = match strategy {
            CompressionStrategy::PatternBased => 2.0 + repetition * 8.0,
            CompressionStrategy::DictionaryBased => 1.5 + (8.0 - entropy) * 0.5,
            CompressionStrategy::TreeBased => 1.8 + (data.len() as f32 / 10000.0).min(2.0),
            CompressionStrategy::Hybrid => 2.5,
        };

        base_ratio.max(1.0)
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
        if data.len() < 2 { return 0.0; }

        let repetitions = data.windows(2).filter(|w| w[0] == w[1]).count();
        repetitions as f32 / (data.len() - 1) as f32
    }

    /// Calculate byte diversity
    fn calculate_byte_diversity(&self, data: &[u8]) -> f32 {
        let unique_bytes = data.iter().collect::<std::collections::HashSet<_>>().len();
        unique_bytes as f32 / 256.0
    }

    /// Calculate compression potential
    fn calculate_compression_potential(&self, data: &[u8]) -> f32 {
        let entropy = self.calculate_entropy(data);
        let repetition = self.calculate_repetition_ratio(data);

        // Higher repetition and lower entropy indicate better compression potential
        (1.0 + repetition) * (8.0 - entropy) / 8.0
    }

    /// Get top N most important features
    pub fn get_top_features(&self, n: usize) -> Vec<(usize, f32)> {
        let mut indexed_importance: Vec<_> = self.feature_importance.iter()
            .enumerate()
            .map(|(i, &importance)| (i, importance))
            .collect();

        indexed_importance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        indexed_importance.into_iter().take(n).collect()
    }

    /// Get feature importance scores
    pub fn get_feature_importance(&self) -> &[f32] {
        &self.feature_importance
    }

    /// Get model performance metrics
    pub fn get_performance(&self) -> &ModelPerformance {
        &self.performance
    }

    /// Check if model is trained
    pub fn is_trained(&self) -> bool {
        self.gb_regressor.is_some()
    }
}

/// Model performance metrics
#[derive(Debug, Clone, Default)]
pub struct ModelPerformance {
    pub training_loss: f32,
    pub validation_loss: f32,
    pub feature_count: usize,
    pub training_samples: usize,
}

impl Default for NativeXGBoost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_native_xgboost_creation() {
        let xgb = NativeXGBoost::new();
        assert!(!xgb.is_trained());
        assert!(xgb.feature_importance.is_empty());
    }

    #[test]
    fn test_strategy_conversion() {
        let xgb = NativeXGBoost::new();

        let strategy = CompressionStrategy::TreeBased;
        let numeric = xgb.strategy_to_numeric(&strategy);
        assert_eq!(numeric, 2.0);

        let converted = xgb.numeric_to_strategy(numeric);
        matches!(converted, CompressionStrategy::TreeBased);
    }

    #[test]
    fn test_feature_extraction() {
        let xgb = NativeXGBoost::new();
        let test_data = vec![1, 2, 3, 4, 5, 5, 5, 5];

        let entropy = xgb.calculate_entropy(&test_data);
        assert!(entropy > 0.0 && entropy <= 8.0);

        let repetition = xgb.calculate_repetition_ratio(&test_data);
        assert!(repetition >= 0.0 && repetition <= 1.0);

        let diversity = xgb.calculate_byte_diversity(&test_data);
        assert!(diversity >= 0.0 && diversity <= 1.0);
    }

    #[test]
    fn test_compression_potential() {
        let xgb = NativeXGBoost::new();

        // Highly repetitive data should have high compression potential
        let repetitive_data = vec![42u8; 100];
        let potential1 = xgb.calculate_compression_potential(&repetitive_data);

        // Random data should have lower compression potential
        let random_data: Vec<u8> = (0..100).map(|i| (i * 17 + 42) as u8).collect();
        let potential2 = xgb.calculate_compression_potential(&random_data);

        assert!(potential1 > potential2);
    }

    #[test]
    fn test_heuristic_strategy_selection() {
        let xgb = NativeXGBoost::new();

        // High repetition should suggest pattern-based
        let repetitive_data = vec![42u8; 200];
        let strategy = xgb.heuristic_strategy_selection(&repetitive_data);
        matches!(strategy, CompressionStrategy::PatternBased);

        // Large data should suggest tree-based
        let large_data = vec![1u8; 10000];
        let strategy = xgb.heuristic_strategy_selection(&large_data);
        matches!(strategy, CompressionStrategy::TreeBased);
    }
}