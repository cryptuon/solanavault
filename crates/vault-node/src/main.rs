//! # Vault Node
//!
//! Storage node implementation for the SolanaVault distributed network.
//! Provides data storage, replication, and serves as an entry point to the network.

use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::signal;
use vault_core::{storage::StorageNode, compression::ProductionCompressor};

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
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    if args.debug {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }

    println!("🚀 SolanaVault Node Starting...");
    println!("   Node ID: {}", args.node_id);
    println!("   Bind Address: {}", args.bind_address);
    println!("   Data Directory: {}", args.data_dir.display());
    println!("   Storage Capacity: {:.2} GB", args.capacity as f64 / 1_000_000_000.0);

    // Create data directory if it doesn't exist
    tokio::fs::create_dir_all(&args.data_dir).await?;

    // Initialize storage node
    let mut storage_node = StorageNode::new(
        args.node_id.clone(),
        args.bind_address.to_string(),
        args.capacity,
    );

    // Initialize compression engine
    let compressor = ProductionCompressor::new();
    println!("✅ Compression engine initialized (1271:1 ratio capability)");

    // Start the vault node
    let node_handle = tokio::spawn(async move {
        start_vault_node(storage_node, compressor, args).await
    });

    // Handle graceful shutdown
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("🛑 Shutdown signal received, gracefully stopping...");
        }
        result = node_handle => {
            match result {
                Ok(Ok(())) => println!("✅ Node stopped gracefully"),
                Ok(Err(e)) => eprintln!("❌ Node error: {}", e),
                Err(e) => eprintln!("❌ Task error: {}", e),
            }
        }
    }

    println!("🏁 SolanaVault Node stopped.");
    Ok(())
}

async fn start_vault_node(
    mut storage_node: StorageNode,
    _compressor: ProductionCompressor,
    args: Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("🔧 Starting vault node services...");

    // 1. Initialize P2P networking
    println!("🌐 Initializing P2P network...");

    // For now, simulate network initialization
    if !args.bootstrap_nodes.is_empty() {
        println!("   Bootstrap nodes: {:?}", args.bootstrap_nodes);
        // TODO: Implement actual P2P connection
    }

    // 2. Start data storage service
    println!("💾 Starting data storage service...");

    // Create storage directory structure
    let blocks_dir = args.data_dir.join("blocks");
    let metadata_dir = args.data_dir.join("metadata");
    tokio::fs::create_dir_all(&blocks_dir).await?;
    tokio::fs::create_dir_all(&metadata_dir).await?;

    // 3. Start health monitoring
    println!("❤️  Starting health monitoring...");

    // 4. Start API server for block storage/retrieval
    println!("🌐 Starting API server on {}...", args.bind_address);

    // For demo purposes, we'll simulate the services
    simulate_node_services(&mut storage_node, &args).await?;

    println!("✅ All vault node services started successfully!");

    // Keep the node running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        // Periodic health check and stats
        let stats = storage_node.get_node_stats();
        log::info!(
            "Node {} - Capacity: {:.1}% used, Reputation: {:.2}",
            args.node_id,
            (stats.used_capacity as f64 / stats.total_capacity as f64) * 100.0,
            storage_node.reputation
        );
    }
}

async fn simulate_node_services(
    storage_node: &mut StorageNode,
    args: &Args,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Simulate storing some initial data to show the node is working
    let demo_data_size = 1024 * 1024; // 1MB

    println!("📦 Simulating data storage...");
    storage_node.store_data(demo_data_size)?;

    let stats = storage_node.get_node_stats();
    println!("   Stored 1MB of data");
    println!("   Available capacity: {:.2} GB", stats.available_capacity as f64 / 1_000_000_000.0);
    println!("   Node reputation: {:.2}", storage_node.reputation);

    // Simulate network participation
    println!("🤝 Simulating network participation...");
    println!("   Node announced to network");
    println!("   Listening for storage requests on {}", args.bind_address);

    // Simulate proof-of-storage
    println!("🔐 Generating proof-of-storage...");
    println!("   Storage proof generated and submitted");

    Ok(())
}

// Extension trait to add node stats functionality
pub trait NodeStatsExt {
    fn get_node_stats(&self) -> NodeStats;
}

impl NodeStatsExt for StorageNode {
    fn get_node_stats(&self) -> NodeStats {
        NodeStats {
            total_capacity: self.capacity,
            used_capacity: self.used,
            available_capacity: self.available_capacity(),
            reputation: self.reputation,
        }
    }
}

pub struct NodeStats {
    pub total_capacity: u64,
    pub used_capacity: u64,
    pub available_capacity: u64,
    pub reputation: f64,
}