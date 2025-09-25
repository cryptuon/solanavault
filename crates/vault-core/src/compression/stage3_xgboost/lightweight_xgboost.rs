//! # Lightweight XGBoost-Style Implementation
//!
//! Custom gradient boosting implementation optimized for compression pattern prediction
//! without heavy external dependencies.

use super::*;
use std::collections::HashMap;
use rand::Rng;

/// Lightweight gradient boosting predictor for compression
#[derive(Debug, Clone)]
pub struct LightweightXGBoost {
    /// Collection of decision trees (weak learners)
    trees: Vec<DecisionTree>,

    /// Feature importance scores
    feature_importance: Vec<f32>,

    /// Learning rate
    learning_rate: f32,

    /// Base prediction (mean of training targets)
    base_prediction: f32,

    /// Configuration
    config: XGBoostConfig,

    /// Training statistics
    stats: XGBoostStats,
}

impl LightweightXGBoost {
    /// Creates a new lightweight XGBoost predictor
    pub fn new() -> Self {
        Self {
            trees: Vec::new(),
            feature_importance: Vec::new(),
            learning_rate: 0.1,
            base_prediction: 0.0,
            config: XGBoostConfig::default(),
            stats: XGBoostStats::default(),
        }
    }

    /// Creates predictor with custom configuration
    pub fn with_config(config: XGBoostConfig) -> Self {
        Self {
            trees: Vec::new(),
            feature_importance: Vec::new(),
            learning_rate: config.learning_rate,
            base_prediction: 0.0,
            stats: XGBoostStats::default(),
            config,
        }
    }

    /// Train gradient boosting model on compression data
    pub fn train(&mut self, features: &[Vec<f32>], targets: &[CompressionStrategy]) -> Result<(), CompressionError> {
        println!("Training Corrected XGBoost on {} samples", features.len());

        if features.is_empty() || features.len() != targets.len() {
            return Err(CompressionError::InvalidFormat);
        }

        // Convert targets to numeric values
        let numeric_targets: Vec<f32> = targets.iter().map(|s| self.strategy_to_numeric(s)).collect();

        // Store base prediction (mean of targets)
        self.base_prediction = numeric_targets.iter().sum::<f32>() / numeric_targets.len() as f32;

        // Initialize predictions with base prediction
        let mut predictions = vec![self.base_prediction; features.len()];

        // Gradient boosting iterations
        for iteration in 0..self.config.n_estimators {
            // Calculate residuals (negative gradients for squared loss)
            let residuals: Vec<f32> = numeric_targets.iter()
                .zip(predictions.iter())
                .map(|(target, pred)| target - pred)
                .collect();

            // Train decision tree on residuals
            let mut tree = DecisionTree::new(self.config.max_depth);
            tree.train(features, &residuals)?;

            // Update predictions for loss calculation
            for (i, feature) in features.iter().enumerate() {
                let tree_prediction = tree.predict(feature);
                predictions[i] += self.learning_rate * tree_prediction;
            }

            // Store the unscaled tree - we'll apply learning rate during prediction
            self.trees.push(tree);

            // Calculate training loss
            let loss = self.calculate_mse(&numeric_targets, &predictions);

            if iteration % 10 == 0 {
                println!("Iteration {}: Loss = {:.6}", iteration, loss);
            }

            // Early stopping
            if loss < 0.001 {
                println!("Early stopping at iteration {} (loss: {:.6})", iteration, loss);
                break;
            }
        }

        // Calculate feature importance
        self.calculate_feature_importance(features)?;

        println!("✅ Corrected XGBoost training completed with {} trees, base prediction: {:.3}",
                 self.trees.len(), self.base_prediction);
        Ok(())
    }

    /// Predict compression strategy for given features
    pub fn predict(&self, features: &[f32]) -> CompressionStrategy {
        if self.trees.is_empty() {
            return CompressionStrategy::DictionaryBased; // Default fallback
        }

        // CORRECTED: Start with base prediction and aggregate trees with learning rate
        let mut prediction = self.base_prediction;
        for tree in &self.trees {
            // Apply learning rate during prediction (this matches training logic)
            prediction += self.learning_rate * tree.predict(features);
        }

        // Convert numeric prediction back to strategy
        self.numeric_to_strategy(prediction)
    }

    /// Predict optimal compression strategy for data
    pub fn predict_strategy(&mut self, data: &[u8], feature_importance: &[FeatureImportanceScore]) -> Result<CompressionStrategy, CompressionError> {
        let features = self.extract_features_for_prediction(data, feature_importance)?;

        // Enhanced strategy selection using blockchain-specific features
        let strategy = self.predict_enhanced_strategy(&features, feature_importance)?;

        self.stats.strategy_predictions += 1;
        Ok(strategy)
    }

    /// Extract features for prediction
    fn extract_features_for_prediction(&self, data: &[u8], feature_importance: &[FeatureImportanceScore]) -> Result<Vec<f32>, CompressionError> {
        let mut features = Vec::new();

        // Basic data characteristics
        features.push(data.len() as f32 / 1000.0); // Normalized size
        features.push(self.calculate_entropy(data));
        features.push(self.calculate_repetition_ratio(data));
        features.push(self.calculate_byte_diversity(data));
        features.push(self.calculate_pattern_complexity(data));

        // Enhanced blockchain-specific features
        features.push(self.extract_blockchain_feature_score(feature_importance, "spl_token_density"));
        features.push(self.extract_blockchain_feature_score(feature_importance, "transfer_instruction_frequency"));
        features.push(self.extract_blockchain_feature_score(feature_importance, "token_account_similarity"));
        features.push(self.extract_blockchain_feature_score(feature_importance, "repetition_scale_32"));
        features.push(self.extract_blockchain_feature_score(feature_importance, "sequence_periodicity"));
        features.push(self.extract_blockchain_feature_score(feature_importance, "hft_patterns"));

        // Feature importance metrics (top 8 to accommodate new features)
        for score in feature_importance.iter().take(8) {
            features.push(score.importance);
            features.push(score.gain);
        }

        // Pad to fixed size (27 features)
        while features.len() < 27 {
            features.push(0.0);
        }

        Ok(features)
    }

    /// Enhanced strategy prediction using blockchain-specific logic
    fn predict_enhanced_strategy(&self, features: &[f32], feature_importance: &[FeatureImportanceScore]) -> Result<CompressionStrategy, CompressionError> {
        // Extract key blockchain feature scores
        let spl_token_score = self.extract_blockchain_feature_score(feature_importance, "spl_token_density");
        let transfer_freq_score = self.extract_blockchain_feature_score(feature_importance, "transfer_instruction_frequency");
        let repetition_score = self.extract_blockchain_feature_score(feature_importance, "repetition_scale_32");
        let periodicity_score = self.extract_blockchain_feature_score(feature_importance, "sequence_periodicity");
        let hft_score = self.extract_blockchain_feature_score(feature_importance, "hft_patterns");

        // Debug: Show all feature scores for analysis
        println!("🔍 Feature scores: SPL={:.3}, Transfer={:.3}, Rep={:.3}, Period={:.3}, HFT={:.3}",
                 spl_token_score, transfer_freq_score, repetition_score, periodicity_score, hft_score);

        // Rule-based strategy selection for blockchain patterns
        // Token Transfer strategy - prioritize when token patterns are strong (very low thresholds for now)
        if spl_token_score > 0.02 || transfer_freq_score > 0.01 {
            println!("🎯 Selecting TokenTransfer strategy: SPL={:.3}, Transfer={:.3}", spl_token_score, transfer_freq_score);
            return Ok(CompressionStrategy::TokenTransfer);
        }

        // Repetitive strategy - prioritize when repetitive patterns are strong
        if repetition_score > 0.4 || periodicity_score > 0.35 || hft_score > 0.3 {
            println!("🎯 Selecting Repetitive strategy: Rep={:.3}, Period={:.3}, HFT={:.3}", repetition_score, periodicity_score, hft_score);
            return Ok(CompressionStrategy::Repetitive);
        }

        // Fall back to ML prediction for other cases
        let prediction = self.predict(features);
        println!("🤖 ML prediction fallback: {:?}", prediction);
        Ok(prediction)
    }

    /// Extract specific blockchain feature score by name
    fn extract_blockchain_feature_score(&self, feature_importance: &[FeatureImportanceScore], feature_name: &str) -> f32 {
        feature_importance
            .iter()
            .find(|score| score.feature_name == feature_name)
            .map(|score| score.importance)
            .unwrap_or(0.0)
    }

    /// Calculate feature importance based on tree splits
    fn calculate_feature_importance(&mut self, training_features: &[Vec<f32>]) -> Result<(), CompressionError> {
        if training_features.is_empty() {
            return Ok(());
        }

        let feature_count = training_features[0].len();
        let mut importance_scores = vec![0.0; feature_count];

        // Aggregate importance from all trees
        for tree in &self.trees {
            let tree_importance = tree.get_feature_importance();
            for (i, &importance) in tree_importance.iter().enumerate() {
                if i < importance_scores.len() {
                    importance_scores[i] += importance;
                }
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

    /// Convert compression strategy to numeric value
    fn strategy_to_numeric(&self, strategy: &CompressionStrategy) -> f32 {
        match strategy {
            CompressionStrategy::DictionaryBased => 0.0,
            CompressionStrategy::PatternBased => 1.0,
            CompressionStrategy::TreeBased => 2.0,
            CompressionStrategy::Hybrid => 3.0,
            CompressionStrategy::TokenTransfer => 4.0,
            CompressionStrategy::Repetitive => 5.0,
        }
    }

    /// Convert numeric value to compression strategy
    fn numeric_to_strategy(&self, value: f32) -> CompressionStrategy {
        let rounded = value.round() as i32;
        match rounded {
            0 => CompressionStrategy::DictionaryBased,
            1 => CompressionStrategy::PatternBased,
            2 => CompressionStrategy::TreeBased,
            3 => CompressionStrategy::Hybrid,
            4 => CompressionStrategy::TokenTransfer,
            5 => CompressionStrategy::Repetitive,
            _ => {
                // For out-of-range values, choose based on the fractional part
                if value < 0.5 {
                    CompressionStrategy::DictionaryBased
                } else if value < 1.5 {
                    CompressionStrategy::PatternBased
                } else if value < 2.5 {
                    CompressionStrategy::TreeBased
                } else if value < 3.5 {
                    CompressionStrategy::Hybrid
                } else if value < 4.5 {
                    CompressionStrategy::TokenTransfer
                } else {
                    CompressionStrategy::Repetitive
                }
            }
        }
    }

    /// Calculate mean squared error
    fn calculate_mse(&self, targets: &[f32], predictions: &[f32]) -> f32 {
        let sum_squared_errors: f32 = targets.iter()
            .zip(predictions.iter())
            .map(|(target, pred)| (target - pred).powi(2))
            .sum();

        sum_squared_errors / targets.len() as f32
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

        entropy / 8.0 // Normalize to [0, 1]
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

    /// Calculate byte diversity
    fn calculate_byte_diversity(&self, data: &[u8]) -> f32 {
        let mut unique_bytes = std::collections::HashSet::new();
        for &byte in data {
            unique_bytes.insert(byte);
        }

        unique_bytes.len() as f32 / 256.0
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

    /// Get feature importance scores
    pub fn get_feature_importance(&self) -> &[f32] {
        &self.feature_importance
    }

    /// Get model statistics
    pub fn get_stats(&self) -> &XGBoostStats {
        &self.stats
    }
}

/// Simple decision tree for gradient boosting
#[derive(Debug, Clone)]
struct DecisionTree {
    root: Option<TreeNode>,
    max_depth: usize,
    feature_importance: Vec<f32>,
}

impl DecisionTree {
    fn new(max_depth: usize) -> Self {
        Self {
            root: None,
            max_depth,
            feature_importance: Vec::new(),
        }
    }

    fn train(&mut self, features: &[Vec<f32>], targets: &[f32]) -> Result<(), CompressionError> {
        if features.is_empty() {
            return Err(CompressionError::InvalidFormat);
        }

        self.feature_importance = vec![0.0; features[0].len()];

        let indices: Vec<usize> = (0..features.len()).collect();
        self.root = Some(self.build_tree(features, targets, &indices, 0));

        Ok(())
    }

    fn build_tree(&mut self, features: &[Vec<f32>], targets: &[f32], indices: &[usize], depth: usize) -> TreeNode {
        // Base cases
        if indices.is_empty() || depth >= self.max_depth {
            let mean_target = if indices.is_empty() {
                0.0
            } else {
                indices.iter().map(|&i| targets[i]).sum::<f32>() / indices.len() as f32
            };
            return TreeNode::Leaf { value: mean_target };
        }

        // Find best split
        let best_split = self.find_best_split(features, targets, indices);

        if let Some((feature_idx, threshold, left_indices, right_indices)) = best_split {
            // Update feature importance
            if feature_idx < self.feature_importance.len() {
                self.feature_importance[feature_idx] += self.calculate_split_importance(targets, indices, &left_indices, &right_indices);
            }

            // Recursively build subtrees
            let left_child = Box::new(self.build_tree(features, targets, &left_indices, depth + 1));
            let right_child = Box::new(self.build_tree(features, targets, &right_indices, depth + 1));

            TreeNode::Split {
                feature_idx,
                threshold,
                left: left_child,
                right: right_child,
            }
        } else {
            // No good split found, create leaf
            let mean_target = indices.iter().map(|&i| targets[i]).sum::<f32>() / indices.len() as f32;
            TreeNode::Leaf { value: mean_target }
        }
    }

    fn find_best_split(&self, features: &[Vec<f32>], targets: &[f32], indices: &[usize]) -> Option<(usize, f32, Vec<usize>, Vec<usize>)> {
        if indices.len() < 2 {
            return None;
        }

        let mut best_split = None;
        let mut best_variance_reduction = 0.0;

        let current_variance = self.calculate_variance(targets, indices);

        // Try splits on each feature
        for feature_idx in 0..features[0].len() {
            // Get unique values for this feature
            let mut feature_values: Vec<f32> = indices.iter()
                .map(|&i| features[i][feature_idx])
                .collect();
            feature_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            feature_values.dedup();

            // Try each unique value as a threshold
            for &threshold in &feature_values {
                let (left_indices, right_indices) = self.split_indices(features, indices, feature_idx, threshold);

                if left_indices.is_empty() || right_indices.is_empty() {
                    continue;
                }

                // Calculate variance reduction
                let left_variance = self.calculate_variance(targets, &left_indices);
                let right_variance = self.calculate_variance(targets, &right_indices);

                let weighted_variance = (left_indices.len() as f32 * left_variance +
                                       right_indices.len() as f32 * right_variance) / indices.len() as f32;

                let variance_reduction = current_variance - weighted_variance;

                if variance_reduction > best_variance_reduction {
                    best_variance_reduction = variance_reduction;
                    best_split = Some((feature_idx, threshold, left_indices, right_indices));
                }
            }
        }

        best_split
    }

    fn split_indices(&self, features: &[Vec<f32>], indices: &[usize], feature_idx: usize, threshold: f32) -> (Vec<usize>, Vec<usize>) {
        let mut left = Vec::new();
        let mut right = Vec::new();

        for &idx in indices {
            if features[idx][feature_idx] <= threshold {
                left.push(idx);
            } else {
                right.push(idx);
            }
        }

        (left, right)
    }

    fn calculate_variance(&self, targets: &[f32], indices: &[usize]) -> f32 {
        if indices.is_empty() {
            return 0.0;
        }

        let mean = indices.iter().map(|&i| targets[i]).sum::<f32>() / indices.len() as f32;
        let variance = indices.iter()
            .map(|&i| (targets[i] - mean).powi(2))
            .sum::<f32>() / indices.len() as f32;

        variance
    }

    fn calculate_split_importance(&self, targets: &[f32], parent_indices: &[usize], left_indices: &[usize], right_indices: &[usize]) -> f32 {
        let parent_variance = self.calculate_variance(targets, parent_indices);
        let left_variance = self.calculate_variance(targets, left_indices);
        let right_variance = self.calculate_variance(targets, right_indices);

        let weighted_child_variance = (left_indices.len() as f32 * left_variance +
                                     right_indices.len() as f32 * right_variance) / parent_indices.len() as f32;

        parent_variance - weighted_child_variance
    }

    fn predict(&self, features: &[f32]) -> f32 {
        match &self.root {
            Some(node) => self.predict_node(node, features),
            None => 0.0,
        }
    }

    fn predict_node(&self, node: &TreeNode, features: &[f32]) -> f32 {
        match node {
            TreeNode::Leaf { value } => *value,
            TreeNode::Split { feature_idx, threshold, left, right } => {
                if *feature_idx < features.len() && features[*feature_idx] <= *threshold {
                    self.predict_node(left, features)
                } else {
                    self.predict_node(right, features)
                }
            }
        }
    }

    fn get_feature_importance(&self) -> &[f32] {
        &self.feature_importance
    }
}

/// Tree node for decision tree
#[derive(Debug, Clone)]
enum TreeNode {
    Leaf {
        value: f32,
    },
    Split {
        feature_idx: usize,
        threshold: f32,
        left: Box<TreeNode>,
        right: Box<TreeNode>,
    },
}

impl Default for LightweightXGBoost {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lightweight_xgboost_creation() {
        let xgb = LightweightXGBoost::new();
        assert!(xgb.trees.is_empty());
        assert_eq!(xgb.learning_rate, 0.1);
    }

    #[test]
    fn test_strategy_conversion() {
        let xgb = LightweightXGBoost::new();

        let strategy = CompressionStrategy::PatternBased;
        let numeric = xgb.strategy_to_numeric(&strategy);
        assert_eq!(numeric, 1.0);

        let converted_back = xgb.numeric_to_strategy(numeric);
        match converted_back {
            CompressionStrategy::PatternBased => assert!(true),
            _ => assert!(false),
        }
    }

    #[test]
    fn test_entropy_calculation() {
        let xgb = LightweightXGBoost::new();

        // Uniform distribution should have high entropy
        let uniform_data = (0..=255u8).collect::<Vec<_>>();
        let entropy = xgb.calculate_entropy(&uniform_data);
        assert!(entropy > 0.9);

        // Single value should have zero entropy
        let constant_data = vec![42u8; 100];
        let entropy = xgb.calculate_entropy(&constant_data);
        assert!(entropy < 0.01);
    }

    #[test]
    fn test_repetition_ratio() {
        let xgb = LightweightXGBoost::new();

        // High repetition
        let repetitive = vec![1u8, 1u8, 1u8, 1u8];
        let ratio = xgb.calculate_repetition_ratio(&repetitive);
        assert!(ratio > 0.9);

        // No repetition
        let non_repetitive = vec![1u8, 2u8, 3u8, 4u8];
        let ratio = xgb.calculate_repetition_ratio(&non_repetitive);
        assert!(ratio < 0.1);
    }

    #[test]
    fn test_decision_tree_basic() {
        let mut tree = DecisionTree::new(3);

        // Simple training data
        let features = vec![
            vec![1.0, 2.0],
            vec![2.0, 1.0],
            vec![3.0, 4.0],
            vec![4.0, 3.0],
        ];
        let targets = vec![0.0, 0.0, 1.0, 1.0];

        tree.train(&features, &targets).unwrap();

        // Test predictions
        let pred1 = tree.predict(&vec![1.5, 1.5]);
        let pred2 = tree.predict(&vec![3.5, 3.5]);

        // Should predict different values for different regions
        assert!((pred1 - pred2).abs() > 0.1);
    }
}