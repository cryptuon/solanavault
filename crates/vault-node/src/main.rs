//! # Vault Node
//!
//! Storage node implementation for the SolanaVault distributed network.
//! Provides data storage, replication, and serves as an entry point to the network.
//!
//! ## Features
//!
//! - `tui`: Enable terminal user interface (--tui flag)
//! - `dashboard`: Enable web dashboard (--dashboard-port flag)
//! - `full`: Enable both TUI and web dashboard

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::RwLock;

use vault_core::{
    compression::ProductionCompressor,
    dashboard::{NodeDashboardApi, SimpleNetworkStatsProvider, NodeStatus},
    storage::StorageNode,
};

#[cfg(feature = "tui")]
mod tui;

#[cfg(feature = "dashboard")]
mod dashboard;

#[derive(Parser)]
#[clap(name = "vault-node", version = "0.1.0", author = "SolanaVault Team")]
struct Args {
    /// Node ID (unique identifier for this node)
    #[clap(long, default_value = "node-1")]
    node_id: String,

    /// Address to bind the node server
    #[clap(long, default_value = "127.0.0.1:8080")]
    bind_address: SocketAddr,

    /// Data directory for storing compressed blocks
    #[clap(long, default_value = "./vault-data")]
    data_dir: PathBuf,

    /// Storage capacity in bytes (default: 100GB)
    #[clap(long, default_value = "107374182400")]
    capacity: u64,

    /// Enable debug logging
    #[clap(long)]
    debug: bool,

    /// Bootstrap nodes for network discovery
    #[clap(long, value_delimiter = ',')]
    bootstrap_nodes: Vec<String>,

    /// Minimum stake required to participate
    #[clap(long, default_value = "1000000000")] // 1B tokens
    min_stake: u64,

    /// Enable TUI mode (terminal user interface)
    #[cfg(feature = "tui")]
    #[clap(long)]
    tui: bool,

    /// Enable web dashboard on specified port
    #[cfg(feature = "dashboard")]
    #[clap(long)]
    dashboard_port: Option<u16>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Check for TUI mode - skip logging initialization if TUI is enabled
    #[cfg(feature = "tui")]
    let tui_mode = args.tui;
    #[cfg(not(feature = "tui"))]
    let tui_mode = false;

    // Initialize logging (skip if TUI mode - TUI handles its own display)
    if !tui_mode {
        if args.debug {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
                .init();
        } else {
            env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
                .init();
        }

        println!("SolanaVault Node Starting...");
        println!("   Node ID: {}", args.node_id);
        println!("   Bind Address: {}", args.bind_address);
        println!("   Data Directory: {}", args.data_dir.display());
        println!(
            "   Storage Capacity: {:.2} GB",
            args.capacity as f64 / 1_000_000_000.0
        );
    }

    // Create data directory if it doesn't exist
    tokio::fs::create_dir_all(&args.data_dir).await?;

    // Initialize storage node
    let storage_node = Arc::new(RwLock::new(StorageNode::new_with_data_dir(
        args.node_id.clone(),
        args.bind_address.to_string(),
        args.capacity,
        args.data_dir.clone(),
    )));

    // Initialize the storage node
    {
        let mut node = storage_node.write().await;
        node.initialize().await?;
    }

    // Create network stats provider
    let network_provider = Arc::new(SimpleNetworkStatsProvider::new());

    // Create Dashboard API (shared between TUI and Web Dashboard)
    let dashboard_api = Arc::new(
        NodeDashboardApi::new(
            args.node_id.clone(),
            args.bind_address.to_string(),
            storage_node.clone(),
        )
        .with_network_provider(network_provider.clone()),
    );

    // Set status to running
    dashboard_api.set_status(NodeStatus::Running).await;

    // Handle TUI mode
    #[cfg(feature = "tui")]
    if args.tui {
        let mut tui_app = tui::TuiApp::new(dashboard_api.clone());
        return tui_app.run().await.map_err(|e| e.into());
    }

    // Handle Web Dashboard mode
    #[cfg(feature = "dashboard")]
    if let Some(port) = args.dashboard_port {
        let web_dashboard = dashboard::WebDashboard::new(dashboard_api.clone(), port);

        // Run web dashboard alongside the node
        let dashboard_handle = tokio::spawn(async move {
            if let Err(e) = web_dashboard.run().await {
                eprintln!("Dashboard error: {}", e);
            }
        });

        // Run the main node loop
        run_node_loop(storage_node.clone(), &args, network_provider).await?;

        // Wait for dashboard to finish (it won't unless there's an error)
        let _ = dashboard_handle.await;
    } else {
        // No dashboard, just run the node
        run_node_loop(storage_node.clone(), &args, network_provider).await?;
    }

    #[cfg(not(feature = "dashboard"))]
    {
        // No dashboard feature, just run the node
        run_node_loop(storage_node.clone(), &args, network_provider).await?;
    }

    Ok(())
}

async fn run_node_loop(
    storage_node: Arc<RwLock<StorageNode>>,
    args: &Args,
    _network_provider: Arc<SimpleNetworkStatsProvider>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize compression engine
    let _compressor = ProductionCompressor::new();
    println!("Compression engine initialized");

    // Initialize P2P networking
    println!("Initializing P2P network...");
    if !args.bootstrap_nodes.is_empty() {
        println!("   Bootstrap nodes: {:?}", args.bootstrap_nodes);
    }

    // Run storage demonstration
    {
        let mut node = storage_node.write().await;
        node.demonstrate_storage().await?;
    }

    println!("All vault node services started successfully!");

    // Main node loop with graceful shutdown
    loop {
        tokio::select! {
            _ = signal::ctrl_c() => {
                println!("Shutdown signal received, stopping...");
                break;
            }
            _ = tokio::time::sleep(tokio::time::Duration::from_secs(30)) => {
                // Periodic health check and stats
                let node = storage_node.read().await;
                let stats = node.get_storage_stats();
                log::info!(
                    "Node {} - Storage: {:.1}% used ({} blocks), Compression: {:.2}:1, Reputation: {:.2}",
                    args.node_id,
                    (stats.used_capacity as f64 / stats.total_capacity as f64) * 100.0,
                    stats.blocks_stored,
                    stats.total_compression_ratio,
                    node.reputation
                );
            }
        }
    }

    println!("SolanaVault Node stopped.");
    Ok(())
}
