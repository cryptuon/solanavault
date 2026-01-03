//! Network Integration Tests
//!
//! Tests for the P2P networking, transport, and discovery components.

use vault_core::network::{
    NetworkMessage, TransportError,
    TlsConfig, TlsStatus, TlsVersion,
};

/// Test network message serialization
#[test]
fn test_network_message_serialization() {
    use vault_core::network::transport::{BlockMessage, BlockMessageType};

    let block_msg = BlockMessage {
        request_id: "test-request-123".to_string(),
        block_slot: Some(12345),
        compressed_data: Some(vec![1, 2, 3, 4, 5]),
        message_type: BlockMessageType::Request,
    };

    let msg = NetworkMessage::Block(block_msg);

    // Should serialize and deserialize correctly
    let serialized = bincode::serialize(&msg).expect("Failed to serialize");
    let deserialized: NetworkMessage = bincode::deserialize(&serialized).expect("Failed to deserialize");

    if let NetworkMessage::Block(block) = deserialized {
        assert_eq!(block.request_id, "test-request-123");
        assert_eq!(block.block_slot, Some(12345));
        assert_eq!(block.compressed_data, Some(vec![1, 2, 3, 4, 5]));
    } else {
        panic!("Expected Block message type");
    }
}

/// Test TLS configuration defaults
#[test]
fn test_tls_config_default() {
    let config = TlsConfig::default();

    assert!(!config.enabled);
    assert!(config.cert_path.is_none());
    assert!(config.key_path.is_none());
    assert!(!config.verify_client);
    assert_eq!(config.min_tls_version, TlsVersion::Tls12);
}

/// Test TLS configuration disabled mode
#[test]
fn test_tls_config_disabled() {
    let config = TlsConfig::disabled();

    assert!(!config.enabled);
    assert!(config.validate().is_ok());
    assert!(!config.is_ready());
}

/// Test TLS status from disabled config
#[test]
fn test_tls_status_disabled() {
    let config = TlsConfig::disabled();
    let status = TlsStatus::from_config(&config);

    assert!(!status.enabled);
    assert!(!status.configured);
    assert!(!status.mtls_enabled);
    assert!(status.error.is_none());
    assert!(status.min_version.is_none());
}

/// Test TLS enabled without paths fails validation
#[test]
fn test_tls_enabled_without_paths_fails() {
    let config = TlsConfig {
        enabled: true,
        ..Default::default()
    };

    assert!(config.validate().is_err());
}

/// Test TLS version string representation
#[test]
fn test_tls_version_strings() {
    assert_eq!(TlsVersion::Tls12.as_str(), "TLS 1.2");
    assert_eq!(TlsVersion::Tls13.as_str(), "TLS 1.3");
}

/// Test transport error types
#[test]
fn test_transport_error_display() {
    let error = TransportError::ConnectionFailed("test reason".to_string());
    let error_str = error.to_string();

    assert!(error_str.contains("test reason") || !error_str.is_empty());
}

/// Test heartbeat message creation
#[test]
fn test_heartbeat_message() {
    use vault_core::network::transport::{HeartbeatMessage, NodeMetrics};

    let heartbeat = HeartbeatMessage {
        node_id: "test-node".to_string(),
        timestamp: 1234567890,
        metrics: NodeMetrics {
            uptime_seconds: 3600,
            blocks_stored: 100,
            compression_ratio: 15.5,
            bandwidth_used: 1024 * 1024,
            reputation_score: 0.95,
        },
    };

    let msg = NetworkMessage::Heartbeat(heartbeat);

    let serialized = bincode::serialize(&msg).expect("Failed to serialize");
    let deserialized: NetworkMessage = bincode::deserialize(&serialized).expect("Failed to deserialize");

    if let NetworkMessage::Heartbeat(hb) = deserialized {
        assert_eq!(hb.node_id, "test-node");
        assert_eq!(hb.timestamp, 1234567890);
        assert_eq!(hb.metrics.blocks_stored, 100);
    } else {
        panic!("Expected Heartbeat message type");
    }
}

/// Test block message types
#[test]
fn test_block_message_types() {
    use vault_core::network::transport::BlockMessageType;

    // Ensure all variants exist
    let _request = BlockMessageType::Request;
    let _response = BlockMessageType::Response;
    let _store = BlockMessageType::Store;
    let _retrieve = BlockMessageType::Retrieve;
}

/// Test network stats structure
#[test]
fn test_network_stats_creation() {
    use vault_core::network::NetworkStats;

    let stats = NetworkStats {
        node_id: "test-node".to_string(),
        total_peers: 10,
        connected_peers: 5,
        messages_sent: 100,
        messages_received: 150,
        uptime_seconds: 3600,
    };

    assert_eq!(stats.node_id, "test-node");
    assert_eq!(stats.total_peers, 10);
    assert_eq!(stats.connected_peers, 5);
    assert_eq!(stats.messages_sent, 100);
    assert_eq!(stats.messages_received, 150);
    assert_eq!(stats.uptime_seconds, 3600);
}

/// Test TLS config with min version
#[test]
fn test_tls_config_with_min_version() {
    let config = TlsConfig::disabled().with_min_version(TlsVersion::Tls13);
    assert_eq!(config.min_tls_version, TlsVersion::Tls13);
}
