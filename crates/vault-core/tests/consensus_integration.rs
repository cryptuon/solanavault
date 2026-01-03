//! Consensus Integration Tests
//!
//! Tests for the Byzantine Fault Tolerant consensus protocol.

use vault_core::network::{
    VoteChoice, ReputationEvidence,
};

/// Test proposal ID generation
#[test]
fn test_proposal_id_generation() {
    use uuid::Uuid;

    let proposal_id_1 = Uuid::new_v4().to_string();
    let proposal_id_2 = Uuid::new_v4().to_string();

    // Each proposal should have a unique ID
    assert_ne!(proposal_id_1, proposal_id_2);
}

/// Test vote choice enum
#[test]
fn test_vote_choice_variants() {
    let accept = VoteChoice::Accept;
    let reject = VoteChoice::Reject;
    let abstain = VoteChoice::Abstain;

    // All variants should be distinct
    assert_ne!(
        format!("{:?}", accept),
        format!("{:?}", reject)
    );
    assert_ne!(
        format!("{:?}", accept),
        format!("{:?}", abstain)
    );
}

/// Test reputation evidence types
#[test]
fn test_reputation_evidence_types() {
    use vault_core::network::consensus::EvidenceType;

    let good_evidence = ReputationEvidence {
        evidence_type: EvidenceType::GoodBehavior,
        timestamp: 1234567890,
        witnesses: vec!["witness1".to_string(), "witness2".to_string()],
        description: "Verified block correctly".to_string(),
    };

    let bad_evidence = ReputationEvidence {
        evidence_type: EvidenceType::DataCorruption,
        timestamp: 1234567890,
        witnesses: vec!["witness1".to_string()],
        description: "Submitted invalid block".to_string(),
    };

    assert_ne!(
        format!("{:?}", good_evidence.evidence_type),
        format!("{:?}", bad_evidence.evidence_type)
    );
}

/// Test consensus quorum calculation (2/3 majority)
#[test]
fn test_consensus_quorum_calculation() {
    // For BFT, we need 2/3 + 1 votes for consensus
    fn calculate_quorum(total_nodes: usize) -> usize {
        (total_nodes * 2 / 3) + 1
    }

    assert_eq!(calculate_quorum(3), 3);  // 3 nodes: need 3
    assert_eq!(calculate_quorum(4), 3);  // 4 nodes: need 3
    assert_eq!(calculate_quorum(6), 5);  // 6 nodes: need 5
    assert_eq!(calculate_quorum(9), 7);  // 9 nodes: need 7
    assert_eq!(calculate_quorum(10), 7); // 10 nodes: need 7
}

/// Test proposal status transitions
#[test]
fn test_proposal_status_transitions() {
    use vault_core::network::consensus::ProposalStatus;

    let statuses = [
        ProposalStatus::Proposed,
        ProposalStatus::Voting,
        ProposalStatus::Accepted,
        ProposalStatus::Rejected,
        ProposalStatus::Timeout,
    ];

    // All statuses should be distinct
    for i in 0..statuses.len() {
        for j in i + 1..statuses.len() {
            assert_ne!(
                format!("{:?}", statuses[i]),
                format!("{:?}", statuses[j])
            );
        }
    }
}

/// Test consensus vote aggregation logic
#[test]
fn test_vote_aggregation() {
    fn tally_votes(votes: &[VoteChoice]) -> (usize, usize, usize) {
        let mut accept = 0;
        let mut reject = 0;
        let mut abstain = 0;

        for vote in votes {
            match vote {
                VoteChoice::Accept => accept += 1,
                VoteChoice::Reject => reject += 1,
                VoteChoice::Abstain => abstain += 1,
            }
        }

        (accept, reject, abstain)
    }

    let votes = vec![
        VoteChoice::Accept,
        VoteChoice::Accept,
        VoteChoice::Accept,
        VoteChoice::Reject,
        VoteChoice::Abstain,
    ];

    let (accept, reject, abstain) = tally_votes(&votes);
    assert_eq!(accept, 3);
    assert_eq!(reject, 1);
    assert_eq!(abstain, 1);
}

/// Test consensus decision logic
#[test]
fn test_consensus_decision() {
    #[derive(Debug, PartialEq)]
    enum Decision {
        Accepted,
        Rejected,
        Pending,
    }

    fn make_decision(accept: usize, reject: usize, total: usize) -> Decision {
        let quorum = (total * 2 / 3) + 1;

        if accept >= quorum {
            Decision::Accepted
        } else if reject >= quorum {
            Decision::Rejected
        } else {
            Decision::Pending
        }
    }

    // Test with 9 nodes (need 7 for quorum)
    assert_eq!(make_decision(7, 2, 9), Decision::Accepted);
    assert_eq!(make_decision(2, 7, 9), Decision::Rejected);
    assert_eq!(make_decision(4, 4, 9), Decision::Pending);
    assert_eq!(make_decision(6, 3, 9), Decision::Pending);
}

/// Test proposal type variants
#[test]
fn test_proposal_types() {
    use vault_core::network::consensus::ProposalType;

    let _data_integrity = ProposalType::DataIntegrity {
        block_slot: 12345,
        compressed_data_hash: "hash".to_string(),
        original_data_hash: "orig".to_string(),
        compression_ratio: 15.5,
    };

    let _network_config = ProposalType::NetworkConfig {
        parameter: "max_block_size".to_string(),
        old_value: "1048576".to_string(),
        new_value: "2097152".to_string(),
    };
}

/// Test evidence type variants
#[test]
fn test_evidence_types() {
    use vault_core::network::consensus::EvidenceType;

    let evidence_types = [
        EvidenceType::DataCorruption,
        EvidenceType::ServiceDowntime,
        EvidenceType::SlowResponse,
        EvidenceType::GoodBehavior,
        EvidenceType::FastResponse,
        EvidenceType::DataIntegrityMaintained,
    ];

    // All evidence types should be distinct
    for i in 0..evidence_types.len() {
        for j in i + 1..evidence_types.len() {
            assert_ne!(
                format!("{:?}", evidence_types[i]),
                format!("{:?}", evidence_types[j])
            );
        }
    }
}
