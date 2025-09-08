//! # Compression Traits
//!
//! Common traits and types for compression algorithms.

use serde::{Deserialize, Serialize};

/// A trait for compression algorithms
pub trait CompressionStrategy {
    /// Compresses the input data
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
    
    /// Decompresses the input data
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
    
    /// Returns the version of this compression algorithm
    fn version(&self) -> CompressionVersion;
}

/// Compression algorithm versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionVersion {
    V1,
    V2,
    V3,
}

/// Error type for compression operations
#[derive(Debug, thiserror::Error)]
pub enum CompressionError {
    #[error("Invalid compression format")]
    InvalidFormat,
    
    #[error("Unsupported compression version: {0:?}")]
    UnsupportedVersion(CompressionVersion),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}