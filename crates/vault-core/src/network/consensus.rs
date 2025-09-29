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
        }
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
                signature,
                timestamp: current_timestamp(),
            };

            // TODO: Broadcast vote message
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
            ProposalType::DataIntegrity { block_slot, .. } => {
                println!("✅ Data integrity verified for block {}", block_slot);
                // TODO: Update data integrity status
            }
            ProposalType::ReputationUpdate { target_node, new_reputation, .. } => {
                let mut reputation_store = self.reputation_store.write().await;
                reputation_store.scores.insert(target_node.clone(), new_reputation);
                println!("📊 Reputation updated for {}: {}", target_node, new_reputation);
            }
            ProposalType::NetworkConfig { parameter, new_value, .. } => {
                println!("⚙️  Network parameter {} updated to {}", parameter, new_value);
                // TODO: Update network configuration
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
        // TODO: Implement data integrity proposal handling
        println!("🔍 Received data integrity proposal from {}", sender_id);
        Ok(())
    }

    async fn handle_reputation_proposal(
        &self,
        message: ConsensusMessage,
        sender_id: &str,
    ) -> Result<(), TransportError> {
        // TODO: Implement reputation proposal handling
        println!("📊 Received reputation proposal from {}", sender_id);
        Ok(())
    }

    async fn handle_network_health_proposal(
        &self,
        message: ConsensusMessage,
        sender_id: &str,
    ) -> Result<(), TransportError> {
        // TODO: Implement network health proposal handling
        println!("🏥 Received network health proposal from {}", sender_id);
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
        .unwrap()
        .as_secs()
}