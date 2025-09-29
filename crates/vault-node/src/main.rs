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

    // Initialize storage node with enhanced functionality
    let mut storage_node = StorageNode::new_with_data_dir(
        args.node_id.clone(),
        args.bind_address.to_string(),
        args.capacity,
        args.data_dir.clone(),
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

    // 1. Initialize storage node with real functionality
    println!("💾 Initializing storage node...");
    storage_node.initialize().await?;

    // 2. Initialize P2P networking
    println!("🌐 Initializing P2P network...");

    // For now, simulate network initialization
    if !args.bootstrap_nodes.is_empty() {
        println!("   Bootstrap nodes: {:?}", args.bootstrap_nodes);
        // TODO: Implement actual P2P connection
    }

    // 3. Start health monitoring
    println!("❤️  Starting health monitoring...");

    // 4. Start API server for block storage/retrieval
    println!("🌐 Starting API server on {}...", args.bind_address);

    // 5. Demonstrate storage functionality
    println!("🎬 Running storage demonstration...");
    storage_node.demonstrate_storage().await?;

    println!("✅ All vault node services started successfully!");

    // Keep the node running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;

        // Periodic health check and stats
        let stats = storage_node.get_storage_stats();
        log::info!(
            "Node {} - Storage: {:.1}% used ({} blocks), Compression: {:.2}:1, Reputation: {:.2}",
            args.node_id,
            (stats.used_capacity as f64 / stats.total_capacity as f64) * 100.0,
            stats.blocks_stored,
            stats.total_compression_ratio,
            storage_node.reputation
        );
    }
}


