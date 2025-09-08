//! # Vault RPC Proxy
//!
//! Drop-in replacement RPC proxy for Solana with historical data support.

use warp::Filter;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Deserialize, Serialize)]
struct RpcRequest {
    jsonrpc: String,
    id: serde_json::Value,
    method: String,
    params: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RpcResponse {
    jsonrpc: String,
    id: serde_json::Value,
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RpcError {
    code: i32,
    message: String,
}

#[tokio::main]
async fn main() {
    println!("SolanaVault RPC Proxy - Starting up...");
    
    // Create a simple route for handling RPC requests
    let rpc_route = warp::post()
        .and(warp::path("rpc"))
        .and(warp::body::json())
        .map(|req: RpcRequest| handle_rpc_request(req))
        .map(|reply| warp::reply::json(&reply));
    
    // Root route for health check
    let health_route = warp::get()
        .and(warp::path::end())
        .map(|| "SolanaVault RPC Proxy is running!");
    
    let routes = health_route.or(rpc_route);
    
    println!("Listening on http://127.0.0.1:3030");
    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}

fn handle_rpc_request(req: RpcRequest) -> RpcResponse {
    println!("Received RPC request: {} {:?}", req.method, req.params);
    
    let result = match req.method.as_str() {
        "getConfirmedBlock" => {
            // Extract slot number from params
            let slot = extract_slot_from_params(&req.params);
            handle_get_confirmed_block(slot)
        },
        _ => {
            // For other methods, return a mock response
            Some(serde_json::json!({
                "message": format!("Method {} not implemented in demo", req.method)
            }))
        }
    };
    
    RpcResponse {
        jsonrpc: req.jsonrpc,
        id: req.id,
        result,
        error: None,
    }
}

fn extract_slot_from_params(params: &Option<Value>) -> Option<u64> {
    params.as_ref().and_then(|p| {
        p.as_array().and_then(|arr| {
            arr.first().and_then(|v| v.as_u64())
        })
    })
}

fn handle_get_confirmed_block(slot: Option<u64>) -> Option<Value> {
    let slot = slot.unwrap_or(0);
    
    // In a real implementation, this would:
    // 1. Check if it's a recent block (route to standard RPC)
    // 2. If historical, route to VaultNetwork
    // 3. Retrieve and decompress the block data
    
    // For demo purposes, we'll simulate different response times
    if slot > 245000000 {
        // Historical block - simulate VaultNetwork retrieval
        println!("Retrieving historical block {} from VaultNetwork", slot);
        Some(serde_json::json!({
            "blockhash": format!("blockhash_{}", slot),
            "parentSlot": slot - 1,
            "transactions": [],
            "rewards": [],
            "blockTime": 1234567890,
            "retrievalTimeMs": 156  // Simulate fast retrieval
        }))
    } else {
        // Recent block - would route to standard RPC in real implementation
        Some(serde_json::json!({
            "blockhash": format!("blockhash_{}", slot),
            "parentSlot": slot - 1,
            "transactions": [],
            "rewards": [],
            "blockTime": 1234567890,
            "retrievalTimeMs": 45  // Simulate faster retrieval for recent blocks
        }))
    }
}