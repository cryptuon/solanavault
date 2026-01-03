//! # P2P Networking
//!
//! Peer-to-peer networking implementation for SolanaVault distributed network.
//! Integrates with the NNG transport layer for actual network communication.

use crate::network::transport::{NngTransport, NetworkMessage, TransportError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;

/// P2P network manager for node discovery and communication
#[derive(Debug)]
pub struct P2PNetwork {
    /// Local node ID
    node_id: String,
    /// Local node address
    local_address: SocketAddr,
    /// Connected peers
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// Bootstrap nodes for initial network discovery
    bootstrap_nodes: Vec<String>,
    /// NNG transport layer for actual communication
    transport: Option<Arc<NngTransport>>,
}

/// Information about a connected peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer's node ID
    pub node_id: String,
    /// Peer's network address
    pub address: SocketAddr,
    /// Connection status
    pub status: PeerStatus,
    /// Reputation score
    pub reputation: f64,
    /// Last seen timestamp
    pub last_seen: u64,
}

/// Status of a peer connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerStatus {
    Connecting,
    Connected,
    Disconnected,
    Failed,
}

impl P2PNetwork {
    /// Create a new P2P network instance
    pub fn new(node_id: String, local_address: SocketAddr) -> Self {
        Self {
            node_id,
            local_address,
            peers: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_nodes: Vec::new(),
            transport: None,
        }
    }

    /// Create a new P2P network with an existing transport layer
    pub fn with_transport(node_id: String, local_address: SocketAddr, transport: Arc<NngTransport>) -> Self {
        Self {
            node_id,
            local_address,
            peers: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_nodes: Vec::new(),
            transport: Some(transport),
        }
    }

    /// Add bootstrap nodes for network discovery
    pub fn add_bootstrap_nodes(&mut self, nodes: Vec<String>) {
        self.bootstrap_nodes.extend(nodes);
    }

    /// Start the P2P network service
    pub async fn start(&mut self) -> Result<(), P2PError> {
        println!("🌐 Starting P2P network on {}", self.local_address);

        // Initialize transport layer if not already provided
        if self.transport.is_none() {
            let transport = NngTransport::new(self.node_id.clone(), self.local_address)
                .await
                .map_err(|e| P2PError::NetworkError(format!("Transport init failed: {}", e)))?;

            let transport = Arc::new(transport);
            transport.start().await
                .map_err(|e| P2PError::NetworkError(format!("Transport start failed: {}", e)))?;

            self.transport = Some(transport);
        }

        // Connect to bootstrap nodes
        if !self.bootstrap_nodes.is_empty() {
            println!("   Connecting to {} bootstrap nodes...", self.bootstrap_nodes.len());
            self.connect_to_bootstrap_nodes().await?;
        }

        println!("✅ P2P network started successfully");
        Ok(())
    }

    /// Connect to a specific peer using the transport layer
    pub async fn connect_peer(&self, address: &str) -> Result<String, P2PError> {
        println!("🤝 Connecting to peer: {}", address);

        // Use transport layer for actual connection
        if let Some(transport) = &self.transport {
            transport.connect_peer(address)
                .await
                .map_err(|e| P2PError::ConnectionFailed)?;
        }

        // Parse address to create peer info
        let peer_id = format!("peer-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap_or("unknown"));
        let socket_addr: SocketAddr = address
            .trim_start_matches("tcp://")
            .parse()
            .map_err(|_| P2PError::InvalidAddress)?;

        let peer_info = PeerInfo {
            node_id: peer_id.clone(),
            address: socket_addr,
            status: PeerStatus::Connected,
            reputation: 1.0,
            last_seen: current_timestamp(),
        };

        let mut peers = self.peers.write().await;
        peers.insert(peer_id.clone(), peer_info);

        println!("✅ Connected to peer: {} ({})", peer_id, address);
        Ok(peer_id)
    }

    /// Get list of connected peers
    pub async fn get_peers(&self) -> HashMap<String, PeerInfo> {
        let peers = self.peers.read().await;
        peers.clone()
    }

    /// Broadcast a message to all connected peers using transport layer
    pub async fn broadcast(&self, message: NetworkMessage) -> Result<(), P2PError> {
        let peers = self.peers.read().await;
        println!("📡 Broadcasting message to {} peers", peers.len());

        if let Some(transport) = &self.transport {
            transport.broadcast(message)
                .await
                .map_err(|e| P2PError::SendFailed)?;

            // Update metrics
            transport.increment_messages_sent();
        } else {
            return Err(P2PError::NetworkError("Transport not initialized".to_string()));
        }

        Ok(())
    }

    /// Broadcast raw bytes to all connected peers
    pub async fn broadcast_bytes(&self, data: &[u8]) -> Result<(), P2PError> {
        let peers = self.peers.read().await;
        println!("📡 Broadcasting {} bytes to {} peers", data.len(), peers.len());

        // For raw bytes, we wrap in a Block message
        let message = NetworkMessage::Block(crate::network::transport::BlockMessage {
            request_id: uuid::Uuid::new_v4().to_string(),
            block_slot: None,
            compressed_data: Some(data.to_vec()),
            message_type: crate::network::transport::BlockMessageType::Store,
        });

        self.broadcast(message).await
    }

    /// Send a message to a specific peer
    pub async fn send_to_peer(&self, peer_id: &str, message: NetworkMessage) -> Result<(), P2PError> {
        let peers = self.peers.read().await;

        if peers.contains_key(peer_id) {
            println!("📤 Sending message to peer: {}", peer_id);

            if let Some(transport) = &self.transport {
                transport.send_to_peer(peer_id, message)
                    .await
                    .map_err(|e| P2PError::SendFailed)?;

                transport.increment_messages_sent();
            } else {
                return Err(P2PError::NetworkError("Transport not initialized".to_string()));
            }

            Ok(())
        } else {
            Err(P2PError::PeerNotFound)
        }
    }

    /// Get network statistics
    pub async fn get_stats(&self) -> NetworkStats {
        let peers = self.peers.read().await;
        NetworkStats {
            total_peers: peers.len(),
            connected_peers: peers.values()
                .filter(|p| matches!(p.status, PeerStatus::Connected))
                .count(),
            bootstrap_nodes: self.bootstrap_nodes.len(),
            local_node_id: self.node_id.clone(),
        }
    }

    /// Update peer status
    pub async fn update_peer_status(&self, peer_id: &str, status: PeerStatus) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(peer_id) {
            peer.status = status;
            peer.last_seen = current_timestamp();
        }
    }

    /// Update peer reputation
    pub async fn update_peer_reputation(&self, peer_id: &str, delta: f64) {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(peer_id) {
            peer.reputation = (peer.reputation + delta).clamp(0.0, 2.0);
            peer.last_seen = current_timestamp();
        }
    }

    /// Remove disconnected peers
    pub async fn prune_disconnected_peers(&self) {
        let mut peers = self.peers.write().await;
        let disconnected: Vec<String> = peers
            .iter()
            .filter(|(_, p)| matches!(p.status, PeerStatus::Disconnected | PeerStatus::Failed))
            .map(|(id, _)| id.clone())
            .collect();

        for peer_id in disconnected {
            peers.remove(&peer_id);
            println!("🗑️ Removed disconnected peer: {}", peer_id);
        }
    }

    /// Get transport reference for advanced operations
    pub fn get_transport(&self) -> Option<Arc<NngTransport>> {
        self.transport.clone()
    }

    async fn connect_to_bootstrap_nodes(&self) -> Result<(), P2PError> {
        let mut connected = 0;
        let mut failed = 0;

        for node in &self.bootstrap_nodes {
            match self.connect_peer(node).await {
                Ok(peer_id) => {
                    println!("   ✅ Connected to bootstrap node: {} ({})", peer_id, node);
                    connected += 1;
                }
                Err(e) => {
                    println!("   ❌ Failed to connect to bootstrap node {}: {:?}", node, e);
                    failed += 1;
                }
            }
        }

        println!("   Bootstrap: {}/{} nodes connected", connected, self.bootstrap_nodes.len());

        // Require at least one bootstrap connection if nodes were specified
        if connected == 0 && !self.bootstrap_nodes.is_empty() {
            return Err(P2PError::NetworkError(
                "Failed to connect to any bootstrap nodes".to_string()
            ));
        }

        Ok(())
    }
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_peers: usize,
    pub connected_peers: usize,
    pub bootstrap_nodes: usize,
    pub local_node_id: String,
}

/// P2P networking errors
#[derive(Debug, thiserror::Error)]
pub enum P2PError {
    #[error("Network connection failed")]
    ConnectionFailed,

    #[error("Invalid network address")]
    InvalidAddress,

    #[error("Peer not found")]
    PeerNotFound,

    #[error("Message send failed")]
    SendFailed,

    #[error("Network error: {0}")]
    NetworkError(String),
}