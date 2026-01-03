//! # VAULT Rewards Program
//!
//! Epoch-based reward distribution for SolanaVault network participants.
//!
//! ## Features
//! - Epoch-based reward distribution (24-hour epochs)
//! - Performance-weighted rewards
//! - Integration with staking program
//! - Fee distribution (95% gateway, 5% network fund)
//! - Automatic emission from token program

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, MintTo, Mint};

declare_id!("11111111111111111111111111111114");

/// Epoch duration: 24 hours in seconds
pub const EPOCH_DURATION: i64 = 24 * 60 * 60;

/// Gateway fee share: 95%
pub const GATEWAY_FEE_SHARE: u64 = 95;

/// Network fund share: 5%
pub const NETWORK_FUND_SHARE: u64 = 5;

/// Minimum epoch rewards to trigger distribution
pub const MIN_EPOCH_REWARDS: u64 = 1_000_000_000; // 1 VAULT

#[program]
pub mod vault_rewards {
    use super::*;

    /// Initialize the rewards system
    pub fn initialize(
        ctx: Context<Initialize>,
        staking_program: Pubkey,
        token_program_id: Pubkey,
    ) -> Result<()> {
        let rewards_config = &mut ctx.accounts.rewards_config;

        rewards_config.authority = ctx.accounts.authority.key();
        rewards_config.vault_mint = ctx.accounts.vault_mint.key();
        rewards_config.rewards_vault = ctx.accounts.rewards_vault.key();
        rewards_config.network_fund = ctx.accounts.network_fund.key();
        rewards_config.staking_program = staking_program;
        rewards_config.token_program_id = token_program_id;
        rewards_config.current_epoch = 0;
        rewards_config.epoch_start_time = Clock::get()?.unix_timestamp;
        rewards_config.total_distributed = 0;
        rewards_config.total_fees_collected = 0;
        rewards_config.is_initialized = true;
        rewards_config.bump = ctx.bumps.rewards_config;

        msg!("Rewards system initialized");
        Ok(())
    }

    /// Advance to next epoch and trigger distribution
    pub fn advance_epoch(
        ctx: Context<AdvanceEpoch>,
    ) -> Result<()> {
        let rewards_config = &mut ctx.accounts.rewards_config;
        let clock = Clock::get()?;

        // Check if epoch duration has passed
        let elapsed = clock.unix_timestamp - rewards_config.epoch_start_time;
        require!(elapsed >= EPOCH_DURATION, RewardsError::EpochNotComplete);

        // Record epoch completion
        let epoch_record = &mut ctx.accounts.epoch_record;
        epoch_record.epoch = rewards_config.current_epoch;
        epoch_record.start_time = rewards_config.epoch_start_time;
        epoch_record.end_time = clock.unix_timestamp;
        epoch_record.total_rewards = ctx.accounts.rewards_vault.amount;
        epoch_record.total_stakers = 0; // Will be updated during distribution
        epoch_record.is_distributed = false;
        epoch_record.bump = ctx.bumps.epoch_record;

        // Advance to next epoch
        rewards_config.current_epoch += 1;
        rewards_config.epoch_start_time = clock.unix_timestamp;

        emit!(EpochAdvanced {
            epoch: rewards_config.current_epoch,
            timestamp: clock.unix_timestamp,
        });

        Ok(())
    }

    /// Record fee payment (called when gateway serves a request)
    pub fn record_fee(
        ctx: Context<RecordFee>,
        amount: u64,
        gateway_operator: Pubkey,
    ) -> Result<()> {
        // Calculate distribution
        let gateway_share = (amount * GATEWAY_FEE_SHARE) / 100;
        let network_share = amount - gateway_share;

        // Extract needed values before CPI calls
        let bump = ctx.accounts.rewards_config.bump;
        let seeds = &[
            b"rewards_config".as_ref(),
            &[bump],
        ];
        let signer = &[&seeds[..]];

        // Transfer to gateway operator
        let cpi_accounts = Transfer {
            from: ctx.accounts.fee_vault.to_account_info(),
            to: ctx.accounts.gateway_token_account.to_account_info(),
            authority: ctx.accounts.rewards_config.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, gateway_share)?;

        // Transfer to network fund
        let cpi_accounts_fund = Transfer {
            from: ctx.accounts.fee_vault.to_account_info(),
            to: ctx.accounts.network_fund.to_account_info(),
            authority: ctx.accounts.rewards_config.to_account_info(),
        };
        let cpi_ctx_fund = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            cpi_accounts_fund,
            signer,
        );
        token::transfer(cpi_ctx_fund, network_share)?;

        // Now get mutable references for state updates
        let rewards_config = &mut ctx.accounts.rewards_config;
        let fee_record = &mut ctx.accounts.fee_record;

        // Update fee record
        fee_record.total_fees += amount;
        fee_record.gateway_earnings += gateway_share;
        fee_record.network_fund_earnings += network_share;
        fee_record.transaction_count += 1;

        // Update global stats
        rewards_config.total_fees_collected += amount;

        emit!(FeeRecorded {
            amount,
            gateway_operator,
            gateway_share,
            network_share,
        });

        Ok(())
    }

    /// Distribute epoch rewards to a staker
    pub fn distribute_staker_reward(
        ctx: Context<DistributeStakerReward>,
        epoch: u64,
    ) -> Result<()> {
        let rewards_config = &ctx.accounts.rewards_config;
        let epoch_record = &mut ctx.accounts.epoch_record;
        let staker_reward = &mut ctx.accounts.staker_reward;

        require!(epoch_record.epoch == epoch, RewardsError::InvalidEpoch);
        require!(!staker_reward.is_claimed, RewardsError::AlreadyClaimed);

        // Calculate reward based on stake and performance
        // This would normally read from the staking program via CPI
        let stake_amount = ctx.accounts.stake_info.staked_amount;
        let performance_score = ctx.accounts.stake_info.performance_score;
        let tier_multiplier = ctx.accounts.stake_info.tier_multiplier;

        // Calculate proportional reward
        // reward = (stake / total_staked) * epoch_rewards * (performance/100) * (tier/100)
        let base_reward = if epoch_record.total_staked > 0 {
            (epoch_record.total_rewards as u128)
                .checked_mul(stake_amount as u128)
                .unwrap()
                .checked_div(epoch_record.total_staked as u128)
                .unwrap() as u64
        } else {
            0
        };

        let performance_adjusted = (base_reward as u128)
            .checked_mul(performance_score as u128)
            .unwrap()
            .checked_div(100)
            .unwrap() as u64;

        let final_reward = (performance_adjusted as u128)
            .checked_mul(tier_multiplier as u128)
            .unwrap()
            .checked_div(100)
            .unwrap() as u64;

        // Transfer reward
        let config_key = rewards_config.key();
        let seeds = &[
            b"rewards_config".as_ref(),
            config_key.as_ref(),
            &[rewards_config.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = Transfer {
            from: ctx.accounts.rewards_vault.to_account_info(),
            to: ctx.accounts.staker_token_account.to_account_info(),
            authority: ctx.accounts.rewards_config.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);
        token::transfer(cpi_ctx, final_reward)?;

        // Mark as claimed
        staker_reward.epoch = epoch;
        staker_reward.staker = ctx.accounts.staker.key();
        staker_reward.amount = final_reward;
        staker_reward.is_claimed = true;
        staker_reward.claimed_at = Clock::get()?.unix_timestamp;

        // Update epoch record
        epoch_record.distributed_rewards += final_reward;
        epoch_record.total_stakers += 1;

        emit!(RewardDistributed {
            epoch,
            staker: ctx.accounts.staker.key(),
            amount: final_reward,
            stake_amount,
            performance_score,
        });

        Ok(())
    }

    /// Fund rewards vault from token emissions
    pub fn fund_rewards(
        ctx: Context<FundRewards>,
        amount: u64,
    ) -> Result<()> {
        let rewards_config = &mut ctx.accounts.rewards_config;

        // Transfer tokens to rewards vault
        let cpi_accounts = Transfer {
            from: ctx.accounts.funder_token_account.to_account_info(),
            to: ctx.accounts.rewards_vault.to_account_info(),
            authority: ctx.accounts.funder.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, amount)?;

        emit!(RewardsFunded {
            amount,
            funder: ctx.accounts.funder.key(),
            total_in_vault: ctx.accounts.rewards_vault.amount + amount,
        });

        Ok(())
    }

    /// Get current epoch info
    pub fn get_epoch_info(ctx: Context<GetEpochInfo>) -> Result<EpochInfo> {
        let rewards_config = &ctx.accounts.rewards_config;
        let clock = Clock::get()?;

        let elapsed = clock.unix_timestamp - rewards_config.epoch_start_time;
        let remaining = EPOCH_DURATION - elapsed;

        Ok(EpochInfo {
            current_epoch: rewards_config.current_epoch,
            epoch_start_time: rewards_config.epoch_start_time,
            time_remaining: remaining.max(0),
            rewards_available: ctx.accounts.rewards_vault.amount,
            total_distributed: rewards_config.total_distributed,
        })
    }
}

// ============================================================================
// Account Structures
// ============================================================================

#[account]
#[derive(Default)]
pub struct RewardsConfig {
    /// Admin authority
    pub authority: Pubkey,
    /// VAULT token mint
    pub vault_mint: Pubkey,
    /// Rewards vault
    pub rewards_vault: Pubkey,
    /// Network fund account
    pub network_fund: Pubkey,
    /// Staking program address
    pub staking_program: Pubkey,
    /// Token program address
    pub token_program_id: Pubkey,
    /// Current epoch number
    pub current_epoch: u64,
    /// Current epoch start timestamp
    pub epoch_start_time: i64,
    /// Total rewards distributed all-time
    pub total_distributed: u64,
    /// Total fees collected all-time
    pub total_fees_collected: u64,
    /// Whether initialized
    pub is_initialized: bool,
    /// PDA bump
    pub bump: u8,
}

impl RewardsConfig {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        32 + // vault_mint
        32 + // rewards_vault
        32 + // network_fund
        32 + // staking_program
        32 + // token_program_id
        8 +  // current_epoch
        8 +  // epoch_start_time
        8 +  // total_distributed
        8 +  // total_fees_collected
        1 +  // is_initialized
        1;   // bump
}

#[account]
#[derive(Default)]
pub struct EpochRecord {
    /// Epoch number
    pub epoch: u64,
    /// Epoch start timestamp
    pub start_time: i64,
    /// Epoch end timestamp
    pub end_time: i64,
    /// Total rewards available for this epoch
    pub total_rewards: u64,
    /// Total staked during this epoch
    pub total_staked: u64,
    /// Rewards distributed so far
    pub distributed_rewards: u64,
    /// Number of stakers who claimed
    pub total_stakers: u64,
    /// Whether distribution is complete
    pub is_distributed: bool,
    /// PDA bump
    pub bump: u8,
}

impl EpochRecord {
    pub const SIZE: usize = 8 + // discriminator
        8 +  // epoch
        8 +  // start_time
        8 +  // end_time
        8 +  // total_rewards
        8 +  // total_staked
        8 +  // distributed_rewards
        8 +  // total_stakers
        1 +  // is_distributed
        1;   // bump
}

#[account]
#[derive(Default)]
pub struct FeeRecord {
    /// Gateway operator
    pub gateway: Pubkey,
    /// Total fees processed
    pub total_fees: u64,
    /// Earnings sent to gateway
    pub gateway_earnings: u64,
    /// Earnings sent to network fund
    pub network_fund_earnings: u64,
    /// Number of transactions
    pub transaction_count: u64,
    /// Last update timestamp
    pub last_updated: i64,
}

impl FeeRecord {
    pub const SIZE: usize = 8 + // discriminator
        32 + // gateway
        8 +  // total_fees
        8 +  // gateway_earnings
        8 +  // network_fund_earnings
        8 +  // transaction_count
        8;   // last_updated
}

#[account]
#[derive(Default)]
pub struct StakerReward {
    /// Epoch number
    pub epoch: u64,
    /// Staker address
    pub staker: Pubkey,
    /// Reward amount
    pub amount: u64,
    /// Whether claimed
    pub is_claimed: bool,
    /// Claim timestamp
    pub claimed_at: i64,
}

impl StakerReward {
    pub const SIZE: usize = 8 + // discriminator
        8 +  // epoch
        32 + // staker
        8 +  // amount
        1 +  // is_claimed
        8;   // claimed_at
}

/// Stake info passed from staking program (simplified)
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct StakeInfo {
    pub staked_amount: u64,
    pub performance_score: u8,
    pub tier_multiplier: u8,
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
        token::authority = rewards_config,
    )]
    pub rewards_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub network_fund: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = authority,
        space = RewardsConfig::SIZE,
        seeds = [b"rewards_config"],
        bump,
    )]
    pub rewards_config: Account<'info, RewardsConfig>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct AdvanceEpoch<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"rewards_config"],
        bump = rewards_config.bump,
        constraint = authority.key() == rewards_config.authority @ RewardsError::Unauthorized
    )]
    pub rewards_config: Account<'info, RewardsConfig>,

    #[account(
        init,
        payer = authority,
        space = EpochRecord::SIZE,
        seeds = [b"epoch", rewards_config.current_epoch.to_le_bytes().as_ref()],
        bump,
    )]
    pub epoch_record: Account<'info, EpochRecord>,

    pub rewards_vault: Account<'info, TokenAccount>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RecordFee<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"rewards_config"],
        bump = rewards_config.bump,
    )]
    pub rewards_config: Account<'info, RewardsConfig>,

    #[account(
        init_if_needed,
        payer = payer,
        space = FeeRecord::SIZE,
        seeds = [b"fee_record", gateway_operator.key().as_ref()],
        bump,
    )]
    pub fee_record: Account<'info, FeeRecord>,

    /// CHECK: Gateway operator pubkey
    pub gateway_operator: AccountInfo<'info>,

    #[account(mut)]
    pub fee_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub gateway_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = network_fund.key() == rewards_config.network_fund @ RewardsError::InvalidNetworkFund
    )]
    pub network_fund: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(epoch: u64)]
pub struct DistributeStakerReward<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,

    #[account(
        seeds = [b"rewards_config"],
        bump = rewards_config.bump,
    )]
    pub rewards_config: Account<'info, RewardsConfig>,

    #[account(
        mut,
        seeds = [b"epoch", epoch.to_le_bytes().as_ref()],
        bump = epoch_record.bump,
    )]
    pub epoch_record: Account<'info, EpochRecord>,

    #[account(
        init_if_needed,
        payer = staker,
        space = StakerReward::SIZE,
        seeds = [b"staker_reward", epoch.to_le_bytes().as_ref(), staker.key().as_ref()],
        bump,
    )]
    pub staker_reward: Account<'info, StakerReward>,

    /// Stake info account from staking program
    /// CHECK: Validated by staking program
    #[account()]
    pub stake_info: Account<'info, StakeInfoAccount>,

    #[account(
        mut,
        constraint = rewards_vault.key() == rewards_config.rewards_vault @ RewardsError::InvalidVault
    )]
    pub rewards_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub staker_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

/// Simplified stake info account structure for CPI
#[account]
pub struct StakeInfoAccount {
    pub owner: Pubkey,
    pub staked_amount: u64,
    pub performance_score: u8,
    pub tier_multiplier: u8,
}

#[derive(Accounts)]
pub struct FundRewards<'info> {
    #[account(mut)]
    pub funder: Signer<'info>,

    #[account(
        mut,
        seeds = [b"rewards_config"],
        bump = rewards_config.bump,
    )]
    pub rewards_config: Account<'info, RewardsConfig>,

    #[account(
        mut,
        constraint = rewards_vault.key() == rewards_config.rewards_vault @ RewardsError::InvalidVault
    )]
    pub rewards_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub funder_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct GetEpochInfo<'info> {
    #[account(
        seeds = [b"rewards_config"],
        bump = rewards_config.bump,
    )]
    pub rewards_config: Account<'info, RewardsConfig>,

    pub rewards_vault: Account<'info, TokenAccount>,
}

// ============================================================================
// Return Types
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct EpochInfo {
    pub current_epoch: u64,
    pub epoch_start_time: i64,
    pub time_remaining: i64,
    pub rewards_available: u64,
    pub total_distributed: u64,
}

// ============================================================================
// Events
// ============================================================================

#[event]
pub struct EpochAdvanced {
    pub epoch: u64,
    pub timestamp: i64,
}

#[event]
pub struct FeeRecorded {
    pub amount: u64,
    pub gateway_operator: Pubkey,
    pub gateway_share: u64,
    pub network_share: u64,
}

#[event]
pub struct RewardDistributed {
    pub epoch: u64,
    pub staker: Pubkey,
    pub amount: u64,
    pub stake_amount: u64,
    pub performance_score: u8,
}

#[event]
pub struct RewardsFunded {
    pub amount: u64,
    pub funder: Pubkey,
    pub total_in_vault: u64,
}

// ============================================================================
// Errors
// ============================================================================

#[error_code]
pub enum RewardsError {
    #[msg("Epoch duration not complete")]
    EpochNotComplete,

    #[msg("Invalid epoch")]
    InvalidEpoch,

    #[msg("Reward already claimed")]
    AlreadyClaimed,

    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Invalid vault")]
    InvalidVault,

    #[msg("Invalid network fund")]
    InvalidNetworkFund,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
