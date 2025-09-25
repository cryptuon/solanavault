//! # Model Training and Optimization
//!
//! Trains and optimizes machine learning models for compression.

use super::*;
use rand::Rng;

/// Model trainer for machine learning compression
#[derive(Debug, Clone)]
pub struct ModelTrainer {
    /// Model weights
    weights: Vec<Vec<f32>>,
    /// Training data buffer
    training_buffer: Vec<TrainingBatch>,
    /// Optimization state
    optimizer_state: OptimizerState,
    /// Training configuration
    config: TrainingConfig,
    /// Training statistics
    stats: TrainingStats,
}

impl ModelTrainer {
    /// Creates a new model trainer
    pub fn new() -> Self {
        Self {
            weights: Vec::new(),
            training_buffer: Vec::new(),
            optimizer_state: OptimizerState::default(),
            config: TrainingConfig::default(),
            stats: TrainingStats::default(),
        }
    }

    /// Creates a model trainer with custom configuration
    pub fn with_config(stage3_config: &Stage3Config) -> Self {
        Self {
            weights: Self::initialize_weights(stage3_config),
            training_buffer: Vec::new(),
            optimizer_state: OptimizerState {
                learning_rate: stage3_config.learning_rate,
                momentum: 0.9,
                decay: 0.0001,
                iteration: 0,
                velocity: Vec::new(),
            },
            config: TrainingConfig {
                batch_size: 32,
                max_epochs: 100,
                learning_rate_schedule: LearningRateSchedule::Exponential { decay: 0.95 },
                regularization: RegularizationType::L2 { lambda: 0.001 },
                early_stopping: EarlyStoppingConfig {
                    patience: 10,
                    min_delta: 0.001,
                },
            },
            stats: TrainingStats::default(),
        }
    }

    /// Train on a batch of features and target data
    pub fn train_batch(&mut self, features: &[f32], target_data: &[u8]) -> Result<(), CompressionError> {
        // Convert target data to training format
        let target_features = self.convert_target_to_features(target_data)?;

        // Create training batch
        let batch = TrainingBatch {
            features: features.to_vec(),
            targets: target_features,
            timestamp: std::time::SystemTime::now(),
        };

        // Add to buffer
        self.training_buffer.push(batch);

        // Train when buffer is full
        if self.training_buffer.len() >= self.config.batch_size {
            self.train_buffered_data()?;
        }

        Ok(())
    }

    /// Finalize training process
    pub fn finalize_training(&mut self) -> Result<(), CompressionError> {
        // Train on remaining buffer data
        if !self.training_buffer.is_empty() {
            self.train_buffered_data()?;
        }

        // Apply final weight updates
        self.apply_final_updates()?;

        println!("Training completed: {} iterations, final loss: {:.6}",
                 self.stats.iterations, self.stats.final_loss);

        Ok(())
    }

    /// Get trained model weights
    pub fn get_weights(&self) -> Vec<Vec<f32>> {
        self.weights.clone()
    }

    /// Get model complexity score
    pub fn get_complexity_score(&self) -> f64 {
        let total_weights: usize = self.weights.iter().map(|layer| layer.len()).sum();
        total_weights as f64 / 1000.0 // Normalize by 1000
    }

    /// Update models with new data (online learning)
    pub fn update_models(&mut self, features: &[f32], data: &[u8]) -> Result<(), CompressionError> {
        // Perform single gradient descent step
        let target_features = self.convert_target_to_features(data)?;
        self.gradient_descent_step(features, &target_features)?;

        self.stats.online_updates += 1;
        Ok(())
    }

    /// Initialize model weights
    fn initialize_weights(config: &Stage3Config) -> Vec<Vec<f32>> {
        let layers = [
            config.feature_dimensions,
            config.feature_dimensions / 2,
            config.feature_dimensions / 4,
            config.prediction_window,
        ];

        let mut weights = Vec::new();
        for i in 0..layers.len() - 1 {
            let layer_size = layers[i] * layers[i + 1] + layers[i + 1]; // Weights + biases
            let limit = (6.0 / (layers[i] + layers[i + 1]) as f32).sqrt();

            let layer_weights: Vec<f32> = (0..layer_size)
                .map(|_| (rand::random::<f32>() * 2.0 - 1.0) * limit)
                .collect();

            weights.push(layer_weights);
        }

        weights
    }

    /// Train on buffered data
    fn train_buffered_data(&mut self) -> Result<(), CompressionError> {
        let batches = self.create_training_batches()?;

        for batch in batches {
            self.train_single_batch(&batch)?;
        }

        // Clear buffer
        self.training_buffer.clear();

        Ok(())
    }

    /// Create training batches from buffer
    fn create_training_batches(&self) -> Result<Vec<Vec<TrainingBatch>>, CompressionError> {
        let mut batches = Vec::new();

        for chunk in self.training_buffer.chunks(self.config.batch_size) {
            batches.push(chunk.to_vec());
        }

        Ok(batches)
    }

    /// Train on a single batch
    fn train_single_batch(&mut self, batch: &[TrainingBatch]) -> Result<(), CompressionError> {
        if batch.is_empty() {
            return Ok(());
        }

        // Calculate batch gradients
        let gradients = self.calculate_batch_gradients(batch)?;

        // Apply optimizer
        self.apply_optimizer_update(gradients)?;

        // Update statistics
        self.stats.iterations += 1;
        self.stats.batches_processed += 1;

        // Calculate and record loss
        let loss = self.calculate_batch_loss(batch)?;
        self.stats.losses.push(loss);
        self.stats.final_loss = loss;

        Ok(())
    }

    /// Calculate gradients for a batch
    fn calculate_batch_gradients(&self, batch: &[TrainingBatch]) -> Result<Vec<Vec<f32>>, CompressionError> {
        let mut batch_gradients: Vec<Vec<f32>> = self.weights.iter().map(|layer| vec![0.0; layer.len()]).collect();

        for sample in batch {
            let sample_gradients = self.calculate_sample_gradients(&sample.features, &sample.targets)?;

            // Accumulate gradients
            for (layer_idx, layer_grad) in sample_gradients.iter().enumerate() {
                for (weight_idx, &grad) in layer_grad.iter().enumerate() {
                    if layer_idx < batch_gradients.len() && weight_idx < batch_gradients[layer_idx].len() {
                        batch_gradients[layer_idx][weight_idx] += grad;
                    }
                }
            }
        }

        // Average gradients
        for layer_grad in &mut batch_gradients {
            for grad in layer_grad {
                *grad /= batch.len() as f32;
            }
        }

        Ok(batch_gradients)
    }

    /// Calculate gradients for a single sample
    fn calculate_sample_gradients(&self, features: &[f32], targets: &[f32]) -> Result<Vec<Vec<f32>>, CompressionError> {
        // Simplified gradient calculation
        let mut gradients: Vec<Vec<f32>> = self.weights.iter().map(|layer| vec![0.0; layer.len()]).collect();

        // Forward pass to get predictions
        let predictions = self.forward_pass(features)?;

        // Calculate output error
        let output_errors: Vec<f32> = predictions.iter()
            .zip(targets.iter())
            .map(|(pred, target)| pred - target)
            .collect();

        // Backpropagate errors (simplified)
        if let Some(last_layer_grad) = gradients.last_mut() {
            for (i, &error) in output_errors.iter().enumerate() {
                if i < last_layer_grad.len() {
                    last_layer_grad[i] = error * self.optimizer_state.learning_rate;
                }
            }
        }

        Ok(gradients)
    }

    /// Forward pass through the network
    fn forward_pass(&self, input: &[f32]) -> Result<Vec<f32>, CompressionError> {
        if self.weights.is_empty() {
            return Ok(input.to_vec());
        }

        let mut current = input.to_vec();

        // Simplified forward pass
        for layer_weights in &self.weights {
            let output_size = layer_weights.len() / (current.len() + 1); // Accounting for biases
            let mut next = vec![0.0; output_size];

            for i in 0..output_size {
                // Weights
                for j in 0..current.len() {
                    if i * current.len() + j < layer_weights.len() {
                        next[i] += current[j] * layer_weights[i * current.len() + j];
                    }
                }
                // Bias
                if current.len() * output_size + i < layer_weights.len() {
                    next[i] += layer_weights[current.len() * output_size + i];
                }
                // Activation (ReLU)
                next[i] = next[i].max(0.0);
            }

            current = next;
        }

        Ok(current)
    }

    /// Apply optimizer update
    fn apply_optimizer_update(&mut self, gradients: Vec<Vec<f32>>) -> Result<(), CompressionError> {
        match &self.config.learning_rate_schedule {
            LearningRateSchedule::Constant => {
                // Keep current learning rate
            }
            LearningRateSchedule::Exponential { decay } => {
                self.optimizer_state.learning_rate *= decay;
            }
            LearningRateSchedule::StepDecay { step_size, gamma } => {
                if self.optimizer_state.iteration % step_size == 0 {
                    self.optimizer_state.learning_rate *= gamma;
                }
            }
        }

        // Apply gradients with momentum
        for (layer_idx, layer_grad) in gradients.iter().enumerate() {
            if layer_idx < self.weights.len() {
                for (weight_idx, &grad) in layer_grad.iter().enumerate() {
                    if weight_idx < self.weights[layer_idx].len() {
                        // Apply L2 regularization
                        let reg_term = match &self.config.regularization {
                            RegularizationType::None => 0.0,
                            RegularizationType::L2 { lambda } => {
                                lambda * self.weights[layer_idx][weight_idx]
                            }
                            RegularizationType::L1 { lambda } => {
                                lambda * self.weights[layer_idx][weight_idx].signum()
                            }
                        };

                        // Update weight
                        self.weights[layer_idx][weight_idx] -=
                            self.optimizer_state.learning_rate * (grad + reg_term);
                    }
                }
            }
        }

        self.optimizer_state.iteration += 1;
        Ok(())
    }

    /// Calculate loss for a batch
    fn calculate_batch_loss(&self, batch: &[TrainingBatch]) -> Result<f32, CompressionError> {
        let mut total_loss = 0.0;

        for sample in batch {
            let predictions = self.forward_pass(&sample.features)?;
            let loss = self.calculate_sample_loss(&predictions, &sample.targets);
            total_loss += loss;
        }

        Ok(total_loss / batch.len() as f32)
    }

    /// Calculate loss for a single sample
    fn calculate_sample_loss(&self, predictions: &[f32], targets: &[f32]) -> f32 {
        // Mean squared error
        let mut mse = 0.0;
        let min_len = predictions.len().min(targets.len());

        for i in 0..min_len {
            let diff = predictions[i] - targets[i];
            mse += diff * diff;
        }

        mse / min_len as f32
    }

    /// Convert target data to feature format
    fn convert_target_to_features(&self, data: &[u8]) -> Result<Vec<f32>, CompressionError> {
        // Simple conversion: normalize bytes to [0, 1]
        let features: Vec<f32> = data.iter()
            .take(32) // Limit to 32 features
            .map(|&b| b as f32 / 255.0)
            .collect();

        // Pad to required size
        let mut padded = features;
        while padded.len() < 32 {
            padded.push(0.0);
        }

        Ok(padded)
    }

    /// Apply final weight updates
    fn apply_final_updates(&mut self) -> Result<(), CompressionError> {
        // Apply any final regularization or normalization
        for layer in &mut self.weights {
            for weight in layer {
                // Clip weights to prevent overflow
                *weight = weight.max(-10.0).min(10.0);
            }
        }

        Ok(())
    }

    /// Perform single gradient descent step
    fn gradient_descent_step(&mut self, features: &[f32], targets: &[f32]) -> Result<(), CompressionError> {
        let gradients = self.calculate_sample_gradients(features, targets)?;
        self.apply_optimizer_update(gradients)?;
        Ok(())
    }
}

impl Default for ModelTrainer {
    fn default() -> Self {
        Self::new()
    }
}

/// Training batch
#[derive(Debug, Clone)]
pub struct TrainingBatch {
    pub features: Vec<f32>,
    pub targets: Vec<f32>,
    pub timestamp: std::time::SystemTime,
}

/// Optimizer state
#[derive(Debug, Clone, Default)]
pub struct OptimizerState {
    pub learning_rate: f32,
    pub momentum: f32,
    pub decay: f32,
    pub iteration: usize,
    pub velocity: Vec<Vec<f32>>,
}

/// Training configuration
#[derive(Debug, Clone)]
pub struct TrainingConfig {
    pub batch_size: usize,
    pub max_epochs: usize,
    pub learning_rate_schedule: LearningRateSchedule,
    pub regularization: RegularizationType,
    pub early_stopping: EarlyStoppingConfig,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            batch_size: 32,
            max_epochs: 100,
            learning_rate_schedule: LearningRateSchedule::Exponential { decay: 0.95 },
            regularization: RegularizationType::L2 { lambda: 0.001 },
            early_stopping: EarlyStoppingConfig {
                patience: 10,
                min_delta: 0.001,
            },
        }
    }
}

/// Learning rate schedule
#[derive(Debug, Clone)]
pub enum LearningRateSchedule {
    Constant,
    Exponential { decay: f32 },
    StepDecay { step_size: usize, gamma: f32 },
}

/// Regularization type
#[derive(Debug, Clone)]
pub enum RegularizationType {
    None,
    L1 { lambda: f32 },
    L2 { lambda: f32 },
}

/// Early stopping configuration
#[derive(Debug, Clone)]
pub struct EarlyStoppingConfig {
    pub patience: usize,
    pub min_delta: f32,
}

/// Training statistics
#[derive(Debug, Clone, Default)]
pub struct TrainingStats {
    pub iterations: usize,
    pub batches_processed: usize,
    pub online_updates: usize,
    pub losses: Vec<f32>,
    pub final_loss: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_trainer_creation() {
        let trainer = ModelTrainer::new();
        assert!(trainer.weights.is_empty());
        assert_eq!(trainer.stats.iterations, 0);
    }

    #[test]
    fn test_weight_initialization() {
        let config = Stage3Config::default();
        let weights = ModelTrainer::initialize_weights(&config);
        assert!(!weights.is_empty());

        // Check that weights are in reasonable range
        for layer in &weights {
            for &weight in layer {
                assert!(weight.abs() < 1.0);
            }
        }
    }

    #[test]
    fn test_training_batch() {
        let mut trainer = ModelTrainer::with_config(&Stage3Config::default());
        let features = vec![0.5; 128];
        let target_data = vec![1, 2, 3, 4];

        trainer.train_batch(&features, &target_data).unwrap();
        assert_eq!(trainer.training_buffer.len(), 1);
    }

    #[test]
    fn test_target_conversion() {
        let trainer = ModelTrainer::new();
        let data = vec![128, 64, 192, 32];

        let features = trainer.convert_target_to_features(&data).unwrap();
        assert_eq!(features.len(), 32);

        // Check normalization
        assert!((features[0] - 128.0/255.0).abs() < 0.01);
        assert!((features[1] - 64.0/255.0).abs() < 0.01);
    }

    #[test]
    fn test_forward_pass() {
        let trainer = ModelTrainer::with_config(&Stage3Config::default());
        let input = vec![0.5; 128];

        let output = trainer.forward_pass(&input).unwrap();
        assert!(!output.is_empty());

        // All outputs should be valid
        for &val in &output {
            assert!(val.is_finite());
        }
    }

    #[test]
    fn test_loss_calculation() {
        let trainer = ModelTrainer::new();
        let predictions = vec![0.8, 0.6, 0.4];
        let targets = vec![1.0, 0.5, 0.3];

        let loss = trainer.calculate_sample_loss(&predictions, &targets);
        assert!(loss >= 0.0);
    }

    #[test]
    fn test_online_learning() {
        let mut trainer = ModelTrainer::with_config(&Stage3Config::default());
        let features = vec![0.5; 128];
        let data = vec![1, 2, 3, 4];

        trainer.update_models(&features, &data).unwrap();
        assert_eq!(trainer.stats.online_updates, 1);
    }
}