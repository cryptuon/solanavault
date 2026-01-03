//! # Web Dashboard Server
//!
//! Main Axum web server for the dashboard.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    routing::get,
    Extension, Router,
};
use tower_http::cors::{Any, CorsLayer};

use vault_core::dashboard::NodeDashboardApi;

use super::routes;
use super::static_files::static_handler;
use super::websocket::websocket_handler;

/// Web Dashboard server
pub struct WebDashboard {
    api: Arc<NodeDashboardApi>,
    port: u16,
}

impl WebDashboard {
    /// Create a new web dashboard server
    pub fn new(api: Arc<NodeDashboardApi>, port: u16) -> Self {
        Self { api, port }
    }

    /// Run the web dashboard server
    pub async fn run(self) -> std::io::Result<()> {
        let api = self.api.clone();

        // Build router
        let app = Router::new()
            // REST API routes
            .route("/api/stats", get(routes::get_all_stats))
            .route("/api/storage", get(routes::get_storage_stats))
            .route("/api/network", get(routes::get_network_stats))
            .route("/api/economics", get(routes::get_economics_stats))
            .route("/api/history", get(routes::get_metrics_history))
            .route("/api/health", get(routes::health_check))
            // WebSocket for real-time updates
            .route("/ws", get(websocket_handler))
            // Static files (Vue.js SPA) - catch-all
            .fallback(static_handler)
            // Middleware
            .layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            )
            .layer(Extension(api));

        let addr = SocketAddr::from(([0, 0, 0, 0], self.port));
        println!("Web Dashboard listening on http://{}", addr);
        println!("  API endpoints: /api/stats, /api/storage, /api/network, /api/economics");
        println!("  WebSocket: ws://{}/ws", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}
