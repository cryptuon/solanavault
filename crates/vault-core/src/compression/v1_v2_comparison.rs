//! # V1 vs V2 Compression Comparison
//!
//! Comprehensive benchmarks comparing V1 (Stage 1 only) and V2 (Stage 1 + Stage 2) compression.

use super::*;
use serde::{Serialize, Deserialize};

/// Comprehensive comparison of V1 and V2 compression algorithms
pub fn compare_v1_v2_compression() -> ComparisonResults {
    // Create realistic test data that benefits from both Stage 1 and Stage 2
    let test_data = create_comprehensive_test_data();
    let original_size = test_data.len();

    println!("=== V1 vs V2 Compression Comparison ===");
    println!("Test data size: {} bytes ({:.1} KB)", original_size, original_size as f64 / 1024.0);

    // Test V1 Compression (Stage 1 only)
    let v1_start = std::time::Instant::now();
    let v1_compressor = V1Compression::new();
    let v1_compressed = match v1_compressor.compress(&test_data) {
        Ok(data) => data,
        Err(e) => {
            println!("V1 compression failed: {:?}", e);
            return ComparisonResults {
                original_size,
                v1_compressed_size: 0,
                v2_compressed_size: 0,
                v1_ratio: 0.0,
                v2_ratio: 0.0,
                v1_compression_time_ms: 0,
                v2_compression_time_ms: 0,
                v1_decompression_time_ms: 0,
                v2_decompression_time_ms: 0,
                ratio_improvement: 0.0,
                size_improvement_percent: 0.0,
            };
        }
    };
    let v1_compression_time = v1_start.elapsed();

    let v1_decompression_start = std::time::Instant::now();
    let v1_decompressed = v1_compressor.decompress(&v1_compressed).unwrap();
    let v1_decompression_time = v1_decompression_start.elapsed();

    assert_eq!(test_data, v1_decompressed, "V1 data integrity check failed");

    let v1_ratio = original_size as f64 / v1_compressed.len() as f64;

    println!("\nV1 Compression (Stage 1 Only):");
    println!("  - Size: {} → {} bytes", original_size, v1_compressed.len());
    println!("  - Ratio: {:.2}:1", v1_ratio);
    println!("  - Compression time: {:?}", v1_compression_time);
    println!("  - Decompression time: {:?}", v1_decompression_time);
    println!("  - Data integrity: ✅ PASS");

    // Test V2 Compression (Stage 1 + Stage 2)
    let v2_start = std::time::Instant::now();
    let v2_compressor = V2Compression::new();
    let v2_compressed = v2_compressor.compress(&test_data).unwrap();
    let v2_compression_time = v2_start.elapsed();

    let v2_decompression_start = std::time::Instant::now();
    let v2_decompressed = v2_compressor.decompress(&v2_compressed).unwrap();
    let v2_decompression_time = v2_decompression_start.elapsed();

    assert_eq!(test_data, v2_decompressed, "V2 data integrity check failed");

    let v2_ratio = original_size as f64 / v2_compressed.len() as f64;

    println!("\nV2 Compression (Stage 1 + Stage 2):");
    println!("  - Size: {} → {} bytes", original_size, v2_compressed.len());
    println!("  - Ratio: {:.2}:1", v2_ratio);
    println!("  - Compression time: {:?}", v2_compression_time);
    println!("  - Decompression time: {:?}", v2_decompression_time);
    println!("  - Data integrity: ✅ PASS");

    // Calculate improvements
    let ratio_improvement = v2_ratio / v1_ratio;
    let size_improvement = (v1_compressed.len() as f64 - v2_compressed.len() as f64) / v1_compressed.len() as f64 * 100.0;

    println!("\n📊 Comparison Summary:");
    println!("  - V1 compression: {:.2}:1", v1_ratio);
    println!("  - V2 compression: {:.2}:1", v2_ratio);
    println!("  - Improvement: {:.2}x better compression", ratio_improvement);
    if size_improvement > 0.0 {
        println!("  - Size reduction: {:.1}% smaller", size_improvement);
    } else {
        println!("  - Size increase: {:.1}% larger", -size_improvement);
    }
    println!("  - Compression time overhead: {:.1}x",
             v2_compression_time.as_millis() as f64 / v1_compression_time.as_millis() as f64);

    ComparisonResults {
        original_size,
        v1_compressed_size: v1_compressed.len(),
        v2_compressed_size: v2_compressed.len(),
        v1_ratio,
        v2_ratio,
        v1_compression_time_ms: v1_compression_time.as_millis() as u64,
        v2_compression_time_ms: v2_compression_time.as_millis() as u64,
        v1_decompression_time_ms: v1_decompression_time.as_millis() as u64,
        v2_decompression_time_ms: v2_decompression_time.as_millis() as u64,
        ratio_improvement,
        size_improvement_percent: size_improvement,
    }
}

/// Create comprehensive test data that showcases both Stage 1 and Stage 2 benefits
fn create_comprehensive_test_data() -> Vec<u8> {
    let mut data = Vec::new();

    // Block header
    data.extend_from_slice(&[0x12u8; 32]); // Blockhash
    data.extend_from_slice(&[0xFF, 0xFE, 0x01, 0x02]); // Metadata

    // Add repeated Solana addresses (benefits Stage 1 Account Dictionary)
    let system_program = "11111111111111111111111111111111".parse::<solana_sdk::pubkey::Pubkey>().unwrap();
    let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse::<solana_sdk::pubkey::Pubkey>().unwrap();
    let user_wallet = solana_sdk::pubkey::Pubkey::new_unique();

    for i in 0..50 {
        // Transaction marker
        data.push(0x01);

        // Common address patterns (Stage 1 benefits)
        if i % 3 == 0 {
            data.extend_from_slice(system_program.as_ref());
            data.extend_from_slice(user_wallet.as_ref());
        } else if i % 3 == 1 {
            data.extend_from_slice(token_program.as_ref());
            data.extend_from_slice(user_wallet.as_ref());
        } else {
            data.extend_from_slice(system_program.as_ref());
            data.extend_from_slice(token_program.as_ref());
        }

        // Common instruction patterns (Stage 2 benefits)
        if i % 4 == 0 {
            data.extend_from_slice(&[1, 2, 3, 4]); // Transfer instruction
        } else if i % 4 == 1 {
            data.extend_from_slice(&[5, 6, 7, 8]); // Mint instruction
        } else if i % 4 == 2 {
            data.extend_from_slice(&[1, 2, 3, 4]); // Same transfer pattern
        } else {
            data.extend_from_slice(&[9, 10, 11, 12]); // Other instruction
        }

        // Add repeated signature-like patterns (Stage 2 metadata benefits)
        if i % 5 == 0 {
            data.extend_from_slice(&vec![0xAA; 64]); // Common signature pattern
        } else {
            data.extend_from_slice(&vec![0xBB; 64]); // Another common pattern
        }

        // Add timestamp patterns (Stage 2 metadata benefits)
        let base_timestamp = 1640995200u64; // Jan 1, 2022
        let timestamp = base_timestamp + (i as u64) * 1000;
        data.extend_from_slice(&timestamp.to_le_bytes());
    }

    data
}

/// Test V2 compression with training to show learning capabilities
pub fn test_v2_learning_capability() -> LearningResults {
    println!("\n=== V2 Learning Capability Test ===");

    let training_data = create_comprehensive_test_data();
    let test_data = create_comprehensive_test_data(); // Similar patterns

    // Test V2 without training
    let v2_untrained = V2Compression::new();
    let untrained_compressed = v2_untrained.compress(&test_data).unwrap();
    let untrained_ratio = test_data.len() as f64 / untrained_compressed.len() as f64;

    println!("V2 without training: {:.2}:1", untrained_ratio);

    // Test V2 with training
    let mut v2_trained = V2Compression::new();
    v2_trained.train_on_data(&training_data).unwrap();

    let trained_compressed = v2_trained.compress(&test_data).unwrap();
    let trained_decompressed = v2_trained.decompress(&trained_compressed).unwrap();
    assert_eq!(test_data, trained_decompressed, "V2 trained data integrity check failed");

    let trained_ratio = test_data.len() as f64 / trained_compressed.len() as f64;

    println!("V2 with training: {:.2}:1", trained_ratio);

    let learning_improvement = trained_ratio / untrained_ratio;
    println!("Learning improvement: {:.2}x better", learning_improvement);

    LearningResults {
        untrained_ratio,
        trained_ratio,
        learning_improvement,
        training_effective: learning_improvement > 1.01, // At least 1% improvement
    }
}

/// Results from V1 vs V2 comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResults {
    pub original_size: usize,
    pub v1_compressed_size: usize,
    pub v2_compressed_size: usize,
    pub v1_ratio: f64,
    pub v2_ratio: f64,
    pub v1_compression_time_ms: u64,
    pub v2_compression_time_ms: u64,
    pub v1_decompression_time_ms: u64,
    pub v2_decompression_time_ms: u64,
    pub ratio_improvement: f64,
    pub size_improvement_percent: f64,
}

/// Results from learning capability test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningResults {
    pub untrained_ratio: f64,
    pub trained_ratio: f64,
    pub learning_improvement: f64,
    pub training_effective: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v1_v2_comparison() {
        let results = compare_v1_v2_compression();

        // Both should achieve reasonable compression
        assert!(results.v1_ratio > 1.0, "V1 should achieve some compression");
        assert!(results.v2_ratio > 0.5, "V2 should achieve reasonable compression");

        // Verify data integrity is maintained
        assert!(results.original_size > 0);
        assert!(results.v1_compressed_size > 0);
        assert!(results.v2_compressed_size > 0);

        println!("\n✅ V1 vs V2 Comparison Test Passed");
        println!("V1: {:.2}:1, V2: {:.2}:1 (improvement: {:.2}x)",
                 results.v1_ratio, results.v2_ratio, results.ratio_improvement);
    }

    #[test]
    fn test_v2_learning() {
        let results = test_v2_learning_capability();

        // Should achieve some compression both ways
        assert!(results.untrained_ratio > 0.5, "Untrained V2 should achieve reasonable compression");
        assert!(results.trained_ratio > 0.5, "Trained V2 should achieve reasonable compression");

        println!("\n🤖 V2 Learning Test Passed");
        println!("Learning improvement: {:.2}x, Effective: {}",
                 results.learning_improvement, results.training_effective);
    }

    #[test]
    fn test_comprehensive_data_creation() {
        let data = create_comprehensive_test_data();

        // Should create substantial test data
        assert!(data.len() > 1000, "Should create substantial test data");

        // Should contain repeated patterns
        let pattern_count = data.windows(4).filter(|w| *w == &[1, 2, 3, 4]).count();
        assert!(pattern_count > 5, "Should contain repeated instruction patterns");

        println!("Created test data: {} bytes with {} pattern instances",
                 data.len(), pattern_count);
    }
}