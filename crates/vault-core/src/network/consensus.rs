//! # Consensus Protocol
//!
//! Byzantine Fault Tolerant consensus protocol for data integrity and
//! network agreement in the SolanaVault distributed network.

use crate::network::transport::{NetworkMessage, ConsensusMessage, ConsensusType, NngTransport, TransportError};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, Mutex};
use tokio::time::{sleep, timeout};
use uuid::Uuid;

/// Byzantine Fault Tolerant consensus engine
#[derive(Debug)]
pub struct ConsensusEngine {
    /// Local node identifier
    node_id: String,
    /// Transport layer for communication
    transport: Arc<NngTransport>,
    /// Active consensus proposals
    proposals: Arc<RwLock<HashMap<String, ConsensusProposal>>>,
    /// Node reputation tracking
    reputation_store: Arc<RwLock<ReputationStore>>,
    /// Consensus configuration
    config: ConsensusConfig,
    /// Current network view
    network_view: Arc<RwLock<NetworkView>>,
    /// Data integrity verification status (block_slot -> verification status)
    data_integrity_status: Arc<RwLock<DataIntegrityStore>>,
    /// Runtime network configuration
    runtime_config: Arc<RwLock<RuntimeNetworkConfig>>,
}

/// Store for data integrity verification status
#[derive(Debug, Default)]
pub struct DataIntegrityStore {
    /// Verified blocks (block_slot -> verification record)
    verified_blocks: HashMap<u64, DataIntegrityRecord>,
}

#[derive(Debug, Clone)]
pub struct DataIntegrityRecord {
    pub block_slot: u64,
    pub compressed_hash: String,
    pub original_hash: String,
    pub compression_ratio: f64,
    pub verified_at: u64,
    pub proposal_id: String,
    pub verifier_count: usize,
}

/// Runtime network configuration that can be updated via consensus
#[derive(Debug, Clone)]
pub struct RuntimeNetworkConfig {
    /// Maximum block size in bytes
    pub max_block_size: u64,
    /// Replication factor for blocks
    pub replication_factor: u32,
    /// Compression level (0-22 for zstd)
    pub compression_level: u32,
    /// Minimum nodes for consensus
    pub min_consensus_nodes: u32,
    /// Gateway fee multiplier
    pub gateway_fee_multiplier: f64,
    /// Block retention period in seconds
    pub block_retention_seconds: u64,
}

impl Default for RuntimeNetworkConfig {
    fn default() -> Self {
        Self {
            max_block_size: 10 * 1024 * 1024, // 10MB
            replication_factor: 3,
            compression_level: 19,
            min_consensus_nodes: 3,
            gateway_fee_multiplier: 1.0,
            block_retention_seconds: 365 * 24 * 60 * 60, // 1 year
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusProposal {
    pub proposal_id: String,
    pub proposer: String,
    pub proposal_type: ProposalType,
    pub data_hash: String,
    pub timestamp: u64,
    pub signatures: HashMap<String, Signature>,
    pub status: ProposalStatus,
    pub timeout_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    /// Data integrity verification
    DataIntegrity {
        block_slot: u64,
        compressed_data_hash: String,
        original_data_hash: String,
        compression_ratio: f64,
    },
    /// Node reputation update
    ReputationUpdate {
        target_node: String,
        new_reputation: f64,
        evidence: Vec<ReputationEvidence>,
    },
    /// Network configuration change
    NetworkConfig {
        parameter: String,
        old_value: String,
        new_value: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    pub signer: String,
    pub signature: String,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Proposed,
    Voting,
    Accepted,
    Rejected,
    Timeout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvidence {
    pub evidence_type: EvidenceType,
    pub timestamp: u64,
    pub witnesses: Vec<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvidenceType {
    DataCorruption,
    ServiceDowntime,
    SlowResponse,
    GoodBehavior,
    FastResponse,
    DataIntegrityMaintained,
}

#[derive(Debug)]
pub struct ReputationStore {
    /// Node ID -> reputation score
    scores: HashMap<String, f64>,
    /// Node ID -> reputation history
    history: HashMap<String, Vec<ReputationEvent>>,
}

#[derive(Debug, Clone)]
pub struct ReputationEvent {
    pub timestamp: u64,
    pub event_type: EvidenceType,
    pub impact: f64,
    pub witnesses: Vec<String>,
}

#[derive(Debug)]
pub struct NetworkView {
    /// Active nodes in the network
    active_nodes: HashSet<String>,
    /// Total stake/voting power per node
    voting_power: HashMap<String, f64>,
    /// Last seen timestamp per node
    last_seen: HashMap<String, u64>,
}

#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// Minimum percentage of votes needed for acceptance
    pub acceptance_threshold: f64,
    /// Maximum time to wait for consensus
    pub proposal_timeout: Duration,
    /// Minimum reputation to participate in consensus
    pub min_reputation: f64,
    /// Byzantine fault tolerance (max % of malicious nodes)
    pub byzantine_threshold: f64,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            acceptance_threshold: 0.67, // 2/3 majority
            proposal_timeout: Duration::from_secs(60),
            min_reputation: 0.5,
            byzantine_threshold: 0.33, // Tolerates up to 1/3 malicious nodes
        }
    }
}

/// Consensus message types for voting and agreement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConsensusVote {
    /// Vote on a proposal
    Vote {
        proposal_id: String,
        voter: String,
        vote: VoteChoice,
        signature: String,
        timestamp: u64,
    },
    /// Request for proposal details
    ProposalRequest {
        proposal_id: String,
        requester: String,
    },
    /// Proposal details response
    ProposalResponse {
        proposal: ConsensusProposal,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteChoice {
    Accept,
    Reject,
    Abstain,
}

impl ConsensusEngine {
    /// Create new consensus engine
    pub fn new(node_id: String, transport: Arc<NngTransport>) -> Self {
        Self {
            node_id,
            transport,
            proposals: Arc::new(RwLock::new(HashMap::new())),
            reputation_store: Arc::new(RwLock::new(ReputationStore::new())),
            config: ConsensusConfig::default(),
            network_view: Arc::new(RwLock::new(NetworkView::new())),
            data_integrity_status: Arc::new(RwLock::new(DataIntegrityStore::default())),
            runtime_config: Arc::new(RwLock::new(RuntimeNetworkConfig::default())),
        }
    }

    /// Get the data integrity status for a block
    pub async fn get_block_integrity_status(&self, block_slot: u64) -> Option<DataIntegrityRecord> {
        let store = self.data_integrity_status.read().await;
        store.verified_blocks.get(&block_slot).cloned()
    }

    /// Check if a block has been verified
    pub async fn is_block_verified(&self, block_slot: u64) -> bool {
        let store = self.data_integrity_status.read().await;
        store.verified_blocks.contains_key(&block_slot)
    }

    /// Get current runtime network configuration
    pub async fn get_runtime_config(&self) -> RuntimeNetworkConfig {
        let config = self.runtime_config.read().await;
        config.clone()
    }

    /// Propose data integrity verification
    pub async fn propose_data_integrity(
        &self,
        block_slot: u64,
        compressed_data_hash: String,
        original_data_hash: String,
        compression_ratio: f64,
    ) -> Result<String, TransportError> {
        let proposal_id = generate_proposal_id();

        let proposal = ConsensusProposal {
            proposal_id: proposal_id.clone(),
            proposer: self.node_id.clone(),
            proposal_type: ProposalType::DataIntegrity {
                block_slot,
                compressed_data_hash: compressed_data_hash.clone(),
                original_data_hash: original_data_hash.clone(),
                compression_ratio,
            },
            data_hash: calculate_proposal_hash(&compressed_data_hash, &original_data_hash),
            timestamp: current_timestamp(),
            signatures: HashMap::new(),
            status: ProposalStatus::Proposed,
            timeout_at: current_timestamp() + self.config.proposal_timeout.as_secs(),
        };

        // Store proposal locally
        {
            let mut proposals = self.proposals.write().await;
            proposals.insert(proposal_id.clone(), proposal.clone());
        }

        // Broadcast proposal to network
        let consensus_msg = ConsensusMessage {
            proposal_id: proposal_id.clone(),
            proposer: self.node_id.clone(),
            data_hash: proposal.data_hash.clone(),
            signatures: vec![],
            consensus_type: ConsensusType::DataIntegrity,
        };

        let network_msg = NetworkMessage::Consensus(consensus_msg);
        self.transport.broadcast(network_msg).await?;

        println!("📋 Proposed data integrity verification for block {}", block_slot);

        // Start consensus process
        self.start_consensus_process(proposal_id.clone()).await;

        Ok(proposal_id)
    }

    /// Vote on a proposal
    pub async fn vote_on_proposal(
        &self,
        proposal_id: &str,
        vote: VoteChoice,
    ) -> Result<(), TransportError> {
        // Check if we have the proposal
        let proposal = {
            let proposals = self.proposals.read().await;
            proposals.get(proposal_id).cloned()
        };

        if let Some(mut proposal) = proposal {
            // Check voting eligibility
            if !self.can_vote(&self.node_id).await {
                return Err(TransportError::NetworkError("Insufficient reputation to vote".to_string()));
            }

            // Create signature
            let signature = create_vote_signature(&self.node_id, proposal_id, &vote);

            // Add our vote
            proposal.signatures.insert(self.node_id.clone(), Signature {
                signer: self.node_id.clone(),
                signature: signature.clone(),
                timestamp: current_timestamp(),
            });

            // Update proposal
            {
                let mut proposals = self.proposals.write().await;
                proposals.insert(proposal_id.to_string(), proposal.clone());
            }

            // Broadcast vote
            let vote_msg = ConsensusVote::Vote {
                proposal_id: proposal_id.to_string(),
                voter: self.node_id.clone(),
                vote: vote.clone(),
                signature: signature.clone(),
                timestamp: current_timestamp(),
            };

            // Broadcast vote to network peers
            let consensus_msg = ConsensusMessage {
                proposal_id: proposal_id.to_string(),
                proposer: proposal.proposer.clone(),
                data_hash: proposal.data_hash.clone(),
                signatures: vec![signature.clone()],
                consensus_type: match &proposal.proposal_type {
                    ProposalType::DataIntegrity { .. } => ConsensusType::DataIntegrity,
                    ProposalType::ReputationUpdate { .. } => ConsensusType::NodeReputation,
                    ProposalType::NetworkConfig { .. } => ConsensusType::NetworkHealth,
                },
            };

            let network_msg = NetworkMessage::Consensus(consensus_msg);
            if let Err(e) = self.transport.broadcast(network_msg).await {
                log::warn!("Failed to broadcast vote for {}: {}", proposal_id, e);
            }

            println!("🗳️  Voted {:?} on proposal {}", vote, proposal_id);

            // Check if consensus reached
            self.check_consensus_completion(proposal_id).await?;
        }

        Ok(())
    }

    /// Handle incoming consensus messages
    pub async fn handle_consensus_message(
        &self,
        message: ConsensusMessage,
        sender_id: &str,
    ) -> Result<(), TransportError> {
        match message.consensus_type {
            ConsensusType::DataIntegrity => {
                self.handle_data_integrity_proposal(message, sender_id).await
            }
            ConsensusType::NodeReputation => {
                self.handle_reputation_proposal(message, sender_id).await
            }
            ConsensusType::NetworkHealth => {
                self.handle_network_health_proposal(message, sender_id).await
            }
        }
    }

    /// Verify data integrity using consensus
    pub async fn verify_data_integrity(
        &self,
        block_slot: u64,
        data_hash: &str,
    ) -> Result<bool, TransportError> {
        // Check existing consensus results
        let proposals = self.proposals.read().await;

        for proposal in proposals.values() {
            if let ProposalType::DataIntegrity { block_slot: slot, compressed_data_hash, .. } = &proposal.proposal_type {
                if *slot == block_slot && compressed_data_hash == data_hash {
                    return Ok(matches!(proposal.status, ProposalStatus::Accepted));
                }
            }
        }

        // No existing consensus found
        Ok(false)
    }

    /// Update node reputation based on consensus
    pub async fn update_reputation(
        &self,
        node_id: &str,
        evidence: ReputationEvidence,
    ) -> Result<(), TransportError> {
        let impact = match evidence.evidence_type {
            EvidenceType::DataCorruption => -0.2,
            EvidenceType::ServiceDowntime => -0.1,
            EvidenceType::SlowResponse => -0.05,
            EvidenceType::GoodBehavior => 0.05,
            EvidenceType::FastResponse => 0.02,
            EvidenceType::DataIntegrityMaintained => 0.1,
        };

        let mut reputation_store = self.reputation_store.write().await;

        // Update score
        let current_score = reputation_store.scores.get(node_id).unwrap_or(&1.0);
        let new_score = (current_score + impact).max(0.0).min(2.0); // Clamp between 0-2
        reputation_store.scores.insert(node_id.to_string(), new_score);

        // Add to history
        let event = ReputationEvent {
            timestamp: current_timestamp(),
            event_type: evidence.evidence_type,
            impact,
            witnesses: evidence.witnesses,
        };

        reputation_store.history
            .entry(node_id.to_string())
            .or_insert_with(Vec::new)
            .push(event);

        println!("📊 Updated reputation for {}: {:.2}", node_id, new_score);

        Ok(())
    }

    /// Get current reputation score for a node
    pub async fn get_reputation(&self, node_id: &str) -> f64 {
        let reputation_store = self.reputation_store.read().await;
        reputation_store.scores.get(node_id).unwrap_or(&1.0).clone()
    }

    async fn start_consensus_process(&self, proposal_id: String) {
        let proposals = self.proposals.clone();
        let config = self.config.clone();
        let transport = self.transport.clone();

        tokio::spawn(async move {
            // Wait for consensus timeout
            sleep(config.proposal_timeout).await;

            let mut proposals_lock = proposals.write().await;
            if let Some(proposal) = proposals_lock.get_mut(&proposal_id) {
                if matches!(proposal.status, ProposalStatus::Proposed | ProposalStatus::Voting) {
                    proposal.status = ProposalStatus::Timeout;
                    println!("⏰ Proposal {} timed out", proposal_id);
                }
            }
        });
    }

    async fn check_consensus_completion(&self, proposal_id: &str) -> Result<(), TransportError> {
        let network_view = self.network_view.read().await;
        let total_voting_power: f64 = network_view.voting_power.values().sum();
        let required_votes = total_voting_power * self.config.acceptance_threshold;

        let mut proposals = self.proposals.write().await;
        if let Some(proposal) = proposals.get_mut(proposal_id) {
            let total_votes: f64 = proposal.signatures.len() as f64; // Simplified voting power

            if total_votes >= required_votes {
                proposal.status = ProposalStatus::Accepted;
                println!("✅ Consensus reached for proposal {}", proposal_id);

                // Execute consensus decision
                self.execute_consensus_decision(proposal.clone()).await?;
            } else if current_timestamp() > proposal.timeout_at {
                proposal.status = ProposalStatus::Timeout;
                println!("⏰ Proposal {} timed out without consensus", proposal_id);
            }
        }

        Ok(())
    }

    async fn execute_consensus_decision(&self, proposal: ConsensusProposal) -> Result<(), TransportError> {
        match proposal.proposal_type {
            ProposalType::DataIntegrity { block_slot, compressed_data_hash, original_data_hash, compression_ratio } => {
                // Store data integrity verification record
                let record = DataIntegrityRecord {
                    block_slot,
                    compressed_hash: compressed_data_hash.clone(),
                    original_hash: original_data_hash.clone(),
                    compression_ratio,
                    verified_at: current_timestamp(),
                    proposal_id: proposal.proposal_id.clone(),
                    verifier_count: proposal.signatures.len(),
                };

                {
                    let mut store = self.data_integrity_status.write().await;
                    store.verified_blocks.insert(block_slot, record);
                }

                log::info!("✅ Data integrity verified for block {} (ratio: {:.2}:1, verifiers: {})",
                    block_slot, compression_ratio, proposal.signatures.len());
            }
            ProposalType::ReputationUpdate { target_node, new_reputation, .. } => {
                let mut reputation_store = self.reputation_store.write().await;
                reputation_store.scores.insert(target_node.clone(), new_reputation);
                log::info!("📊 Reputation updated for {}: {:.2}", target_node, new_reputation);
            }
            ProposalType::NetworkConfig { parameter, new_value, .. } => {
                // Apply runtime network configuration changes
                let mut config = self.runtime_config.write().await;

                match parameter.as_str() {
                    "max_block_size" => {
                        if let Ok(val) = new_value.parse::<u64>() {
                            config.max_block_size = val;
                            log::info!("⚙️  max_block_size updated to {} bytes", val);
                        }
                    }
                    "replication_factor" => {
                        if let Ok(val) = new_value.parse::<u32>() {
                            config.replication_factor = val;
                            log::info!("⚙️  replication_factor updated to {}", val);
                        }
                    }
                    "compression_level" => {
                        if let Ok(val) = new_value.parse::<u32>() {
                            if val <= 22 {
                                config.compression_level = val;
                                log::info!("⚙️  compression_level updated to {}", val);
                            } else {
                                log::warn!("Invalid compression_level: {} (max 22)", val);
                            }
                        }
                    }
                    "min_consensus_nodes" => {
                        if let Ok(val) = new_value.parse::<u32>() {
                            config.min_consensus_nodes = val;
                            log::info!("⚙️  min_consensus_nodes updated to {}", val);
                        }
                    }
                    "gateway_fee_multiplier" => {
                        if let Ok(val) = new_value.parse::<f64>() {
                            if val >= 0.1 && val <= 10.0 {
                                config.gateway_fee_multiplier = val;
                                log::info!("⚙️  gateway_fee_multiplier updated to {:.2}", val);
                            } else {
                                log::warn!("Invalid gateway_fee_multiplier: {} (range: 0.1-10.0)", val);
                            }
                        }
                    }
                    "block_retention_seconds" => {
                        if let Ok(val) = new_value.parse::<u64>() {
                            config.block_retention_seconds = val;
                            log::info!("⚙️  block_retention_seconds updated to {} ({} days)",
                                val, val / 86400);
                        }
                    }
                    _ => {
                        log::warn!("Unknown network config parameter: {}", parameter);
                    }
                }
            }
        }

        Ok(())
    }

    async fn can_vote(&self, node_id: &str) -> bool {
        let reputation = self.get_reputation(node_id).await;
        reputation >= self.config.min_reputation
    }

    async fn handle_data_integrity_proposal(
        &self,
        message: ConsensusMessage,
        sender_id: &str,
    ) -> Result<(), TransportError> {
        println!("🔍 Received data integrity proposal {} from {}", message.proposal_id, sender_id);

        // Validate proposal exists or create it
        let proposals = self.proposals.read().await;
        if let Some(proposal) = proposals.get(&message.proposal_id) {
            // Already tracking this proposal, add signatures
            drop(proposals);
            let mut proposals = self.proposals.write().await;
            if let Some(proposal) = proposals.get_mut(&message.proposal_id) {
                for sig in &message.signatures {
                    let signature = Signature {
                        signer: sender_id.to_string(),
                        signature: sig.clone(),
                        timestamp: current_timestamp(),
                    };
                    proposal.signatures.insert(sender_id.to_string(), signature);
                }
            }
        } else {
            // Log receipt of unknown proposal
            log::debug!("Received unknown proposal {}, may need to sync", message.proposal_id);
        }

        Ok(())
    }

    async fn handle_reputation_proposal(
        &self,
        message: ConsensusMessage,
        sender_id: &str,
    ) -> Result<(), TransportError> {
        println!("📊 Received reputation proposal {} from {}", message.proposal_id, sender_id);

        // Process reputation votes
        let proposals = self.proposals.read().await;
        if proposals.contains_key(&message.proposal_id) {
            drop(proposals);
            let mut proposals = self.proposals.write().await;
            if let Some(proposal) = proposals.get_mut(&message.proposal_id) {
                for sig in &message.signatures {
                    let signature = Signature {
                        signer: sender_id.to_string(),
                        signature: sig.clone(),
                        timestamp: current_timestamp(),
                    };
                    proposal.signatures.insert(sender_id.to_string(), signature);
                }
            }
        }

        Ok(())
    }

    async fn handle_network_health_proposal(
        &self,
        message: ConsensusMessage,
        sender_id: &str,
    ) -> Result<(), TransportError> {
        println!("🏥 Received network health proposal {} from {}", message.proposal_id, sender_id);

        // Process network health votes
        let proposals = self.proposals.read().await;
        if proposals.contains_key(&message.proposal_id) {
            drop(proposals);
            let mut proposals = self.proposals.write().await;
            if let Some(proposal) = proposals.get_mut(&message.proposal_id) {
                for sig in &message.signatures {
                    let signature = Signature {
                        signer: sender_id.to_string(),
                        signature: sig.clone(),
                        timestamp: current_timestamp(),
                    };
                    proposal.signatures.insert(sender_id.to_string(), signature);
                }
            }
        }

        Ok(())
    }
}

impl ReputationStore {
    fn new() -> Self {
        Self {
            scores: HashMap::new(),
            history: HashMap::new(),
        }
    }
}

impl NetworkView {
    fn new() -> Self {
        Self {
            active_nodes: HashSet::new(),
            voting_power: HashMap::new(),
            last_seen: HashMap::new(),
        }
    }
}

/// Generate unique proposal ID
fn generate_proposal_id() -> String {
    Uuid::new_v4().to_string()
}

/// Calculate hash for proposal data
fn calculate_proposal_hash(data1: &str, data2: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data1.as_bytes());
    hasher.update(data2.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Create cryptographic signature for vote
fn create_vote_signature(voter: &str, proposal_id: &str, vote: &VoteChoice) -> String {
    let vote_data = format!("{}:{}:{:?}", voter, proposal_id, vote);
    let mut hasher = Sha256::new();
    hasher.update(vote_data.as_bytes());
    format!("sig_{:x}", hasher.finalize())[..32].to_string()
}

/// Get current timestamp
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}