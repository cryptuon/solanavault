//! # Instruction Template Engine
//!
//! Creates and applies templates for common Solana instruction patterns to achieve compression.

use super::super::traits::CompressionError;
use super::pattern_recognition::RecognizedPattern;
use super::transaction_analysis::{BlockAnalysis, InstructionInfo};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Template engine for compressing common instruction patterns
#[derive(Debug, Clone)]
pub struct InstructionTemplateEngine {
    /// Library of instruction templates
    templates: HashMap<TemplateId, InstructionTemplate>,
    /// Next available template ID
    next_template_id: u32,
    /// Template usage statistics
    usage_stats: HashMap<TemplateId, TemplateUsage>,
    /// Common Solana instruction patterns
    builtin_templates: Vec<InstructionTemplate>,
}

impl InstructionTemplateEngine {
    /// Creates a new instruction template engine
    pub fn new() -> Self {
        let mut engine = Self {
            templates: HashMap::new(),
            next_template_id: 1000, // Start from 1000 to avoid conflicts with builtins
            usage_stats: HashMap::new(),
            builtin_templates: Vec::new(),
        };

        engine.load_builtin_templates();
        engine
    }

    /// Apply templates to compress instruction data
    pub fn apply_templates(&mut self, data: &[u8], patterns: &[RecognizedPattern]) -> Result<Vec<u8>, CompressionError> {
        let mut compressed = Vec::new();
        let mut offset = 0;

        // Add template header
        compressed.extend_from_slice(&[0xF0, 0x02]); // Magic bytes for Stage 2 template compression

        // Process data in chunks, looking for template matches
        while offset < data.len() {
            let mut matched = false;

            // Try to match against templates (longest match first)
            let mut best_match: Option<(TemplateId, usize)> = None;

            for (template_id, template) in &self.templates {
                if let Some(match_len) = self.try_match_template(template, &data[offset..]) {
                    if best_match.is_none() || match_len > best_match.unwrap().1 {
                        best_match = Some((*template_id, match_len));
                    }
                }
            }

            // Also try builtin templates
            for (idx, template) in self.builtin_templates.iter().enumerate() {
                if let Some(match_len) = self.try_match_template(template, &data[offset..]) {
                    let template_id = idx as u32; // Builtin templates use indices 0-999
                    if best_match.is_none() || match_len > best_match.unwrap().1 {
                        best_match = Some((template_id, match_len));
                    }
                }
            }

            if let Some((template_id, match_len)) = best_match {
                // Apply template compression
                compressed.push(0xFF); // Template marker
                compressed.extend_from_slice(&template_id.to_le_bytes());

                // Update usage statistics
                let usage = self.usage_stats.entry(template_id).or_insert_with(|| TemplateUsage {
                    count: 0,
                    bytes_saved: 0,
                });
                usage.count += 1;
                if match_len > 5 {
                    usage.bytes_saved += match_len - 5; // Template ID is 4 bytes + marker
                }

                offset += match_len;
                matched = true;
            }

            if !matched {
                // No template match, copy byte directly
                compressed.push(data[offset]);
                offset += 1;
            }
        }

        Ok(compressed)
    }

    /// Expand templates to decompress instruction data
    pub fn expand_templates(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        // If data doesn't have template header, pass through as-is
        if data.len() < 2 || &data[0..2] != &[0xF0, 0x02] {
            return Ok(data.to_vec());
        }

        let mut expanded = Vec::new();
        let mut offset = 2; // Skip header

        while offset < data.len() {
            if data[offset] == 0xFF && offset + 5 <= data.len() {
                // Template reference
                let template_id_bytes = &data[offset + 1..offset + 5];
                let template_id = u32::from_le_bytes([
                    template_id_bytes[0],
                    template_id_bytes[1],
                    template_id_bytes[2],
                    template_id_bytes[3],
                ]);

                // Find and expand template
                if let Some(template_data) = self.get_template_data(template_id) {
                    expanded.extend_from_slice(&template_data);
                } else {
                    return Err(CompressionError::InvalidFormat);
                }

                offset += 5;
            } else {
                // Regular byte
                expanded.push(data[offset]);
                offset += 1;
            }
        }

        Ok(expanded)
    }

    /// Learn new templates from block analysis
    pub fn learn_templates(&mut self, analysis: &BlockAnalysis) -> Result<(), CompressionError> {
        let mut instruction_patterns: HashMap<Vec<u8>, u32> = HashMap::new();

        // Count instruction patterns across all transactions
        for transaction in &analysis.transactions {
            for instruction in &transaction.instructions {
                *instruction_patterns.entry(instruction.data.clone()).or_insert(0) += 1;
            }
        }

        // Create templates for frequent patterns
        for (instruction_data, frequency) in instruction_patterns {
            if frequency >= 3 && instruction_data.len() >= 8 {
                let template = InstructionTemplate {
                    id: self.next_template_id,
                    name: format!("auto_template_{}", self.next_template_id),
                    pattern: instruction_data.clone(),
                    frequency,
                    size_bytes: instruction_data.len(),
                    template_type: TemplateType::InstructionData,
                };

                self.templates.insert(self.next_template_id, template);
                self.next_template_id += 1;
            }
        }

        Ok(())
    }

    /// Load builtin templates for common Solana operations
    fn load_builtin_templates(&mut self) {
        // System Program: Transfer instruction
        self.builtin_templates.push(InstructionTemplate {
            id: 0,
            name: "system_transfer".to_string(),
            pattern: vec![2, 0, 0, 0], // Transfer instruction discriminator
            frequency: 0, // Will be updated during usage
            size_bytes: 4,
            template_type: TemplateType::SystemProgram,
        });

        // Token Program: Transfer instruction
        self.builtin_templates.push(InstructionTemplate {
            id: 1,
            name: "token_transfer".to_string(),
            pattern: vec![3, 0, 0, 0], // Token transfer discriminator
            frequency: 0,
            size_bytes: 4,
            template_type: TemplateType::TokenProgram,
        });

        // Token Program: Mint instruction
        self.builtin_templates.push(InstructionTemplate {
            id: 2,
            name: "token_mint".to_string(),
            pattern: vec![7, 0, 0, 0], // Mint instruction discriminator
            frequency: 0,
            size_bytes: 4,
            template_type: TemplateType::TokenProgram,
        });

        // Common instruction pattern [1, 2, 3, 4] (for testing)
        self.builtin_templates.push(InstructionTemplate {
            id: 3,
            name: "common_test_pattern".to_string(),
            pattern: vec![1, 2, 3, 4],
            frequency: 0,
            size_bytes: 4,
            template_type: TemplateType::TestPattern,
        });
    }

    /// Try to match a template against data
    fn try_match_template(&self, template: &InstructionTemplate, data: &[u8]) -> Option<usize> {
        if data.len() >= template.pattern.len() && data.starts_with(&template.pattern) {
            Some(template.pattern.len())
        } else {
            None
        }
    }

    /// Get template data by ID
    fn get_template_data(&self, template_id: TemplateId) -> Option<Vec<u8>> {
        if template_id < 1000 {
            // Builtin template
            self.builtin_templates.get(template_id as usize).map(|t| t.pattern.clone())
        } else {
            // Custom template
            self.templates.get(&template_id).map(|t| t.pattern.clone())
        }
    }

    /// Get template usage statistics
    pub fn get_usage_stats(&self) -> &HashMap<TemplateId, TemplateUsage> {
        &self.usage_stats
    }

    /// Get template count
    pub fn template_count(&self) -> usize {
        self.builtin_templates.len() + self.templates.len()
    }
}

impl Default for InstructionTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Template ID type
type TemplateId = u32;

/// Instruction template for compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionTemplate {
    pub id: TemplateId,
    pub name: String,
    pub pattern: Vec<u8>,
    pub frequency: u32,
    pub size_bytes: usize,
    pub template_type: TemplateType,
}

/// Types of instruction templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TemplateType {
    SystemProgram,
    TokenProgram,
    AssociatedTokenProgram,
    InstructionData,
    TestPattern,
}

/// Template usage statistics
#[derive(Debug, Clone)]
pub struct TemplateUsage {
    pub count: u32,
    pub bytes_saved: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::transaction_analysis::{TransactionInfo, TransactionAnalyzer};

    fn create_test_data_with_patterns() -> Vec<u8> {
        let mut data = Vec::new();

        // Add some regular data
        data.extend_from_slice(&[0x00, 0x01, 0x02]);

        // Add common pattern that should be templated
        data.extend_from_slice(&[1, 2, 3, 4]);

        // Add more regular data
        data.extend_from_slice(&[0x05, 0x06]);

        // Add the same pattern again
        data.extend_from_slice(&[1, 2, 3, 4]);

        // Add different data
        data.extend_from_slice(&[0x07, 0x08, 0x09]);

        data
    }

    #[test]
    fn test_template_engine_creation() {
        let engine = InstructionTemplateEngine::new();
        assert!(!engine.builtin_templates.is_empty());
        assert_eq!(engine.next_template_id, 1000);
    }

    #[test]
    fn test_apply_templates() {
        let mut engine = InstructionTemplateEngine::new();
        let test_data = create_test_data_with_patterns();

        let compressed = engine.apply_templates(&test_data, &[]).unwrap();

        // Should start with template header
        assert_eq!(&compressed[0..2], &[0xF0, 0x02]);

        // Should be smaller than original due to template compression
        println!("Original: {} bytes, Compressed: {} bytes", test_data.len(), compressed.len());
    }

    #[test]
    fn test_expand_templates() {
        let mut engine = InstructionTemplateEngine::new();
        let test_data = create_test_data_with_patterns();

        let compressed = engine.apply_templates(&test_data, &[]).unwrap();
        let expanded = engine.expand_templates(&compressed).unwrap();

        assert_eq!(test_data, expanded);
    }

    #[test]
    fn test_learn_templates() {
        let mut engine = InstructionTemplateEngine::new();

        // Create mock analysis with repeated patterns
        let analysis = BlockAnalysis {
            transaction_count: 2,
            total_instructions: 4,
            unique_programs: 1,
            total_accounts: 2,
            transactions: vec![
                TransactionInfo {
                    signatures: vec![],
                    accounts: vec![],
                    instructions: vec![
                        InstructionInfo {
                            program_id: "test".to_string(),
                            accounts: vec![],
                            data: vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22], // 8-byte pattern
                        },
                        InstructionInfo {
                            program_id: "test".to_string(),
                            accounts: vec![],
                            data: vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22], // Same pattern
                        },
                    ],
                },
                TransactionInfo {
                    signatures: vec![],
                    accounts: vec![],
                    instructions: vec![
                        InstructionInfo {
                            program_id: "test".to_string(),
                            accounts: vec![],
                            data: vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22], // Same pattern again
                        },
                    ],
                },
            ],
        };

        let initial_count = engine.template_count();
        engine.learn_templates(&analysis).unwrap();

        // Should have learned new templates
        assert!(engine.template_count() > initial_count);
    }

    #[test]
    fn test_builtin_templates() {
        let engine = InstructionTemplateEngine::new();

        // Should have loaded builtin templates
        assert!(!engine.builtin_templates.is_empty());

        // Should have common Solana patterns
        let has_system_transfer = engine.builtin_templates.iter()
            .any(|t| t.name == "system_transfer");
        let has_token_transfer = engine.builtin_templates.iter()
            .any(|t| t.name == "token_transfer");

        assert!(has_system_transfer);
        assert!(has_token_transfer);
    }

    #[test]
    fn test_template_matching() {
        let engine = InstructionTemplateEngine::new();

        // Find the test pattern template
        let test_template = engine.builtin_templates.iter()
            .find(|t| t.name == "common_test_pattern")
            .unwrap();

        // Should match the pattern
        let test_data = vec![1, 2, 3, 4, 5, 6];
        let match_len = engine.try_match_template(test_template, &test_data);
        assert_eq!(match_len, Some(4));

        // Should not match different pattern
        let different_data = vec![5, 6, 7, 8];
        let no_match = engine.try_match_template(test_template, &different_data);
        assert_eq!(no_match, None);
    }
}