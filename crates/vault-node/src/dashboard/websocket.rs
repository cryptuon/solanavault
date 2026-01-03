//! # WebSocket Handler
//!
//! WebSocket handler for real-time dashboard updates.

use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Extension,
    },
    response::IntoResponse,
};
use tokio::time::interval;

use vault_core::dashboard::NodeDashboardApi;

/// WebSocket upgrade handler
pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Extension(api): Extension<Arc<NodeDashboardApi>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, api))
}

/// Handle WebSocket connection
async fn handle_socket(mut socket: WebSocket, api: Arc<NodeDashboardApi>) {
    // Send updates every second
    let mut ticker = interval(Duration::from_secs(1));

    loop {
        ticker.tick().await;

        // Get current snapshot
        let snapshot = api.get_snapshot().await;

        // Serialize to JSON
        let json = match serde_json::to_string(&snapshot) {
            Ok(j) => j,
            Err(e) => {
                eprintln!("Failed to serialize snapshot: {}", e);
                continue;
            }
        };

        // Send to client
        if socket.send(Message::Text(json)).await.is_err() {
            // Client disconnected
            break;
        }
    }
}
