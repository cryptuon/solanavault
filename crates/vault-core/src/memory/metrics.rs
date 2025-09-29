//! # Memory Metrics and Performance Tracking
//!
//! Comprehensive monitoring system for memory usage, performance metrics,
//! and system health indicators.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use serde::{Serialize, Deserialize};

/// Performance tracking intervals
const METRIC_WINDOW_SIZE: usize = 1000;
const HISTOGRAM_BUCKETS: usize = 50;

/// System memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMetrics {
    /// Timestamp of the measurement
    pub timestamp: u64,

    /// Process memory usage
    pub process_memory: ProcessMemoryInfo,

    /// Vault-specific memory usage
    pub vault_memory: VaultMemoryInfo,

    /// Cache performance metrics
    pub cache_performance: CachePerformanceInfo,

    /// Storage engine metrics
    pub storage_metrics: StorageMetricsInfo,

    /// Compression metrics
    pub compression_metrics: CompressionMetricsInfo,
}

/// Process-level memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessMemoryInfo {
    /// Virtual memory size in bytes
    pub virtual_memory: u64,
    /// Resident set size in bytes
    pub resident_memory: u64,
    /// Shared memory in bytes
    pub shared_memory: u64,
    /// Heap usage in bytes
    pub heap_usage: u64,
    /// Stack usage in bytes
    pub stack_usage: u64,
}

/// Vault-specific memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultMemoryInfo {
    /// L1 cache usage in bytes
    pub l1_cache_usage: u64,
    /// L2 cache usage in bytes
    pub l2_cache_usage: u64,
    /// Memory pool usage in bytes
    pub memory_pool_usage: u64,
    /// Compression buffer usage in bytes
    pub compression_buffers: u64,
    /// Network buffer usage in bytes
    pub network_buffers: u64,
    /// Total vault memory usage
    pub total_vault_memory: u64,
}

/// Cache performance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachePerformanceInfo {
    /// Cache hit rate (0.0 - 1.0)
    pub hit_rate: f64,
    /// Average access latency in microseconds
    pub avg_access_latency_us: f64,
    /// Cache eviction rate (evictions per second)
    pub eviction_rate: f64,
    /// Cache compression ratio
    pub compression_ratio: f64,
    /// Number of cache misses that resulted in storage reads
    pub storage_fallback_rate: f64,
}

/// Storage engine metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageMetricsInfo {
    /// RocksDB memory usage in bytes
    pub rocksdb_memory: u64,
    /// Number of SST files
    pub sst_file_count: u64,
    /// Total storage size in bytes
    pub total_storage_size: u64,
    /// Read IOPS
    pub read_iops: f64,
    /// Write IOPS
    pub write_iops: f64,
    /// Average read latency in microseconds
    pub avg_read_latency_us: f64,
    /// Average write latency in microseconds
    pub avg_write_latency_us: f64,
}

/// Compression performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionMetricsInfo {
    /// Average compression ratio
    pub avg_compression_ratio: f64,
    /// Compression throughput in MB/s
    pub compression_throughput_mbps: f64,
    /// Decompression throughput in MB/s
    pub decompression_throughput_mbps: f64,
    /// Average compression latency in microseconds
    pub avg_compression_latency_us: f64,
    /// Average decompression latency in microseconds
    pub avg_decompression_latency_us: f64,
}

/// Performance sample for time-series analysis
#[derive(Debug, Clone)]
struct PerformanceSample {
    timestamp: Instant,
    value: f64,
    metadata: Option<HashMap<String, String>>,
}

/// Latency histogram for detailed performance analysis
#[derive(Debug, Clone)]
pub struct LatencyHistogram {
    buckets: Vec<u64>,
    bucket_boundaries: Vec<f64>,
    total_count: u64,
    sum: f64,
}

impl LatencyHistogram {
    fn new() -> Self {
        // Create exponential buckets from 1μs to 1s
        let mut boundaries = Vec::with_capacity(HISTOGRAM_BUCKETS);
        for i in 0..HISTOGRAM_BUCKETS {
            boundaries.push(10.0_f64.powf((i as f64 / HISTOGRAM_BUCKETS as f64) * 6.0));
        }

        Self {
            buckets: vec![0; HISTOGRAM_BUCKETS],
            bucket_boundaries: boundaries,
            total_count: 0,
            sum: 0.0,
        }
    }

    fn add_sample(&mut self, value_us: f64) {
        self.sum += value_us;
        self.total_count += 1;

        // Find appropriate bucket
        for (i, &boundary) in self.bucket_boundaries.iter().enumerate() {
            if value_us <= boundary {
                self.buckets[i] += 1;
                break;
            }
        }
    }

    pub fn percentile(&self, p: f64) -> f64 {
        if self.total_count == 0 {
            return 0.0;
        }

        let target_count = (self.total_count as f64 * p / 100.0) as u64;
        let mut count = 0;

        for (i, &bucket_count) in self.buckets.iter().enumerate() {
            count += bucket_count;
            if count >= target_count {
                return self.bucket_boundaries[i];
            }
        }

        self.bucket_boundaries.last().copied().unwrap_or(0.0)
    }

    pub fn average(&self) -> f64 {
        if self.total_count == 0 {
            0.0
        } else {
            self.sum / self.total_count as f64
        }
    }
}

/// Performance tracker for detailed monitoring
pub struct PerformanceTracker {
    // Memory usage over time
    memory_samples: Arc<Mutex<VecDeque<PerformanceSample>>>,

    // Latency tracking
    read_latencies: Arc<Mutex<LatencyHistogram>>,
    write_latencies: Arc<Mutex<LatencyHistogram>>,
    compression_latencies: Arc<Mutex<LatencyHistogram>>,
    decompression_latencies: Arc<Mutex<LatencyHistogram>>,

    // Throughput tracking
    throughput_samples: Arc<Mutex<HashMap<String, VecDeque<PerformanceSample>>>>,

    // Current metrics snapshot
    current_metrics: Arc<RwLock<MemoryMetrics>>,

    // System info
    start_time: Instant,
    collection_interval: Duration,
}

impl PerformanceTracker {
    /// Create a new performance tracker
    pub fn new(collection_interval: Duration) -> Self {
        Self {
            memory_samples: Arc::new(Mutex::new(VecDeque::with_capacity(METRIC_WINDOW_SIZE))),
            read_latencies: Arc::new(Mutex::new(LatencyHistogram::new())),
            write_latencies: Arc::new(Mutex::new(LatencyHistogram::new())),
            compression_latencies: Arc::new(Mutex::new(LatencyHistogram::new())),
            decompression_latencies: Arc::new(Mutex::new(LatencyHistogram::new())),
            throughput_samples: Arc::new(Mutex::new(HashMap::new())),
            current_metrics: Arc::new(RwLock::new(MemoryMetrics::default())),
            start_time: Instant::now(),
            collection_interval,
        }
    }

    /// Record a read operation latency
    pub fn record_read_latency(&self, latency_us: f64) {
        let mut histogram = self.read_latencies.lock().unwrap();
        histogram.add_sample(latency_us);
    }

    /// Record a write operation latency
    pub fn record_write_latency(&self, latency_us: f64) {
        let mut histogram = self.write_latencies.lock().unwrap();
        histogram.add_sample(latency_us);
    }

    /// Record compression operation latency
    pub fn record_compression_latency(&self, latency_us: f64) {
        let mut histogram = self.compression_latencies.lock().unwrap();
        histogram.add_sample(latency_us);
    }

    /// Record decompression operation latency
    pub fn record_decompression_latency(&self, latency_us: f64) {
        let mut histogram = self.decompression_latencies.lock().unwrap();
        histogram.add_sample(latency_us);
    }

    /// Record throughput sample
    pub fn record_throughput(&self, operation: &str, bytes_per_second: f64) {
        let now = Instant::now();
        let sample = PerformanceSample {
            timestamp: now,
            value: bytes_per_second,
            metadata: None,
        };

        let mut throughput = self.throughput_samples.lock().unwrap();
        let samples = throughput.entry(operation.to_string())
            .or_insert_with(|| VecDeque::with_capacity(METRIC_WINDOW_SIZE));

        samples.push_back(sample);

        // Keep only recent samples
        while samples.len() > METRIC_WINDOW_SIZE {
            samples.pop_front();
        }
    }

    /// Update memory metrics
    pub fn update_memory_metrics(&self, metrics: MemoryMetrics) {
        // Update current metrics
        {
            let mut current = self.current_metrics.write().unwrap();
            *current = metrics.clone();
        }

        // Add to historical samples
        let total_memory = metrics.vault_memory.total_vault_memory as f64;
        let sample = PerformanceSample {
            timestamp: Instant::now(),
            value: total_memory,
            metadata: None,
        };

        let mut samples = self.memory_samples.lock().unwrap();
        samples.push_back(sample);

        // Keep only recent samples
        while samples.len() > METRIC_WINDOW_SIZE {
            samples.pop_front();
        }
    }

    /// Get current memory metrics
    pub fn get_current_metrics(&self) -> MemoryMetrics {
        self.current_metrics.read().unwrap().clone()
    }

    /// Get latency statistics
    pub fn get_latency_stats(&self) -> LatencyStats {
        LatencyStats {
            read_p50: self.read_latencies.lock().unwrap().percentile(50.0),
            read_p95: self.read_latencies.lock().unwrap().percentile(95.0),
            read_p99: self.read_latencies.lock().unwrap().percentile(99.0),
            write_p50: self.write_latencies.lock().unwrap().percentile(50.0),
            write_p95: self.write_latencies.lock().unwrap().percentile(95.0),
            write_p99: self.write_latencies.lock().unwrap().percentile(99.0),
            compression_avg: self.compression_latencies.lock().unwrap().average(),
            decompression_avg: self.decompression_latencies.lock().unwrap().average(),
        }
    }

    /// Get throughput statistics
    pub fn get_throughput_stats(&self) -> HashMap<String, ThroughputStats> {
        let throughput = self.throughput_samples.lock().unwrap();
        let mut stats = HashMap::new();

        for (operation, samples) in throughput.iter() {
            if samples.is_empty() {
                continue;
            }

            let values: Vec<f64> = samples.iter().map(|s| s.value).collect();
            let avg = values.iter().sum::<f64>() / values.len() as f64;
            let max = values.iter().copied().fold(0.0, f64::max);
            let min = values.iter().copied().fold(f64::INFINITY, f64::min);

            stats.insert(operation.clone(), ThroughputStats {
                avg_mbps: avg,
                max_mbps: max,
                min_mbps: min,
                sample_count: values.len(),
            });
        }

        stats
    }

    /// Get memory usage trend (positive = increasing, negative = decreasing)
    pub fn get_memory_trend(&self) -> f64 {
        let samples = self.memory_samples.lock().unwrap();

        if samples.len() < 10 {
            return 0.0;
        }

        let recent_avg = samples.iter().rev().take(5)
            .map(|s| s.value)
            .sum::<f64>() / 5.0;

        let older_avg = samples.iter().rev().skip(5).take(5)
            .map(|s| s.value)
            .sum::<f64>() / 5.0;

        (recent_avg - older_avg) / older_avg
    }

    /// Generate performance report
    pub fn generate_report(&self) -> PerformanceReport {
        let current_metrics = self.get_current_metrics();
        let latency_stats = self.get_latency_stats();
        let throughput_stats = self.get_throughput_stats();
        let memory_trend = self.get_memory_trend();

        PerformanceReport {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
            memory_metrics: current_metrics,
            latency_stats,
            throughput_stats,
            memory_trend,
            health_score: self.calculate_health_score(),
        }
    }

    /// Calculate overall health score (0.0 = poor, 1.0 = excellent)
    fn calculate_health_score(&self) -> f64 {
        let mut score: f64 = 1.0;

        // Penalize high memory usage
        let current = self.get_current_metrics();
        let memory_usage_ratio = current.vault_memory.total_vault_memory as f64 / (1024.0 * 1024.0 * 1024.0); // GB
        if memory_usage_ratio > 0.8 {
            score *= 0.8;
        }

        // Penalize poor cache performance
        if current.cache_performance.hit_rate < 0.7 {
            score *= 0.9;
        }

        // Penalize high latencies
        let latency = self.get_latency_stats();
        if latency.read_p95 > 10000.0 { // 10ms
            score *= 0.85;
        }

        // Penalize increasing memory trend
        let trend = self.get_memory_trend();
        if trend > 0.1 { // 10% increase
            score *= 0.9;
        }

        score.max(0.0f64).min(1.0f64)
    }

    /// Reset all metrics
    pub fn reset(&self) {
        self.memory_samples.lock().unwrap().clear();
        *self.read_latencies.lock().unwrap() = LatencyHistogram::new();
        *self.write_latencies.lock().unwrap() = LatencyHistogram::new();
        *self.compression_latencies.lock().unwrap() = LatencyHistogram::new();
        *self.decompression_latencies.lock().unwrap() = LatencyHistogram::new();
        self.throughput_samples.lock().unwrap().clear();
    }
}

/// Latency statistics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub read_p50: f64,
    pub read_p95: f64,
    pub read_p99: f64,
    pub write_p50: f64,
    pub write_p95: f64,
    pub write_p99: f64,
    pub compression_avg: f64,
    pub decompression_avg: f64,
}

/// Throughput statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputStats {
    pub avg_mbps: f64,
    pub max_mbps: f64,
    pub min_mbps: f64,
    pub sample_count: usize,
}

/// Comprehensive performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    pub timestamp: u64,
    pub uptime_seconds: u64,
    pub memory_metrics: MemoryMetrics,
    pub latency_stats: LatencyStats,
    pub throughput_stats: HashMap<String, ThroughputStats>,
    pub memory_trend: f64,
    pub health_score: f64,
}

impl Default for MemoryMetrics {
    fn default() -> Self {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        Self {
            timestamp: now,
            process_memory: ProcessMemoryInfo {
                virtual_memory: 0,
                resident_memory: 0,
                shared_memory: 0,
                heap_usage: 0,
                stack_usage: 0,
            },
            vault_memory: VaultMemoryInfo {
                l1_cache_usage: 0,
                l2_cache_usage: 0,
                memory_pool_usage: 0,
                compression_buffers: 0,
                network_buffers: 0,
                total_vault_memory: 0,
            },
            cache_performance: CachePerformanceInfo {
                hit_rate: 0.0,
                avg_access_latency_us: 0.0,
                eviction_rate: 0.0,
                compression_ratio: 1.0,
                storage_fallback_rate: 0.0,
            },
            storage_metrics: StorageMetricsInfo {
                rocksdb_memory: 0,
                sst_file_count: 0,
                total_storage_size: 0,
                read_iops: 0.0,
                write_iops: 0.0,
                avg_read_latency_us: 0.0,
                avg_write_latency_us: 0.0,
            },
            compression_metrics: CompressionMetricsInfo {
                avg_compression_ratio: 1.0,
                compression_throughput_mbps: 0.0,
                decompression_throughput_mbps: 0.0,
                avg_compression_latency_us: 0.0,
                avg_decompression_latency_us: 0.0,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_latency_histogram() {
        let mut histogram = LatencyHistogram::new();

        // Add some samples
        for i in 1..=100 {
            histogram.add_sample(i as f64);
        }

        assert_eq!(histogram.total_count, 100);
        assert!(histogram.percentile(50.0) > 0.0);
        assert!(histogram.percentile(95.0) > histogram.percentile(50.0));
        assert!(histogram.average() > 0.0);
    }

    #[test]
    fn test_performance_tracker() {
        let tracker = PerformanceTracker::new(Duration::from_secs(1));

        // Record some metrics
        tracker.record_read_latency(100.0);
        tracker.record_write_latency(200.0);
        tracker.record_throughput("compression", 50.0);

        let latency_stats = tracker.get_latency_stats();
        assert!(latency_stats.read_p50 > 0.0);

        let throughput_stats = tracker.get_throughput_stats();
        assert!(throughput_stats.contains_key("compression"));
    }

    #[test]
    fn test_health_score_calculation() {
        let tracker = PerformanceTracker::new(Duration::from_secs(1));
        let report = tracker.generate_report();

        // Default health score should be good
        assert!(report.health_score > 0.5);
    }
}