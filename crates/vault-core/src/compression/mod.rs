//! # Compression Module
//!
//! This module provides versioned compression algorithms for Solana block data.

/// Stage 1 structural compression algorithms
pub mod stage1;

/// Stage 2 bot intelligence compression algorithms
pub mod stage2;

/// Version 1 compression algorithm
pub mod v1;

/// Version 2 compression algorithm
pub mod v2;

/// Version 3 compression algorithm
pub mod v3;

/// Common traits and types for compression
pub mod traits;

/// Comprehensive tests with real Solana data
pub mod tests;

/// Benchmark compression algorithms
pub mod benchmark;

/// V1 vs V2 compression comparison
pub mod v1_v2_comparison;

pub use traits::{CompressionStrategy, CompressionVersion, CompressionError};
pub use stage1::{Stage1Compressor, AccountDictionary, ProgramCluster, BlockhashDelta};
pub use stage2::{Stage2Compressor, PatternRecognizer, InstructionTemplateEngine, MetadataCompressor};
pub use v1::V1Compression;
pub use v2::V2Compression;
pub use v3::V3Compression;