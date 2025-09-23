//! # Program Clustering Compression
//!
//! Groups and compresses references to common Solana programs that appear
//! frequently across transactions.

use super::super::traits::CompressionError;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Program clustering for compressing common program references
#[derive(Debug, Clone)]
pub struct ProgramCluster {
    /// Map from program pubkey to cluster ID
    program_to_cluster: HashMap<Pubkey, u8>,
    /// Map from cluster ID to program pubkey
    cluster_to_program: HashMap<u8, Pubkey>,
    /// Next available cluster ID
    next_cluster_id: u8,
    /// Reference counts for each program
    reference_counts: HashMap<Pubkey, u32>,
}

impl ProgramCluster {
    /// Create a new program cluster with common Solana programs
    pub fn new() -> Self {
        let mut cluster = Self {
            program_to_cluster: HashMap::new(),
            cluster_to_program: HashMap::new(),
            next_cluster_id: 20, // Reserve 0-19 for most common programs
            reference_counts: HashMap::new(),
        };

        cluster.add_common_programs();
        cluster
    }

    /// Add the most commonly used Solana programs
    fn add_common_programs(&mut self) {
        let common_programs = [
            // Most frequent programs get lowest IDs for better compression
            ("11111111111111111111111111111111", 0), // System Program
            ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", 1), // Token Program
            ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", 2), // Associated Token
            ("ComputeBudget111111111111111111111111111111", 3), // Compute Budget
            ("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr", 4), // Memo Program
            ("BPFLoaderUpgradeab1e11111111111111111111111", 5), // BPF Loader
            ("AddressLookupTab1e1111111111111111111111111", 6), // Address Lookup
            ("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb", 7), // Token-2022
            ("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", 8), // Serum DEX v3
            ("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", 9), // Raydium AMM v4
            ("EhhTKczWMGQt46ynNeRX1WfeagwwJd7ufHvCDjRxjo5Q", 10), // Raydium Staking
            ("SwaPpA9LAaLfeLi3a68M4DjnLqgtticKg6CnyNwgAC8", 11), // Orca Swap
            ("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc", 12), // Orca Whirlpool
            ("PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY", 13), // Phoenix DEX
            ("opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb", 14), // OpenBook DEX
            ("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", 15), // Jupiter v6
            ("JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB", 16), // Jupiter v4
            ("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s", 17), // Metaplex
            ("namesLPneVptA9Z5rqUDD9tMTWEJwofgaYwp8cawRkX", 18), // Solana Name Service
            ("SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f", 19), // Switchboard
        ];

        for (program_str, cluster_id) in common_programs {
            if let Ok(pubkey) = program_str.parse::<Pubkey>() {
                self.program_to_cluster.insert(pubkey, cluster_id);
                self.cluster_to_program.insert(cluster_id, pubkey);
                // Initialize with high reference count since these are common
                self.reference_counts.insert(pubkey, 1000);
            }
        }
    }

    /// Add a program reference and return its cluster ID
    pub fn add_program_reference(&mut self, program: Pubkey) -> u8 {
        // Increment reference count
        *self.reference_counts.entry(program).or_insert(0) += 1;

        // Return existing cluster ID if already mapped
        if let Some(&cluster_id) = self.program_to_cluster.get(&program) {
            return cluster_id;
        }

        // Create new cluster for new program
        let cluster_id = self.next_cluster_id;
        self.program_to_cluster.insert(program, cluster_id);
        self.cluster_to_program.insert(cluster_id, program);
        self.next_cluster_id = self.next_cluster_id.wrapping_add(1);

        cluster_id
    }

    /// Get cluster ID for a program
    pub fn get_cluster_id(&self, program: &Pubkey) -> Option<u8> {
        self.program_to_cluster.get(program).copied()
    }

    /// Get program from cluster ID
    pub fn get_program(&self, cluster_id: u8) -> Option<Pubkey> {
        self.cluster_to_program.get(&cluster_id).copied()
    }

    /// Get reference count for a program
    pub fn get_reference_count(&self, program: &Pubkey) -> u32 {
        self.reference_counts.get(program).copied().unwrap_or(0)
    }

    /// Compress data by finding and replacing program references
    pub fn compress_data(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.len() < 32 {
            return Ok(data.to_vec());
        }

        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            // Look for 32-byte sequences that could be program pubkeys
            if i + 32 <= data.len() {
                if let Ok(bytes) = <[u8; 32]>::try_from(&data[i..i + 32]) {
                    let pubkey = Pubkey::from(bytes);

                    // Check if this is a known program or looks like one
                    if self.is_likely_program(&pubkey) {
                        let cluster_id = self.add_program_reference(pubkey);

                        // Write compressed marker (0xFE) followed by 1-byte cluster ID
                        compressed.push(0xFE);
                        compressed.push(cluster_id);
                        i += 32;
                        continue;
                    }
                }
            }

            // If not a program reference, copy the byte as-is
            compressed.push(data[i]);
            i += 1;
        }

        Ok(compressed)
    }

    /// Decompress data by replacing cluster IDs with full program addresses
    pub fn decompress_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < compressed_data.len() {
            if compressed_data[i] == 0xFE && i + 1 < compressed_data.len() {
                // Read 1-byte cluster ID
                let cluster_id = compressed_data[i + 1];

                // Look up program
                if let Some(program) = self.get_program(cluster_id) {
                    decompressed.extend_from_slice(program.as_ref());
                    i += 2;
                } else {
                    return Err(CompressionError::InvalidFormat);
                }
            } else {
                decompressed.push(compressed_data[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    /// Heuristic to determine if a pubkey is likely a program
    fn is_likely_program(&self, pubkey: &Pubkey) -> bool {
        // If it's already in our program map, it's definitely a program
        if self.program_to_cluster.contains_key(pubkey) {
            return true;
        }

        let bytes = pubkey.as_ref();

        // Heuristics for program addresses:
        // 1. Programs often have specific patterns or are derived addresses
        // 2. Check if it's a PDA (Program Derived Address)
        // For now, use basic heuristics

        // Not all zeros or all 0xFF
        if bytes.iter().all(|&b| b == 0) || bytes.iter().all(|&b| b == 0xFF) {
            return false;
        }

        // Programs often have some structure - check for patterns
        // This is a simple heuristic and could be improved with ML
        let unique_bytes = bytes.iter().collect::<std::collections::HashSet<_>>().len();

        // If there's reasonable entropy (not too repetitive), might be a program
        unique_bytes >= 8
    }

    /// Get the number of programs in the cluster
    pub fn entry_count(&self) -> usize {
        self.program_to_cluster.len()
    }

    /// Get the most frequently referenced programs
    pub fn get_top_programs(&self, count: usize) -> Vec<(Pubkey, u32)> {
        let mut programs: Vec<_> = self.reference_counts.iter()
            .map(|(&program, &count)| (program, count))
            .collect();

        programs.sort_by(|a, b| b.1.cmp(&a.1));
        programs.into_iter().take(count).collect()
    }

    /// Optimize cluster assignments based on usage frequency
    pub fn optimize_clusters(&mut self) {
        // Get programs sorted by reference count
        let mut programs_by_usage = self.get_top_programs(self.entry_count());

        // Clear existing non-system mappings (keep IDs 0-19 for system programs)
        self.program_to_cluster.retain(|_, &mut id| id < 20);
        self.cluster_to_program.retain(|&id, _| id < 20);

        // Reassign cluster IDs based on usage frequency
        let mut next_id = 20;
        for (program, _count) in programs_by_usage {
            if !self.program_to_cluster.contains_key(&program) {
                self.program_to_cluster.insert(program, next_id);
                self.cluster_to_program.insert(next_id, program);
                next_id = next_id.wrapping_add(1);
            }
        }

        self.next_cluster_id = next_id;
    }

    /// Calculate compression efficiency
    pub fn compression_efficiency(&self) -> f64 {
        let total_references: u32 = self.reference_counts.values().sum();
        if total_references == 0 {
            return 1.0;
        }

        // Each reference saves 31 bytes (32 - 1)
        let bytes_saved = total_references as usize * 31;
        let cluster_overhead = self.entry_count() * 32; // Store full addresses in cluster map

        if cluster_overhead > bytes_saved {
            1.0 // No compression benefit
        } else {
            (bytes_saved - cluster_overhead) as f64 / (total_references as usize * 32) as f64
        }
    }
}

impl Default for ProgramCluster {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_cluster_basic() {
        let mut cluster = ProgramCluster::new();

        let program1 = Pubkey::new_unique();
        let program2 = Pubkey::new_unique();

        let id1 = cluster.add_program_reference(program1);
        let id2 = cluster.add_program_reference(program2);

        assert_ne!(id1, id2);
        assert_eq!(cluster.get_program(id1), Some(program1));
        assert_eq!(cluster.get_program(id2), Some(program2));
    }

    #[test]
    fn test_system_programs_preloaded() {
        let cluster = ProgramCluster::new();

        // System program should be pre-loaded with ID 0
        let system_program = "11111111111111111111111111111111".parse::<Pubkey>().unwrap();
        assert_eq!(cluster.get_cluster_id(&system_program), Some(0));
    }

    #[test]
    fn test_reference_counting() {
        let mut cluster = ProgramCluster::new();

        let program = Pubkey::new_unique();

        // Add multiple references
        cluster.add_program_reference(program);
        cluster.add_program_reference(program);
        cluster.add_program_reference(program);

        assert_eq!(cluster.get_reference_count(&program), 3);
    }

    #[test]
    fn test_compression_decompression() {
        let mut cluster = ProgramCluster::new();

        // Create test data with a program pubkey
        let test_program = Pubkey::new_unique();
        let mut test_data = vec![1, 2, 3, 4];
        test_data.extend_from_slice(test_program.as_ref());
        test_data.extend_from_slice(&[5, 6, 7, 8]);

        let compressed = cluster.compress_data(&test_data).unwrap();
        let decompressed = cluster.decompress_data(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
    }

    #[test]
    fn test_optimization() {
        let mut cluster = ProgramCluster::new();

        let program1 = Pubkey::new_unique();
        let program2 = Pubkey::new_unique();

        // Add many references to program1
        for _ in 0..100 {
            cluster.add_program_reference(program1);
        }

        // Add few references to program2
        for _ in 0..10 {
            cluster.add_program_reference(program2);
        }

        cluster.optimize_clusters();

        // program1 should have a lower cluster ID due to higher usage
        let id1 = cluster.get_cluster_id(&program1).unwrap();
        let id2 = cluster.get_cluster_id(&program2).unwrap();

        assert!(id1 < id2);
    }
}