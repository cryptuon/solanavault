//! # REST API Routes
//!
//! HTTP route handlers for the dashboard API.

use std::sync::Arc;

use axum::{response::IntoResponse, Extension, Json};
use serde::Serialize;

use vault_core::dashboard::{
    DashboardSnapshot, EconomicsMetrics, MetricsHistory, NetworkMetrics, NodeDashboardApi,
    StorageMetrics,
};

/// Get all statistics
pub async fn get_all_stats(
    Extension(api): Extension<Arc<NodeDashboardApi>>,
) -> impl IntoResponse {
    let snapshot = api.get_snapshot().await;
    Json(snapshot)
}

/// Get storage statistics only
pub async fn get_storage_stats(
    Extension(api): Extension<Arc<NodeDashboardApi>>,
) -> impl IntoResponse {
    let snapshot = api.get_snapshot().await;
    Json(snapshot.storage)
}

/// Get network statistics only
pub async fn get_network_stats(
    Extension(api): Extension<Arc<NodeDashboardApi>>,
) -> impl IntoResponse {
    let snapshot = api.get_snapshot().await;
    Json(snapshot.network)
}

/// Get economics statistics only
pub async fn get_economics_stats(
    Extension(api): Extension<Arc<NodeDashboardApi>>,
) -> impl IntoResponse {
    let snapshot = api.get_snapshot().await;
    Json(snapshot.economics)
}

/// Get metrics history for charts
pub async fn get_metrics_history(
    Extension(api): Extension<Arc<NodeDashboardApi>>,
) -> impl IntoResponse {
    let history = api.get_history().await;
    Json(history)
}

/// Health check endpoint
#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

pub async fn health_check() -> impl IntoResponse {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
