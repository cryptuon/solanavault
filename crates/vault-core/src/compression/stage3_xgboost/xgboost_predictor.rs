//! # XGBoost-Based Pattern Predictor
//!
//! Uses gradient boosting to predict optimal compression patterns and strategies.

use super::*;
use super::lightweight_xgboost::LightweightXGBoost;
use std::collections::HashMap;

/// XGBoost-style gradient boosting predictor for compression patterns
#[derive(Debug, Clone)]
pub struct XGBoostPredictor {
    /// Lightweight XGBoost implementation
    lightweight_xgb: LightweightXGBoost,

    /// Training history
    training_history: Vec<TrainingRecord>,

    /// Configuration
    config: XGBoostConfig,

    /// Prediction statistics
    stats: PredictorStats,
}

impl XGBoostPredictor {
    /// Creates a new XGBoost predictor
    pub fn new() -> Self {
        Self {
            lightweight_xgb: LightweightXGBoost::new(),
            training_history: Vec::new(),
            config: XGBoostConfig::default(),
            stats: PredictorStats::default(),
        }
    }

    /// Creates predictor with custom configuration
    pub fn with_config(config: &XGBoostConfig) -> Self {
        Self {
            lightweight_xgb: LightweightXGBoost::with_config(config.clone()),
            training_history: Vec::new(),
            config: config.clone(),
            stats: PredictorStats::default(),
        }
    }

    /// Predict optimal compression strategy using gradient boosting
    pub fn predict_strategy(&mut self, data: &[u8], feature_importance: &[FeatureImportanceScore]) -> Result<CompressionStrategy, CompressionError> {
        let start_time = std::time::Instant::now();

        // Use lightweight XGBoost for prediction
        let strategy = self.lightweight_xgb.predict_strategy(data, feature_importance)?;

        self.stats.prediction_count += 1;
        self.stats.total_prediction_time += start_time.elapsed();

        Ok(strategy)
    }

    /// Train gradient boosting models on feature/strategy pairs
    pub fn train(&mut self, features: &[Vec<FeatureImportanceScore>], targets: &[CompressionStrategy]) -> Result<(), CompressionError> {
        println!("Training XGBoost predictor on {} samples", features.len());

        if features.len() != targets.len() {
            return Err(CompressionError::InvalidFormat);
        }

        // Convert feature importance scores to numeric features for training
        let numeric_features: Vec<Vec<f32>> = features.iter()
            .map(|feature_set| {
                let mut numeric_feature = Vec::new();
                for score in feature_set.iter().take(20) {
                    numeric_feature.push(score.importance);
                    numeric_feature.push(score.gain);
                    numeric_feature.push(score.cover);
                }
                // Pad to fixed size
                while numeric_feature.len() < 60 {
                    numeric_feature.push(0.0);
                }
                numeric_feature
            })
            .collect();

        // Train the lightweight XGBoost model
        self.lightweight_xgb.train(&numeric_features, targets)?;

        self.stats.feature_importance_calculated = true;
        println!("✅ XGBoost predictor training completed!");
        Ok(())
    }



    /// Get feature importance scores
    pub fn get_feature_importance(&self) -> &[f32] {
        self.lightweight_xgb.get_feature_importance()
    }

    /// Get prediction statistics
    pub fn get_stats(&self) -> &PredictorStats {
        &self.stats
    }
}

impl Default for XGBoostPredictor {
    fn default() -> Self {
        Self::new()
    }
}

/// Training record for history tracking
#[derive(Debug, Clone)]
struct TrainingRecord {
    features: Vec<f32>,
    target: CompressionStrategy,
    prediction_accuracy: f32,
    timestamp: std::time::SystemTime,
}

/// Predictor statistics
#[derive(Debug, Clone, Default)]
pub struct PredictorStats {
    pub prediction_count: usize,
    pub total_prediction_time: std::time::Duration,
    pub accuracy_score: f32,
    pub feature_importance_calculated: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xgboost_predictor_creation() {
        let predictor = XGBoostPredictor::new();
        assert_eq!(predictor.stats.prediction_count, 0);
        assert!(!predictor.stats.feature_importance_calculated);
    }

    #[test]
    fn test_lightweight_xgboost_integration() {
        let mut predictor = XGBoostPredictor::new();

        // Test basic functionality
        assert_eq!(predictor.stats.prediction_count, 0);
        assert!(!predictor.stats.feature_importance_calculated);

        // Create dummy feature importance data
        let test_features = vec![
            FeatureImportanceScore {
                feature_name: "entropy".to_string(),
                importance: 0.5,
                gain: 0.3,
                cover: 0.2,
            },
            FeatureImportanceScore {
                feature_name: "repetition".to_string(),
                importance: 0.8,
                gain: 0.6,
                cover: 0.4,
            },
        ];

        let test_data = vec![42u8; 100];

        // Test prediction (should work even without training)
        let result = predictor.predict_strategy(&test_data, &test_features);
        assert!(result.is_ok());
        assert_eq!(predictor.stats.prediction_count, 1);
    }
}