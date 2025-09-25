//! Debug XGBoost pipeline roundtrip failures

use super::{XGBoostStage3Compressor, CompressionStrategy};
use crate::compression::traits::CompressionError;

pub fn debug_xgboost_pipeline() -> Result<(), CompressionError> {
    println!("🔍 DEBUGGING XGBOOST PIPELINE ROUNDTRIP");

    // Test 1: Simple repetitive data
    let simple_data = vec![1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 5, 6, 7, 8];
    test_pipeline_roundtrip("Simple repetitive", &simple_data)?;

    // Test 2: Simulated Solana-like data
    let mut solana_like = Vec::new();
    // Add some account-like patterns (32-byte chunks)
    for i in 0..3 {
        let mut account = vec![0u8; 32];
        account[0] = i as u8;
        account[31] = i as u8;
        solana_like.extend_from_slice(&account);
    }
    // Add some instruction data
    let instruction_data = vec![0xA1, 0xB2, 0xC3, 0xD4, 0xE5, 0xF6];
    for _ in 0..5 {
        solana_like.extend_from_slice(&instruction_data);
    }
    test_pipeline_roundtrip("Solana-like data", &solana_like)?;

    // Test 3: Larger realistic data
    let mut large_data = Vec::new();
    let pattern = vec![0xAA, 0xBB, 0xCC, 0xDD, 0xEE, 0xFF, 0x11, 0x22];
    for _ in 0..100 {
        large_data.extend_from_slice(&pattern);
        // Add some variation
        large_data.push((large_data.len() % 256) as u8);
    }
    test_pipeline_roundtrip("Large realistic data", &large_data)?;

    println!("✅ All XGBoost pipeline tests completed");
    Ok(())
}

fn test_pipeline_roundtrip(test_name: &str, data: &[u8]) -> Result<(), CompressionError> {
    println!("\n🧪 Testing: {} ({} bytes)", test_name, data.len());

    let mut compressor = XGBoostStage3Compressor::new();

    // Compress
    println!("  Step 1: Compressing...");
    let compressed = compressor.compress_block_data(data)?;
    println!("  Compressed: {} -> {} bytes ({:.1}:1 ratio)",
             data.len(), compressed.len(),
             data.len() as f32 / compressed.len() as f32);

    // Decompress
    println!("  Step 2: Decompressing...");
    let decompressed = compressor.decompress_block_data(&compressed)?;
    println!("  Decompressed: {} bytes", decompressed.len());

    // Verify roundtrip
    if data == decompressed {
        println!("  ✅ Pipeline roundtrip successful");
    } else {
        println!("  ❌ Pipeline roundtrip failed!");
        println!("  Original length: {}", data.len());
        println!("  Decompressed length: {}", decompressed.len());

        // Show differences
        let max_show = 32.min(data.len()).min(decompressed.len());
        println!("  First {} bytes comparison:", max_show);
        print!("  Original:     ");
        for i in 0..max_show {
            print!("{:02X} ", data[i]);
        }
        println!();
        print!("  Decompressed: ");
        for i in 0..max_show {
            if i < decompressed.len() {
                print!("{:02X} ", decompressed[i]);
            } else {
                print!("-- ");
            }
        }
        println!();

        // Find first difference
        for i in 0..data.len().min(decompressed.len()) {
            if data[i] != decompressed[i] {
                println!("  First difference at byte {}: {:02X} != {:02X}", i, data[i], decompressed[i]);
                break;
            }
        }

        return Err(CompressionError::InvalidFormat);
    }

    Ok(())
}

pub fn debug_individual_components() -> Result<(), CompressionError> {
    println!("\n🔧 DEBUGGING INDIVIDUAL PIPELINE COMPONENTS");

    let test_data = vec![1, 2, 3, 4, 1, 2, 3, 4, 5, 6, 7, 8];

    // Test RepetitiveCompressor (we know this works)
    println!("\n📊 Testing RepetitiveCompressor:");
    let mut repetitive = super::ensemble_compressor::RepetitiveCompressor::new();
    let compressed = repetitive.compress(&test_data)?;
    let decompressed = repetitive.decompress(&compressed)?;
    println!("  RepetitiveCompressor: {} -> {} bytes, roundtrip: {}",
             test_data.len(), compressed.len(), test_data == decompressed);

    // Test EnsembleCompressor with Repetitive strategy
    println!("\n📊 Testing EnsembleCompressor with Repetitive strategy:");
    let mut ensemble = super::ensemble_compressor::EnsembleCompressor::new();
    let strategy = CompressionStrategy::Repetitive;
    let compressed = ensemble.compress_with_strategy(&test_data, &strategy)?;
    let decompressed = ensemble.decompress_with_strategy(&compressed, &strategy)?;
    println!("  EnsembleCompressor: {} -> {} bytes, roundtrip: {}",
             test_data.len(), compressed.len(), test_data == decompressed);

    // Test TreeCompressor
    println!("\n📊 Testing TreeCompressor:");
    let tree_compressor = super::tree_compressor::TreeCompressor::new();
    let compressed = tree_compressor.apply_tree_compression(&test_data)?;
    let decompressed = tree_compressor.reverse_tree_compression(&compressed)?;
    println!("  TreeCompressor: {} -> {} bytes, roundtrip: {}",
             test_data.len(), compressed.len(), test_data == decompressed);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_xgboost_pipeline() {
        debug_xgboost_pipeline().unwrap();
    }

    #[test]
    fn test_debug_individual_components() {
        debug_individual_components().unwrap();
    }
}