//! Integration test for blockchain-compression adapter

use vault_core::compression::{BlockchainCompressionAdapter, CompressionStrategy};

#[test]
fn test_basic_compression_workflow() {
    println!("🚀 Testing Basic Compression Workflow");
    println!("====================================");

    // Create compression adapter
    let adapter = BlockchainCompressionAdapter::for_transactions();
    println!("✅ Created blockchain compression adapter");

    // Create test data
    let test_data = create_test_solana_data();
    println!("📦 Created test data: {} bytes", test_data.len());

    // Compress
    let compressed = adapter.compress(&test_data).expect("Compression should work");
    println!("🔧 Compressed to: {} bytes", compressed.len());

    // Decompress
    let decompressed = adapter.decompress(&compressed).expect("Decompression should work");

    // Verify integrity
    assert_eq!(test_data, decompressed, "Perfect data integrity required");
    println!("✅ Perfect data integrity verified!");

    // Calculate compression ratio
    let ratio = test_data.len() as f64 / compressed.len() as f64;
    println!("📊 Compression ratio: {:.2}:1", ratio);

    // Verify we get good compression on Solana data
    assert!(ratio > 5.0, "Should achieve >5:1 compression on Solana patterns");

    println!("🎉 Basic compression workflow test passed!");
}

#[test]
fn test_different_presets() {
    println!("🔍 Testing Different Compression Presets");
    println!("========================================");

    let test_data = create_test_solana_data();

    let presets = vec![
        ("Transactions", BlockchainCompressionAdapter::for_transactions()),
        ("Accounts", BlockchainCompressionAdapter::for_accounts()),
        ("Mixed", BlockchainCompressionAdapter::for_mixed_data()),
        ("Archival", BlockchainCompressionAdapter::for_archival()),
    ];

    for (name, adapter) in presets {
        let compressed = adapter.compress(&test_data).expect("Compression should work");
        let decompressed = adapter.decompress(&compressed).expect("Decompression should work");

        assert_eq!(test_data, decompressed);

        let ratio = test_data.len() as f64 / compressed.len() as f64;
        println!("{:12}: {} bytes ({:.2}:1)", name, compressed.len(), ratio);
    }

    println!("✅ All presets working correctly!");
}

fn create_test_solana_data() -> Vec<u8> {
    let mut data = Vec::new();

    // Add common Solana program IDs (compress well due to repetition)
    for _ in 0..20 {
        data.extend_from_slice("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".as_bytes());
        data.extend_from_slice("11111111111111111111111111111112".as_bytes());
    }

    // Add common amounts
    for _ in 0..10 {
        data.extend_from_slice(&1_000_000_000u64.to_le_bytes()); // 1 SOL
        data.extend_from_slice(&100_000_000u64.to_le_bytes());   // 0.1 SOL
    }

    // Add instruction patterns
    for _ in 0..15 {
        data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Transfer
        data.extend_from_slice(&[0x01, 0x00, 0x00, 0x00]); // Initialize
    }

    data
}