//! Debug RepetitiveCompressor roundtrip failures

use super::ensemble_compressor::RepetitiveCompressor;
use crate::compression::traits::CompressionError;

pub fn debug_repetitive_roundtrip() -> Result<(), CompressionError> {
    println!("🔍 DEBUGGING REPETITIVE COMPRESSOR ROUNDTRIP");

    // Test 1: Simple repetitive data
    let simple_data = vec![1, 2, 3, 4, 1, 2, 3, 4, 1, 2, 3, 4, 5, 6, 7, 8];
    test_roundtrip("Simple repetitive", &simple_data)?;

    // Test 2: Highly repetitive data
    let mut repetitive_data = Vec::new();
    let pattern = vec![0xAA, 0xBB, 0xCC, 0xDD];
    for _ in 0..10 {
        repetitive_data.extend_from_slice(&pattern);
    }
    test_roundtrip("Highly repetitive", &repetitive_data)?;

    // Test 3: Mixed data with some repetition
    let mixed_data = vec![
        1, 2, 3, 4,
        1, 2, 3, 4,  // repeat
        5, 6, 7, 8,
        1, 2, 3, 4,  // repeat again
        9, 10, 11, 12,
        0xFF, 0x80, 0x7F, 0x00, // edge case bytes
    ];
    test_roundtrip("Mixed data", &mixed_data)?;

    println!("✅ All RepetitiveCompressor tests completed");
    Ok(())
}

fn test_roundtrip(test_name: &str, data: &[u8]) -> Result<(), CompressionError> {
    println!("\n🧪 Testing: {} ({} bytes)", test_name, data.len());

    let mut compressor = RepetitiveCompressor::new();

    // Compress
    let compressed = compressor.compress(data)?;
    println!("  Compressed: {} -> {} bytes ({:.1}:1 ratio)",
             data.len(), compressed.len(),
             data.len() as f32 / compressed.len() as f32);

    // Decompress
    let decompressed = compressor.decompress(&compressed)?;
    println!("  Decompressed: {} bytes", decompressed.len());

    // Verify roundtrip
    if data == decompressed {
        println!("  ✅ Roundtrip successful");
    } else {
        println!("  ❌ Roundtrip failed!");
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

        return Err(CompressionError::InvalidFormat);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_repetitive_roundtrip() {
        debug_repetitive_roundtrip().unwrap();
    }
}