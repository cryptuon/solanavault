//! # Real-World Compression Benchmarks
//!
//! Test our compression algorithms on realistic Solana data patterns.

use super::stage1::*;
use solana_sdk::pubkey::Pubkey;

/// Create realistic Solana block data for testing
pub fn create_mock_solana_block_data() -> Vec<u8> {
    let mut block_data = Vec::new();

    // Common Solana system programs that appear frequently
    let system_program = "11111111111111111111111111111111".parse::<Pubkey>().unwrap();
    let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse::<Pubkey>().unwrap();
    let associated_token = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse::<Pubkey>().unwrap();

    // Common user addresses (simulated frequent traders)
    let user1 = Pubkey::new_unique();
    let user2 = Pubkey::new_unique();
    let user3 = Pubkey::new_unique();

    // Block header with blockhash (32 bytes)
    let blockhash = [0x12u8; 32];
    block_data.extend_from_slice(&blockhash);

    // Add some metadata
    block_data.extend_from_slice(&[0xFF, 0xFE, 0x01, 0x02]); // 4 bytes metadata

    // Simulate 100 transactions in the block
    for tx_idx in 0..100 {
        // Transaction header
        block_data.push(0x01); // Transaction marker

        // Accounts in transaction (highly repeated addresses)
        if tx_idx % 3 == 0 {
            // System program transactions
            block_data.extend_from_slice(system_program.as_ref());
            block_data.extend_from_slice(user1.as_ref());
        } else if tx_idx % 3 == 1 {
            // Token program transactions
            block_data.extend_from_slice(token_program.as_ref());
            block_data.extend_from_slice(user2.as_ref());
            block_data.extend_from_slice(associated_token.as_ref());
        } else {
            // Mixed transactions
            block_data.extend_from_slice(user3.as_ref());
            block_data.extend_from_slice(system_program.as_ref());
        }

        // Add some instruction data
        block_data.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]); // 4 bytes instruction data
    }

    block_data
}

/// Benchmark Stage 1 compression algorithms
pub fn benchmark_stage1_compression() -> Stage1BenchmarkResults {
    let test_data = create_mock_solana_block_data();
    let original_size = test_data.len();

    println!("=== Stage 1 Compression Benchmark ===");
    println!("Original block size: {} bytes", original_size);

    // Test Account Dictionary
    let mut account_dict = AccountDictionary::new();
    let dict_start = std::time::Instant::now();
    let dict_compressed = account_dict.compress_data(&test_data).unwrap();
    let dict_time = dict_start.elapsed();
    let dict_ratio = original_size as f64 / dict_compressed.len() as f64;

    println!("Account Dictionary:");
    println!("  - Size: {} → {} bytes", original_size, dict_compressed.len());
    println!("  - Ratio: {:.2}:1", dict_ratio);
    println!("  - Time: {:?}", dict_time);
    println!("  - Dictionary entries: {}", account_dict.entry_count());

    // Test Program Clustering
    let mut program_cluster = ProgramCluster::new();
    let prog_start = std::time::Instant::now();
    let prog_compressed = program_cluster.compress_data(&test_data).unwrap();
    let prog_time = prog_start.elapsed();
    let prog_ratio = original_size as f64 / prog_compressed.len() as f64;

    println!("Program Clustering:");
    println!("  - Size: {} → {} bytes", original_size, prog_compressed.len());
    println!("  - Ratio: {:.2}:1", prog_ratio);
    println!("  - Time: {:?}", prog_time);
    println!("  - Cluster entries: {}", program_cluster.entry_count());

    // Test Blockhash Delta
    let mut blockhash_delta = BlockhashDelta::new();
    let hash_start = std::time::Instant::now();
    let hash_compressed = blockhash_delta.compress_data(&test_data).unwrap();
    let hash_time = hash_start.elapsed();
    let hash_ratio = original_size as f64 / hash_compressed.len() as f64;

    println!("Blockhash Delta:");
    println!("  - Size: {} → {} bytes", original_size, hash_compressed.len());
    println!("  - Ratio: {:.2}:1", hash_ratio);
    println!("  - Time: {:?}", hash_time);

    // Test combined Stage 1 compression
    let mut stage1 = Stage1Compressor::new();
    let combined_start = std::time::Instant::now();
    let combined_compressed = stage1.compress_block_data(&test_data).unwrap();
    let combined_time = combined_start.elapsed();
    let combined_ratio = original_size as f64 / combined_compressed.len() as f64;

    println!("Combined Stage 1:");
    println!("  - Size: {} → {} bytes", original_size, combined_compressed.len());
    println!("  - Ratio: {:.2}:1", combined_ratio);
    println!("  - Time: {:?}", combined_time);

    // Verify decompression works
    let decompressed = stage1.decompress_block_data(&combined_compressed).unwrap();
    let integrity_check = test_data == decompressed;
    println!("  - Data integrity: {}", if integrity_check { "✅ PASS" } else { "❌ FAIL" });

    Stage1BenchmarkResults {
        original_size,
        dict_compressed_size: dict_compressed.len(),
        prog_compressed_size: prog_compressed.len(),
        hash_compressed_size: hash_compressed.len(),
        combined_compressed_size: combined_compressed.len(),
        dict_ratio,
        prog_ratio,
        hash_ratio,
        combined_ratio,
        dict_time_ms: dict_time.as_millis() as u64,
        prog_time_ms: prog_time.as_millis() as u64,
        hash_time_ms: hash_time.as_millis() as u64,
        combined_time_ms: combined_time.as_millis() as u64,
        integrity_check,
    }
}

/// Results from Stage 1 benchmarking
#[derive(Debug, Clone)]
pub struct Stage1BenchmarkResults {
    pub original_size: usize,
    pub dict_compressed_size: usize,
    pub prog_compressed_size: usize,
    pub hash_compressed_size: usize,
    pub combined_compressed_size: usize,
    pub dict_ratio: f64,
    pub prog_ratio: f64,
    pub hash_ratio: f64,
    pub combined_ratio: f64,
    pub dict_time_ms: u64,
    pub prog_time_ms: u64,
    pub hash_time_ms: u64,
    pub combined_time_ms: u64,
    pub integrity_check: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stage1_benchmark() {
        let results = benchmark_stage1_compression();

        // Verify all algorithms work
        assert!(results.integrity_check, "Data integrity must be maintained");

        // Check that we achieve some compression
        assert!(results.combined_ratio > 1.0, "Should achieve compression ratio > 1:1");

        // Dictionary should be very effective on repeated addresses
        assert!(results.dict_ratio > 2.0, "Account dictionary should achieve > 2:1 on repeated addresses");

        // Program clustering should be effective on repeated programs
        assert!(results.prog_ratio > 1.5, "Program clustering should achieve > 1.5:1 on repeated programs");

        println!("\n🎯 Stage 1 Compression Summary:");
        println!("Overall compression: {:.2}:1", results.combined_ratio);
        println!("Best component: Account Dictionary at {:.2}:1", results.dict_ratio);
        println!("Total compression time: {}ms", results.combined_time_ms);
    }

    #[test]
    fn test_realistic_solana_patterns() {
        // Test with patterns that mimic real Solana usage
        let data = create_mock_solana_block_data();

        // Should contain repeated addresses
        let system_program_bytes = "11111111111111111111111111111111".parse::<Pubkey>().unwrap();
        let occurrences = data.windows(32).filter(|window| {
            *window == system_program_bytes.as_ref()
        }).count();

        assert!(occurrences > 10, "Should have multiple system program references");
        println!("System program appears {} times in mock data", occurrences);
    }

    #[test]
    fn test_large_scale_compression() {
        println!("\n=== Large Scale Compression Test ===");

        // Create a larger mock block (simulating 1000 transactions)
        let mut large_block = Vec::new();
        let system_program = "11111111111111111111111111111111".parse::<Pubkey>().unwrap();
        let token_program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse::<Pubkey>().unwrap();

        // Add many repeated transactions (common in real Solana blocks)
        for _ in 0..1000 {
            large_block.extend_from_slice(system_program.as_ref());
            large_block.extend_from_slice(token_program.as_ref());
            // Add some varying data
            large_block.extend_from_slice(&[0x01, 0x02, 0x03, 0x04]);
        }

        let original_size = large_block.len();
        println!("Large block original size: {} bytes ({:.1} KB)",
                 original_size, original_size as f64 / 1024.0);

        // Test compression
        let mut stage1 = Stage1Compressor::new();
        let start = std::time::Instant::now();
        let compressed = stage1.compress_block_data(&large_block).unwrap();
        let compression_time = start.elapsed();

        let compressed_size = compressed.len();
        let ratio = original_size as f64 / compressed_size as f64;

        println!("Compressed size: {} bytes ({:.1} KB)",
                 compressed_size, compressed_size as f64 / 1024.0);
        println!("Compression ratio: {:.2}:1", ratio);
        println!("Compression time: {:?}", compression_time);
        println!("Throughput: {:.1} MB/s",
                 (original_size as f64 / 1024.0 / 1024.0) / compression_time.as_secs_f64());

        // Test decompression
        let start = std::time::Instant::now();
        let decompressed = stage1.decompress_block_data(&compressed).unwrap();
        let decompression_time = start.elapsed();

        println!("Decompression time: {:?}", decompression_time);
        println!("Decompression throughput: {:.1} MB/s",
                 (compressed_size as f64 / 1024.0 / 1024.0) / decompression_time.as_secs_f64());

        assert_eq!(large_block, decompressed, "Data integrity check");
        assert!(ratio > 5.0, "Should achieve significant compression on repeated data");

        println!("✅ Large scale test passed!");
    }
}