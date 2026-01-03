//! # VAULT Staking Program
//!
//! On-chain staking for SolanaVault network validators and operators.
//!
//! ## Features
//! - Stake VAULT tokens to participate in the network
//! - Tiered staking (Bronze, Silver, Gold, Platinum)
//! - 14-day unbonding period
//! - Performance score tracking
//! - Slashing integration

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

declare_id!("VSTKxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

/// Minimum stake: 1,000 VAULT tokens (with 9 decimals)
pub const MINIMUM_STAKE: u64 = 1_000 * 1_000_000_000;

/// Unbonding period: 14 days in seconds
pub const UNBONDING_PERIOD: i64 = 14 * 24 * 60 * 60;

/// Staking tier thresholds
pub const TIER_BRONZE: u64 = 1_000 * 1_000_000_000;      // 1,000 VAULT
pub const TIER_SILVER: u64 = 10_000 * 1_000_000_000;     // 10,000 VAULT
pub const TIER_GOLD: u64 = 100_000 * 1_000_000_000;      // 100,000 VAULT
pub const TIER_PLATINUM: u64 = 1_000_000 * 1_000_000_000; // 1,000,000 VAULT

#[program]
pub mod vault_staking {
    use super::*;

    /// Initialize the staking pool
    pub fn initialize(
        ctx: Context<Initialize>,
        rewards_authority: Pubkey,
        slashing_authority: Pubkey,
    ) -> Result<()> {
        let staking_pool = &mut ctx.accounts.staking_pool;

        staking_pool.authority = ctx.accounts.authority.key();
        staking_pool.vault_mint = ctx.accounts.vault_mint.key();
        staking_pool.staking_vault = ctx.accounts.staking_vault.key();
        staking_pool.rewards_authority = rewards_authority;
        staking_pool.slashing_authority = slashing_authority;
        staking_pool.total_staked = 0;
        staking_pool.total_stakers = 0;
        staking_pool.is_initialized = true;
        staking_pool.bump = ctx.bumps.staking_pool;

        msg!("Staking pool initialized");
        Ok(())
    }

    /// Stake VAULT tokens
    pub fn stake(
        ctx: Context<Stake>,
        amount: u64,
    ) -> Result<()> {
        require!(amount >= MINIMUM_STAKE, StakingError::InsufficientStake);

        let staking_pool = &mut ctx.accounts.staking_pool;
        let stake_account = &mut ctx.accounts.stake_account;
        let clock = Clock::get()?;

        // Initialize stake account if new
        if !stake_account.is_initialized {
            stake_account.owner = ctx.accounts.staker.key();
            stake_account.staking_pool = staking_pool.key();
            stake_account.staked_amount = 0;
            stake_account.stake_timestamp = clock.unix_timestamp;
            stake_account.performance_score = 100; // Base score of 100%
            stake_account.pending_rewards = 0;
            stake_account.total_rewards_claimed = 0;
            stake_account.slash_count = 0;
            stake_account.total_slashed = 0;
            stake_account.is_initialized = true;
            stake_account.bump = ctx.bumps.stake_account;
            stake_account.unbonding_amount = 0;
            stake_account.unbonding_end = 0;

            staking_pool.total_stakers += 1;
        }

        // Transfer tokens to staking vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.staker_token_account.to_account_info(),
            to: ctx.accounts.staking_vault.to_account_info(),
            authority: ctx.accounts.staker.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        // Update stake account
        stake_account.staked_amount = stake_account.staked_amount
            .checked_add(amount)
            .ok_or(StakingError::ArithmeticOverflow)?;

        // Update pool totals
        staking_pool.total_staked = staking_pool.total_staked
            .checked_add(amount)
            .ok_or(StakingError::ArithmeticOverflow)?;

        emit!(Staked {
            staker: ctx.accounts.staker.key(),
            amount,
            total_staked: stake_account.staked_amount,
            tier: stake_account.get_tier(),
        });

        Ok(())
    }

    /// Request unstaking (starts unbonding period)
    pub fn request_unstake(
        ctx: Context<RequestUnstake>,
        amount: u64,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let staking_pool = &mut ctx.accounts.staking_pool;
        let clock = Clock::get()?;

        require!(
            amount <= stake_account.staked_amount,
            StakingError::InsufficientStake
        );

        // Check if there's already an unbonding request
        require!(
            stake_account.unbonding_amount == 0,
            StakingError::UnbondingInProgress
        );

        // Check minimum remaining stake
        let remaining = stake_account.staked_amount.saturating_sub(amount);
        require!(
            remaining == 0 || remaining >= MINIMUM_STAKE,
            StakingError::BelowMinimumStake
        );

        // Start unbonding
        stake_account.staked_amount = stake_account.staked_amount
            .checked_sub(amount)
            .ok_or(StakingError::ArithmeticOverflow)?;
        stake_account.unbonding_amount = amount;
        stake_account.unbonding_end = clock.unix_timestamp + UNBONDING_PERIOD;

        // Update pool totals
        staking_pool.total_staked = staking_pool.total_staked
            .checked_sub(amount)
            .ok_or(StakingError::ArithmeticOverflow)?;

        emit!(UnstakeRequested {
            staker: ctx.accounts.staker.key(),
            amount,
            unbonding_end: stake_account.unbonding_end,
        });

        Ok(())
    }

    /// Complete unstaking (after unbonding period)
    pub fn complete_unstake(
        ctx: Context<CompleteUnstake>,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let staking_pool = &ctx.accounts.staking_pool;
        let clock = Clock::get()?;

        require!(
            stake_account.unbonding_amount > 0,
            StakingError::NoUnbondingRequest
        );

        require!(
            clock.unix_timestamp >= stake_account.unbonding_end,
            StakingError::UnbondingNotComplete
        );

        let amount = stake_account.unbonding_amount;

        // Transfer tokens back to staker
        let pool_key = staking_pool.key();
        let seeds = &[
            b"staking_pool".as_ref(),
            pool_key.as_ref(),
            &[staking_pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.staking_vault.to_account_info(),
            to: ctx.accounts.staker_token_account.to_account_info(),
            authority: ctx.accounts.staking_pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        // Reset unbonding state
        stake_account.unbonding_amount = 0;
        stake_account.unbonding_end = 0;

        emit!(UnstakeCompleted {
            staker: ctx.accounts.staker.key(),
            amount,
        });

        Ok(())
    }

    /// Update performance score (called by rewards authority)
    pub fn update_performance(
        ctx: Context<UpdatePerformance>,
        new_score: u8,
    ) -> Result<()> {
        require!(new_score <= 200, StakingError::InvalidPerformanceScore);

        let stake_account = &mut ctx.accounts.stake_account;
        let old_score = stake_account.performance_score;
        stake_account.performance_score = new_score;

        emit!(PerformanceUpdated {
            staker: stake_account.owner,
            old_score,
            new_score,
        });

        Ok(())
    }

    /// Apply slashing (called by slashing authority)
    pub fn apply_slash(
        ctx: Context<ApplySlash>,
        amount: u64,
        reason: SlashReason,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let staking_pool = &mut ctx.accounts.staking_pool;

        // Calculate actual slash amount (can't slash more than staked)
        let actual_slash = amount.min(stake_account.staked_amount);

        // Reduce stake
        stake_account.staked_amount = stake_account.staked_amount
            .checked_sub(actual_slash)
            .ok_or(StakingError::ArithmeticOverflow)?;
        stake_account.slash_count += 1;
        stake_account.total_slashed = stake_account.total_slashed
            .checked_add(actual_slash)
            .ok_or(StakingError::ArithmeticOverflow)?;

        // Reduce pool total
        staking_pool.total_staked = staking_pool.total_staked
            .checked_sub(actual_slash)
            .ok_or(StakingError::ArithmeticOverflow)?;

        // Burn slashed tokens (transfer to burn address)
        // Note: In production, this would call the token burn instruction

        emit!(Slashed {
            staker: stake_account.owner,
            amount: actual_slash,
            reason,
            remaining_stake: stake_account.staked_amount,
        });

        Ok(())
    }

    /// Add pending rewards (called by rewards program)
    pub fn add_pending_rewards(
        ctx: Context<AddPendingRewards>,
        amount: u64,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;

        stake_account.pending_rewards = stake_account.pending_rewards
            .checked_add(amount)
            .ok_or(StakingError::ArithmeticOverflow)?;

        emit!(RewardsAdded {
            staker: stake_account.owner,
            amount,
            total_pending: stake_account.pending_rewards,
        });

        Ok(())
    }

    /// Claim pending rewards
    pub fn claim_rewards(
        ctx: Context<ClaimRewards>,
    ) -> Result<()> {
        let stake_account = &mut ctx.accounts.stake_account;
        let staking_pool = &ctx.accounts.staking_pool;

        require!(
            stake_account.pending_rewards > 0,
            StakingError::NoRewardsToClaim
        );

        let amount = stake_account.pending_rewards;

        // Transfer rewards from rewards vault
        // Note: In production, this would integrate with the rewards program
        let pool_key = staking_pool.key();
        let seeds = &[
            b"staking_pool".as_ref(),
            pool_key.as_ref(),
            &[staking_pool.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.rewards_vault.to_account_info(),
            to: ctx.accounts.staker_token_account.to_account_info(),
            authority: ctx.accounts.staking_pool.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, amount)?;

        // Update stake account
        stake_account.pending_rewards = 0;
        stake_account.total_rewards_claimed = stake_account.total_rewards_claimed
            .checked_add(amount)
            .ok_or(StakingError::ArithmeticOverflow)?;

        emit!(RewardsClaimed {
            staker: ctx.accounts.staker.key(),
            amount,
            total_claimed: stake_account.total_rewards_claimed,
        });

        Ok(())
    }

    /// Get stake information
    pub fn get_stake_info(ctx: Context<GetStakeInfo>) -> Result<StakeInfo> {
        let stake_account = &ctx.accounts.stake_account;

        Ok(StakeInfo {
            owner: stake_account.owner,
            staked_amount: stake_account.staked_amount,
            tier: stake_account.get_tier(),
            tier_multiplier: stake_account.get_tier_multiplier(),
            performance_score: stake_account.performance_score,
            pending_rewards: stake_account.pending_rewards,
            unbonding_amount: stake_account.unbonding_amount,
            unbonding_end: stake_account.unbonding_end,
            slash_count: stake_account.slash_count,
        })
    }
}

// ============================================================================
// Account Structures
// ============================================================================

#[account]
#[derive(Default)]
pub struct StakingPool {
    /// Pool authority
    pub authority: Pubkey,
    /// VAULT token mint
    pub vault_mint: Pubkey,
    /// Staking vault (holds staked tokens)
    pub staking_vault: Pubkey,
    /// Rewards authority (rewards program)
    pub rewards_authority: Pubkey,
    /// Slashing authority (slashing program)
    pub slashing_authority: Pubkey,
    /// Total tokens staked
    pub total_staked: u64,
    /// Total number of stakers
    pub total_stakers: u64,
    /// Whether initialized
    pub is_initialized: bool,
    /// PDA bump
    pub bump: u8,
}

impl StakingPool {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // vault_mint
        32 + // staking_vault
        32 + // rewards_authority
        32 + // slashing_authority
        8 +  // total_staked
        8 +  // total_stakers
        1 +  // is_initialized
        1;   // bump
}

#[account]
#[derive(Default)]
pub struct StakeAccount {
    /// Owner of this stake
    pub owner: Pubkey,
    /// Associated staking pool
    pub staking_pool: Pubkey,
    /// Amount currently staked
    pub staked_amount: u64,
    /// Timestamp of initial stake
    pub stake_timestamp: i64,
    /// Performance score (0-200, 100 = baseline)
    pub performance_score: u8,
    /// Pending rewards to claim
    pub pending_rewards: u64,
    /// Total rewards claimed
    pub total_rewards_claimed: u64,
    /// Number of times slashed
    pub slash_count: u8,
    /// Total amount slashed
    pub total_slashed: u64,
    /// Whether initialized
    pub is_initialized: bool,
    /// PDA bump
    pub bump: u8,
    /// Amount currently unbonding
    pub unbonding_amount: u64,
    /// Unbonding end timestamp
    pub unbonding_end: i64,
}

impl StakeAccount {
    pub const SIZE: usize = 8 + // discriminator
        32 + // owner
        32 + // staking_pool
        8 +  // staked_amount
        8 +  // stake_timestamp
        1 +  // performance_score
        8 +  // pending_rewards
        8 +  // total_rewards_claimed
        1 +  // slash_count
        8 +  // total_slashed
        1 +  // is_initialized
        1 +  // bump
        8 +  // unbonding_amount
        8;   // unbonding_end

    /// Get staking tier based on staked amount
    pub fn get_tier(&self) -> StakingTier {
        if self.staked_amount >= TIER_PLATINUM {
            StakingTier::Platinum
        } else if self.staked_amount >= TIER_GOLD {
            StakingTier::Gold
        } else if self.staked_amount >= TIER_SILVER {
            StakingTier::Silver
        } else {
            StakingTier::Bronze
        }
    }

    /// Get reward multiplier based on tier
    pub fn get_tier_multiplier(&self) -> u8 {
        match self.get_tier() {
            StakingTier::Bronze => 100,   // 1.0x
            StakingTier::Silver => 120,   // 1.2x
            StakingTier::Gold => 150,     // 1.5x
            StakingTier::Platinum => 200, // 2.0x
        }
    }
}

// ============================================================================
// Enums
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum StakingTier {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum SlashReason {
    DataUnavailability,
    DataCorruption,
    ExtendedDowntime,
    DoubleVoting,
    InvalidProposal,
    MaliciousBehavior,
}

// ============================================================================
// Instruction Contexts
// ============================================================================

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// CHECK: VAULT token mint
    pub vault_mint: AccountInfo<'info>,

    #[account(
        init,
        payer = authority,
        token::mint = vault_mint,
        token::authority = staking_pool,
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        space = StakingPool::SIZE,
        seeds = [b"staking_pool", vault_mint.key().as_ref()],
        bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking_pool", staking_pool.vault_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        init_if_needed,
        payer = staker,
        space = StakeAccount::SIZE,
        seeds = [b"stake_account", staking_pool.key().as_ref(), staker.key().as_ref()],
        bump,
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        constraint = staking_vault.key() == staking_pool.staking_vault @ StakingError::InvalidVault
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = staker_token_account.owner == staker.key() @ StakingError::InvalidTokenAccount
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RequestUnstake<'info> {
    pub staker: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking_pool", staking_pool.vault_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"stake_account", staking_pool.key().as_ref(), staker.key().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == staker.key() @ StakingError::Unauthorized
    )]
    pub stake_account: Account<'info, StakeAccount>,
}

#[derive(Accounts)]
pub struct CompleteUnstake<'info> {
    pub staker: Signer<'info>,

    #[account(
        seeds = [b"staking_pool", staking_pool.vault_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"stake_account", staking_pool.key().as_ref(), staker.key().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == staker.key() @ StakingError::Unauthorized
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        constraint = staking_vault.key() == staking_pool.staking_vault @ StakingError::InvalidVault
    )]
    pub staking_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = staker_token_account.owner == staker.key() @ StakingError::InvalidTokenAccount
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdatePerformance<'info> {
    #[account(
        constraint = authority.key() == staking_pool.rewards_authority @ StakingError::Unauthorized
    )]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"staking_pool", staking_pool.vault_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        constraint = stake_account.staking_pool == staking_pool.key() @ StakingError::InvalidPool
    )]
    pub stake_account: Account<'info, StakeAccount>,
}

#[derive(Accounts)]
pub struct ApplySlash<'info> {
    #[account(
        constraint = authority.key() == staking_pool.slashing_authority @ StakingError::Unauthorized
    )]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"staking_pool", staking_pool.vault_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        constraint = stake_account.staking_pool == staking_pool.key() @ StakingError::InvalidPool
    )]
    pub stake_account: Account<'info, StakeAccount>,
}

#[derive(Accounts)]
pub struct AddPendingRewards<'info> {
    #[account(
        constraint = authority.key() == staking_pool.rewards_authority @ StakingError::Unauthorized
    )]
    pub authority: Signer<'info>,

    #[account(
        seeds = [b"staking_pool", staking_pool.vault_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        constraint = stake_account.staking_pool == staking_pool.key() @ StakingError::InvalidPool
    )]
    pub stake_account: Account<'info, StakeAccount>,
}

#[derive(Accounts)]
pub struct ClaimRewards<'info> {
    pub staker: Signer<'info>,

    #[account(
        seeds = [b"staking_pool", staking_pool.vault_mint.as_ref()],
        bump = staking_pool.bump,
    )]
    pub staking_pool: Account<'info, StakingPool>,

    #[account(
        mut,
        seeds = [b"stake_account", staking_pool.key().as_ref(), staker.key().as_ref()],
        bump = stake_account.bump,
        constraint = stake_account.owner == staker.key() @ StakingError::Unauthorized
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(mut)]
    pub rewards_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = staker_token_account.owner == staker.key() @ StakingError::InvalidTokenAccount
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct GetStakeInfo<'info> {
    pub stake_account: Account<'info, StakeAccount>,
}

// ============================================================================
// Return Types
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct StakeInfo {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub tier: StakingTier,
    pub tier_multiplier: u8,
    pub performance_score: u8,
    pub pending_rewards: u64,
    pub unbonding_amount: u64,
    pub unbonding_end: i64,
    pub slash_count: u8,
}

// ============================================================================
// Events
// ============================================================================

#[event]
pub struct Staked {
    pub staker: Pubkey,
    pub amount: u64,
    pub total_staked: u64,
    pub tier: StakingTier,
}

#[event]
pub struct UnstakeRequested {
    pub staker: Pubkey,
    pub amount: u64,
    pub unbonding_end: i64,
}

#[event]
pub struct UnstakeCompleted {
    pub staker: Pubkey,
    pub amount: u64,
}

#[event]
pub struct PerformanceUpdated {
    pub staker: Pubkey,
    pub old_score: u8,
    pub new_score: u8,
}

#[event]
pub struct Slashed {
    pub staker: Pubkey,
    pub amount: u64,
    pub reason: SlashReason,
    pub remaining_stake: u64,
}

#[event]
pub struct RewardsAdded {
    pub staker: Pubkey,
    pub amount: u64,
    pub total_pending: u64,
}

#[event]
pub struct RewardsClaimed {
    pub staker: Pubkey,
    pub amount: u64,
    pub total_claimed: u64,
}

// ============================================================================
// Errors
// ============================================================================

#[error_code]
pub enum StakingError {
    #[msg("Stake amount is below minimum required")]
    InsufficientStake,

    #[msg("Would leave stake below minimum")]
    BelowMinimumStake,

    #[msg("Unbonding already in progress")]
    UnbondingInProgress,

    #[msg("No unbonding request found")]
    NoUnbondingRequest,

    #[msg("Unbonding period not complete")]
    UnbondingNotComplete,

    #[msg("Invalid performance score (must be 0-200)")]
    InvalidPerformanceScore,

    #[msg("No rewards to claim")]
    NoRewardsToClaim,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Invalid vault")]
    InvalidVault,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Invalid staking pool")]
    InvalidPool,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
