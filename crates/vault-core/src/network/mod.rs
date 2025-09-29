//! # Network Module
//!
//! This module handles node communication and data transfer for the
//! SolanaVault decentralized network.

/// Legacy P2P networking implementation (for compatibility)
pub mod p2p;

/// High-performance NNG transport layer
pub mod transport;

/// Peer discovery and DHT implementation
pub mod discovery;

/// Byzantine Fault Tolerant consensus protocol
pub mod consensus;

/// Decentralized network manager coordinating all components
pub mod decentralized;

/// Light client for non-node users
pub mod light_client;

/// Gateway node for monetizing network access
pub mod gateway;

// Re-export key types
pub use p2p::{P2PNetwork, PeerInfo, P2PError};
pub use transport::{NngTransport, NetworkMessage, TransportError, NetworkStats};
pub use discovery::{DhtNode, NodeInfo, ContentMetadata};
pub use consensus::{ConsensusEngine, ConsensusProposal, VoteChoice, ReputationEvidence};
pub use decentralized::{DecentralizedVaultNode, DecentralizedRpcHandler, NodeConfig};
pub use light_client::{LightClient, LightClientConfig, RequestPriority, WalletInfo};
pub use gateway::{GatewayNode, GatewayConfig, PricingInfo, GatewayStats};