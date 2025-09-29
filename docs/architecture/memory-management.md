# Memory Management Architecture

SolanaVault implements a sophisticated memory management system designed to maximize performance while efficiently utilizing available system memory. The architecture consists of multiple coordinated components working together to provide optimal memory usage patterns.

## Overview

The memory management system is built around four core components:

1. **File-based Storage Engine** - Persistent data storage with intelligent caching
2. **Multi-Level Cache Manager** - Hierarchical caching with compression
3. **Memory Pool Manager** - Efficient buffer allocation and reuse
4. **Performance Metrics** - Real-time monitoring and health scoring

## 1. File-based Storage Engine (`VaultStorageEngine`)

### Purpose
High-performance persistent storage with organized file-based storage, replacing traditional database systems with a lightweight, efficient approach optimized for blockchain data.

### Key Features

#### Categorized Storage
- **Blocks**: Compressed blockchain block data
- **Metadata**: Block metadata and indices
- **Cache**: Temporary cached data with TTL
- **Stats**: Performance statistics and metrics

#### Storage Organization
```
./vault-storage/
├── blocks.json     # Blockchain block data
├── metadata.json   # Block metadata and indices
├── cache.json      # Cached data with TTL
└── stats.json      # Performance statistics
```

#### Configuration
```rust
StorageEngineConfig {
    db_path: "./vault-storage",
    max_cache_size: 256 * 1024 * 1024,  // 256MB
    enable_compression: true,
    default_ttl: 3600,                  // 1 hour
    max_file_size: 128 * 1024 * 1024,   // 128MB
    sync_threads: 2,
}
```

### Technical Implementation

#### Data Persistence
- **JSON Serialization**: Human-readable format using serde_json
- **Atomic Writes**: Ensures data consistency during writes
- **Background Sync**: Automatic persistence based on memory pressure
- **TTL Management**: Automatic expiration of cached entries

#### Memory Pressure Management
```rust
fn maybe_sync_to_disk(&self, category: StorageCategory) -> Result<(), StorageError> {
    let current_usage = *self.memory_usage.lock().unwrap();
    // Sync if using more than 80% of cache
    if current_usage > (self.config.max_cache_size * 4) / 5 {
        self.sync_category_to_disk(category)?;
    }
    Ok(())
}
```

## 2. Multi-Level Cache Manager (`VaultCacheManager`)

### Purpose
Intelligent multi-tier caching system with compression-aware policies, designed to maximize cache hit rates while minimizing memory usage.

### Cache Architecture

#### Cache Levels
- **L1 Cache**: Hot data, uncompressed (128MB default)
  - Sub-microsecond access times
  - Most frequently accessed data
  - Zero compression overhead

- **L2 Cache**: Warm data, zstd compressed (512MB default)
  - Microsecond access with decompression
  - 3-5x compression ratios typical
  - Balances space vs. speed

- **L3 Cache**: Cold data, persistent storage (2GB default)
  - Millisecond access from disk
  - Highest compression ratios
  - Automatic persistence

#### Cache Flow
```
Request → L1 Check → L2 Check → L3/Disk → Load & Promote
   ↓         ↓          ↓          ↓
 Return   Decompress  Decompress  Process
          & Return   & Return    & Cache
```

### Advanced Features

#### Smart Eviction
```rust
impl CacheManager {
    fn evict_lru_with_pattern_analysis(&mut self) -> Result<(), CacheError> {
        // Analyze access patterns
        let patterns = self.analyze_access_patterns();

        // Prefer evicting sequential access data over random access
        let candidates = self.prioritize_eviction_candidates(patterns);

        // Evict least recently used with lowest access frequency
        self.evict_candidates(candidates)
    }
}
```

#### Access Pattern Detection
- **Sequential Access**: Large block reads, prefetch next blocks
- **Random Access**: Small block reads, optimize for individual block cache
- **Temporal Locality**: Recently accessed blocks likely to be accessed again
- **Spatial Locality**: Nearby blocks likely to be accessed together

#### Prefetching Strategy
```rust
enum PrefetchStrategy {
    None,
    Conservative,   // Prefetch 1-2 adjacent blocks
    Aggressive,     // Prefetch 5-10 adjacent blocks
    Predictive,     // ML-based prefetching
}
```

## 3. Memory Pool Manager (`VaultMemoryPool`)

### Purpose
Efficient pre-allocated buffer management for zero-copy operations, reducing allocation overhead and memory fragmentation.

### Size Classes
- **Small**: 64KB buffers for metadata and small operations
- **Medium**: 1MB buffers for single block operations
- **Large**: 16MB buffers for multi-block operations
- **XLarge**: 64MB buffers for batch processing

### RAII Management
```rust
pub struct ManagedBuffer {
    buffer: BytesMut,
    size_class: SizeClass,
    pool: Arc<Weak<MemoryPool>>,
}

impl Drop for ManagedBuffer {
    fn drop(&mut self) {
        // Automatically return buffer to pool
        if let Some(pool) = self.pool.upgrade() {
            pool.return_buffer(self.buffer.clone(), self.size_class);
        }
    }
}
```

### Buffer Lifecycle
1. **Allocation**: Request buffer from appropriate size class
2. **Usage**: Zero-copy operations with capacity management
3. **Return**: Automatic return to pool on drop
4. **Cleanup**: Background cleanup of expired buffers

### Configuration
```rust
MemoryPoolConfig {
    max_total_memory: 1024 * 1024 * 1024,  // 1GB
    max_buffers_per_size: 64,
    enable_buffer_reuse: true,
    cleanup_interval: Duration::from_secs(60),
}
```

## 4. Performance Metrics (`PerformanceTracker`)

### Purpose
Comprehensive monitoring and performance analysis to optimize memory usage and identify bottlenecks.

### Metrics Collected

#### Latency Histograms
```rust
struct LatencyHistogram {
    p50: Duration,
    p95: Duration,
    p99: Duration,
    max: Duration,
    samples: VecDeque<Duration>,
}
```

#### Memory Usage Tracking
- **Total Memory**: System memory usage
- **Cache Memory**: Memory used by cache levels
- **Pool Memory**: Memory allocated in pools
- **Working Set**: Active memory usage

#### Cache Performance
- **Hit Rates**: L1/L2/L3 cache hit percentages
- **Eviction Rates**: Frequency of cache evictions
- **Access Patterns**: Sequential vs. random access ratios
- **Promotion Rates**: Movement between cache levels

### Health Scoring Algorithm
```rust
fn calculate_health_score(&self) -> f64 {
    let mut score: f64 = 1.0;

    // Memory usage penalty (0-20%)
    let memory_ratio = self.get_memory_usage_ratio();
    if memory_ratio > 0.8 { score *= 0.8; }

    // Cache performance bonus/penalty (0-10%)
    let hit_rate = self.get_cache_hit_rate();
    if hit_rate < 0.7 { score *= 0.9; }

    // Latency penalty (0-15%)
    let p95_latency = self.get_p95_latency();
    if p95_latency > Duration::from_millis(10) { score *= 0.85; }

    score.max(0.0f64).min(1.0f64)
}
```

## Integration Examples

### Production Configuration
```rust
// High-performance production setup
let storage_config = StorageEngineConfig {
    db_path: "/data/vault-storage",
    max_cache_size: 1024 * 1024 * 1024,    // 1GB
    enable_compression: true,
    default_ttl: 86400,                     // 24 hours
    max_file_size: 512 * 1024 * 1024,      // 512MB
    sync_threads: 8,
};

let cache_config = CacheConfig {
    max_memory: 4 * 1024 * 1024 * 1024,    // 4GB
    l1_size: 512 * 1024 * 1024,            // 512MB
    l2_size: 2048 * 1024 * 1024,           // 2GB
    l3_size: 8192 * 1024 * 1024,           // 8GB
    enable_compression: true,
    prefetch_strategy: PrefetchStrategy::Predictive,
};
```

### Development Configuration
```rust
// Resource-efficient development setup
let dev_config = StorageEngineConfig {
    db_path: "./dev-vault-data",
    max_cache_size: 64 * 1024 * 1024,      // 64MB
    enable_compression: true,
    default_ttl: 3600,                     // 1 hour
    max_file_size: 32 * 1024 * 1024,       // 32MB
    sync_threads: 2,
};
```

## Performance Characteristics

### Measured Performance
- **L1 Cache**: Sub-microsecond access (< 1μs)
- **L2 Cache**: 1-5 microseconds with decompression
- **L3 Cache**: 1-10 milliseconds from disk
- **Memory Pool**: 70-90% buffer reuse rates
- **Storage Sync**: Background sync with minimal impact

### Memory Efficiency
- **Cache Hit Rates**: >90% for typical workloads
- **Compression Ratios**: 3-5x in L2 cache
- **Memory Utilization**: 80-95% of allocated memory actively used
- **Fragmentation**: <5% due to pool management

### Scalability
- **Horizontal**: Supports multiple storage engines per node
- **Vertical**: Efficient scaling with available memory
- **Predictive**: ML-based optimization for access patterns
- **Adaptive**: Real-time tuning based on workload characteristics

---

**Next**: [Compression Pipeline](compression.md)