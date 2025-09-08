//! # Compression Module
//!
//! This module provides versioned compression algorithms for Solana block data.

/// Version 1 compression algorithm
pub mod v1;

/// Version 2 compression algorithm
pub mod v2;

/// Version 3 compression algorithm
pub mod v3;

/// Common traits and types for compression
pub mod traits;

pub use traits::CompressionStrategy;
pub use traits::CompressionVersion;