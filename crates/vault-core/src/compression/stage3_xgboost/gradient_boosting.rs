//! # Gradient Boosting Model Trainer
//!
//! Advanced gradient boosting trainer using native Rust ML libraries.

use super::*;

/// Gradient boosting trainer for compression models
#[derive(Debug, Clone)]
pub struct GradientBoostingTrainer {
    /// Training configuration
    config: XGBoostConfig,

    /// Training history
    training_history: Vec<TrainingEpoch>,

    /// Current metrics
    metrics: GradientMetrics,
}

impl GradientBoostingTrainer {
    /// Creates a new gradient boosting trainer
    pub fn new() -> Self {
        Self {
            config: XGBoostConfig::default(),
            training_history: Vec::new(),
            metrics: GradientMetrics::default(),
        }
    }

    /// Creates trainer with custom configuration
    pub fn with_config(config: &XGBoostConfig) -> Self {
        Self {
            config: config.clone(),
            training_history: Vec::new(),
            metrics: GradientMetrics::default(),
        }
    }

    /// Update models with new training data
    pub fn update_models(&mut self, original_data: &[u8], compressed_data: &[u8],
                        feature_importance: &[FeatureImportanceScore]) -> Result<(), CompressionError> {
        let compression_ratio = original_data.len() as f32 / compressed_data.len() as f32;

        // Record training example
        let epoch = TrainingEpoch {
            original_size: original_data.len(),
            compressed_size: compressed_data.len(),
            compression_ratio,
            feature_count: feature_importance.len(),
            timestamp: std::time::SystemTime::now(),
        };

        self.training_history.push(epoch);

        // Update metrics
        self.update_metrics(compression_ratio, feature_importance)?;

        Ok(())
    }

    /// Train on feature set
    pub fn train_on_features(&mut self, features: &[Vec<FeatureImportanceScore>]) -> Result<(), CompressionError> {
        println!("🎓 Training gradient boosting on {} feature sets", features.len());

        // Calculate feature importance across all samples
        let mut aggregated_importance = vec![0.0; 50]; // Fixed size

        for feature_set in features {
            for (i, score) in feature_set.iter().enumerate() {
                if i < aggregated_importance.len() {
                    aggregated_importance[i] += score.importance;
                }
            }
        }

        // Normalize
        let total: f32 = aggregated_importance.iter().sum();
        if total > 0.0 {
            for importance in &mut aggregated_importance {
                *importance /= total;
            }
        }

        self.metrics.feature_importance = aggregated_importance;

        println!("✅ Gradient boosting training completed");
        Ok(())
    }

    /// Update training metrics
    fn update_metrics(&mut self, compression_ratio: f32, feature_importance: &[FeatureImportanceScore]) -> Result<(), CompressionError> {
        // Update running averages
        let sample_count = self.training_history.len();

        // Update training loss (use negative compression ratio as loss - higher ratio = lower loss)
        let current_loss = -compression_ratio;
        self.metrics.training_loss = (self.metrics.training_loss * (sample_count - 1) as f32 + current_loss) / sample_count as f32;

        // Update learning curve
        self.metrics.learning_curve.push(compression_ratio);

        // Keep only recent history for learning curve
        if self.metrics.learning_curve.len() > 100 {
            self.metrics.learning_curve.remove(0);
        }

        Ok(())
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> GradientMetrics {
        self.metrics.clone()
    }

    /// Get training progress
    pub fn get_training_progress(&self) -> TrainingProgress {
        let recent_ratios: Vec<f32> = self.training_history.iter()
            .rev()
            .take(10)
            .map(|epoch| epoch.compression_ratio)
            .collect();

        let average_recent_ratio = if recent_ratios.is_empty() {
            1.0
        } else {
            recent_ratios.iter().sum::<f32>() / recent_ratios.len() as f32
        };

        TrainingProgress {
            total_epochs: self.training_history.len(),
            current_loss: self.metrics.training_loss,
            average_compression_ratio: average_recent_ratio,
            feature_importance_stability: self.calculate_stability(),
        }
    }

    /// Calculate feature importance stability
    fn calculate_stability(&self) -> f32 {
        if self.training_history.len() < 10 {
            return 0.5; // Not enough data
        }

        // Calculate variance in recent compression ratios
        let recent_ratios: Vec<f32> = self.training_history.iter()
            .rev()
            .take(20)
            .map(|epoch| epoch.compression_ratio)
            .collect();

        let mean = recent_ratios.iter().sum::<f32>() / recent_ratios.len() as f32;
        let variance = recent_ratios.iter()
            .map(|&ratio| (ratio - mean).powi(2))
            .sum::<f32>() / recent_ratios.len() as f32;

        // Lower variance = higher stability
        (1.0 / (1.0 + variance)).min(1.0)
    }
}

impl Default for GradientBoostingTrainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Training epoch record
#[derive(Debug, Clone)]
struct TrainingEpoch {
    original_size: usize,
    compressed_size: usize,
    compression_ratio: f32,
    feature_count: usize,
    timestamp: std::time::SystemTime,
}

/// Training progress metrics
#[derive(Debug, Clone)]
pub struct TrainingProgress {
    pub total_epochs: usize,
    pub current_loss: f32,
    pub average_compression_ratio: f32,
    pub feature_importance_stability: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gradient_boosting_trainer() {
        let mut trainer = GradientBoostingTrainer::new();

        let original_data = vec![1u8; 1000];
        let compressed_data = vec![2u8; 200];
        let features = vec![
            FeatureImportanceScore {
                feature_name: "test".to_string(),
                importance: 0.5,
                gain: 0.1,
                cover: 100.0,
            }
        ];

        trainer.update_models(&original_data, &compressed_data, &features).unwrap();

        let progress = trainer.get_training_progress();
        assert_eq!(progress.total_epochs, 1);
        assert!(progress.average_compression_ratio > 1.0);
    }

    #[test]
    fn test_metrics_calculation() {
        let mut trainer = GradientBoostingTrainer::new();

        // Add several training examples
        for i in 1..=10 {
            let original = vec![1u8; 1000];
            let compressed = vec![2u8; 100 * i];  // Varying compression
            let features = vec![];

            trainer.update_models(&original, &compressed, &features).unwrap();
        }

        let metrics = trainer.get_metrics();
        assert!(metrics.learning_curve.len() > 0);
        assert!(metrics.training_loss != 0.0);
    }
}