//! # SolanaVault Economics Demo
//!
//! Demonstrates how the economic model works for light clients and gateway nodes.

use vault_core::network::{
    LightClient, LightClientConfig, RequestPriority, GatewayStrategy,
    GatewayNode, GatewayConfig,
    DecentralizedVaultNode, NodeConfig,
};
use std::path::PathBuf;
use std::time::Duration;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("💰 SolanaVault Economics Demo");
    println!("=============================");
    println!();

    // Create a decentralized vault node (the underlying storage network)
    let vault_config = NodeConfig {
        node_id: "vault-demo-node".to_string(),
        address: "127.0.0.1:4040".parse().unwrap(),
        bootstrap_nodes: vec![],
        storage_capacity: 10 * 1024 * 1024 * 1024, // 10GB
        compression_enabled: true,
        consensus_participation: true,
    };

    println!("🏗️  Creating decentralized vault node...");
    let vault_node = Arc::new(DecentralizedVaultNode::new(vault_config).await?);

    // Start the vault node
    // vault_node.start().await?;
    println!("✅ Vault node initialized");
    println!();

    // Create a gateway node (provides paid access)
    let gateway_config = GatewayConfig {
        gateway_id: "demo-gateway".to_string(),
        client_endpoint: "tcp://0.0.0.0:5050".to_string(),
        base_pricing: vault_core::network::gateway::PricingConfig {
            base_fee: 100, // 100 micro-tokens base fee
            data_fee_per_kb: 50, // 50 micro-tokens per KB
            priority_multiplier: 1.5,
            volume_discounts: vec![
                vault_core::network::gateway::VolumeDiscount {
                    min_monthly_volume: 1000,
                    discount_percentage: 0.1, // 10% discount
                },
                vault_core::network::gateway::VolumeDiscount {
                    min_monthly_volume: 10000,
                    discount_percentage: 0.2, // 20% discount
                },
            ],
            dynamic_pricing: vault_core::network::gateway::DynamicPricingConfig {
                enabled: true,
                max_surge_multiplier: 2.0,
                surge_threshold: 0.8,
                adjustment_interval: Duration::from_secs(300),
            },
        },
        network_fee_percentage: 0.05, // 5% goes to network
        settlement_frequency: Duration::from_secs(60),
    };

    println!("🚪 Creating gateway node...");
    let gateway = GatewayNode::new(vault_node.clone(), gateway_config).await?;

    // Start gateway
    // gateway.start().await?;
    println!("✅ Gateway node initialized");
    println!();

    // Create a light client (for apps that don't run full nodes)
    let client_config = LightClientConfig {
        client_id: "demo-app".to_string(),
        ipc_path: PathBuf::from("/tmp/solanavault-demo.sock"),
        max_payment_per_request: 1000, // 1000 micro-tokens max per request
        daily_spending_limit: 100_000, // 100,000 micro-tokens per day
        cache_ttl: Duration::from_secs(300), // 5 minute cache
        gateway_strategy: GatewayStrategy::Balanced,
    };

    println!("📱 Creating light client...");
    let light_client = LightClient::new(client_config).await?;

    // Add funds to the client wallet
    println!("💰 Adding funds to client wallet...");
    light_client.add_funds(50_000).await?; // 50,000 micro-tokens
    println!();

    // Demonstrate the economic flow
    demonstrate_economic_flow(&light_client, &gateway).await?;

    Ok(())
}

async fn demonstrate_economic_flow(
    light_client: &LightClient,
    gateway: &GatewayNode,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Economic Flow Demonstration");
    println!("==============================");
    println!();

    // Show initial wallet state
    let wallet_info = light_client.get_wallet_info().await;
    println!("💳 Initial Wallet State:");
    println!("   Balance: {} micro-tokens", wallet_info.balance);
    println!("   Daily Limit: {} micro-tokens", wallet_info.daily_limit);
    println!();

    // Show gateway pricing
    let pricing = gateway.get_pricing_info().await;
    println!("💰 Gateway Pricing:");
    println!("   Base Fee: {} micro-tokens", pricing.base_fee);
    println!("   Data Fee: {} micro-tokens/KB", pricing.data_fee_per_kb);
    println!("   Current Load: {:.1}%", pricing.current_load * 100.0);

    if pricing.surge_pricing_active {
        println!("   🔥 Surge Pricing Active: {:.2}x multiplier", pricing.surge_multiplier);
    }
    println!();

    // Simulate different types of requests
    println!("📊 Request Cost Simulation:");

    let request_scenarios = vec![
        ("getBlock", serde_json::json!([245000000]), RequestPriority::Normal, "Retrieve historical block"),
        ("getAccountInfo", serde_json::json!(["TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"]), RequestPriority::Normal, "Get account info"),
        ("getTransaction", serde_json::json!(["signature123"]), RequestPriority::High, "Priority transaction lookup"),
    ];

    for (method, params, priority, description) in request_scenarios {
        println!("  {} ({})", description, match priority {
            RequestPriority::High => "Priority",
            RequestPriority::Normal => "Normal",
            RequestPriority::Low => "Economy",
        });

        // Calculate estimated cost
        let estimated_cost = estimate_request_cost(method, &params, &priority, &pricing);
        println!("    Estimated Cost: {} micro-tokens", estimated_cost);

        // Make the request
        match light_client.make_request(method, &params, priority).await {
            Ok(response) => {
                println!("    ✅ Request successful");
                if let Some(gateway_info) = response.get("gatewayInfo") {
                    if let Some(actual_cost) = gateway_info.get("cost").and_then(|c| c.as_u64()) {
                        println!("    💳 Actual Cost: {} micro-tokens", actual_cost);
                    }
                }
            }
            Err(e) => {
                println!("    ❌ Request failed: {}", e);
            }
        }
        println!();
    }

    // Show updated wallet state
    let final_wallet_info = light_client.get_wallet_info().await;
    println!("💳 Final Wallet State:");
    println!("   Balance: {} micro-tokens", final_wallet_info.balance);
    println!("   Spent Today: {} micro-tokens", final_wallet_info.daily_spent);
    println!("   Total Requests: {}", final_wallet_info.total_requests);

    if final_wallet_info.total_requests > 0 {
        println!("   Average Cost: {:.1} micro-tokens/request", final_wallet_info.average_cost);
    }

    if final_wallet_info.cache_savings > 0.0 {
        println!("   💾 Cache Savings: {:.1}%", final_wallet_info.cache_savings * 100.0);
    }
    println!();

    // Show gateway revenue
    let gateway_stats = gateway.get_gateway_stats().await;
    println!("🏪 Gateway Revenue:");
    println!("   Total Revenue: {} micro-tokens", gateway_stats.total_revenue);
    println!("   Requests Served: {}", gateway_stats.requests_served);
    println!("   Active Clients: {}", gateway_stats.active_clients);

    if gateway_stats.requests_served > 0 {
        println!("   Average Revenue/Request: {:.1} micro-tokens",
                 gateway_stats.total_revenue as f64 / gateway_stats.requests_served as f64);
    }
    println!();

    // Explain the economic model
    explain_economic_model();

    Ok(())
}

fn estimate_request_cost(
    method: &str,
    _params: &serde_json::Value,
    priority: &RequestPriority,
    pricing: &vault_core::network::gateway::PricingInfo,
) -> u64 {
    let base_cost = match method {
        "getBlock" | "getConfirmedBlock" => pricing.base_fee + (pricing.data_fee_per_kb * 10), // ~10KB block
        "getAccountInfo" => pricing.base_fee + pricing.data_fee_per_kb, // ~1KB account
        "getTransaction" => pricing.base_fee + (pricing.data_fee_per_kb * 2), // ~2KB transaction
        _ => pricing.base_fee,
    };

    let priority_multiplier = match priority {
        RequestPriority::High => 1.5,
        RequestPriority::Normal => 1.0,
        RequestPriority::Low => 0.8,
    };

    let surge_cost = if pricing.surge_pricing_active {
        (base_cost as f64 * pricing.surge_multiplier) as u64
    } else {
        base_cost
    };

    (surge_cost as f64 * priority_multiplier) as u64
}

fn explain_economic_model() {
    println!("🧠 Economic Model Explanation:");
    println!("=============================");
    println!();

    println!("💡 How It Works:");
    println!("   1. Light Clients pay micro-transactions for network access");
    println!("   2. Gateway Nodes earn revenue by serving requests");
    println!("   3. Full Vault Nodes earn from consensus participation");
    println!("   4. Network fees fund infrastructure and development");
    println!();

    println!("🎯 Key Benefits:");
    println!("   • Pay-per-use model - only pay for what you need");
    println!("   • Automatic caching reduces costs for repeated requests");
    println!("   • Competition between gateways keeps prices fair");
    println!("   • Volume discounts reward high-usage applications");
    println!("   • Dynamic pricing balances supply and demand");
    println!();

    println!("📊 Revenue Distribution:");
    println!("   • 95% to Gateway Node operators");
    println!("   • 5% to SolanaVault network fund");
    println!("   • Consensus rewards for vault node operators");
    println!("   • Reputation bonuses for reliable service");
    println!();

    println!("🔧 For Developers:");
    println!("   • Drop-in replacement for Solana RPC");
    println!("   • Transparent pricing with cost prediction");
    println!("   • Built-in caching and optimization");
    println!("   • Multiple payment options (prepaid, channels, bulk)");
    println!("   • Real-time usage monitoring and alerts");
    println!();
}