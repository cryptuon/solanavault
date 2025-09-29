//! Integration tests for SolanaVault with blockchain-compression
//!
//! This test demonstrates the full workflow:
//! 1. Fetch Solana block data
//! 2. Compress with blockchain-compression library
//! 3. Store compressed data
//! 4. Retrieve and decompress
//! 5. Verify perfect data integrity

use vault_core::compression::{BlockchainCompressionAdapter, CompressionStrategy};
use vault_core::data::{SolanaBlockClient, BlockCache};

#[tokio::test]
async fn test_end_to_end_compression_workflow() {
    println!("🚀 SolanaVault End-to-End Compression Test");
    println!("==========================================");

    // Step 1: Create compression adapter
    let adapter = BlockchainCompressionAdapter::for_transactions();
    println!("✅ Created blockchain compression adapter");

    // Step 2: Simulate realistic Solana block data
    let block_data = create_realistic_solana_block_data();
    println!("📦 Created simulated block data: {} bytes", block_data.len());

    // Step 3: Compress the block data
    let start_time = std::time::Instant::now();
    let compressed_data = adapter.compress(&block_data).expect("Compression should succeed");
    let compression_time = start_time.elapsed();

    println!("🔧 Compressed in {:?}", compression_time);
    println!("   Original size: {} bytes", block_data.len());
    println!("   Compressed size: {} bytes", compressed_data.len());

    let compression_ratio = block_data.len() as f64 / compressed_data.len() as f64;
    println!("   Compression ratio: {:.2}:1", compression_ratio);

    // Step 4: Decompress and verify integrity
    let start_time = std::time::Instant::now();
    let decompressed_data = adapter.decompress(&compressed_data).expect("Decompression should succeed");
    let decompression_time = start_time.elapsed();

    println!("🔍 Decompressed in {:?}", decompression_time);

    // Step 5: Verify perfect data integrity
    assert_eq!(block_data, decompressed_data, "Data integrity must be perfect");
    println!("✅ Perfect data integrity verified!");

    // Step 6: Performance analysis
    let compression_throughput = block_data.len() as f64 / compression_time.as_secs_f64() / 1_024_1024.0;
    let decompression_throughput = block_data.len() as f64 / decompression_time.as_secs_f64() / 1_024_1024.0;

    println!("📊 Performance Metrics:");
    println!("   Compression throughput: {:.2} MB/s", compression_throughput);
    println!("   Decompression throughput: {:.2} MB/s", decompression_throughput);

    // Step 7: Get compression statistics
    let stats = adapter.get_stats().expect("Should get stats");
    println!("📈 Compression Statistics:");
    println!("   Total compressions: {}", stats.compressions);
    println!("   Average ratio: {:.2}:1", stats.average_ratio);
    println!("   Best ratio: {:.2}:1", stats.best_ratio);

    // Verify performance targets
    assert!(compression_ratio > 10.0, "Should achieve >10:1 compression on Solana data");
    assert!(compression_throughput > 1.0, "Should achieve >1 MB/s compression");
    assert!(decompression_throughput > 10.0, "Should achieve >10 MB/s decompression");

    println!("🎉 All tests passed! SolanaVault compression workflow is working perfectly.");
}

#[tokio::test]
async fn test_different_compression_presets() {
    println!("🔍 Testing Different Compression Presets");
    println!("========================================");

    let test_data = create_realistic_solana_block_data();

    let adapters = vec![
        ("Transactions", BlockchainCompressionAdapter::for_transactions()),
        ("Accounts", BlockchainCompressionAdapter::for_accounts()),
        ("Mixed Data", BlockchainCompressionAdapter::for_mixed_data()),
        ("Archival", BlockchainCompressionAdapter::for_archival()),
    ];

    println!("Test data size: {} bytes\n", test_data.len());

    for (name, adapter) in adapters {
        let compressed = adapter.compress(&test_data).expect("Compression should succeed");
        let decompressed = adapter.decompress(&compressed).expect("Decompression should succeed");

        // Verify integrity
        assert_eq!(test_data, decompressed);

        let ratio = test_data.len() as f64 / compressed.len() as f64;
        println!("{:12}: {:4} bytes ({:5.2}:1)", name, compressed.len(), ratio);
    }

    println!("\n✅ All presets working with perfect data integrity!");
}

#[test]
fn test_compression_adapter_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    println!("🧵 Testing Thread Safety");
    println!("========================");

    let adapter = Arc::new(BlockchainCompressionAdapter::for_transactions());
    let test_data = Arc::new(create_realistic_solana_block_data());

    let mut handles = vec![];

    // Spawn multiple threads to test concurrent compression
    for i in 0..5 {
        let adapter_clone = Arc::clone(&adapter);
        let data_clone = Arc::clone(&test_data);

        let handle = thread::spawn(move || {
            let compressed = adapter_clone.compress(&data_clone).expect("Compression should succeed");
            let decompressed = adapter_clone.decompress(&compressed).expect("Decompression should succeed");

            assert_eq!(**data_clone, decompressed);

            let ratio = data_clone.len() as f64 / compressed.len() as f64;
            println!("Thread {}: {:.2}:1 compression ratio", i, ratio);

            ratio
        });

        handles.push(handle);
    }

    // Wait for all threads and collect results
    let ratios: Vec<f64> = handles.into_iter()
        .map(|h| h.join().expect("Thread should complete"))
        .collect();

    // Verify all threads got similar results (deterministic compression)
    let first_ratio = ratios[0];
    for ratio in &ratios {
        assert!((ratio - first_ratio).abs() < 0.1, "Compression should be deterministic");
    }

    println!("✅ Thread safety verified with consistent results!");
}

#[test]
fn test_large_block_compression() {
    println!("📏 Testing Large Block Compression");
    println!("==================================");

    // Create a large block (simulating a busy block with many transactions)
    let large_block = create_large_solana_block();
    println!("Large block size: {:.2} MB", large_block.len() as f64 / 1_024_1024.0);

    let adapter = BlockchainCompressionAdapter::for_archival(); // Use max compression

    let start_time = std::time::Instant::now();
    let compressed = adapter.compress(&large_block).expect("Should compress large blocks");
    let compression_time = start_time.elapsed();

    let start_time = std::time::Instant::now();
    let decompressed = adapter.decompress(&compressed).expect("Should decompress large blocks");
    let decompression_time = start_time.elapsed();

    assert_eq!(large_block, decompressed);

    let ratio = large_block.len() as f64 / compressed.len() as f64;
    let compression_throughput = large_block.len() as f64 / compression_time.as_secs_f64() / 1_024_1024.0;

    println!("Compressed to: {:.2} MB ({:.2}:1 ratio)", compressed.len() as f64 / 1_024_1024.0, ratio);
    println!("Compression time: {:?} ({:.2} MB/s)", compression_time, compression_throughput);
    println!("Decompression time: {:?}", decompression_time);

    // Verify performance on large blocks
    assert!(ratio > 15.0, "Should achieve >15:1 compression on large Solana blocks");
    assert!(compression_throughput > 0.5, "Should achieve >0.5 MB/s on large blocks");

    println!("✅ Large block compression test passed!");
}

// Helper functions to create test data

fn create_realistic_solana_block_data() -> Vec<u8> {
    let mut block_data = Vec::new();

    // Block header (simplified)
    block_data.extend_from_slice(b"BLOCK_HEADER_V1");
    block_data.extend_from_slice(&12345678u64.to_le_bytes()); // Slot
    block_data.extend_from_slice(&[0; 32]); // Blockhash
    block_data.extend_from_slice(&[1; 32]); // Parent hash

    // Multiple transactions with common Solana patterns
    for i in 0..50 {
        // Transaction header
        block_data.extend_from_slice(&[0x01]); // Version
        block_data.extend_from_slice(&[0x02]); // Signature count

        // Common Solana program IDs (high compression due to dictionary)
        block_data.extend_from_slice("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".as_bytes());
        block_data.extend_from_slice("11111111111111111111111111111112".as_bytes());

        // Transaction amounts (common values)
        if i % 3 == 0 {
            block_data.extend_from_slice(&1_000_000_000u64.to_le_bytes()); // 1 SOL
        } else if i % 3 == 1 {
            block_data.extend_from_slice(&100_000_000u64.to_le_bytes());   // 0.1 SOL
        } else {
            block_data.extend_from_slice(&10_000_000u64.to_le_bytes());    // 0.01 SOL
        }

        // Common instruction patterns
        block_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Transfer instruction
        block_data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Initialize account

        // Some variable data
        block_data.extend_from_slice(&(i as u32).to_le_bytes());
    }

    // Block metadata
    block_data.extend_from_slice(&1234567890u64.to_le_bytes()); // Timestamp
    block_data.extend_from_slice(&[0xFF; 8]); // End marker

    block_data
}

fn create_large_solana_block() -> Vec<u8> {
    let mut block_data = Vec::new();

    // Simulate a very busy block with many transactions
    for tx_id in 0..1000 {
        // Transaction with multiple instructions
        for inst_id in 0..5 {
            // Common program IDs (will compress very well)
            block_data.extend_from_slice("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".as_bytes());
            block_data.extend_from_slice("11111111111111111111111111111112".as_bytes());
            block_data.extend_from_slice("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".as_bytes());

            // Transaction data
            block_data.extend_from_slice(&(tx_id as u32).to_le_bytes());
            block_data.extend_from_slice(&(inst_id as u32).to_le_bytes());

            // Common amounts
            block_data.extend_from_slice(&1_000_000_000u64.to_le_bytes());

            // Instruction data (repetitive patterns)
            block_data.extend_from_slice(&[0x00, 0x01, 0x02, 0x03]);
            block_data.extend_from_slice(&[0x04, 0x05, 0x06, 0x07]);
        }
    }

    block_data
}