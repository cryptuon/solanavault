//! # VAULT Token Program
//!
//! SPL Token wrapper with controlled minting for the SolanaVault network.
//!
//! ## Features
//! - 1 billion total supply cap
//! - Controlled minting via mint authority
//! - Token burning for deflationary mechanics
//! - Emission schedule enforcement

use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, MintTo, Burn};

declare_id!("VTKNxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

/// Total supply cap: 1 billion tokens with 9 decimals
pub const TOTAL_SUPPLY_CAP: u64 = 1_000_000_000 * 1_000_000_000; // 1B * 10^9

/// Token decimals (same as SOL)
pub const TOKEN_DECIMALS: u8 = 9;

#[program]
pub mod vault_token {
    use super::*;

    /// Initialize the VAULT token mint and token config
    pub fn initialize(
        ctx: Context<Initialize>,
        emission_authority: Pubkey,
    ) -> Result<()> {
        let token_config = &mut ctx.accounts.token_config;

        token_config.mint = ctx.accounts.mint.key();
        token_config.mint_authority = ctx.accounts.authority.key();
        token_config.emission_authority = emission_authority;
        token_config.total_minted = 0;
        token_config.total_burned = 0;
        token_config.is_initialized = true;
        token_config.bump = ctx.bumps.token_config;

        // Set emission schedule (tokens per year, decreasing)
        token_config.emission_schedule = EmissionSchedule {
            year_1_emission: 100_000_000 * 1_000_000_000,  // 100M tokens
            year_2_emission: 80_000_000 * 1_000_000_000,   // 80M tokens
            year_3_emission: 60_000_000 * 1_000_000_000,   // 60M tokens
            year_4_plus_emission: 40_000_000 * 1_000_000_000, // 40M tokens/year
            genesis_timestamp: Clock::get()?.unix_timestamp as u64,
        };

        msg!("VAULT Token initialized with 1B supply cap");
        Ok(())
    }

    /// Mint tokens to a recipient (only callable by emission authority)
    pub fn mint_tokens(
        ctx: Context<MintTokens>,
        amount: u64,
    ) -> Result<()> {
        let token_config = &mut ctx.accounts.token_config;

        // Check total supply cap
        require!(
            token_config.total_minted.checked_add(amount).unwrap() <= TOTAL_SUPPLY_CAP,
            VaultTokenError::SupplyCapExceeded
        );

        // Check emission schedule
        let current_time = Clock::get()?.unix_timestamp as u64;
        let allowed_emission = token_config.calculate_allowed_emission(current_time)?;

        require!(
            token_config.total_minted.checked_add(amount).unwrap() <= allowed_emission,
            VaultTokenError::EmissionScheduleExceeded
        );

        // Mint tokens
        let seeds = &[
            b"token_config".as_ref(),
            &[token_config.bump],
        ];
        let signer = &[&seeds[..]];

        let cpi_accounts = MintTo {
            mint: ctx.accounts.mint.to_account_info(),
            to: ctx.accounts.recipient_token_account.to_account_info(),
            authority: ctx.accounts.token_config.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new_with_signer(cpi_program, cpi_accounts, signer);

        token::mint_to(cpi_ctx, amount)?;

        token_config.total_minted = token_config.total_minted.checked_add(amount).unwrap();

        emit!(TokensMinted {
            recipient: ctx.accounts.recipient.key(),
            amount,
            total_minted: token_config.total_minted,
        });

        Ok(())
    }

    /// Burn tokens (callable by token holder)
    pub fn burn_tokens(
        ctx: Context<BurnTokens>,
        amount: u64,
    ) -> Result<()> {
        let token_config = &mut ctx.accounts.token_config;

        let cpi_accounts = Burn {
            mint: ctx.accounts.mint.to_account_info(),
            from: ctx.accounts.token_account.to_account_info(),
            authority: ctx.accounts.owner.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

        token::burn(cpi_ctx, amount)?;

        token_config.total_burned = token_config.total_burned.checked_add(amount).unwrap();

        emit!(TokensBurned {
            owner: ctx.accounts.owner.key(),
            amount,
            total_burned: token_config.total_burned,
        });

        Ok(())
    }

    /// Update emission authority (only current emission authority)
    pub fn update_emission_authority(
        ctx: Context<UpdateEmissionAuthority>,
        new_authority: Pubkey,
    ) -> Result<()> {
        let token_config = &mut ctx.accounts.token_config;

        let old_authority = token_config.emission_authority;
        token_config.emission_authority = new_authority;

        emit!(EmissionAuthorityUpdated {
            old_authority,
            new_authority,
        });

        Ok(())
    }

    /// Get token statistics
    pub fn get_token_stats(ctx: Context<GetTokenStats>) -> Result<TokenStats> {
        let token_config = &ctx.accounts.token_config;
        let current_time = Clock::get()?.unix_timestamp as u64;

        Ok(TokenStats {
            total_supply_cap: TOTAL_SUPPLY_CAP,
            total_minted: token_config.total_minted,
            total_burned: token_config.total_burned,
            circulating_supply: token_config.total_minted.saturating_sub(token_config.total_burned),
            allowed_emission: token_config.calculate_allowed_emission(current_time)?,
            remaining_emission: TOTAL_SUPPLY_CAP.saturating_sub(token_config.total_minted),
        })
    }
}

// ============================================================================
// Account Structures
// ============================================================================

#[account]
#[derive(Default)]
pub struct TokenConfig {
    /// The VAULT token mint address
    pub mint: Pubkey,
    /// Authority that can update config
    pub mint_authority: Pubkey,
    /// Authority that can trigger emissions (rewards program)
    pub emission_authority: Pubkey,
    /// Total tokens minted to date
    pub total_minted: u64,
    /// Total tokens burned to date
    pub total_burned: u64,
    /// Whether the config is initialized
    pub is_initialized: bool,
    /// PDA bump seed
    pub bump: u8,
    /// Emission schedule
    pub emission_schedule: EmissionSchedule,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct EmissionSchedule {
    /// Year 1 emission (10% = 100M tokens)
    pub year_1_emission: u64,
    /// Year 2 emission (8% = 80M tokens)
    pub year_2_emission: u64,
    /// Year 3 emission (6% = 60M tokens)
    pub year_3_emission: u64,
    /// Year 4+ emission (4% = 40M tokens per year)
    pub year_4_plus_emission: u64,
    /// Genesis timestamp for calculating years
    pub genesis_timestamp: u64,
}

impl TokenConfig {
    pub const SIZE: usize = 8 + // discriminator
        32 + // mint
        32 + // mint_authority
        32 + // emission_authority
        8 +  // total_minted
        8 +  // total_burned
        1 +  // is_initialized
        1 +  // bump
        (8 * 5); // emission_schedule (5 u64s)

    /// Calculate the total allowed emission based on current time
    pub fn calculate_allowed_emission(&self, current_time: u64) -> Result<u64> {
        let elapsed_seconds = current_time.saturating_sub(self.emission_schedule.genesis_timestamp);
        let seconds_per_year: u64 = 365 * 24 * 60 * 60;

        let years_elapsed = elapsed_seconds / seconds_per_year;
        let partial_year = (elapsed_seconds % seconds_per_year) as f64 / seconds_per_year as f64;

        let mut total_allowed: u64 = 0;

        // Calculate full years
        for year in 0..years_elapsed {
            total_allowed = total_allowed.checked_add(match year {
                0 => self.emission_schedule.year_1_emission,
                1 => self.emission_schedule.year_2_emission,
                2 => self.emission_schedule.year_3_emission,
                _ => self.emission_schedule.year_4_plus_emission,
            }).unwrap_or(TOTAL_SUPPLY_CAP);
        }

        // Add partial year
        let current_year_emission = match years_elapsed {
            0 => self.emission_schedule.year_1_emission,
            1 => self.emission_schedule.year_2_emission,
            2 => self.emission_schedule.year_3_emission,
            _ => self.emission_schedule.year_4_plus_emission,
        };

        total_allowed = total_allowed.checked_add(
            (current_year_emission as f64 * partial_year) as u64
        ).unwrap_or(TOTAL_SUPPLY_CAP);

        // Cap at total supply
        Ok(total_allowed.min(TOTAL_SUPPLY_CAP))
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
        mint::decimals = TOKEN_DECIMALS,
        mint::authority = token_config,
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = authority,
        space = TokenConfig::SIZE,
        seeds = [b"token_config"],
        bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct MintTokens<'info> {
    #[account(
        constraint = emission_authority.key() == token_config.emission_authority @ VaultTokenError::UnauthorizedMinter
    )]
    pub emission_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_config"],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    #[account(
        mut,
        constraint = mint.key() == token_config.mint @ VaultTokenError::InvalidMint
    )]
    pub mint: Account<'info, Mint>,

    /// CHECK: Recipient pubkey for event logging
    pub recipient: AccountInfo<'info>,

    #[account(
        mut,
        constraint = recipient_token_account.mint == mint.key() @ VaultTokenError::InvalidMint,
        constraint = recipient_token_account.owner == recipient.key() @ VaultTokenError::InvalidTokenAccount,
    )]
    pub recipient_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct BurnTokens<'info> {
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_config"],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,

    #[account(
        mut,
        constraint = mint.key() == token_config.mint @ VaultTokenError::InvalidMint
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        constraint = token_account.mint == mint.key() @ VaultTokenError::InvalidMint,
        constraint = token_account.owner == owner.key() @ VaultTokenError::InvalidTokenAccount,
    )]
    pub token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

#[derive(Accounts)]
pub struct UpdateEmissionAuthority<'info> {
    #[account(
        constraint = current_authority.key() == token_config.emission_authority @ VaultTokenError::UnauthorizedMinter
    )]
    pub current_authority: Signer<'info>,

    #[account(
        mut,
        seeds = [b"token_config"],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,
}

#[derive(Accounts)]
pub struct GetTokenStats<'info> {
    #[account(
        seeds = [b"token_config"],
        bump = token_config.bump,
    )]
    pub token_config: Account<'info, TokenConfig>,
}

// ============================================================================
// Return Types
// ============================================================================

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct TokenStats {
    pub total_supply_cap: u64,
    pub total_minted: u64,
    pub total_burned: u64,
    pub circulating_supply: u64,
    pub allowed_emission: u64,
    pub remaining_emission: u64,
}

// ============================================================================
// Events
// ============================================================================

#[event]
pub struct TokensMinted {
    pub recipient: Pubkey,
    pub amount: u64,
    pub total_minted: u64,
}

#[event]
pub struct TokensBurned {
    pub owner: Pubkey,
    pub amount: u64,
    pub total_burned: u64,
}

#[event]
pub struct EmissionAuthorityUpdated {
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
}

// ============================================================================
// Errors
// ============================================================================

#[error_code]
pub enum VaultTokenError {
    #[msg("Total supply cap would be exceeded")]
    SupplyCapExceeded,

    #[msg("Emission schedule limit exceeded")]
    EmissionScheduleExceeded,

    #[msg("Unauthorized minter")]
    UnauthorizedMinter,

    #[msg("Invalid mint address")]
    InvalidMint,

    #[msg("Invalid token account")]
    InvalidTokenAccount,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
}
