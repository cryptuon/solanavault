//! # NNG Transport Layer
//!
//! High-performance peer-to-peer transport layer using NNG (nanomsg-next-generation)
//! for efficient, scalable communication in the SolanaVault network.

use async_trait::async_trait;
use bytes::Bytes;
use futures::future::try_join_all;
use nng::{Message, Protocol, Socket};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

/// Network message types for P2P communication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Peer discovery and handshake
    Discovery(DiscoveryMessage),
    /// Block data requests and responses
    Block(BlockMessage),
    /// Network consensus and validation
    Consensus(ConsensusMessage),
    /// Heartbeat and health checks
    Heartbeat(HeartbeatMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryMessage {
    pub node_id: String,
    pub address: String,
    pub capabilities: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMessage {
    pub request_id: String,
    pub block_slot: Option<u64>,
    pub compressed_data: Option<Vec<u8>>,
    pub message_type: BlockMessageType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BlockMessageType {
    Request,
    Response,
    Store,
    Retrieve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusMessage {
    pub proposal_id: String,
    pub proposer: String,
    pub data_hash: String,
    pub signatures: Vec<String>,
    pub consensus_type: ConsensusType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusType {
    DataIntegrity,
    NodeReputation,
    NetworkHealth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatMessage {
    pub node_id: String,
    pub timestamp: u64,
    pub metrics: NodeMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetrics {
    pub uptime_seconds: u64,
    pub blocks_stored: u64,
    pub compression_ratio: f64,
    pub bandwidth_used: u64,
    pub reputation_score: f64,
}

/// High-performance NNG transport layer
#[derive(Debug)]
pub struct NngTransport {
    /// Unique node identifier
    node_id: String,
    /// Local network address
    local_address: SocketAddr,
    /// NNG socket for publishing messages
    pub_socket: Arc<Mutex<Socket>>,
    /// NNG socket for subscribing to messages
    sub_socket: Arc<Mutex<Socket>>,
    /// Connected peers map
    peers: Arc<RwLock<HashMap<String, PeerConnection>>>,
    /// Message handlers
    handlers: Arc<RwLock<HashMap<String, Box<dyn MessageHandler + Send + Sync>>>>,
    /// Transport configuration
    config: TransportConfig,
}

#[derive(Debug, Clone)]
pub struct PeerConnection {
    pub node_id: String,
    pub address: String,
    pub socket: Arc<Mutex<Socket>>,
    pub last_heartbeat: u64,
    pub metrics: NodeMetrics,
    pub status: ConnectionStatus,
}

#[derive(Debug, Clone)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Authenticated,
    Disconnected,
    Failed,
}

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub connection_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub max_peers: usize,
    pub buffer_size: usize,
    pub retry_attempts: u32,
}

impl Default for TransportConfig {
    fn default() -> Self {
        Self {
            connection_timeout: Duration::from_secs(30),
            heartbeat_interval: Duration::from_secs(10),
            max_peers: 100,
            buffer_size: 1024 * 1024, // 1MB
            retry_attempts: 3,
        }
    }
}

/// Trait for handling different message types
#[async_trait]
pub trait MessageHandler {
    async fn handle(&self, message: NetworkMessage, sender: &str) -> Result<Option<NetworkMessage>, TransportError>;
}

impl NngTransport {
    /// Create new NNG transport instance
    pub async fn new(node_id: String, local_address: SocketAddr) -> Result<Self, TransportError> {
        let config = TransportConfig::default();

        // Create NNG publisher socket
        let pub_socket = Socket::new(Protocol::Pub0)
            .map_err(|e| TransportError::SocketCreation(format!("Publisher: {}", e)))?;

        // Create NNG subscriber socket
        let sub_socket = Socket::new(Protocol::Sub0)
            .map_err(|e| TransportError::SocketCreation(format!("Subscriber: {}", e)))?;

        // Subscribe to all message types
        sub_socket.subscribe(b"").map_err(|e| TransportError::SocketCreation(format!("Subscribe: {}", e)))?;

        let transport = Self {
            node_id: node_id.clone(),
            local_address,
            pub_socket: Arc::new(Mutex::new(pub_socket)),
            sub_socket: Arc::new(Mutex::new(sub_socket)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            handlers: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        Ok(transport)
    }

    /// Start the transport layer
    pub async fn start(&self) -> Result<(), TransportError> {
        println!("🚀 Starting NNG transport layer for node: {}", self.node_id);

        // Bind publisher socket
        let pub_addr = format!("tcp://{}:{}", self.local_address.ip(), self.local_address.port());
        {
            let socket = self.pub_socket.lock().await;
            socket.listen(&pub_addr)
                .map_err(|e| TransportError::BindFailed(format!("Publisher bind failed: {}", e)))?;
        }

        // Bind subscriber socket for incoming connections
        let sub_addr = format!("tcp://{}:{}", self.local_address.ip(), self.local_address.port() + 1);
        {
            let socket = self.sub_socket.lock().await;
            socket.listen(&sub_addr)
                .map_err(|e| TransportError::BindFailed(format!("Subscriber bind failed: {}", e)))?;
        }

        println!("✅ NNG transport listening on:");
        println!("   Publisher:  {}", pub_addr);
        println!("   Subscriber: {}", sub_addr);

        // Start background tasks
        self.start_message_processor().await?;
        self.start_heartbeat_service().await?;

        Ok(())
    }

    /// Connect to a peer node
    pub async fn connect_peer(&self, peer_address: &str) -> Result<(), TransportError> {
        println!("🤝 Connecting to peer: {}", peer_address);

        // Create dedicated socket for this peer
        let peer_socket = Socket::new(Protocol::Req0)
            .map_err(|e| TransportError::SocketCreation(format!("Peer socket: {}", e)))?;

        // Connect to peer
        peer_socket.dial(peer_address)
            .map_err(|e| TransportError::ConnectionFailed(format!("Failed to connect to {}: {}", peer_address, e)))?;

        // Perform discovery handshake
        let discovery_msg = NetworkMessage::Discovery(DiscoveryMessage {
            node_id: self.node_id.clone(),
            address: format!("tcp://{}:{}", self.local_address.ip(), self.local_address.port()),
            capabilities: vec!["compression".to_string(), "storage".to_string()],
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        });

        let serialized = bincode::serialize(&discovery_msg)
            .map_err(|e| TransportError::SerializationError(e.to_string()))?;

        let nng_msg = Message::from(serialized.as_slice());
        peer_socket.send(nng_msg)
            .map_err(|e| TransportError::SendFailed(format!("Discovery message: {}", e)))?;

        // Wait for response
        let response = peer_socket.recv()
            .map_err(|e| TransportError::ReceiveFailed(format!("Discovery response: {}", e)))?;

        let peer_discovery: NetworkMessage = bincode::deserialize(response.as_slice())
            .map_err(|e| TransportError::SerializationError(e.to_string()))?;

        if let NetworkMessage::Discovery(discovery) = peer_discovery {
            // Store peer connection
            let peer_conn = PeerConnection {
                node_id: discovery.node_id.clone(),
                address: peer_address.to_string(),
                socket: Arc::new(Mutex::new(peer_socket)),
                last_heartbeat: discovery.timestamp,
                metrics: NodeMetrics {
                    uptime_seconds: 0,
                    blocks_stored: 0,
                    compression_ratio: 0.0,
                    bandwidth_used: 0,
                    reputation_score: 1.0,
                },
                status: ConnectionStatus::Connected,
            };

            let mut peers = self.peers.write().await;
            peers.insert(discovery.node_id.clone(), peer_conn);

            println!("✅ Connected to peer: {} ({})", discovery.node_id, peer_address);
        }

        Ok(())
    }

    /// Broadcast message to all connected peers
    pub async fn broadcast(&self, message: NetworkMessage) -> Result<(), TransportError> {
        let serialized = bincode::serialize(&message)
            .map_err(|e| TransportError::SerializationError(e.to_string()))?;

        let peers = self.peers.read().await;
        let broadcast_tasks: Vec<_> = peers.values().map(|peer| {
            let data = serialized.clone();
            let socket = peer.socket.clone();
            async move {
                let socket = socket.lock().await;
                let nng_msg = Message::from(data.as_slice());
                socket.send(nng_msg)
                    .map_err(|e| TransportError::SendFailed(format!("Broadcast to {}: {}", peer.node_id, e)))
            }
        }).collect();

        // Execute all broadcasts concurrently
        let results = try_join_all(broadcast_tasks).await;

        match results {
            Ok(_) => {
                println!("📡 Broadcast sent to {} peers", peers.len());
                Ok(())
            }
            Err(e) => {
                eprintln!("❌ Broadcast failed: {}", e);
                Err(e)
            }
        }
    }

    /// Send message to specific peer
    pub async fn send_to_peer(&self, peer_id: &str, message: NetworkMessage) -> Result<(), TransportError> {
        let peers = self.peers.read().await;

        if let Some(peer) = peers.get(peer_id) {
            let serialized = bincode::serialize(&message)
                .map_err(|e| TransportError::SerializationError(e.to_string()))?;

            let socket = peer.socket.lock().await;
            let nng_msg = Message::from(serialized.as_slice());
            socket.send(nng_msg)
                .map_err(|e| TransportError::SendFailed(format!("Send to {}: {}", peer_id, e)))?;

            println!("📤 Message sent to peer: {}", peer_id);
            Ok(())
        } else {
            Err(TransportError::PeerNotFound(peer_id.to_string()))
        }
    }

    /// Register message handler for specific message types
    pub async fn register_handler<H>(&self, message_type: String, handler: H)
    where
        H: MessageHandler + Send + Sync + 'static
    {
        let mut handlers = self.handlers.write().await;
        handlers.insert(message_type, Box::new(handler));
    }

    /// Get list of connected peers
    pub async fn get_peers(&self) -> Vec<String> {
        let peers = self.peers.read().await;
        peers.keys().cloned().collect()
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let peers = self.peers.read().await;
        let connected_count = peers.values()
            .filter(|p| matches!(p.status, ConnectionStatus::Connected | ConnectionStatus::Authenticated))
            .count();

        NetworkStats {
            node_id: self.node_id.clone(),
            total_peers: peers.len(),
            connected_peers: connected_count,
            messages_sent: 0, // TODO: Track this
            messages_received: 0, // TODO: Track this
            uptime_seconds: 0, // TODO: Track this
        }
    }

    async fn start_message_processor(&self) -> Result<(), TransportError> {
        println!("🔄 Starting message processor...");
        // TODO: Implement background message processing loop
        Ok(())
    }

    async fn start_heartbeat_service(&self) -> Result<(), TransportError> {
        println!("💓 Starting heartbeat service...");
        // TODO: Implement heartbeat monitoring
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub node_id: String,
    pub total_peers: usize,
    pub connected_peers: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub uptime_seconds: u64,
}

/// Transport layer errors
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("Socket creation failed: {0}")]
    SocketCreation(String),

    #[error("Failed to bind socket: {0}")]
    BindFailed(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Send failed: {0}")]
    SendFailed(String),

    #[error("Receive failed: {0}")]
    ReceiveFailed(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Peer not found: {0}")]
    PeerNotFound(String),

    #[error("Network error: {0}")]
    NetworkError(String),
}