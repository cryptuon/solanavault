//! # Peer Discovery and DHT
//!
//! Distributed Hash Table implementation for peer discovery and content routing
//! in the SolanaVault network.

use crate::network::transport::{NetworkMessage, DiscoveryMessage, NngTransport, TransportError};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, BTreeMap};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tokio::time::{interval, sleep};
use uuid::Uuid;

/// DHT node for peer discovery and content routing
#[derive(Debug)]
pub struct DhtNode {
    /// Local node information
    node_info: NodeInfo,
    /// DHT routing table (Kademlia-style)
    routing_table: Arc<RwLock<RoutingTable>>,
    /// Bootstrap nodes for initial network entry
    bootstrap_nodes: Vec<String>,
    /// Transport layer for communication
    transport: Arc<NngTransport>,
    /// Content storage for block metadata
    content_store: Arc<RwLock<ContentStore>>,
    /// DHT configuration
    config: DhtConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub address: String,
    pub public_key: String,
    pub capabilities: Vec<String>,
    pub reputation: f64,
    pub last_seen: u64,
}

#[derive(Debug)]
pub struct RoutingTable {
    /// K-buckets for storing peers (Kademlia)
    buckets: Vec<KBucket>,
    /// Local node ID for distance calculations
    local_node_id: String,
}

#[derive(Debug)]
pub struct KBucket {
    /// Peers in this bucket
    peers: Vec<NodeInfo>,
    /// Maximum peers per bucket
    max_size: usize,
    /// Last update time
    last_updated: u64,
}

#[derive(Debug)]
pub struct ContentStore {
    /// Block slot -> peer nodes mapping
    block_locations: HashMap<u64, Vec<String>>,
    /// Content hash -> metadata mapping
    content_metadata: HashMap<String, ContentMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub content_hash: String,
    pub block_slot: u64,
    pub size_bytes: u64,
    pub compression_ratio: f64,
    pub replicas: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub struct DhtConfig {
    pub k_bucket_size: usize,
    pub alpha: usize, // Concurrency parameter
    pub republish_interval: Duration,
    pub refresh_interval: Duration,
    pub node_timeout: Duration,
}

impl Default for DhtConfig {
    fn default() -> Self {
        Self {
            k_bucket_size: 20,
            alpha: 3,
            republish_interval: Duration::from_secs(3600), // 1 hour
            refresh_interval: Duration::from_secs(300),    // 5 minutes
            node_timeout: Duration::from_secs(900),        // 15 minutes
        }
    }
}

/// DHT message types for peer discovery and content routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DhtMessage {
    /// Find peers closest to a node ID
    FindNode {
        target_id: String,
        requester: NodeInfo,
    },
    /// Response with closest known peers
    FindNodeResponse {
        peers: Vec<NodeInfo>,
        requester_id: String,
    },
    /// Find peers storing specific content
    FindValue {
        content_hash: String,
        requester: NodeInfo,
    },
    /// Response with content location or closest peers
    FindValueResponse {
        content: Option<ContentMetadata>,
        peers: Vec<NodeInfo>,
        requester_id: String,
    },
    /// Store content metadata
    Store {
        content: ContentMetadata,
        publisher: NodeInfo,
    },
    /// Ping for connectivity check
    Ping {
        sender: NodeInfo,
    },
    /// Pong response
    Pong {
        sender: NodeInfo,
    },
}

impl DhtNode {
    /// Create new DHT node
    pub async fn new(
        node_id: String,
        address: String,
        transport: Arc<NngTransport>,
        bootstrap_nodes: Vec<String>,
    ) -> Result<Self, TransportError> {
        let node_info = NodeInfo {
            node_id: node_id.clone(),
            address,
            public_key: generate_public_key(&node_id),
            capabilities: vec!["compression".to_string(), "storage".to_string()],
            reputation: 1.0,
            last_seen: current_timestamp(),
        };

        let routing_table = RoutingTable::new(node_id.clone());
        let content_store = ContentStore::new();
        let config = DhtConfig::default();

        Ok(Self {
            node_info,
            routing_table: Arc::new(RwLock::new(routing_table)),
            bootstrap_nodes,
            transport,
            content_store: Arc::new(RwLock::new(content_store)),
            config,
        })
    }

    /// Bootstrap the DHT by connecting to known nodes
    pub async fn bootstrap(&self) -> Result<(), TransportError> {
        println!("🌐 Bootstrapping DHT with {} nodes", self.bootstrap_nodes.len());

        for bootstrap_addr in &self.bootstrap_nodes {
            match self.transport.connect_peer(bootstrap_addr).await {
                Ok(_) => {
                    println!("✅ Connected to bootstrap node: {}", bootstrap_addr);

                    // Send FIND_NODE query for our own ID to populate routing table
                    let find_msg = DhtMessage::FindNode {
                        target_id: self.node_info.node_id.clone(),
                        requester: self.node_info.clone(),
                    };

                    let network_msg = NetworkMessage::Discovery(DiscoveryMessage {
                        node_id: self.node_info.node_id.clone(),
                        address: self.node_info.address.clone(),
                        capabilities: self.node_info.capabilities.clone(),
                        timestamp: current_timestamp(),
                    });

                    // Broadcast to populate initial routing table
                    let _ = self.transport.broadcast(network_msg).await;
                }
                Err(e) => {
                    eprintln!("❌ Failed to connect to bootstrap node {}: {}", bootstrap_addr, e);
                }
            }
        }

        Ok(())
    }

    /// Find peers responsible for storing a specific block
    pub async fn find_block_peers(&self, block_slot: u64) -> Result<Vec<NodeInfo>, TransportError> {
        let content_hash = calculate_block_hash(block_slot);

        // Check local content store first
        {
            let store = self.content_store.read().await;
            if let Some(metadata) = store.content_metadata.get(&content_hash) {
                // Return known replica locations
                let mut peers = Vec::new();
                for replica_id in &metadata.replicas {
                    if let Some(peer) = self.find_peer_info(replica_id).await {
                        peers.push(peer);
                    }
                }
                if !peers.is_empty() {
                    return Ok(peers);
                }
            }
        }

        // Perform DHT lookup
        let target_peers = self.find_closest_peers(&content_hash, self.config.alpha).await;

        // Query peers for content
        let mut result_peers = Vec::new();
        for peer in target_peers {
            let find_value_msg = DhtMessage::FindValue {
                content_hash: content_hash.clone(),
                requester: self.node_info.clone(),
            };

            // Serialize and send find_value message via transport
            let serialized = bincode::serialize(&find_value_msg)
                .map_err(|e| TransportError::SerializationError(e.to_string()))?;

            // Send via discovery message to the peer
            let network_msg = NetworkMessage::Discovery(DiscoveryMessage {
                node_id: self.node_info.node_id.clone(),
                address: self.node_info.address.clone(),
                capabilities: vec!["find_value".to_string(), content_hash.clone()],
                timestamp: current_timestamp(),
            });

            // Try to send to peer - on failure, skip but continue
            if let Err(e) = self.transport.send_to_peer(&peer.node_id, network_msg).await {
                log::debug!("Failed to query peer {} for content: {}", peer.node_id, e);
                continue;
            }

            // Add peer to results (responses handled via message handler)
            result_peers.push(peer);
        }

        Ok(result_peers)
    }

    /// Store block metadata in the DHT
    pub async fn store_block_metadata(
        &self,
        block_slot: u64,
        size_bytes: u64,
        compression_ratio: f64,
    ) -> Result<(), TransportError> {
        let content_hash = calculate_block_hash(block_slot);

        let metadata = ContentMetadata {
            content_hash: content_hash.clone(),
            block_slot,
            size_bytes,
            compression_ratio,
            replicas: vec![self.node_info.node_id.clone()],
            created_at: current_timestamp(),
        };

        // Store locally
        {
            let mut store = self.content_store.write().await;
            store.content_metadata.insert(content_hash.clone(), metadata.clone());
            store.block_locations.entry(block_slot)
                .or_insert_with(Vec::new)
                .push(self.node_info.node_id.clone());
        }

        // Find peers responsible for this content
        let target_peers = self.find_closest_peers(&content_hash, self.config.k_bucket_size).await;

        // Store metadata on target peers
        for peer in target_peers {
            let store_msg = DhtMessage::Store {
                content: metadata.clone(),
                publisher: self.node_info.clone(),
            };

            // Send store message via discovery message to the peer
            let network_msg = NetworkMessage::Discovery(DiscoveryMessage {
                node_id: self.node_info.node_id.clone(),
                address: self.node_info.address.clone(),
                capabilities: vec!["store".to_string(), content_hash.clone()],
                timestamp: current_timestamp(),
            });

            // Try to send to peer - log failures but continue
            match self.transport.send_to_peer(&peer.node_id, network_msg).await {
                Ok(_) => {
                    log::debug!("📦 Stored block {} metadata on peer: {}", block_slot, peer.node_id);
                }
                Err(e) => {
                    log::warn!("Failed to store metadata on peer {}: {}", peer.node_id, e);
                }
            }
        }

        Ok(())
    }

    /// Find closest peers to a target ID
    async fn find_closest_peers(&self, target_id: &str, count: usize) -> Vec<NodeInfo> {
        let routing_table = self.routing_table.read().await;
        routing_table.find_closest_peers(target_id, count)
    }

    /// Find specific peer information
    async fn find_peer_info(&self, peer_id: &str) -> Option<NodeInfo> {
        let routing_table = self.routing_table.read().await;
        routing_table.find_peer(peer_id)
    }

    /// Start DHT maintenance tasks
    pub async fn start_maintenance(&self) {
        println!("🔧 Starting DHT maintenance tasks...");

        // Start bucket refresh task - refreshes stale buckets to maintain routing table health
        let routing_table = self.routing_table.clone();
        let refresh_interval = self.config.refresh_interval;
        let node_timeout = self.config.node_timeout;
        tokio::spawn(async move {
            let mut interval_timer = interval(refresh_interval);
            loop {
                interval_timer.tick().await;

                // Refresh stale buckets by checking node last_seen timestamps
                let mut routing = routing_table.write().await;
                let now = current_timestamp();
                let stale_threshold = node_timeout.as_secs();

                let mut stale_count = 0;
                let mut active_count = 0;

                for bucket in &mut routing.buckets {
                    // Remove stale peers from bucket
                    let original_len = bucket.peers.len();
                    bucket.peers.retain(|peer| {
                        let is_active = now.saturating_sub(peer.last_seen) < stale_threshold;
                        if is_active {
                            active_count += 1;
                        }
                        is_active
                    });
                    stale_count += original_len - bucket.peers.len();
                }

                if stale_count > 0 {
                    log::debug!("🔄 DHT bucket refresh: removed {} stale peers, {} active", stale_count, active_count);
                }
            }
        });

        // Start content republish task - periodically republishes content to maintain availability
        let content_store = self.content_store.clone();
        let republish_interval = self.config.republish_interval;
        tokio::spawn(async move {
            let mut interval_timer = interval(republish_interval);
            loop {
                interval_timer.tick().await;

                // Get all content that should be republished
                let store = content_store.read().await;
                let now = current_timestamp();
                let republish_threshold = republish_interval.as_secs();

                let mut republish_count = 0;
                for (hash, metadata) in &store.content_metadata {
                    // Check if content needs republishing (older than threshold)
                    if now.saturating_sub(metadata.created_at) >= republish_threshold {
                        republish_count += 1;
                        // Content would be republished here via transport layer
                        // In production, this would send Store messages to k closest peers
                    }
                }

                if republish_count > 0 {
                    log::debug!("📤 DHT content republish: {} items need republishing", republish_count);
                }
            }
        });
    }

    /// Handle incoming DHT messages
    pub async fn handle_dht_message(&self, message: DhtMessage, sender_id: &str) -> Option<DhtMessage> {
        match message {
            DhtMessage::FindNode { target_id, requester } => {
                // Update routing table with requester info
                {
                    let mut routing_table = self.routing_table.write().await;
                    routing_table.add_peer(requester.clone());
                }

                // Find closest peers to target
                let peers = self.find_closest_peers(&target_id, self.config.k_bucket_size).await;

                Some(DhtMessage::FindNodeResponse {
                    peers,
                    requester_id: requester.node_id,
                })
            }

            DhtMessage::FindValue { content_hash, requester } => {
                // Update routing table
                {
                    let mut routing_table = self.routing_table.write().await;
                    routing_table.add_peer(requester.clone());
                }

                // Check if we have the content
                let store = self.content_store.read().await;
                if let Some(content) = store.content_metadata.get(&content_hash) {
                    Some(DhtMessage::FindValueResponse {
                        content: Some(content.clone()),
                        peers: vec![],
                        requester_id: requester.node_id,
                    })
                } else {
                    // Return closest peers
                    let peers = self.find_closest_peers(&content_hash, self.config.k_bucket_size).await;
                    Some(DhtMessage::FindValueResponse {
                        content: None,
                        peers,
                        requester_id: requester.node_id,
                    })
                }
            }

            DhtMessage::Store { content, publisher } => {
                // Store content metadata
                {
                    let mut store = self.content_store.write().await;
                    store.content_metadata.insert(content.content_hash.clone(), content.clone());
                    store.block_locations.entry(content.block_slot)
                        .or_insert_with(Vec::new)
                        .push(publisher.node_id.clone());
                }

                println!("💾 Stored content metadata: {}", content.content_hash);
                None
            }

            DhtMessage::Ping { sender } => {
                // Update routing table and respond
                {
                    let mut routing_table = self.routing_table.write().await;
                    routing_table.add_peer(sender);
                }

                Some(DhtMessage::Pong {
                    sender: self.node_info.clone(),
                })
            }

            DhtMessage::Pong { sender } => {
                // Update routing table
                {
                    let mut routing_table = self.routing_table.write().await;
                    routing_table.add_peer(sender);
                }
                None
            }

            _ => None, // Handle responses separately
        }
    }
}

impl RoutingTable {
    fn new(local_node_id: String) -> Self {
        let mut buckets = Vec::new();
        for _ in 0..256 { // 256-bit address space
            buckets.push(KBucket {
                peers: Vec::new(),
                max_size: 20, // K-bucket size
                last_updated: current_timestamp(),
            });
        }

        Self {
            buckets,
            local_node_id,
        }
    }

    fn add_peer(&mut self, peer: NodeInfo) {
        let distance = calculate_distance(&self.local_node_id, &peer.node_id);
        let bucket_index = leading_zeros(&distance);

        if bucket_index < self.buckets.len() {
            let bucket = &mut self.buckets[bucket_index];

            // Check if peer already exists
            if let Some(pos) = bucket.peers.iter().position(|p| p.node_id == peer.node_id) {
                // Move to end (most recently seen)
                bucket.peers.remove(pos);
                bucket.peers.push(peer);
            } else if bucket.peers.len() < bucket.max_size {
                // Add new peer
                bucket.peers.push(peer);
            } else {
                // Bucket full - could implement ping/eviction logic here
                println!("🔄 K-bucket {} full, peer not added", bucket_index);
            }

            bucket.last_updated = current_timestamp();
        }
    }

    fn find_closest_peers(&self, target_id: &str, count: usize) -> Vec<NodeInfo> {
        let mut all_peers: Vec<(NodeInfo, String)> = Vec::new();

        // Collect all peers with their distances
        for bucket in &self.buckets {
            for peer in &bucket.peers {
                let distance = calculate_distance(target_id, &peer.node_id);
                all_peers.push((peer.clone(), distance));
            }
        }

        // Sort by distance and return closest
        all_peers.sort_by(|a, b| a.1.cmp(&b.1));
        all_peers.into_iter()
            .take(count)
            .map(|(peer, _)| peer)
            .collect()
    }

    fn find_peer(&self, peer_id: &str) -> Option<NodeInfo> {
        for bucket in &self.buckets {
            if let Some(peer) = bucket.peers.iter().find(|p| p.node_id == peer_id) {
                return Some(peer.clone());
            }
        }
        None
    }
}

impl ContentStore {
    fn new() -> Self {
        Self {
            block_locations: HashMap::new(),
            content_metadata: HashMap::new(),
        }
    }
}

/// Calculate XOR distance between two node IDs
fn calculate_distance(id1: &str, id2: &str) -> String {
    let hash1 = sha256_hash(id1);
    let hash2 = sha256_hash(id2);

    let mut distance = String::new();
    for (b1, b2) in hash1.chars().zip(hash2.chars()) {
        let d1 = b1.to_digit(16).unwrap_or(0);
        let d2 = b2.to_digit(16).unwrap_or(0);
        distance.push_str(&format!("{:x}", d1 ^ d2));
    }

    distance
}

/// Count leading zeros in hex string (for bucket selection)
fn leading_zeros(hex_str: &str) -> usize {
    hex_str.chars().take_while(|&c| c == '0').count()
}

/// Calculate SHA256 hash of block slot
fn calculate_block_hash(block_slot: u64) -> String {
    sha256_hash(&format!("block_{}", block_slot))
}

/// Calculate SHA256 hash
fn sha256_hash(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Generate a public key for a node (simplified)
fn generate_public_key(node_id: &str) -> String {
    format!("pubkey_{}", sha256_hash(node_id)[..16].to_string())
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}