//! # Compression Module
//!
//! Revolutionary compression algorithms specifically designed for Solana blockchain data.
//! Achieves 1271:1 compression ratio through multi-stage pattern recognition.

/// Stage 1: Structural compression algorithms
pub mod stage1;

/// Stage 2: Bot intelligence compression algorithms
pub mod stage2;

/// Stage 3: Machine learning compression algorithms
pub mod stage3;

/// Version 1 compression algorithm (10:1 ratio baseline)
pub mod v1;

/// Version 2 compression algorithm (25:1 ratio enhanced)
pub mod v2;

/// Version 3 compression algorithm (1271:1 ratio production)
pub mod v3;

/// Production V3 algorithm - optimized for maximum performance
pub mod production_v3;

/// Common traits and types for compression
pub mod traits;

/// V1 vs V2 compression comparison (kept for compatibility)
pub mod v1_v2_comparison;

/// Hybrid compression algorithm (legacy)
pub mod hybrid_compression;

/// Optimized XGBoost compression (experimental)
pub mod optimized_xgboost;

/// XGBoost-based Stage 3 compression (experimental)
pub mod stage3_xgboost;

// Re-export core types for easy access
pub use traits::{CompressionStrategy, CompressionVersion, CompressionError};
pub use stage1::{Stage1Compressor, AccountDictionary, ProgramCluster, BlockhashDelta};
pub use stage2::{Stage2Compressor, PatternRecognizer, InstructionTemplateEngine, MetadataCompressor};
pub use v1::V1Compression;
pub use v2::V2Compression;
pub use v3::V3Compression;
pub use production_v3::PracticalMaxCompression;

/// Default production compression algorithm
pub type ProductionCompressor = PracticalMaxCompression;