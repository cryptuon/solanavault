//! # Metrics History
//!
//! Time-series data storage for dashboard sparklines and charts.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::metrics::DashboardSnapshot;

/// Default history size (5 minutes at 1-second intervals)
const DEFAULT_HISTORY_SIZE: usize = 300;

/// Time-series metrics history for charts and sparklines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsHistory {
    /// Storage usage percentage over time
    pub storage_used_percent: VecDeque<f64>,
    /// Compression ratio over time
    pub compression_ratio: VecDeque<f64>,
    /// Connected peers over time
    pub connected_peers: VecDeque<f64>,
    /// Messages per second over time
    pub messages_per_second: VecDeque<f64>,
    /// Cache hit rate over time
    pub cache_hit_rate: VecDeque<f64>,
    /// Cumulative rewards over time
    pub rewards_earned: VecDeque<f64>,
    /// Maximum history size
    max_size: usize,
    /// Last messages count (for calculating per-second)
    #[serde(skip)]
    last_messages: u64,
    /// Last timestamp
    #[serde(skip)]
    last_timestamp: u64,
}

impl MetricsHistory {
    /// Create a new metrics history with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_HISTORY_SIZE)
    }

    /// Create a new metrics history with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            storage_used_percent: VecDeque::with_capacity(capacity),
            compression_ratio: VecDeque::with_capacity(capacity),
            connected_peers: VecDeque::with_capacity(capacity),
            messages_per_second: VecDeque::with_capacity(capacity),
            cache_hit_rate: VecDeque::with_capacity(capacity),
            rewards_earned: VecDeque::with_capacity(capacity),
            max_size: capacity,
            last_messages: 0,
            last_timestamp: 0,
        }
    }

    /// Push a new snapshot's metrics into history
    pub fn push(&mut self, snapshot: &DashboardSnapshot) {
        let max_size = self.max_size;

        // Calculate storage usage percentage
        let storage_percent = if snapshot.storage.total_capacity > 0 {
            (snapshot.storage.used_capacity as f64 / snapshot.storage.total_capacity as f64) * 100.0
        } else {
            0.0
        };
        Self::push_to_queue(&mut self.storage_used_percent, storage_percent, max_size);

        // Compression ratio
        Self::push_to_queue(&mut self.compression_ratio, snapshot.storage.compression_ratio, max_size);

        // Connected peers
        Self::push_to_queue(&mut self.connected_peers, snapshot.network.connected_peers as f64, max_size);

        // Calculate messages per second
        let total_messages = snapshot.network.messages_sent + snapshot.network.messages_received;
        let time_diff = if self.last_timestamp > 0 {
            snapshot.timestamp.saturating_sub(self.last_timestamp)
        } else {
            1
        };
        let messages_per_sec = if time_diff > 0 && self.last_messages > 0 {
            (total_messages.saturating_sub(self.last_messages)) as f64 / time_diff as f64
        } else {
            0.0
        };
        Self::push_to_queue(&mut self.messages_per_second, messages_per_sec, max_size);
        self.last_messages = total_messages;
        self.last_timestamp = snapshot.timestamp;

        // Cache hit rate
        Self::push_to_queue(&mut self.cache_hit_rate, snapshot.storage.cache_hit_rate * 100.0, max_size);

        // Rewards earned
        Self::push_to_queue(&mut self.rewards_earned, snapshot.economics.rewards.total_earned as f64, max_size);
    }

    /// Push to queue with size management (static to avoid borrow issues)
    fn push_to_queue(queue: &mut VecDeque<f64>, value: f64, max_size: usize) {
        if queue.len() >= max_size {
            queue.pop_front();
        }
        queue.push_back(value);
    }

    /// Get data as u64 vector for sparkline rendering
    pub fn as_sparkline_data(&self, metric: &str) -> Vec<u64> {
        let queue = match metric {
            "storage_used" => &self.storage_used_percent,
            "compression_ratio" => &self.compression_ratio,
            "connected_peers" => &self.connected_peers,
            "messages_per_second" => &self.messages_per_second,
            "cache_hit_rate" => &self.cache_hit_rate,
            "rewards_earned" => &self.rewards_earned,
            _ => return vec![],
        };
        queue.iter().map(|v| *v as u64).collect()
    }

    /// Get the length of history
    pub fn len(&self) -> usize {
        self.storage_used_percent.len()
    }

    /// Check if history is empty
    pub fn is_empty(&self) -> bool {
        self.storage_used_percent.is_empty()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.storage_used_percent.clear();
        self.compression_ratio.clear();
        self.connected_peers.clear();
        self.messages_per_second.clear();
        self.cache_hit_rate.clear();
        self.rewards_earned.clear();
        self.last_messages = 0;
        self.last_timestamp = 0;
    }
}

impl Default for MetricsHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dashboard::metrics::*;

    #[test]
    fn test_metrics_history() {
        let mut history = MetricsHistory::with_capacity(5);

        for i in 0..10 {
            let snapshot = DashboardSnapshot {
                timestamp: i as u64,
                storage: StorageMetrics {
                    total_capacity: 100,
                    used_capacity: i * 10,
                    compression_ratio: 15.0 + i as f64,
                    ..Default::default()
                },
                network: NetworkMetrics {
                    connected_peers: i as usize,
                    messages_sent: i as u64 * 100,
                    messages_received: i as u64 * 50,
                    ..Default::default()
                },
                ..Default::default()
            };
            history.push(&snapshot);
        }

        // Should only keep last 5 entries
        assert_eq!(history.len(), 5);

        // Check sparkline data
        let storage_data = history.as_sparkline_data("storage_used");
        assert_eq!(storage_data.len(), 5);
    }
}
