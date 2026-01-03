//! # Web Dashboard
//!
//! Web dashboard server for vault-node using Axum.

pub mod server;
pub mod routes;
pub mod websocket;
pub mod static_files;

pub use server::WebDashboard;
