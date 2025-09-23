//! # Pattern Recognition for Solana Transactions
//!
//! Identifies common patterns in Solana transaction structures for intelligent compression.

use super::super::traits::CompressionError;
use super::transaction_analysis::{BlockAnalysis, TransactionPattern};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Recognizes patterns in Solana transaction data
#[derive(Debug, Clone)]
pub struct PatternRecognizer {
    /// Known instruction patterns with their frequencies
    instruction_patterns: HashMap<Vec<u8>, PatternInfo>,
    /// Account usage patterns
    account_patterns: HashMap<String, AccountUsagePattern>,
    /// Signature patterns
    signature_patterns: HashMap<String, u32>,
    /// Minimum frequency for a pattern to be considered significant
    min_frequency: u32,
}

impl PatternRecognizer {
    /// Creates a new pattern recognizer
    pub fn new() -> Self {
        Self {
            instruction_patterns: HashMap::new(),
            account_patterns: HashMap::new(),
            signature_patterns: HashMap::new(),
            min_frequency: 2, // Pattern must appear at least 2 times
        }
    }

    /// Find patterns in the analyzed block data
    pub fn find_patterns(&mut self, analysis: &BlockAnalysis) -> Result<Vec<RecognizedPattern>, CompressionError> {
        let mut patterns = Vec::new();

        // Find instruction patterns
        patterns.extend(self.find_instruction_patterns(analysis)?);

        // Find account usage patterns
        patterns.extend(self.find_account_patterns(analysis)?);

        // Find signature patterns
        patterns.extend(self.find_signature_patterns(analysis)?);

        // Sort by compression potential (frequency * size)
        patterns.sort_by(|a, b| {
            let a_potential = a.occurrences * a.size_bytes;
            let b_potential = b.occurrences * b.size_bytes;
            b_potential.cmp(&a_potential)
        });

        Ok(patterns)
    }

    /// Learn new patterns from block analysis
    pub fn learn_patterns(&mut self, analysis: &BlockAnalysis) -> Result<(), CompressionError> {
        // Learn instruction patterns
        for transaction in &analysis.transactions {
            for instruction in &transaction.instructions {
                let pattern_key = instruction.data.clone();
                let entry = self.instruction_patterns.entry(pattern_key).or_insert_with(|| {
                    PatternInfo {
                        frequency: 0,
                        total_size: 0,
                        first_seen: std::time::SystemTime::now(),
                        last_seen: std::time::SystemTime::now(),
                    }
                });
                entry.frequency += 1;
                entry.total_size += instruction.data.len();
                entry.last_seen = std::time::SystemTime::now();
            }
        }

        // Learn account patterns
        for transaction in &analysis.transactions {
            for account in &transaction.accounts {
                let pattern_key = account.clone();
                let entry = self.account_patterns.entry(pattern_key).or_insert_with(|| {
                    AccountUsagePattern {
                        frequency: 0,
                        roles: Vec::new(),
                        associated_programs: Vec::new(),
                    }
                });
                entry.frequency += 1;
                // TODO: Analyze account roles and associated programs
            }
        }

        Ok(())
    }

    /// Find common instruction patterns
    fn find_instruction_patterns(&self, analysis: &BlockAnalysis) -> Result<Vec<RecognizedPattern>, CompressionError> {
        let mut pattern_counts: HashMap<Vec<u8>, usize> = HashMap::new();

        // Count instruction data patterns
        for transaction in &analysis.transactions {
            for instruction in &transaction.instructions {
                *pattern_counts.entry(instruction.data.clone()).or_insert(0) += 1;
            }
        }

        let mut patterns = Vec::new();
        for (instruction_data, count) in pattern_counts {
            if count >= self.min_frequency as usize && instruction_data.len() > 4 {
                patterns.push(RecognizedPattern {
                    pattern_type: PatternType::InstructionData,
                    data: instruction_data.clone(),
                    occurrences: count,
                    size_bytes: instruction_data.len(),
                    compression_potential: count * instruction_data.len(),
                });
            }
        }

        Ok(patterns)
    }

    /// Find account usage patterns
    fn find_account_patterns(&self, analysis: &BlockAnalysis) -> Result<Vec<RecognizedPattern>, CompressionError> {
        let mut account_counts: HashMap<String, usize> = HashMap::new();

        // Count account usage
        for transaction in &analysis.transactions {
            for account in &transaction.accounts {
                *account_counts.entry(account.clone()).or_insert(0) += 1;
            }
        }

        let mut patterns = Vec::new();
        for (account, count) in account_counts {
            if count >= self.min_frequency as usize {
                patterns.push(RecognizedPattern {
                    pattern_type: PatternType::AccountUsage,
                    data: account.as_bytes().to_vec(),
                    occurrences: count,
                    size_bytes: 32, // Solana addresses are 32 bytes
                    compression_potential: count * 32,
                });
            }
        }

        Ok(patterns)
    }

    /// Find signature patterns (for multi-sig or repeated operations)
    fn find_signature_patterns(&self, analysis: &BlockAnalysis) -> Result<Vec<RecognizedPattern>, CompressionError> {
        let mut signature_counts: HashMap<Vec<u8>, usize> = HashMap::new();

        // Count signature patterns (first few bytes for pattern recognition)
        for transaction in &analysis.transactions {
            for signature in &transaction.signatures {
                // Use first 8 bytes as signature pattern
                if signature.len() >= 8 {
                    let pattern = signature[0..8].to_vec();
                    *signature_counts.entry(pattern).or_insert(0) += 1;
                }
            }
        }

        let mut patterns = Vec::new();
        for (signature_pattern, count) in signature_counts {
            if count >= self.min_frequency as usize {
                patterns.push(RecognizedPattern {
                    pattern_type: PatternType::SignaturePattern,
                    data: signature_pattern,
                    occurrences: count,
                    size_bytes: 64, // Full signatures are 64 bytes
                    compression_potential: count * 64,
                });
            }
        }

        Ok(patterns)
    }

    /// Get pattern statistics
    pub fn get_pattern_stats(&self) -> PatternStats {
        PatternStats {
            instruction_patterns: self.instruction_patterns.len(),
            account_patterns: self.account_patterns.len(),
            signature_patterns: self.signature_patterns.len(),
            total_patterns: self.instruction_patterns.len() + self.account_patterns.len() + self.signature_patterns.len(),
        }
    }
}

impl Default for PatternRecognizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a recognized pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternInfo {
    pub frequency: u32,
    pub total_size: usize,
    pub first_seen: std::time::SystemTime,
    pub last_seen: std::time::SystemTime,
}

/// Account usage pattern information
#[derive(Debug, Clone)]
pub struct AccountUsagePattern {
    pub frequency: u32,
    pub roles: Vec<String>, // signer, writable, etc.
    pub associated_programs: Vec<String>,
}

/// A recognized pattern that can be compressed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecognizedPattern {
    pub pattern_type: PatternType,
    pub data: Vec<u8>,
    pub occurrences: usize,
    pub size_bytes: usize,
    pub compression_potential: usize, // occurrences * size_bytes
}

/// Types of patterns that can be recognized
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    InstructionData,
    AccountUsage,
    SignaturePattern,
    TransactionStructure,
    MetadataPattern,
}

/// Statistics about recognized patterns
#[derive(Debug, Clone)]
pub struct PatternStats {
    pub instruction_patterns: usize,
    pub account_patterns: usize,
    pub signature_patterns: usize,
    pub total_patterns: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::transaction_analysis::{TransactionInfo, InstructionInfo};

    fn create_test_analysis() -> BlockAnalysis {
        BlockAnalysis {
            transaction_count: 2,
            total_instructions: 4,
            unique_programs: 2,
            total_accounts: 6,
            transactions: vec![
                TransactionInfo {
                    signatures: vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]],
                    accounts: vec!["11111111111111111111111111111111".to_string()],
                    instructions: vec![
                        InstructionInfo {
                            program_id: "11111111111111111111111111111111".to_string(),
                            accounts: vec![0],
                            data: vec![1, 2, 3, 4], // Common pattern
                        },
                        InstructionInfo {
                            program_id: "11111111111111111111111111111111".to_string(),
                            accounts: vec![0],
                            data: vec![1, 2, 3, 4], // Repeated pattern
                        },
                    ],
                },
                TransactionInfo {
                    signatures: vec![vec![1, 2, 3, 4, 5, 6, 7, 8, 17, 18, 19, 20, 21, 22, 23, 24]],
                    accounts: vec!["11111111111111111111111111111111".to_string()],
                    instructions: vec![
                        InstructionInfo {
                            program_id: "11111111111111111111111111111111".to_string(),
                            accounts: vec![0],
                            data: vec![1, 2, 3, 4], // Same pattern again
                        },
                        InstructionInfo {
                            program_id: "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".to_string(),
                            accounts: vec![0],
                            data: vec![5, 6, 7, 8], // Different pattern
                        },
                    ],
                },
            ],
        }
    }

    #[test]
    fn test_pattern_recognizer_creation() {
        let recognizer = PatternRecognizer::new();
        assert_eq!(recognizer.min_frequency, 3);
    }

    #[test]
    fn test_find_instruction_patterns() {
        let mut recognizer = PatternRecognizer::new();
        recognizer.min_frequency = 2; // Lower threshold for testing

        let analysis = create_test_analysis();
        let patterns = recognizer.find_patterns(&analysis).unwrap();

        // Should find the repeated instruction pattern [1, 2, 3, 4]
        let instruction_patterns: Vec<_> = patterns.iter()
            .filter(|p| matches!(p.pattern_type, PatternType::InstructionData))
            .collect();

        assert!(!instruction_patterns.is_empty());
        assert!(instruction_patterns.iter().any(|p| p.data == vec![1, 2, 3, 4] && p.occurrences >= 2));
    }

    #[test]
    fn test_learn_patterns() {
        let mut recognizer = PatternRecognizer::new();
        let analysis = create_test_analysis();

        recognizer.learn_patterns(&analysis).unwrap();

        let stats = recognizer.get_pattern_stats();
        assert!(stats.instruction_patterns > 0);
        assert!(stats.account_patterns > 0);
    }
}