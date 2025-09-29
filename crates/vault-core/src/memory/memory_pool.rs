//! # Memory Pool Management
//!
//! Efficient memory pool for large block processing with pre-allocated
//! buffers and zero-copy operations where possible.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::time::{Duration, Instant};
use thiserror::Error;
use bytes::{Bytes, BytesMut};

/// Memory pool configuration
#[derive(Debug, Clone)]
pub struct MemoryPoolConfig {
    /// Minimum buffer size
    pub min_buffer_size: usize,
    /// Maximum buffer size
    pub max_buffer_size: usize,
    /// Maximum number of buffers per size class
    pub max_buffers_per_size: usize,
    /// Maximum total memory usage
    pub max_total_memory: usize,
    /// Enable buffer reuse
    pub enable_buffer_reuse: bool,
    /// Buffer cleanup interval
    pub cleanup_interval: Duration,
    /// Maximum idle time before buffer cleanup
    pub max_idle_time: Duration,
}

impl Default for MemoryPoolConfig {
    fn default() -> Self {
        Self {
            min_buffer_size: 4 * 1024,        // 4KB
            max_buffer_size: 64 * 1024 * 1024, // 64MB
            max_buffers_per_size: 32,
            max_total_memory: 512 * 1024 * 1024, // 512MB
            enable_buffer_reuse: true,
            cleanup_interval: Duration::from_secs(60),
            max_idle_time: Duration::from_secs(300),
        }
    }
}

/// Buffer size classes for efficient allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum SizeClass {
    Small,   // 4KB - 64KB
    Medium,  // 64KB - 1MB
    Large,   // 1MB - 16MB
    XLarge,  // 16MB+
}

impl SizeClass {
    fn from_size(size: usize) -> Self {
        match size {
            0..=65536 => SizeClass::Small,        // 64KB
            65537..=1048576 => SizeClass::Medium, // 1MB
            1048577..=16777216 => SizeClass::Large, // 16MB
            _ => SizeClass::XLarge,
        }
    }

    fn default_size(&self) -> usize {
        match self {
            SizeClass::Small => 64 * 1024,      // 64KB
            SizeClass::Medium => 1024 * 1024,   // 1MB
            SizeClass::Large => 16 * 1024 * 1024, // 16MB
            SizeClass::XLarge => 64 * 1024 * 1024, // 64MB
        }
    }

    fn all() -> [SizeClass; 4] {
        [SizeClass::Small, SizeClass::Medium, SizeClass::Large, SizeClass::XLarge]
    }
}

/// Pooled buffer with metadata
#[derive(Debug)]
struct PooledBuffer {
    data: BytesMut,
    size_class: SizeClass,
    allocated_at: Instant,
    last_used: Instant,
    use_count: usize,
}

impl PooledBuffer {
    fn new(size: usize, size_class: SizeClass) -> Self {
        let now = Instant::now();
        Self {
            data: BytesMut::with_capacity(size),
            size_class,
            allocated_at: now,
            last_used: now,
            use_count: 0,
        }
    }

    fn reset(&mut self) {
        self.data.clear();
        self.last_used = Instant::now();
        self.use_count += 1;
    }

    fn is_idle(&self, max_idle_time: Duration) -> bool {
        self.last_used.elapsed() > max_idle_time
    }
}

/// Statistics for the memory pool
#[derive(Debug, Default, Clone)]
pub struct MemoryPoolStats {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub current_buffers: usize,
    pub total_memory_used: usize,
    pub fragmentation_ratio: f64,
}

impl MemoryPoolStats {
    pub fn hit_rate(&self) -> f64 {
        let total_requests = self.cache_hits + self.cache_misses;
        if total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total_requests as f64
        }
    }
}

/// High-performance memory pool for block processing
pub struct VaultMemoryPool {
    config: MemoryPoolConfig,

    // Buffer pools organized by size class
    pools: Arc<Mutex<std::collections::HashMap<SizeClass, VecDeque<PooledBuffer>>>>,

    // Memory usage tracking
    total_memory_used: Arc<Mutex<usize>>,

    // Statistics
    stats: Arc<Mutex<MemoryPoolStats>>,

    // Cleanup coordination
    cleanup_signal: Arc<(Mutex<bool>, Condvar)>,
}

impl VaultMemoryPool {
    /// Create a new memory pool
    pub fn new(config: MemoryPoolConfig) -> Self {
        let pool = Self {
            config,
            pools: Arc::new(Mutex::new(std::collections::HashMap::new())),
            total_memory_used: Arc::new(Mutex::new(0)),
            stats: Arc::new(Mutex::new(MemoryPoolStats::default())),
            cleanup_signal: Arc::new((Mutex::new(false), Condvar::new())),
        };

        // Pre-allocate some buffers for each size class
        pool.preallocate_buffers();

        // Start cleanup thread
        pool.start_cleanup_thread();

        pool
    }

    /// Allocate a buffer of the requested size
    pub fn allocate(&self, size: usize) -> Result<ManagedBuffer, MemoryPoolError> {
        if size > self.config.max_buffer_size {
            return Err(MemoryPoolError::SizeTooLarge(size));
        }

        let size_class = SizeClass::from_size(size);
        let actual_size = std::cmp::max(size, self.config.min_buffer_size);

        // Try to get from pool first
        if let Some(buffer) = self.get_from_pool(size_class, actual_size) {
            self.record_cache_hit();
            return Ok(ManagedBuffer::new(buffer, self.clone()));
        }

        // Create new buffer if pool miss
        self.record_cache_miss();
        let buffer = self.create_new_buffer(actual_size, size_class)?;
        Ok(ManagedBuffer::new(buffer, self.clone()))
    }

    /// Allocate a buffer with specific data
    pub fn allocate_with_data(&self, data: &[u8]) -> Result<ManagedBuffer, MemoryPoolError> {
        let mut buffer = self.allocate(data.len())?;
        buffer.extend_from_slice(data);
        Ok(buffer)
    }

    /// Allocate a zero-filled buffer
    pub fn allocate_zeroed(&self, size: usize) -> Result<ManagedBuffer, MemoryPoolError> {
        let mut buffer = self.allocate(size)?;
        buffer.resize(size, 0);
        Ok(buffer)
    }

    /// Get buffer from pool if available
    fn get_from_pool(&self, size_class: SizeClass, min_size: usize) -> Option<PooledBuffer> {
        let mut pools = self.pools.lock().unwrap();

        if let Some(pool) = pools.get_mut(&size_class) {
            while let Some(mut buffer) = pool.pop_front() {
                // Check if buffer is large enough
                if buffer.data.capacity() >= min_size {
                    buffer.reset();
                    return Some(buffer);
                }
                // Buffer too small, discard it
                self.update_memory_usage(-(buffer.data.capacity() as i64));
            }
        }

        None
    }

    /// Create a new buffer
    fn create_new_buffer(&self, size: usize, size_class: SizeClass) -> Result<PooledBuffer, MemoryPoolError> {
        // Check memory limits
        {
            let current_memory = *self.total_memory_used.lock().unwrap();
            if current_memory + size > self.config.max_total_memory {
                return Err(MemoryPoolError::OutOfMemory);
            }
        }

        let buffer = PooledBuffer::new(size, size_class);
        self.update_memory_usage(size as i64);
        self.record_allocation();

        Ok(buffer)
    }

    /// Return buffer to pool for reuse
    fn return_to_pool(&self, buffer: PooledBuffer) {
        if !self.config.enable_buffer_reuse {
            self.update_memory_usage(-(buffer.data.capacity() as i64));
            self.record_deallocation();
            return;
        }

        let size_class = buffer.size_class;

        let mut pools = self.pools.lock().unwrap();
        let pool = pools.entry(size_class).or_insert_with(VecDeque::new);

        // Limit pool size
        if pool.len() >= self.config.max_buffers_per_size {
            // Remove oldest buffer
            if let Some(old_buffer) = pool.pop_front() {
                self.update_memory_usage(-(old_buffer.data.capacity() as i64));
                self.record_deallocation();
            }
        }

        pool.push_back(buffer);
    }

    /// Pre-allocate buffers for better performance
    fn preallocate_buffers(&self) {
        for size_class in SizeClass::all() {
            let size = size_class.default_size();
            let count = std::cmp::min(4, self.config.max_buffers_per_size);

            for _ in 0..count {
                if let Ok(buffer) = self.create_new_buffer(size, size_class) {
                    self.return_to_pool(buffer);
                }
            }
        }
    }

    /// Start background cleanup thread
    fn start_cleanup_thread(&self) {
        let pools = Arc::clone(&self.pools);
        let config = self.config.clone();
        let total_memory = Arc::clone(&self.total_memory_used);
        let stats = Arc::clone(&self.stats);
        let cleanup_signal = Arc::clone(&self.cleanup_signal);

        std::thread::spawn(move || {
            let (lock, cvar) = &*cleanup_signal;

            loop {
                let mut should_stop = lock.lock().unwrap();
                let result = cvar.wait_timeout(should_stop, config.cleanup_interval).unwrap();
                should_stop = result.0;

                if *should_stop {
                    break;
                }

                drop(should_stop);

                // Perform cleanup
                Self::cleanup_idle_buffers(&pools, &config, &total_memory, &stats);
            }
        });
    }

    /// Clean up idle buffers
    fn cleanup_idle_buffers(
        pools: &Arc<Mutex<std::collections::HashMap<SizeClass, VecDeque<PooledBuffer>>>>,
        config: &MemoryPoolConfig,
        total_memory: &Arc<Mutex<usize>>,
        stats: &Arc<Mutex<MemoryPoolStats>>,
    ) {
        let mut pools = pools.lock().unwrap();
        let mut memory_freed = 0usize;
        let mut buffers_freed = 0usize;

        for pool in pools.values_mut() {
            pool.retain(|buffer| {
                if buffer.is_idle(config.max_idle_time) {
                    memory_freed += buffer.data.capacity();
                    buffers_freed += 1;
                    false
                } else {
                    true
                }
            });
        }

        if memory_freed > 0 {
            let mut total = total_memory.lock().unwrap();
            *total = total.saturating_sub(memory_freed);

            let mut stats = stats.lock().unwrap();
            stats.total_deallocations += buffers_freed as u64;
            stats.current_buffers = stats.current_buffers.saturating_sub(buffers_freed);
            stats.total_memory_used = *total;
        }
    }

    /// Update memory usage tracking
    fn update_memory_usage(&self, delta: i64) {
        let mut total = self.total_memory_used.lock().unwrap();
        if delta < 0 {
            *total = total.saturating_sub((-delta) as usize);
        } else {
            *total += delta as usize;
        }

        let mut stats = self.stats.lock().unwrap();
        stats.total_memory_used = *total;
    }

    /// Record allocation
    fn record_allocation(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_allocations += 1;
        stats.current_buffers += 1;
    }

    /// Record deallocation
    fn record_deallocation(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.total_deallocations += 1;
        stats.current_buffers = stats.current_buffers.saturating_sub(1);
    }

    /// Record cache hit
    fn record_cache_hit(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.cache_hits += 1;
    }

    /// Record cache miss
    fn record_cache_miss(&self) {
        let mut stats = self.stats.lock().unwrap();
        stats.cache_misses += 1;
    }

    /// Get memory pool statistics
    pub fn get_stats(&self) -> MemoryPoolStats {
        let mut stats = self.stats.lock().unwrap();
        stats.total_memory_used = *self.total_memory_used.lock().unwrap();

        // Calculate fragmentation ratio
        let pools = self.pools.lock().unwrap();
        let mut total_capacity = 0usize;
        let mut total_used = 0usize;

        for pool in pools.values() {
            for buffer in pool {
                total_capacity += buffer.data.capacity();
                total_used += buffer.data.len();
            }
        }

        stats.fragmentation_ratio = if total_capacity > 0 {
            1.0 - (total_used as f64 / total_capacity as f64)
        } else {
            0.0
        };

        stats.clone()
    }

    /// Get memory usage by size class
    pub fn get_usage_by_size_class(&self) -> std::collections::HashMap<String, usize> {
        let pools = self.pools.lock().unwrap();
        let mut usage = std::collections::HashMap::new();

        for (&size_class, pool) in pools.iter() {
            let class_name = format!("{:?}", size_class);
            let class_memory: usize = pool.iter().map(|b| b.data.capacity()).sum();
            usage.insert(class_name, class_memory);
        }

        usage
    }

    /// Force cleanup of all idle buffers
    pub fn cleanup_now(&self) {
        Self::cleanup_idle_buffers(
            &self.pools,
            &self.config,
            &self.total_memory_used,
            &self.stats,
        );
    }

    /// Shutdown the memory pool
    pub fn shutdown(&self) {
        // Signal cleanup thread to stop
        let (lock, cvar) = &*self.cleanup_signal;
        let mut should_stop = lock.lock().unwrap();
        *should_stop = true;
        cvar.notify_all();

        // Clear all pools
        let mut pools = self.pools.lock().unwrap();
        pools.clear();

        let mut total = self.total_memory_used.lock().unwrap();
        *total = 0;
    }
}

impl Clone for VaultMemoryPool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pools: Arc::clone(&self.pools),
            total_memory_used: Arc::clone(&self.total_memory_used),
            stats: Arc::clone(&self.stats),
            cleanup_signal: Arc::clone(&self.cleanup_signal),
        }
    }
}

/// RAII managed buffer that automatically returns to pool
pub struct ManagedBuffer {
    buffer: Option<PooledBuffer>,
    pool: VaultMemoryPool,
}

impl ManagedBuffer {
    fn new(buffer: PooledBuffer, pool: VaultMemoryPool) -> Self {
        Self {
            buffer: Some(buffer),
            pool,
        }
    }

    /// Get the capacity of the buffer
    pub fn capacity(&self) -> usize {
        self.buffer.as_ref().map(|b| b.data.capacity()).unwrap_or(0)
    }

    /// Get the length of data in the buffer
    pub fn len(&self) -> usize {
        self.buffer.as_ref().map(|b| b.data.len()).unwrap_or(0)
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Extend the buffer with data
    pub fn extend_from_slice(&mut self, data: &[u8]) {
        if let Some(ref mut buffer) = self.buffer {
            buffer.data.extend_from_slice(data);
        }
    }

    /// Resize the buffer
    pub fn resize(&mut self, new_len: usize, value: u8) {
        if let Some(ref mut buffer) = self.buffer {
            buffer.data.resize(new_len, value);
        }
    }

    /// Clear the buffer
    pub fn clear(&mut self) {
        if let Some(ref mut buffer) = self.buffer {
            buffer.data.clear();
        }
    }

    /// Get a slice of the buffer data
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_ref().map(|b| b.data.as_ref()).unwrap_or(&[])
    }

    /// Get a mutable slice of the buffer data
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.buffer.as_mut().map(|b| b.data.as_mut()).unwrap_or(&mut [])
    }

    /// Convert to Bytes for zero-copy operations
    pub fn freeze(mut self) -> Bytes {
        if let Some(buffer) = self.buffer.take() {
            buffer.data.freeze()
        } else {
            Bytes::new()
        }
    }
}

impl Drop for ManagedBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            self.pool.return_to_pool(buffer);
        }
    }
}

impl std::ops::Deref for ManagedBuffer {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl std::ops::DerefMut for ManagedBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

/// Memory pool errors
#[derive(Debug, Error)]
pub enum MemoryPoolError {
    #[error("Requested size {0} exceeds maximum buffer size")]
    SizeTooLarge(usize),

    #[error("Out of memory")]
    OutOfMemory,

    #[error("Pool is shut down")]
    PoolShutdown,

    #[error("Invalid buffer operation")]
    InvalidOperation,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_pool_allocation() {
        let config = MemoryPoolConfig {
            max_total_memory: 1024 * 1024, // 1MB
            ..Default::default()
        };

        let pool = VaultMemoryPool::new(config);

        // Test basic allocation
        let buffer = pool.allocate(1024).unwrap();
        assert!(buffer.capacity() >= 1024);

        // Test allocation with data
        let data = vec![1, 2, 3, 4, 5];
        let buffer_with_data = pool.allocate_with_data(&data).unwrap();
        assert_eq!(buffer_with_data.as_slice(), &data);

        let stats = pool.get_stats();
        assert!(stats.total_allocations > 0);
    }

    #[test]
    fn test_buffer_reuse() {
        let config = MemoryPoolConfig {
            enable_buffer_reuse: true,
            max_buffers_per_size: 4,
            ..Default::default()
        };

        let pool = VaultMemoryPool::new(config);

        // Allocate and drop buffers
        for _ in 0..5 {
            let _buffer = pool.allocate(1024).unwrap();
            // Buffer automatically returned to pool on drop
        }

        let stats = pool.get_stats();
        assert!(stats.cache_hits > 0);
    }

    #[test]
    fn test_memory_limits() {
        let config = MemoryPoolConfig {
            max_total_memory: 1024, // 1KB limit
            ..Default::default()
        };

        let pool = VaultMemoryPool::new(config);

        // Should fail when exceeding memory limit
        let result = pool.allocate(2048);
        assert!(result.is_err());
    }
}