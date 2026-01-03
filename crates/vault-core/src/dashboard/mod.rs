//! # Dashboard API
//!
//! Unified metrics interface for TUI and Web Dashboard.
//! Provides a common API that aggregates data from storage, network, and economics modules.

pub mod metrics;
pub mod history;
pub mod api;

pub use api::{NodeDashboardApi, NetworkStatsProvider, SimpleNetworkStatsProvider};
pub use metrics::*;
pub use history::MetricsHistory;
