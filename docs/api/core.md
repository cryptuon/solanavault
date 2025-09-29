# SolanaVault Core API Reference

This document provides comprehensive API reference for the SolanaVault core library.

## Overview

The SolanaVault core library (`vault-core`) provides the fundamental compression, storage, and memory management capabilities. It's designed to be used both standalone and as part of the larger SolanaVault network.

## Main Modules

### Compression Module

#### `vault_core::compression`

Core compression functionality with multi-stage pipeline.

```rust
use vault_core::compression::{
    CompressionStrategy,
    BlockchainCompressionAdapter,
    CompressionConfig,
    CompressionError,
};
```

##### Key Types

**`CompressionStrategy`**
```rust
pub enum CompressionStrategy {
    Fast,      // Optimized for speed
    Balanced,  // Balance of speed and ratio
    Maximum,   // Maximum compression ratio
}
```

**`CompressionConfig`**
```rust
pub struct CompressionConfig {
    pub strategy: CompressionStrategy,
    pub max_compression_time: Duration,
    pub target_ratio: f32,
    pub enable_ml_optimization: bool,
}
```

##### Main Functions

**`compress_block`**
```rust
pub fn compress_block(
    block_data: &[u8],
    config: &CompressionConfig,
) -> Result<CompressedBlock, CompressionError>
```

Compresses a Solana block using the multi-stage pipeline.

- **Parameters:**
  - `block_data`: Raw block data to compress
  - `config`: Compression configuration
- **Returns:** Compressed block with metadata
- **Errors:** `CompressionError` if compression fails

**Example:**
```rust
use vault_core::compression::*;

let block_data = get_solana_block_data();
let config = CompressionConfig {
    strategy: CompressionStrategy::Balanced,
    max_compression_time: Duration::from_secs(5),
    target_ratio: 20.0,
    enable_ml_optimization: true,
};

let compressed = compress_block(&block_data, &config)?;
println!("Compression ratio: {:.2}:1", compressed.compression_ratio);
```

**`decompress_block`**
```rust
pub fn decompress_block(
    compressed: &CompressedBlock,
) -> Result<Vec<u8>, CompressionError>
```

Decompresses a previously compressed block.

- **Parameters:**
  - `compressed`: Compressed block with metadata
- **Returns:** Original block data
- **Errors:** `CompressionError` if decompression fails

### Memory Management Module

#### `vault_core::memory`

Advanced memory management with multi-level caching and file-based storage.

```rust
use vault_core::memory::{
    VaultStorageEngine,
    VaultCacheManager,
    VaultMemoryPool,
    StorageEngineConfig,
    CacheConfig,
    MemoryPoolConfig,
};
```

##### Storage Engine

**`VaultStorageEngine`**
```rust
impl VaultStorageEngine {
    pub fn new(config: StorageEngineConfig) -> Result<Self, StorageError>;
    pub fn store_block(&self, slot: u64, data: &[u8]) -> Result<(), StorageError>;
    pub fn get_block(&self, slot: u64) -> Result<Option<Vec<u8>>, StorageError>;
    pub fn store_metadata<T: Serialize>(&self, key: &str, metadata: &T) -> Result<(), StorageError>;
    pub fn get_metadata<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Result<Option<T>, StorageError>;
    pub fn get_stats(&self) -> Result<DatabaseStats, StorageError>;
    pub fn compact(&self) -> Result<u64, StorageError>;
}
```

**Example:**
```rust
use vault_core::memory::*;

// Create storage engine
let config = StorageEngineConfig {
    db_path: "./vault-data".to_string(),
    max_cache_size: 256 * 1024 * 1024, // 256MB
    enable_compression: true,
    default_ttl: 3600, // 1 hour
    ..Default::default()
};

let storage = VaultStorageEngine::new(config)?;

// Store a block
storage.store_block(12345, &compressed_block_data)?;

// Retrieve a block
if let Some(data) = storage.get_block(12345)? {
    println!("Retrieved block: {} bytes", data.len());
}

// Get storage statistics
let stats = storage.get_stats()?;
println!("Stored blocks: {}", stats.blocks_count);
```

##### Cache Manager

**`VaultCacheManager`**
```rust
impl VaultCacheManager {
    pub fn new(config: CacheConfig) -> Result<Self, CacheError>;
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, CacheError>;
    pub fn put(&self, key: &str, data: Vec<u8>) -> Result<(), CacheError>;
    pub fn evict(&self, key: &str) -> Result<bool, CacheError>;
    pub fn clear_level(&self, level: CacheLevel) -> Result<(), CacheError>;
    pub fn get_metrics(&self) -> CacheMetrics;
}
```

### Storage Module

#### `vault_core::storage`

Distributed storage network functionality.

```rust
use vault_core::storage::{
    StorageNode,
    StorageConfig,
    StorageError,
};
```

**`StorageNode`**
```rust
impl StorageNode {
    pub fn new(config: StorageConfig) -> Result<Self, StorageError>;
    pub async fn start(&self) -> Result<(), StorageError>;
    pub async fn store_block(&self, block: CompressedBlock) -> Result<(), StorageError>;
    pub async fn retrieve_block(&self, slot: u64) -> Result<Option<CompressedBlock>, StorageError>;
    pub fn get_storage_stats(&self) -> StorageStats;
}
```

### Data Module

#### `vault_core::data`

Solana blockchain data access and caching.

```rust
use vault_core::data::{
    SolanaBlockClient,
    BlockCache,
    CachedBlock,
};
```

**`SolanaBlockClient`**
```rust
impl SolanaBlockClient {
    pub fn new(rpc_url: String) -> Self;
    pub async fn get_block(&self, slot: u64) -> Result<Vec<u8>, DataError>;
    pub async fn get_block_with_cache(&self, slot: u64) -> Result<CachedBlock, DataError>;
    pub fn enable_caching(&mut self, cache_config: CacheConfig);
}
```

## Error Handling

### Error Types

All modules use typed errors for better error handling:

```rust
// Compression errors
pub enum CompressionError {
    InvalidInput(String),
    CompressionFailed(String),
    DecompressionFailed(String),
    ModelLoadError(String),
}

// Storage errors
pub enum StorageError {
    IoError(String),
    SerializationError(String),
    ConfigurationError(String),
    StorageFull,
    EntryNotFound,
}

// Network errors
pub enum NetworkError {
    ConnectionFailed(String),
    Timeout,
    InvalidResponse(String),
    PeerNotFound(String),
}
```

### Error Conversion

Errors implement `From` traits for easy conversion:

```rust
use vault_core::prelude::*;

fn process_block(slot: u64) -> Result<CompressedBlock, Box<dyn std::error::Error>> {
    let storage = VaultStorageEngine::new(config)?; // StorageError -> Box<dyn Error>
    let block_data = storage.get_block(slot)?;       // StorageError -> Box<dyn Error>
    let compressed = compress_block(&block_data, &config)?; // CompressionError -> Box<dyn Error>
    Ok(compressed)
}
```

## Configuration

### Default Configurations

```rust
// Development configuration
let dev_config = StorageEngineConfig {
    db_path: "./dev-vault-data".to_string(),
    max_cache_size: 64 * 1024 * 1024,      // 64MB
    enable_compression: true,
    default_ttl: 3600,                     // 1 hour
    max_file_size: 32 * 1024 * 1024,       // 32MB
    sync_threads: 2,
};

// Production configuration
let prod_config = StorageEngineConfig {
    db_path: "/data/vault-storage".to_string(),
    max_cache_size: 1024 * 1024 * 1024,    // 1GB
    enable_compression: true,
    default_ttl: 86400,                     // 24 hours
    max_file_size: 512 * 1024 * 1024,      // 512MB
    sync_threads: 8,
};
```

### Environment Variables

Configuration can be overridden with environment variables:

```bash
export VAULT_DATA_DIR="/custom/path"
export VAULT_MAX_CACHE_SIZE="512MB"
export VAULT_COMPRESSION_LEVEL="maximum"
export VAULT_DEFAULT_TTL="7200"
```

## Performance Considerations

### Memory Usage

- **Cache Memory**: Configured via `max_cache_size`
- **Working Memory**: ~200-400MB during compression
- **Buffer Pool**: Memory pools reduce allocation overhead

### Concurrency

- **Thread Safety**: All public APIs are thread-safe
- **Async Support**: Storage and network operations support async/await
- **Parallel Processing**: Compression can utilize multiple cores

### Optimization Tips

```rust
// Pre-allocate configurations for better performance
let config = CompressionConfig::balanced(); // Use predefined configs

// Reuse storage engines across requests
let storage = Arc::new(VaultStorageEngine::new(config)?);

// Use bulk operations when possible
storage.store_blocks(block_batch)?;

// Enable compression for better I/O performance
let config = StorageEngineConfig {
    enable_compression: true,
    ..Default::default()
};
```

## Examples

### Complete Example: Block Processing Pipeline

```rust
use vault_core::prelude::*;
use std::sync::Arc;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize components
    let storage_config = StorageEngineConfig::default();
    let storage = Arc::new(VaultStorageEngine::new(storage_config)?);

    let compression_config = CompressionConfig::balanced();

    // Fetch block data
    let client = SolanaBlockClient::new("https://api.mainnet-beta.solana.com".to_string());
    let block_data = client.get_block(245000000).await?;

    // Compress block
    let compressed = compress_block(&block_data, &compression_config)?;
    println!("Compression ratio: {:.2}:1", compressed.compression_ratio);

    // Store compressed block
    storage.store_block(245000000, &compressed.data)?;

    // Retrieve and decompress
    if let Some(stored_data) = storage.get_block(245000000)? {
        let decompressed = decompress_block_data(&stored_data)?;
        assert_eq!(decompressed, block_data);
        println!("Round-trip successful!");
    }

    // Get storage statistics
    let stats = storage.get_stats()?;
    println!("Storage stats: {} blocks, {} MB used",
             stats.blocks_count,
             stats.total_memory_usage / 1024 / 1024);

    Ok(())
}
```

---

**Next**: [RPC Proxy API](rpc-proxy.md)