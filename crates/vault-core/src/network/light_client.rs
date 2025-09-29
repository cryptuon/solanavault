//! # Light Client Architecture
//!
//! Lightweight client that provides SolanaVault network access without running
//! a full node. Handles payments, IPC communication, and seamless RPC proxy.

use crate::network::{
    transport::{NetworkMessage, TransportError},
    consensus::{ReputationEvidence, EvidenceType},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;
use uuid::Uuid;

/// Lightweight client for SolanaVault network access
#[derive(Debug)]
pub struct LightClient {
    /// Client configuration
    config: LightClientConfig,
    /// Payment wallet for network fees
    wallet: Arc<Mutex<ClientWallet>>,
    /// Known gateway nodes for network access
    gateways: Arc<RwLock<HashMap<String, GatewayNode>>>,
    /// Active sessions with gateway nodes
    sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
    /// Usage metrics for billing
    usage_metrics: Arc<RwLock<UsageMetrics>>,
    /// Request cache for efficiency
    cache: Arc<RwLock<RequestCache>>,
}

#[derive(Debug, Clone)]
pub struct LightClientConfig {
    /// Client identifier
    pub client_id: String,
    /// IPC socket path for local communication
    pub ipc_path: PathBuf,
    /// Maximum payment per request (in micro-tokens)
    pub max_payment_per_request: u64,
    /// Daily spending limit
    pub daily_spending_limit: u64,
    /// Cache settings
    pub cache_ttl: Duration,
    /// Gateway selection strategy
    pub gateway_strategy: GatewayStrategy,
}

#[derive(Debug, Clone)]
pub enum GatewayStrategy {
    /// Use cheapest available gateway
    Cheapest,
    /// Use fastest responding gateway
    Fastest,
    /// Balance cost and performance
    Balanced,
    /// Use specific preferred gateways
    Preferred(Vec<String>),
}

/// Client-side payment wallet
#[derive(Debug)]
pub struct ClientWallet {
    /// Available balance (in micro-tokens)
    balance: u64,
    /// Payment history
    payments: Vec<PaymentRecord>,
    /// Daily spending tracking
    daily_spent: u64,
    /// Last reset timestamp
    last_reset: u64,
}

#[derive(Debug, Clone)]
pub struct PaymentRecord {
    pub payment_id: String,
    pub gateway: String,
    pub amount: u64,
    pub service_type: ServiceType,
    pub timestamp: u64,
    pub confirmed: bool,
}

#[derive(Debug, Clone)]
pub enum ServiceType {
    BlockRetrieval,
    TransactionSubmission,
    AccountInfo,
    ProgramExecution,
    DataStorage,
}

/// Gateway node that provides network access
#[derive(Debug, Clone)]
pub struct GatewayNode {
    pub node_id: String,
    pub address: String,
    pub reputation: f64,
    pub pricing: GatewayPricing,
    pub capabilities: Vec<String>,
    pub response_time_ms: f64,
    pub uptime_percentage: f64,
    pub last_seen: u64,
}

#[derive(Debug, Clone)]
pub struct GatewayPricing {
    /// Base fee per request (micro-tokens)
    pub base_fee: u64,
    /// Fee per KB of data
    pub data_fee_per_kb: u64,
    /// Premium for fast response
    pub priority_multiplier: f64,
    /// Bulk discount tiers
    pub bulk_discounts: Vec<BulkDiscount>,
}

#[derive(Debug, Clone)]
pub struct BulkDiscount {
    pub min_requests: u32,
    pub discount_percentage: f64,
}

/// Active session with a gateway
#[derive(Debug, Clone)]
pub struct ClientSession {
    pub session_id: String,
    pub gateway_id: String,
    pub started_at: u64,
    pub requests_made: u32,
    pub total_paid: u64,
    pub average_response_time: f64,
    pub last_activity: u64,
}

/// Usage tracking for billing and optimization
#[derive(Debug, Default)]
pub struct UsageMetrics {
    pub total_requests: u64,
    pub total_spent: u64,
    pub average_cost_per_request: f64,
    pub cache_hit_rate: f64,
    pub preferred_gateways: HashMap<String, u32>,
    pub service_usage: HashMap<ServiceType, u64>,
}

/// Request cache for avoiding redundant network calls
#[derive(Debug)]
pub struct RequestCache {
    entries: HashMap<String, CacheEntry>,
    max_size: usize,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub timestamp: u64,
    pub ttl: Duration,
    pub cost_saved: u64,
}

/// Payment channel for microtransactions
#[derive(Debug)]
pub struct PaymentChannel {
    pub channel_id: String,
    pub gateway: String,
    pub client_deposit: u64,
    pub gateway_deposit: u64,
    pub balance: u64,
    pub sequence: u64,
    pub expires_at: u64,
}

impl LightClient {
    /// Create new light client
    pub async fn new(config: LightClientConfig) -> Result<Self, TransportError> {
        let wallet = ClientWallet::new();

        Ok(Self {
            config,
            wallet: Arc::new(Mutex::new(wallet)),
            gateways: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            usage_metrics: Arc::new(RwLock::new(UsageMetrics::default())),
            cache: Arc::new(RwLock::new(RequestCache::new(1000))), // 1000 entries
        })
    }

    /// Start the light client service
    pub async fn start(&self) -> Result<(), TransportError> {
        println!("🔮 Starting SolanaVault Light Client");
        println!("   Client ID: {}", self.config.client_id);
        println!("   IPC Path: {:?}", self.config.ipc_path);

        // Discover gateway nodes
        self.discover_gateways().await?;

        // Start background tasks
        self.start_background_tasks().await;

        // Start IPC server for local applications
        self.start_ipc_server().await?;

        println!("✅ Light client ready for connections");
        Ok(())
    }

    /// Make a request to the SolanaVault network
    pub async fn make_request(
        &self,
        method: &str,
        params: &serde_json::Value,
        priority: RequestPriority,
    ) -> Result<serde_json::Value, TransportError> {
        let request_id = Uuid::new_v4().to_string();

        // Check cache first
        if let Some(cached) = self.check_cache(method, params).await {
            println!("💰 Cache hit - saved payment");
            return Ok(cached);
        }

        // Select best gateway based on strategy
        let gateway = self.select_gateway(&priority).await?;

        // Calculate payment for this request
        let payment_amount = self.calculate_payment(&gateway, method, params).await;

        // Check if we can afford this request
        {
            let wallet = self.wallet.lock().await;
            if !wallet.can_afford(payment_amount) {
                return Err(TransportError::NetworkError(
                    "Insufficient balance for request".to_string()
                ));
            }
        }

        // Execute payment and request
        let result = self.execute_paid_request(
            &gateway,
            &request_id,
            method,
            params,
            payment_amount,
        ).await?;

        // Cache successful result
        self.cache_result(method, params, &result).await;

        // Update metrics
        self.update_usage_metrics(&gateway.node_id, payment_amount).await;

        Ok(result)
    }

    /// Add funds to the client wallet
    pub async fn add_funds(&self, amount: u64) -> Result<(), TransportError> {
        let mut wallet = self.wallet.lock().await;
        wallet.balance += amount;
        println!("💰 Added {} micro-tokens to wallet. Balance: {}", amount, wallet.balance);
        Ok(())
    }

    /// Get current wallet balance and usage stats
    pub async fn get_wallet_info(&self) -> WalletInfo {
        let wallet = self.wallet.lock().await;
        let metrics = self.usage_metrics.read().await;

        WalletInfo {
            balance: wallet.balance,
            daily_spent: wallet.daily_spent,
            daily_limit: self.config.daily_spending_limit,
            total_requests: metrics.total_requests,
            average_cost: metrics.average_cost_per_request,
            cache_savings: metrics.cache_hit_rate,
        }
    }

    async fn discover_gateways(&self) -> Result<(), TransportError> {
        println!("🔍 Discovering SolanaVault gateway nodes...");

        // In production, this would use DHT or bootstrap nodes
        // For now, use hardcoded gateways
        let gateway_addresses = vec![
            "tcp://gateway1.solanavault.com:4040",
            "tcp://gateway2.solanavault.com:4040",
            "tcp://gateway3.solanavault.com:4040",
        ];

        let mut gateways = self.gateways.write().await;

        for (i, address) in gateway_addresses.iter().enumerate() {
            let gateway = GatewayNode {
                node_id: format!("gateway-{}", i + 1),
                address: address.to_string(),
                reputation: 0.9 + (i as f64 * 0.03), // Varied reputation
                pricing: GatewayPricing {
                    base_fee: 100, // 100 micro-tokens base
                    data_fee_per_kb: 50,
                    priority_multiplier: 1.5,
                    bulk_discounts: vec![
                        BulkDiscount { min_requests: 10, discount_percentage: 0.05 },
                        BulkDiscount { min_requests: 100, discount_percentage: 0.15 },
                        BulkDiscount { min_requests: 1000, discount_percentage: 0.25 },
                    ],
                },
                capabilities: vec!["compression".to_string(), "fast_retrieval".to_string()],
                response_time_ms: 50.0 + (i as f64 * 15.0),
                uptime_percentage: 99.5 + (i as f64 * 0.1),
                last_seen: current_timestamp(),
            };

            gateways.insert(gateway.node_id.clone(), gateway);
        }

        println!("✅ Found {} gateway nodes", gateways.len());
        Ok(())
    }

    async fn select_gateway(&self, priority: &RequestPriority) -> Result<GatewayNode, TransportError> {
        let gateways = self.gateways.read().await;

        if gateways.is_empty() {
            return Err(TransportError::NetworkError("No gateway nodes available".to_string()));
        }

        let selected = match (&self.config.gateway_strategy, priority) {
            (GatewayStrategy::Cheapest, _) => {
                gateways.values()
                    .min_by(|a, b| a.pricing.base_fee.cmp(&b.pricing.base_fee))
            }
            (GatewayStrategy::Fastest, _) | (_, RequestPriority::High) => {
                gateways.values()
                    .min_by(|a, b| a.response_time_ms.partial_cmp(&b.response_time_ms).unwrap())
            }
            (GatewayStrategy::Balanced, _) => {
                // Score based on cost and performance
                gateways.values()
                    .min_by(|a, b| {
                        let score_a = (a.pricing.base_fee as f64) + (a.response_time_ms * 2.0);
                        let score_b = (b.pricing.base_fee as f64) + (b.response_time_ms * 2.0);
                        score_a.partial_cmp(&score_b).unwrap()
                    })
            }
            (GatewayStrategy::Preferred(preferred), _) => {
                preferred.iter()
                    .find_map(|id| gateways.get(id))
                    .or_else(|| gateways.values().next())
            }
        };

        selected.cloned()
            .ok_or_else(|| TransportError::NetworkError("No suitable gateway found".to_string()))
    }

    async fn calculate_payment(
        &self,
        gateway: &GatewayNode,
        method: &str,
        params: &serde_json::Value,
    ) -> u64 {
        let base_fee = gateway.pricing.base_fee;

        // Estimate data size for this request type
        let estimated_kb = match method {
            "getBlock" | "getConfirmedBlock" => 10, // ~10KB average block
            "getAccountInfo" => 1,
            "getTransaction" => 2,
            _ => 1,
        };

        let data_fee = gateway.pricing.data_fee_per_kb * estimated_kb;

        base_fee + data_fee
    }

    async fn execute_paid_request(
        &self,
        gateway: &GatewayNode,
        request_id: &str,
        method: &str,
        params: &serde_json::Value,
        payment_amount: u64,
    ) -> Result<serde_json::Value, TransportError> {
        // Record payment
        {
            let mut wallet = self.wallet.lock().await;
            wallet.make_payment(payment_amount, gateway.node_id.clone(), ServiceType::BlockRetrieval)?;
        }

        // In production, this would make actual network request to gateway
        // For demo, simulate response
        tokio::time::sleep(Duration::from_millis(gateway.response_time_ms as u64)).await;

        println!("💳 Paid {} micro-tokens to {} for {}", payment_amount, gateway.node_id, method);

        // Simulate successful response
        Ok(serde_json::json!({
            "result": format!("Response for {} from {}", method, gateway.node_id),
            "gatewayInfo": {
                "nodeId": gateway.node_id,
                "cost": payment_amount,
                "responseTime": format!("{}ms", gateway.response_time_ms)
            }
        }))
    }

    async fn check_cache(&self, method: &str, params: &serde_json::Value) -> Option<serde_json::Value> {
        let cache_key = format!("{}:{}", method, serde_json::to_string(params).unwrap_or_default());
        let cache = self.cache.read().await;

        if let Some(entry) = cache.entries.get(&cache_key) {
            if entry.timestamp + entry.ttl.as_secs() > current_timestamp() {
                if let Ok(value) = serde_json::from_slice(&entry.data) {
                    return Some(value);
                }
            }
        }

        None
    }

    async fn cache_result(&self, method: &str, params: &serde_json::Value, result: &serde_json::Value) {
        let cache_key = format!("{}:{}", method, serde_json::to_string(params).unwrap_or_default());
        let mut cache = self.cache.write().await;

        if let Ok(data) = serde_json::to_vec(result) {
            let entry = CacheEntry {
                data,
                timestamp: current_timestamp(),
                ttl: self.config.cache_ttl,
                cost_saved: 100, // Estimate saved cost
            };

            cache.entries.insert(cache_key, entry);

            // Evict oldest entries if cache is full
            if cache.entries.len() > cache.max_size {
                cache.evict_oldest();
            }
        }
    }

    async fn update_usage_metrics(&self, gateway_id: &str, amount: u64) {
        let mut metrics = self.usage_metrics.write().await;
        metrics.total_requests += 1;
        metrics.total_spent += amount;
        metrics.average_cost_per_request = metrics.total_spent as f64 / metrics.total_requests as f64;
        *metrics.preferred_gateways.entry(gateway_id.to_string()).or_insert(0) += 1;
    }

    async fn start_background_tasks(&self) {
        // Gateway health monitoring
        let gateways = self.gateways.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                // TODO: Ping gateways and update health metrics
                println!("🔍 Monitoring gateway health");
            }
        });

        // Cache cleanup
        let cache = self.cache.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                let mut cache_lock = cache.write().await;
                cache_lock.cleanup_expired();
            }
        });
    }

    async fn start_ipc_server(&self) -> Result<(), TransportError> {
        println!("🔌 Starting IPC server at {:?}", self.config.ipc_path);
        // TODO: Implement actual IPC server (Unix domain socket or named pipe)
        // This would listen for local application requests
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WalletInfo {
    pub balance: u64,
    pub daily_spent: u64,
    pub daily_limit: u64,
    pub total_requests: u64,
    pub average_cost: f64,
    pub cache_savings: f64,
}

impl ClientWallet {
    fn new() -> Self {
        Self {
            balance: 0,
            payments: Vec::new(),
            daily_spent: 0,
            last_reset: current_timestamp(),
        }
    }

    fn can_afford(&self, amount: u64) -> bool {
        self.balance >= amount && self.daily_spent + amount <= 1_000_000 // Daily limit
    }

    fn make_payment(
        &mut self,
        amount: u64,
        gateway: String,
        service_type: ServiceType,
    ) -> Result<(), TransportError> {
        if !self.can_afford(amount) {
            return Err(TransportError::NetworkError("Insufficient funds".to_string()));
        }

        self.balance -= amount;
        self.daily_spent += amount;

        let payment = PaymentRecord {
            payment_id: Uuid::new_v4().to_string(),
            gateway,
            amount,
            service_type,
            timestamp: current_timestamp(),
            confirmed: true,
        };

        self.payments.push(payment);
        Ok(())
    }
}

impl RequestCache {
    fn new(max_size: usize) -> Self {
        Self {
            entries: HashMap::new(),
            max_size,
        }
    }

    fn evict_oldest(&mut self) {
        if let Some((oldest_key, _)) = self.entries.iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.entries.remove(&oldest_key);
        }
    }

    fn cleanup_expired(&mut self) {
        let now = current_timestamp();
        self.entries.retain(|_, entry| {
            entry.timestamp + entry.ttl.as_secs() > now
        });
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}