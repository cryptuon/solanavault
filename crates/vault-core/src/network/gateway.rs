//! # Gateway Node Economics
//!
//! Gateway nodes provide network access to light clients and earn fees
//! for their services. This implements the economic incentive layer.

use crate::network::{
    decentralized::{DecentralizedVaultNode, NodeConfig},
    light_client::{ServiceType, PaymentRecord},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex};
use tokio::time::interval;

/// Gateway node that provides paid access to SolanaVault network
#[derive(Debug)]
pub struct GatewayNode {
    /// Underlying vault node
    vault_node: Arc<DecentralizedVaultNode>,
    /// Gateway-specific configuration
    gateway_config: GatewayConfig,
    /// Revenue tracking
    revenue_tracker: Arc<RwLock<RevenueTracker>>,
    /// Active client sessions
    client_sessions: Arc<RwLock<HashMap<String, ClientSession>>>,
    /// Pricing engine
    pricing_engine: Arc<PricingEngine>,
    /// Payment processor
    payment_processor: Arc<Mutex<PaymentProcessor>>,
}

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    /// Gateway identifier
    pub gateway_id: String,
    /// Public endpoint for clients
    pub client_endpoint: String,
    /// Base pricing configuration
    pub base_pricing: PricingConfig,
    /// Revenue sharing with network
    pub network_fee_percentage: f64,
    /// Payment settlement frequency
    pub settlement_frequency: Duration,
}

#[derive(Debug, Clone)]
pub struct PricingConfig {
    /// Base fee per request (micro-tokens)
    pub base_fee: u64,
    /// Fee per KB of data served
    pub data_fee_per_kb: u64,
    /// Premium multiplier for priority requests
    pub priority_multiplier: f64,
    /// Bulk discount schedule
    pub volume_discounts: Vec<VolumeDiscount>,
    /// Dynamic pricing parameters
    pub dynamic_pricing: DynamicPricingConfig,
}

#[derive(Debug, Clone)]
pub struct VolumeDiscount {
    pub min_monthly_volume: u64,
    pub discount_percentage: f64,
}

#[derive(Debug, Clone)]
pub struct DynamicPricingConfig {
    /// Enable dynamic pricing based on demand
    pub enabled: bool,
    /// Maximum price multiplier during high demand
    pub max_surge_multiplier: f64,
    /// Load threshold for surge pricing
    pub surge_threshold: f64,
    /// Price adjustment frequency
    pub adjustment_interval: Duration,
}

/// Tracks revenue and performance metrics
#[derive(Debug, Default)]
pub struct RevenueTracker {
    /// Total revenue earned (micro-tokens)
    pub total_revenue: u64,
    /// Revenue by service type
    pub revenue_by_service: HashMap<ServiceType, u64>,
    /// Revenue by time period
    pub daily_revenue: HashMap<String, u64>, // date -> revenue
    /// Client statistics
    pub client_stats: HashMap<String, ClientStats>,
    /// Performance metrics
    pub performance: PerformanceMetrics,
}

#[derive(Debug, Clone)]
pub struct ClientStats {
    pub total_requests: u64,
    pub total_paid: u64,
    pub average_request_size: f64,
    pub first_seen: u64,
    pub last_seen: u64,
    pub satisfaction_score: f64,
}

#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub requests_per_second: f64,
    pub average_response_time: f64,
    pub cache_hit_rate: f64,
    pub uptime_percentage: f64,
    pub error_rate: f64,
}

/// Active client session with payment tracking
#[derive(Debug, Clone)]
pub struct ClientSession {
    pub client_id: String,
    pub session_id: String,
    pub started_at: u64,
    pub payment_channel: Option<PaymentChannel>,
    pub requests_served: u32,
    pub total_paid: u64,
    pub last_activity: u64,
}

#[derive(Debug, Clone)]
pub struct PaymentChannel {
    pub channel_id: String,
    pub client_deposit: u64,
    pub gateway_stake: u64,
    pub current_balance: u64,
    pub sequence_number: u64,
    pub expires_at: u64,
}

/// Handles payment processing and settlement
#[derive(Debug)]
pub struct PaymentProcessor {
    /// Pending payments awaiting confirmation
    pending_payments: HashMap<String, PendingPayment>,
    /// Confirmed payments ready for settlement
    confirmed_payments: Vec<ConfirmedPayment>,
    /// Payment channels for microtransactions
    payment_channels: HashMap<String, PaymentChannel>,
    /// Settlement statistics
    settlement_stats: SettlementStats,
}

#[derive(Debug, Clone)]
pub struct PendingPayment {
    pub payment_id: String,
    pub client_id: String,
    pub amount: u64,
    pub service_type: ServiceType,
    pub timestamp: u64,
    pub confirmation_deadline: u64,
}

#[derive(Debug, Clone)]
pub struct ConfirmedPayment {
    pub payment_id: String,
    pub client_id: String,
    pub amount: u64,
    pub service_type: ServiceType,
    pub confirmed_at: u64,
    pub gateway_fee: u64,
    pub network_fee: u64,
}

#[derive(Debug, Default)]
pub struct SettlementStats {
    pub total_settled: u64,
    pub settlement_count: u32,
    pub average_settlement_time: f64,
    pub failed_settlements: u32,
}

/// Dynamic pricing engine that adjusts based on demand
#[derive(Debug)]
pub struct PricingEngine {
    config: PricingConfig,
    current_load: Arc<RwLock<f64>>,
    demand_history: Arc<RwLock<Vec<DemandDataPoint>>>,
}

#[derive(Debug, Clone)]
pub struct DemandDataPoint {
    pub timestamp: u64,
    pub requests_per_minute: f64,
    pub average_response_time: f64,
    pub queue_length: u32,
}

impl GatewayNode {
    /// Create new gateway node
    pub async fn new(
        vault_node: Arc<DecentralizedVaultNode>,
        gateway_config: GatewayConfig,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let pricing_engine = Arc::new(PricingEngine::new(gateway_config.base_pricing.clone()));
        let payment_processor = Arc::new(Mutex::new(PaymentProcessor::new()));

        Ok(Self {
            vault_node,
            gateway_config,
            revenue_tracker: Arc::new(RwLock::new(RevenueTracker::default())),
            client_sessions: Arc::new(RwLock::new(HashMap::new())),
            pricing_engine,
            payment_processor,
        })
    }

    /// Start the gateway service
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("💰 Starting SolanaVault Gateway Node");
        println!("   Gateway ID: {}", self.gateway_config.gateway_id);
        println!("   Client Endpoint: {}", self.gateway_config.client_endpoint);

        // Start the underlying vault node
        // self.vault_node.start().await?;

        // Start gateway-specific services
        self.start_pricing_engine().await;
        self.start_payment_processor().await;
        self.start_client_listener().await?;

        println!("✅ Gateway node ready to serve clients");
        Ok(())
    }

    /// Handle a paid request from a light client
    pub async fn handle_paid_request(
        &self,
        client_id: &str,
        request_id: &str,
        method: &str,
        params: &serde_json::Value,
        payment: PaymentRecord,
    ) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        println!("💳 Processing paid request from {}: {} ({}μ-tokens)", client_id, method, payment.amount);

        // Validate payment
        self.validate_payment(&payment).await?;

        // Process payment
        {
            let mut processor = self.payment_processor.lock().await;
            processor.process_payment(payment.clone())?;
        }

        // Calculate pricing for this request
        let actual_cost = self.pricing_engine.calculate_price(method, params).await;

        // Serve the request through the vault network
        let result = match method {
            "getBlock" | "getConfirmedBlock" => {
                let slot = params.as_array()
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);

                // Use the decentralized network to retrieve the block
                self.vault_node.get_block(slot).await
                    .map(|data| serde_json::json!({
                        "blockhash": format!("vault_block_{}", slot),
                        "slot": slot,
                        "compressed": true,
                        "gatewayInfo": {
                            "gatewayId": self.gateway_config.gateway_id,
                            "cost": actual_cost,
                            "networkNodes": 42
                        }
                    }))
                    .map_err(|e| -> Box<dyn std::error::Error> { format!("Block retrieval failed: {}", e).into() })
            }
            _ => {
                Ok(serde_json::json!({
                    "result": format!("Gateway response for {}", method),
                    "gatewayInfo": {
                        "gatewayId": self.gateway_config.gateway_id,
                        "cost": actual_cost
                    }
                }))
            }
        }?;

        // Update revenue tracking
        self.track_revenue(client_id, &payment).await;

        Ok(result)
    }

    /// Create payment channel for microtransactions
    pub async fn create_payment_channel(
        &self,
        client_id: &str,
        client_deposit: u64,
    ) -> Result<PaymentChannel, Box<dyn std::error::Error>> {
        let channel = PaymentChannel {
            channel_id: uuid::Uuid::new_v4().to_string(),
            client_deposit,
            gateway_stake: client_deposit / 10, // 10% stake from gateway
            current_balance: client_deposit,
            sequence_number: 0,
            expires_at: current_timestamp() + 86400, // 24 hours
        };

        {
            let mut processor = self.payment_processor.lock().await;
            processor.payment_channels.insert(channel.channel_id.clone(), channel.clone());
        }

        println!("💰 Created payment channel {} for client {}", channel.channel_id, client_id);
        Ok(channel)
    }

    /// Get current pricing for services
    pub async fn get_pricing_info(&self) -> PricingInfo {
        self.pricing_engine.get_current_pricing().await
    }

    /// Get gateway statistics and revenue info
    pub async fn get_gateway_stats(&self) -> GatewayStats {
        let revenue = self.revenue_tracker.read().await;
        let sessions = self.client_sessions.read().await;

        GatewayStats {
            gateway_id: self.gateway_config.gateway_id.clone(),
            total_revenue: revenue.total_revenue,
            active_clients: sessions.len() as u32,
            requests_served: revenue.client_stats.values()
                .map(|s| s.total_requests)
                .sum(),
            uptime_percentage: revenue.performance.uptime_percentage,
            average_response_time: revenue.performance.average_response_time,
        }
    }

    async fn validate_payment(&self, payment: &PaymentRecord) -> Result<(), Box<dyn std::error::Error>> {
        // Validate payment signature, amount, etc.
        if payment.amount == 0 {
            return Err("Invalid payment amount".into());
        }

        // In production, verify cryptographic signature
        if !payment.confirmed {
            return Err("Payment not confirmed".into());
        }

        Ok(())
    }

    async fn track_revenue(&self, client_id: &str, payment: &PaymentRecord) {
        let mut revenue = self.revenue_tracker.write().await;

        // Update total revenue
        revenue.total_revenue += payment.amount;

        // Update service-specific revenue
        *revenue.revenue_by_service.entry(payment.service_type.clone()).or_insert(0) += payment.amount;

        // Update daily revenue
        let today = format!("{}", current_timestamp() / 86400); // Day since epoch
        *revenue.daily_revenue.entry(today).or_insert(0) += payment.amount;

        // Update client stats
        let client_stat = revenue.client_stats.entry(client_id.to_string()).or_insert(ClientStats {
            total_requests: 0,
            total_paid: 0,
            average_request_size: 0.0,
            first_seen: current_timestamp(),
            last_seen: current_timestamp(),
            satisfaction_score: 1.0,
        });

        client_stat.total_requests += 1;
        client_stat.total_paid += payment.amount;
        client_stat.last_seen = current_timestamp();
        client_stat.average_request_size = client_stat.total_paid as f64 / client_stat.total_requests as f64;
    }

    async fn start_pricing_engine(&self) {
        let pricing_engine = self.pricing_engine.clone();
        tokio::spawn(async move {
            pricing_engine.start_dynamic_adjustment().await;
        });
    }

    async fn start_payment_processor(&self) {
        let processor = self.payment_processor.clone();
        let settlement_frequency = self.gateway_config.settlement_frequency;

        tokio::spawn(async move {
            let mut interval = interval(settlement_frequency);
            loop {
                interval.tick().await;
                let mut proc = processor.lock().await;
                if let Err(e) = proc.settle_payments().await {
                    eprintln!("❌ Payment settlement failed: {}", e);
                }
            }
        });
    }

    async fn start_client_listener(&self) -> Result<(), Box<dyn std::error::Error>> {
        let endpoint = self.gateway_config.client_endpoint.clone();
        println!("🔌 Starting client listener on {}", endpoint);

        // Parse the endpoint to extract host and port
        let addr = endpoint
            .trim_start_matches("tcp://")
            .trim_start_matches("http://")
            .trim_start_matches("https://")
            .to_string();

        // Start a basic TCP listener for client connections
        // In production, this would be replaced with a full Axum/WebSocket server
        let addr_clone = addr.clone();
        tokio::spawn(async move {
            match tokio::net::TcpListener::bind(&addr).await {
                Ok(listener) => {
                    println!("✅ Client listener started on {}", addr_clone);

                    loop {
                        match listener.accept().await {
                            Ok((socket, peer_addr)) => {
                                log::debug!("📥 New client connection from: {}", peer_addr);
                                // Handle client connection in background
                                tokio::spawn(async move {
                                    // Connection handling would go here
                                    // For now, just log and close
                                    drop(socket);
                                });
                            }
                            Err(e) => {
                                log::warn!("Failed to accept client connection: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to start client listener on {}: {}", addr, e);
                }
            }
        });

        Ok(())
    }
}

impl PricingEngine {
    fn new(config: PricingConfig) -> Self {
        Self {
            config,
            current_load: Arc::new(RwLock::new(0.0)),
            demand_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn calculate_price(&self, method: &str, _params: &serde_json::Value) -> u64 {
        let base_price = match method {
            "getBlock" | "getConfirmedBlock" => self.config.base_fee + (self.config.data_fee_per_kb * 10),
            "getAccountInfo" => self.config.base_fee + self.config.data_fee_per_kb,
            _ => self.config.base_fee,
        };

        // Apply dynamic pricing if enabled
        if self.config.dynamic_pricing.enabled {
            let load = *self.current_load.read().await;
            let surge_multiplier = if load > self.config.dynamic_pricing.surge_threshold {
                1.0 + (load - self.config.dynamic_pricing.surge_threshold) *
                      (self.config.dynamic_pricing.max_surge_multiplier - 1.0)
            } else {
                1.0
            };

            (base_price as f64 * surge_multiplier) as u64
        } else {
            base_price
        }
    }

    async fn get_current_pricing(&self) -> PricingInfo {
        let load = *self.current_load.read().await;
        let surge_active = load > self.config.dynamic_pricing.surge_threshold;

        PricingInfo {
            base_fee: self.config.base_fee,
            data_fee_per_kb: self.config.data_fee_per_kb,
            current_load: load,
            surge_pricing_active: surge_active,
            surge_multiplier: if surge_active {
                1.0 + (load - self.config.dynamic_pricing.surge_threshold) *
                      (self.config.dynamic_pricing.max_surge_multiplier - 1.0)
            } else {
                1.0
            },
        }
    }

    async fn start_dynamic_adjustment(&self) {
        if !self.config.dynamic_pricing.enabled {
            return;
        }

        let current_load = self.current_load.clone();
        let demand_history = self.demand_history.clone();
        let adjustment_interval = self.config.dynamic_pricing.adjustment_interval;

        tokio::spawn(async move {
            let mut interval_timer = interval(adjustment_interval);
            loop {
                interval_timer.tick().await;

                // Calculate new load based on demand history
                let mut history = demand_history.write().await;
                let now = current_timestamp();

                // Remove old demand entries (older than 5 minutes)
                history.retain(|dp| now.saturating_sub(dp.timestamp) < 300);

                // Calculate average load from recent data points
                let request_count = history.len();
                let total_rpm: f64 = history.iter().map(|dp| dp.requests_per_minute).sum();
                let load = if request_count > 0 {
                    (total_rpm / request_count as f64 / 100.0).min(1.0) // Normalize to 0-1 range
                } else {
                    0.0
                };

                // Update current load
                {
                    let mut current = current_load.write().await;
                    *current = load;
                }

                log::debug!("⚖️ Dynamic pricing: load={:.2}, data_points={}", load, request_count);
            }
        });
    }

    /// Record a request for demand tracking
    pub async fn record_request(&self) {
        let mut history = self.demand_history.write().await;
        history.push(DemandDataPoint {
            timestamp: current_timestamp(),
            requests_per_minute: 1.0, // Single request recorded
            average_response_time: 0.0,
            queue_length: 0,
        });
    }
}

impl PaymentProcessor {
    fn new() -> Self {
        Self {
            pending_payments: HashMap::new(),
            confirmed_payments: Vec::new(),
            payment_channels: HashMap::new(),
            settlement_stats: SettlementStats::default(),
        }
    }

    fn process_payment(&mut self, payment: PaymentRecord) -> Result<(), Box<dyn std::error::Error>> {
        if payment.confirmed {
            let gateway_fee = payment.amount * 95 / 100; // 95% to gateway
            let network_fee = payment.amount - gateway_fee; // 5% to network

            // Extract client_id from payment - use gateway as client identifier
            // or derive from payment_id if gateway is not meaningful
            let client_id = if payment.gateway.is_empty() {
                // Derive client ID from payment_id prefix
                payment.payment_id.split('-').next()
                    .unwrap_or("unknown")
                    .to_string()
            } else {
                payment.gateway.clone()
            };

            let confirmed = ConfirmedPayment {
                payment_id: payment.payment_id,
                client_id,
                amount: payment.amount,
                service_type: payment.service_type,
                confirmed_at: current_timestamp(),
                gateway_fee,
                network_fee,
            };

            self.confirmed_payments.push(confirmed);
            println!("✅ Payment confirmed: {} μ-tokens", payment.amount);
        } else {
            return Err("Payment not confirmed".into());
        }

        Ok(())
    }

    async fn settle_payments(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.confirmed_payments.is_empty() {
            return Ok(());
        }

        let total_to_settle: u64 = self.confirmed_payments.iter()
            .map(|p| p.gateway_fee)
            .sum();

        let network_fees: u64 = self.confirmed_payments.iter()
            .map(|p| p.network_fee)
            .sum();

        println!("💰 Settling {} payments: {}μ-tokens to gateway, {}μ-tokens to network",
                 self.confirmed_payments.len(), total_to_settle, network_fees);

        // Clear settled payments
        self.confirmed_payments.clear();

        // Update settlement stats
        self.settlement_stats.total_settled += total_to_settle;
        self.settlement_stats.settlement_count += 1;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingInfo {
    pub base_fee: u64,
    pub data_fee_per_kb: u64,
    pub current_load: f64,
    pub surge_pricing_active: bool,
    pub surge_multiplier: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GatewayStats {
    pub gateway_id: String,
    pub total_revenue: u64,
    pub active_clients: u32,
    pub requests_served: u64,
    pub uptime_percentage: f64,
    pub average_response_time: f64,
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}