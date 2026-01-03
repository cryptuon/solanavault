//! # VAULT Governance Program
//!
//! Token-weighted DAO governance for SolanaVault protocol parameters.
//!
//! ## Features
//! - Stake-weighted voting power
//! - Time-based voting power multiplier
//! - Multiple proposal types (parameter, treasury, upgrade)
//! - Quorum and approval thresholds
//! - Timelock for execution

use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111115");

/// Voting period: 3 days
pub const VOTING_PERIOD: i64 = 3 * 24 * 60 * 60;

/// Quorum thresholds by proposal type (percentage of total staked)
pub const QUORUM_PARAMETER_CHANGE: u8 = 10;
pub const QUORUM_TREASURY_ALLOCATION: u8 = 20;
pub const QUORUM_PROTOCOL_UPGRADE: u8 = 30;
pub const QUORUM_EMERGENCY: u8 = 5;

/// Approval thresholds (percentage of votes)
pub const APPROVAL_PARAMETER_CHANGE: u8 = 50;
pub const APPROVAL_TREASURY_ALLOCATION: u8 = 66;
pub const APPROVAL_PROTOCOL_UPGRADE: u8 = 75;
pub const APPROVAL_EMERGENCY: u8 = 90;

/// Timelock periods
pub const TIMELOCK_PARAMETER: i64 = 48 * 60 * 60;  // 48 hours
pub const TIMELOCK_TREASURY: i64 = 7 * 24 * 60 * 60;  // 7 days
pub const TIMELOCK_UPGRADE: i64 = 14 * 24 * 60 * 60;  // 14 days
pub const TIMELOCK_EMERGENCY: i64 = 6 * 60 * 60;  // 6 hours

/// Minimum stake to create proposal
pub const MIN_PROPOSAL_STAKE: u64 = 10_000 * 1_000_000_000; // 10,000 VAULT

#[program]
pub mod vault_governance {
    use super::*;

    /// Initialize governance
    pub fn initialize(
        ctx: Context<Initialize>,
        staking_program: Pubkey,
    ) -> Result<()> {
        let governance_config = &mut ctx.accounts.governance_config;

        governance_config.authority = ctx.accounts.authority.key();
        governance_config.staking_program = staking_program;
        governance_config.proposal_count = 0;
        governance_config.total_staked_snapshot = 0;
        governance_config.is_initialized = true;
        governance_config.bump = ctx.bumps.governance_config;

        msg!("Governance initialized");
        Ok(())
    }

    /// Create a new proposal
    pub fn create_proposal(
        ctx: Context<CreateProposal>,
        proposal_type: ProposalType,
        title: [u8; 64],
        title_len: u8,
        description_hash: [u8; 32],
        action_programs: Vec<Pubkey>,
    ) -> Result<()> {
        require!(title_len <= 64, GovernanceError::TitleTooLong);
        require!(action_programs.len() <= 5, GovernanceError::TooManyActions);

        // Verify proposer has minimum stake
        let proposer_stake = ctx.accounts.proposer_stake.staked_amount;
        require!(
            proposer_stake >= MIN_PROPOSAL_STAKE,
            GovernanceError::InsufficientStake
        );

        let governance_config = &mut ctx.accounts.governance_config;
        let proposal = &mut ctx.accounts.proposal;
        let clock = Clock::get()?;

        proposal.id = governance_config.proposal_count;
        proposal.proposer = ctx.accounts.proposer.key();
        proposal.proposal_type = proposal_type.clone();
        proposal.title = title;
        proposal.title_len = title_len;
        proposal.description_hash = description_hash;
        proposal.action_count = action_programs.len() as u8;

        // Copy action programs to fixed array
        let mut action_array = [Pubkey::default(); 5];
        for (i, program) in action_programs.iter().enumerate() {
            action_array[i] = *program;
        }
        proposal.action_programs = action_array;

        proposal.created_at = clock.unix_timestamp;
        proposal.voting_ends_at = clock.unix_timestamp + VOTING_PERIOD;
        proposal.votes_for = 0;
        proposal.votes_against = 0;
        proposal.votes_abstain = 0;
        proposal.total_voted = 0;
        proposal.status = ProposalStatus::Active;
        proposal.executed = false;
        proposal.bump = ctx.bumps.proposal;

        // Set thresholds based on proposal type
        let (quorum, approval, timelock) = match proposal_type {
            ProposalType::ParameterChange => (
                QUORUM_PARAMETER_CHANGE,
                APPROVAL_PARAMETER_CHANGE,
                TIMELOCK_PARAMETER,
            ),
            ProposalType::TreasuryAllocation => (
                QUORUM_TREASURY_ALLOCATION,
                APPROVAL_TREASURY_ALLOCATION,
                TIMELOCK_TREASURY,
            ),
            ProposalType::ProtocolUpgrade => (
                QUORUM_PROTOCOL_UPGRADE,
                APPROVAL_PROTOCOL_UPGRADE,
                TIMELOCK_UPGRADE,
            ),
            ProposalType::EmergencyAction => (
                QUORUM_EMERGENCY,
                APPROVAL_EMERGENCY,
                TIMELOCK_EMERGENCY,
            ),
        };

        proposal.quorum_threshold = quorum;
        proposal.approval_threshold = approval;
        proposal.timelock_period = timelock;

        // Snapshot total staked for quorum calculation
        proposal.total_staked_snapshot = governance_config.total_staked_snapshot;

        governance_config.proposal_count += 1;

        emit!(ProposalCreated {
            id: proposal.id,
            proposer: proposal.proposer,
            proposal_type,
            voting_ends_at: proposal.voting_ends_at,
        });

        Ok(())
    }

    /// Cast a vote on a proposal
    pub fn cast_vote(
        ctx: Context<CastVote>,
        vote_type: VoteType,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let vote_record = &mut ctx.accounts.vote_record;
        let clock = Clock::get()?;

        // Check proposal is still active
        require!(
            proposal.status == ProposalStatus::Active,
            GovernanceError::ProposalNotActive
        );

        require!(
            clock.unix_timestamp < proposal.voting_ends_at,
            GovernanceError::VotingEnded
        );

        // Check voter hasn't already voted
        require!(!vote_record.has_voted, GovernanceError::AlreadyVoted);

        // Get voting power from stake
        let stake_amount = ctx.accounts.voter_stake.staked_amount;
        let stake_time = clock.unix_timestamp - ctx.accounts.voter_stake.stake_timestamp;

        // Calculate time multiplier (longer stake = more voting power)
        let time_multiplier = calculate_time_multiplier(stake_time);
        let voting_power = (stake_amount as u128)
            .checked_mul(time_multiplier as u128)
            .unwrap()
            .checked_div(100)
            .unwrap() as u64;

        // Record vote
        match vote_type {
            VoteType::For => proposal.votes_for += voting_power,
            VoteType::Against => proposal.votes_against += voting_power,
            VoteType::Abstain => proposal.votes_abstain += voting_power,
        }

        proposal.total_voted += voting_power;

        // Update vote record
        vote_record.proposal = proposal.key();
        vote_record.voter = ctx.accounts.voter.key();
        vote_record.vote_type = vote_type.clone();
        vote_record.voting_power = voting_power;
        vote_record.has_voted = true;
        vote_record.voted_at = clock.unix_timestamp;

        emit!(VoteCast {
            proposal_id: proposal.id,
            voter: ctx.accounts.voter.key(),
            vote_type,
            voting_power,
        });

        Ok(())
    }

    /// Finalize a proposal after voting ends
    pub fn finalize_proposal(
        ctx: Context<FinalizeProposal>,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let clock = Clock::get()?;

        require!(
            proposal.status == ProposalStatus::Active,
            GovernanceError::ProposalNotActive
        );

        require!(
            clock.unix_timestamp >= proposal.voting_ends_at,
            GovernanceError::VotingNotEnded
        );

        // Check quorum
        let quorum_votes = (proposal.total_staked_snapshot as u128)
            .checked_mul(proposal.quorum_threshold as u128)
            .unwrap()
            .checked_div(100)
            .unwrap() as u64;

        if proposal.total_voted < quorum_votes {
            proposal.status = ProposalStatus::Defeated;
            emit!(ProposalFinalized {
                id: proposal.id,
                status: ProposalStatus::Defeated,
                approval_percentage: 0,
            });
            return Ok(());
        }

        // Check approval threshold
        let total_non_abstain = proposal.votes_for + proposal.votes_against;
        if total_non_abstain == 0 {
            proposal.status = ProposalStatus::Defeated;
            return Ok(());
        }

        let approval_percentage = (proposal.votes_for as u128)
            .checked_mul(100)
            .unwrap()
            .checked_div(total_non_abstain as u128)
            .unwrap() as u8;

        if approval_percentage >= proposal.approval_threshold {
            proposal.status = ProposalStatus::Succeeded;
            proposal.execution_eta = clock.unix_timestamp + proposal.timelock_period;

            emit!(ProposalFinalized {
                id: proposal.id,
                status: ProposalStatus::Succeeded,
                approval_percentage,
            });
        } else {
            proposal.status = ProposalStatus::Defeated;

            emit!(ProposalFinalized {
                id: proposal.id,
                status: ProposalStatus::Defeated,
                approval_percentage,
            });
        }

        Ok(())
    }

    /// Execute a successful proposal after timelock
    pub fn execute_proposal(
        ctx: Context<ExecuteProposal>,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;
        let clock = Clock::get()?;

        require!(
            proposal.status == ProposalStatus::Succeeded,
            GovernanceError::ProposalNotSucceeded
        );

        require!(!proposal.executed, GovernanceError::AlreadyExecuted);

        require!(
            clock.unix_timestamp >= proposal.execution_eta,
            GovernanceError::TimelockNotExpired
        );

        // Execute actions
        // In a real implementation, this would use CPI to call other programs
        // For now, we just mark as executed

        proposal.executed = true;
        proposal.status = ProposalStatus::Executed;
        proposal.executed_at = clock.unix_timestamp;

        emit!(ProposalExecuted {
            id: proposal.id,
            executed_at: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Cancel a proposal (only proposer or emergency)
    pub fn cancel_proposal(
        ctx: Context<CancelProposal>,
    ) -> Result<()> {
        let proposal = &mut ctx.accounts.proposal;

        require!(
            proposal.status == ProposalStatus::Active ||
            proposal.status == ProposalStatus::Succeeded,
            GovernanceError::CannotCancel
        );

        require!(
            ctx.accounts.canceller.key() == proposal.proposer ||
            ctx.accounts.canceller.key() == ctx.accounts.governance_config.authority,
            GovernanceError::Unauthorized
        );

        proposal.status = ProposalStatus::Cancelled;

        emit!(ProposalCancelled {
            id: proposal.id,
            cancelled_by: ctx.accounts.canceller.key(),
        });

        Ok(())
    }

    /// Update total staked snapshot (called by staking program)
    pub fn update_staked_snapshot(
        ctx: Context<UpdateStakedSnapshot>,
        total_staked: u64,
    ) -> Result<()> {
        let governance_config = &mut ctx.accounts.governance_config;
        governance_config.total_staked_snapshot = total_staked;

        Ok(())
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Calculate time-based voting power multiplier
fn calculate_time_multiplier(stake_seconds: i64) -> u8 {
    let months = stake_seconds / (30 * 24 * 60 * 60);

    match months {
        0..=2 => 100,     // 1.0x
        3..=5 => 125,     // 1.25x
        6..=11 => 150,    // 1.5x
        _ => 200,         // 2.0x (12+ months)
    }
}

// ============================================================================
// Account Structures
// ============================================================================

#[account]
#[derive(Default)]
pub struct GovernanceConfig {
    /// Admin authority
    pub authority: Pubkey,
    /// Staking program address
    pub staking_program: Pubkey,
    /// Total proposals created
    pub proposal_count: u64,
    /// Snapshot of total staked for quorum
    pub total_staked_snapshot: u64,
    /// Whether initialized
    pub is_initialized: bool,
    /// PDA bump
    pub bump: u8,
}

impl GovernanceConfig {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // staking_program
        8 +  // proposal_count
        8 +  // total_staked_snapshot
        1 +  // is_initialized
        1;   // bump
}

#[account]
pub struct Proposal {
    /// Proposal ID
    pub id: u64,
    /// Proposer address
    pub proposer: Pubkey,
    /// Type of proposal
    pub proposal_type: ProposalType,
    /// Title (fixed 64 bytes)
    pub title: [u8; 64],
    /// Title length
    pub title_len: u8,
    /// Description hash (IPFS CID or SHA256)
    pub description_hash: [u8; 32],
    /// Number of actions
    pub action_count: u8,
    /// Action program IDs (max 5)
    pub action_programs: [Pubkey; 5],
    /// Creation timestamp
    pub created_at: i64,
    /// Voting end timestamp
    pub voting_ends_at: i64,
    /// Votes in favor
    pub votes_for: u64,
    /// Votes against
    pub votes_against: u64,
    /// Abstain votes
    pub votes_abstain: u64,
    /// Total voting power used
    pub total_voted: u64,
    /// Total staked at proposal creation
    pub total_staked_snapshot: u64,
    /// Quorum threshold (percentage)
    pub quorum_threshold: u8,
    /// Approval threshold (percentage)
    pub approval_threshold: u8,
    /// Timelock period in seconds
    pub timelock_period: i64,
    /// Execution ETA (after timelock)
    pub execution_eta: i64,
    /// Current status
    pub status: ProposalStatus,
    /// Whether executed
    pub executed: bool,
    /// Execution timestamp
    pub executed_at: i64,
    /// PDA bump
    pub bump: u8,
}

impl Proposal {
    pub const SIZE: usize = 8 + // discriminator
        8 +   // id
        32 +  // proposer
        1 +   // proposal_type
        64 +  // title
        1 +   // title_len
        32 +  // description_hash
        1 +   // action_count
        (32 * 5) + // action_programs
        8 +   // created_at
        8 +   // voting_ends_at
        8 +   // votes_for
        8 +   // votes_against
        8 +   // votes_abstain
        8 +   // total_voted
        8 +   // total_staked_snapshot
        1 +   // quorum_threshold
        1 +   // approval_threshold
        8 +   // timelock_period
        8 +   // execution_eta
        1 +   // status
        1 +   // executed
        8 +   // executed_at
        1;    // bump
}


#[account]
#[derive(Default)]
pub struct VoteRecord {
    /// Proposal address
    pub proposal: Pubkey,
    /// Voter address
    pub voter: Pubkey,
    /// Vote type
    pub vote_type: VoteType,
    /// Voting power used
    pub voting_power: u64,
    /// Whether has voted
    pub has_voted: bool,
    /// Vote timestamp
    pub voted_at: i64,
}

impl VoteRecord {
    pub const SIZE: usize = 8 + // discriminator
        32 + // proposal
        32 + // voter
        1 +  // vote_type
        8 +  // voting_power
        1 +  // has_voted
        8;   // voted_at
}

/// Simplified stake account for reading
#[account]
pub struct VoterStakeAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub stake_timestamp: i64,
}

// ============================================================================
// Enums
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalType {
    ParameterChange,
    TreasuryAllocation,
    ProtocolUpgrade,
    EmergencyAction,
}

impl Default for ProposalType {
    fn default() -> Self {
        ProposalType::ParameterChange
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ProposalStatus {
    Active,
    Succeeded,
    Defeated,
    Executed,
    Cancelled,
}

impl Default for ProposalStatus {
    fn default() -> Self {
        ProposalStatus::Active
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum VoteType {
    For,
    Against,
    Abstain,
}

impl Default for VoteType {
    fn default() -> Self {
        VoteType::Abstain
    }
}

// ============================================================================
// Instruction Contexts
// ============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = GovernanceConfig::SIZE,
        seeds = [b"governance_config"],
        bump,
    )]
    pub governance_config: Account<'info, GovernanceConfig>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CreateProposal<'info> {
    #[account(mut)]
    pub proposer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"governance_config"],
        bump = governance_config.bump,
    )]
    pub governance_config: Account<'info, GovernanceConfig>,

    #[account(
        init,
        payer = proposer,
        space = Proposal::SIZE,
        seeds = [b"proposal", governance_config.proposal_count.to_le_bytes().as_ref()],
        bump,
    )]
    pub proposal: Account<'info, Proposal>,

    /// Proposer's stake account
    pub proposer_stake: Account<'info, VoterStakeAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct CastVote<'info> {
    #[account(mut)]
    pub voter: Signer<'info>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        init_if_needed,
        payer = voter,
        space = VoteRecord::SIZE,
        seeds = [b"vote", proposal.key().as_ref(), voter.key().as_ref()],
        bump,
    )]
    pub vote_record: Account<'info, VoteRecord>,

    /// Voter's stake account
    pub voter_stake: Account<'info, VoterStakeAccount>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct FinalizeProposal<'info> {
    #[account(mut)]
    pub proposal: Account<'info, Proposal>,
}

#[derive(Accounts)]
pub struct ExecuteProposal<'info> {
    pub executor: Signer<'info>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        seeds = [b"governance_config"],
        bump = governance_config.bump,
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
}

#[derive(Accounts)]
pub struct CancelProposal<'info> {
    pub canceller: Signer<'info>,

    #[account(mut)]
    pub proposal: Account<'info, Proposal>,

    #[account(
        seeds = [b"governance_config"],
        bump = governance_config.bump,
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
}

#[derive(Accounts)]
pub struct UpdateStakedSnapshot<'info> {
    #[account(
        constraint = authority.key() == governance_config.authority @ GovernanceError::Unauthorized
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"governance_config"],
        bump = governance_config.bump,
    )]
    pub governance_config: Account<'info, GovernanceConfig>,
}

// ============================================================================
// Events
// ============================================================================

#[event]
pub struct ProposalCreated {
    pub id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub voting_ends_at: i64,
}

#[event]
pub struct VoteCast {
    pub proposal_id: u64,
    pub voter: Pubkey,
    pub vote_type: VoteType,
    pub voting_power: u64,
}

#[event]
pub struct ProposalFinalized {
    pub id: u64,
    pub status: ProposalStatus,
    pub approval_percentage: u8,
}

#[event]
pub struct ProposalExecuted {
    pub id: u64,
    pub executed_at: i64,
}

#[event]
pub struct ProposalCancelled {
    pub id: u64,
    pub cancelled_by: Pubkey,
}

// ============================================================================
// Errors
// ============================================================================

#[error_code]
pub enum GovernanceError {
    #[msg("Title too long (max 64 bytes)")]
    TitleTooLong,

    #[msg("Too many actions (max 5)")]
    TooManyActions,

    #[msg("Insufficient stake to create proposal")]
    InsufficientStake,

    #[msg("Proposal is not active")]
    ProposalNotActive,

    #[msg("Voting period has ended")]
    VotingEnded,

    #[msg("Voting period has not ended")]
    VotingNotEnded,

    #[msg("Already voted on this proposal")]
    AlreadyVoted,

    #[msg("Proposal did not succeed")]
    ProposalNotSucceeded,

    #[msg("Proposal already executed")]
    AlreadyExecuted,

    #[msg("Timelock period not expired")]
    TimelockNotExpired,

    #[msg("Cannot cancel this proposal")]
    CannotCancel,

    #[msg("Unauthorized")]
    Unauthorized,
}
