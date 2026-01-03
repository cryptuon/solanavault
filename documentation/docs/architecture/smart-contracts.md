# Smart Contracts

> On-chain Solana programs implementing the VAULT tokenomics

---

## Overview

SolanaVault's tokenomics are enforced on-chain through four Anchor programs:

| Program | Purpose | Key Features |
|---------|---------|--------------|
| **vault-token** | VAULT SPL Token | 1B supply cap, emission schedule, burning |
| **vault-staking** | Staking & Unbonding | Tiers, 14-day unbonding, slashing |
| **vault-rewards** | Epoch Rewards | 24h epochs, performance-weighted, fee distribution |
| **vault-governance** | DAO Voting | Stake-weighted, time multiplier, timelocked execution |

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        VAULT Token                               │
│            (1B supply, emission schedule, burning)               │
└─────────────────────────┬───────────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
┌─────────────────┐ ┌───────────┐ ┌─────────────────┐
│    Staking      │ │  Rewards  │ │   Governance    │
│  - Stake/Unstake│ │  - Epochs │ │   - Proposals   │
│  - Tiers        │ │  - Fees   │ │   - Voting      │
│  - Slashing     │ │  - Dist.  │ │   - Execution   │
└─────────────────┘ └───────────┘ └─────────────────┘
```

---

## Program Details

### 1. VAULT Token Program

**Location:** `programs/vault-token/src/lib.rs`

The VAULT token is an SPL token with controlled minting and enforced emission schedule.

#### Constants

```rust
/// Total supply cap: 1 billion tokens with 9 decimals
pub const TOTAL_SUPPLY_CAP: u64 = 1_000_000_000 * 1_000_000_000; // 1B * 10^9

/// Token decimals (same as SOL)
pub const TOKEN_DECIMALS: u8 = 9;
```

#### Instructions

| Instruction | Access | Description |
|-------------|--------|-------------|
| `initialize` | Authority | Create token mint and config |
| `mint_tokens` | Emission Authority | Mint tokens within emission limits |
| `burn_tokens` | Token Holder | Burn tokens (deflationary) |
| `update_emission_authority` | Current Authority | Transfer emission rights |
| `get_token_stats` | Public | Query supply statistics |

#### Emission Schedule

The emission schedule is enforced on-chain:

```rust
pub struct EmissionSchedule {
    pub year_1_emission: u64,      // 100M tokens (10%)
    pub year_2_emission: u64,      // 80M tokens (8%)
    pub year_3_emission: u64,      // 60M tokens (6%)
    pub year_4_plus_emission: u64, // 40M tokens/year (4%)
    pub genesis_timestamp: u64,
}
```

#### Example: Mint Tokens

```typescript
await tokenProgram.methods
  .mintTokens(new BN(1_000_000_000)) // 1 VAULT
  .accounts({
    emissionAuthority: wallet.publicKey,
    tokenConfig: tokenConfigPda,
    mint: mintAddress,
    recipient: recipientPubkey,
    recipientTokenAccount: recipientAta,
    tokenProgram: TOKEN_PROGRAM_ID,
  })
  .rpc();
```

---

### 2. Staking Program

**Location:** `programs/vault-staking/src/lib.rs`

Manages stake deposits, unbonding, and tier-based rewards.

#### Constants

```rust
/// Minimum stake: 1,000 VAULT tokens
pub const MINIMUM_STAKE: u64 = 1_000 * 1_000_000_000;

/// Unbonding period: 14 days in seconds
pub const UNBONDING_PERIOD: i64 = 14 * 24 * 60 * 60;
```

#### Staking Tiers

| Tier | Minimum | Reward Multiplier | Constant |
|------|---------|-------------------|----------|
| Bronze | 1,000 VAULT | 1.0x | `TIER_BRONZE` |
| Silver | 10,000 VAULT | 1.2x | `TIER_SILVER` |
| Gold | 100,000 VAULT | 1.5x | `TIER_GOLD` |
| Platinum | 1,000,000 VAULT | 2.0x | `TIER_PLATINUM` |

#### Instructions

| Instruction | Access | Description |
|-------------|--------|-------------|
| `initialize` | Authority | Initialize staking pool |
| `stake` | User | Deposit tokens and create stake account |
| `request_unstake` | Staker | Begin 14-day unbonding |
| `complete_unstake` | Staker | Withdraw after unbonding |
| `claim_rewards` | Staker | Claim pending rewards |
| `apply_slash` | Slashing Authority | Apply slashing penalty |

#### Slashing Severity

```rust
pub enum SlashingSeverity {
    Minor,      // 5% slash
    Moderate,   // 15% slash
    Severe,     // 30% slash
    Critical,   // 50% slash
}
```

#### Example: Stake Tokens

```typescript
await stakingProgram.methods
  .stake(new BN(10_000_000_000_000)) // 10,000 VAULT
  .accounts({
    staker: wallet.publicKey,
    stakerTokenAccount: stakerAta,
    stakingPool: stakingPoolPda,
    stakingVault: vaultAddress,
    stakeAccount: stakeAccountPda,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

---

### 3. Rewards Program

**Location:** `programs/vault-rewards/src/lib.rs`

Distributes rewards based on epochs (24-hour periods).

#### Constants

```rust
/// Epoch duration: 24 hours
pub const EPOCH_DURATION: i64 = 24 * 60 * 60;

/// Gateway fee share: 95%
pub const GATEWAY_FEE_SHARE: u64 = 95;

/// Network fund share: 5%
pub const NETWORK_FEE_SHARE: u64 = 5;
```

#### Fee Distribution

```
Client Payment Flow:
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│   Gateway   │────▶│   Network   │
│  Pays 100%  │     │ Keeps 95%   │     │  Fund 5%    │
└─────────────┘     └─────────────┘     └─────────────┘
```

#### Instructions

| Instruction | Access | Description |
|-------------|--------|-------------|
| `initialize` | Authority | Set up rewards config |
| `advance_epoch` | Authority | Finalize epoch, prepare next |
| `record_fee` | Gateway | Record fee payment |
| `distribute_staker_reward` | Staker | Claim epoch rewards |
| `fund_rewards` | Authority | Add tokens to rewards pool |

#### Example: Record Fee

```typescript
await rewardsProgram.methods
  .recordFee(
    new BN(100_000_000), // 0.1 VAULT fee
    gatewayOperator.publicKey
  )
  .accounts({
    payer: gateway.publicKey,
    rewardsConfig: rewardsConfigPda,
    feeRecord: feeRecordPda,
    gatewayOperator: gatewayOperator.publicKey,
    feeVault: feeVaultAddress,
    gatewayTokenAccount: gatewayAta,
    networkFund: networkFundAta,
    tokenProgram: TOKEN_PROGRAM_ID,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

---

### 4. Governance Program

**Location:** `programs/vault-governance/src/lib.rs`

Token-weighted DAO governance with timelocked execution.

#### Constants

```rust
/// Voting period: 3 days
pub const VOTING_PERIOD: i64 = 3 * 24 * 60 * 60;

/// Quorum thresholds (percentage of total staked)
pub const QUORUM_PARAMETER_CHANGE: u8 = 10;
pub const QUORUM_TREASURY_ALLOCATION: u8 = 20;
pub const QUORUM_PROTOCOL_UPGRADE: u8 = 30;
pub const QUORUM_EMERGENCY_ACTION: u8 = 5;
```

#### Proposal Types

| Type | Quorum | Approval | Timelock |
|------|--------|----------|----------|
| Parameter Change | 10% | 50% | 48 hours |
| Treasury Allocation | 20% | 66% | 7 days |
| Protocol Upgrade | 30% | 75% | 14 days |
| Emergency Action | 5% | 90% | 6 hours |

#### Voting Power Calculation

```rust
// Time multiplier based on stake duration
fn calculate_time_multiplier(stake_duration_days: u64) -> u8 {
    match stake_duration_days {
        0..=89 => 100,      // 1.0x
        90..=179 => 125,    // 1.25x
        180..=364 => 150,   // 1.5x
        _ => 200,           // 2.0x
    }
}

// Voting power = staked_amount * time_multiplier / 100
```

#### Instructions

| Instruction | Access | Description |
|-------------|--------|-------------|
| `initialize` | Authority | Set up governance config |
| `update_staked_snapshot` | Authority | Update total staked for quorum |
| `create_proposal` | Gold+ Staker | Submit new proposal |
| `cast_vote` | Staker | Vote on active proposal |
| `finalize_proposal` | Anyone | End voting, determine outcome |
| `execute_proposal` | Anyone | Execute after timelock |
| `cancel_proposal` | Proposer/Authority | Cancel pending proposal |

#### Example: Create Proposal

```typescript
// Create parameter change proposal
const title = Buffer.alloc(64);
Buffer.from("Reduce base fee to 80 micro-tokens").copy(title);

const descriptionHash = Buffer.alloc(32);
// SHA-256 hash of full description

await governanceProgram.methods
  .createProposal(
    { parameterChange: {} },
    Array.from(title),
    64, // title length
    Array.from(descriptionHash),
    1,  // action count
    [executorProgramId, Pubkey.default, Pubkey.default, Pubkey.default, Pubkey.default]
  )
  .accounts({
    proposer: wallet.publicKey,
    governanceConfig: govConfigPda,
    proposal: proposalPda,
    stakeAccount: stakeAccountPda,
    systemProgram: SystemProgram.programId,
  })
  .rpc();
```

---

## Account Structures

### Token Config

```rust
pub struct TokenConfig {
    pub mint: Pubkey,
    pub mint_authority: Pubkey,
    pub emission_authority: Pubkey,
    pub total_minted: u64,
    pub total_burned: u64,
    pub is_initialized: bool,
    pub bump: u8,
    pub emission_schedule: EmissionSchedule,
}
```

### Stake Account

```rust
pub struct StakeAccount {
    pub owner: Pubkey,
    pub staking_pool: Pubkey,
    pub staked_amount: u64,
    pub pending_rewards: u64,
    pub stake_timestamp: i64,
    pub last_claim_timestamp: i64,
    pub unbonding_amount: u64,
    pub unbonding_start: i64,
    pub tier: StakingTier,
    pub performance_score: u8,
    pub is_active: bool,
    pub bump: u8,
}
```

### Proposal

```rust
pub struct Proposal {
    pub id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
    pub title: [u8; 64],
    pub description_hash: [u8; 32],
    pub votes_for: u64,
    pub votes_against: u64,
    pub total_eligible_votes: u64,
    pub status: ProposalStatus,
    pub created_at: i64,
    pub voting_ends_at: i64,
    pub execution_time: i64,
    pub executed: bool,
    pub bump: u8,
}
```

---

## Program IDs

> **Note:** These are placeholder IDs. Update after deployment.

| Program | Placeholder | Deployed |
|---------|-------------|----------|
| vault-token | `11111111111111111111111111111112` | TBD |
| vault-staking | `11111111111111111111111111111113` | TBD |
| vault-rewards | `11111111111111111111111111111114` | TBD |
| vault-governance | `11111111111111111111111111111115` | TBD |

---

## Deployment

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install Solana CLI
sh -c "$(curl -sSfL https://release.solana.com/v1.18.0/install)"

# Install Anchor
cargo install --git https://github.com/coral-xyz/anchor anchor-cli --locked
```

### Build

```bash
cd programs
anchor build
```

### Deploy

```bash
# Devnet
./scripts/deploy.sh devnet

# Mainnet (requires funded wallet)
./scripts/deploy.sh mainnet
```

### Initialize

```bash
# After deployment
npx ts-node scripts/initialize.ts
```

---

## Security Considerations

### Access Control

| Operation | Required Authority |
|-----------|-------------------|
| Mint tokens | Emission Authority (rewards program) |
| Apply slashing | Slashing Authority |
| Advance epoch | Rewards Authority |
| Execute proposal | Anyone (after timelock) |

### PDA Seeds

All program accounts use deterministic PDA derivation:

```rust
// Token config
seeds = [b"token_config"]

// Stake account
seeds = [b"stake_account", owner.key().as_ref()]

// Proposal
seeds = [b"proposal", proposal_id.to_le_bytes().as_ref()]

// Epoch record
seeds = [b"epoch", epoch_number.to_le_bytes().as_ref()]
```

### Slashing Protection

- 7-day grace period before slashing takes effect
- Progressive slashing for repeat offenders
- Appeals process through governance

---

## Integration

### TypeScript SDK

```typescript
import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { VaultToken } from "../target/types/vault_token";
import { VaultStaking } from "../target/types/vault_staking";

// Load programs
const tokenProgram = anchor.workspace.VaultToken as Program<VaultToken>;
const stakingProgram = anchor.workspace.VaultStaking as Program<VaultStaking>;

// Get PDAs
const [tokenConfig] = PublicKey.findProgramAddressSync(
  [Buffer.from("token_config")],
  tokenProgram.programId
);

const [stakeAccount] = PublicKey.findProgramAddressSync(
  [Buffer.from("stake_account"), wallet.publicKey.toBuffer()],
  stakingProgram.programId
);
```

### Events

All programs emit events for off-chain indexing:

```rust
// Token events
#[event]
pub struct TokensMinted {
    pub recipient: Pubkey,
    pub amount: u64,
    pub total_minted: u64,
}

// Staking events
#[event]
pub struct Staked {
    pub staker: Pubkey,
    pub amount: u64,
    pub new_tier: StakingTier,
}

// Governance events
#[event]
pub struct ProposalCreated {
    pub proposal_id: u64,
    pub proposer: Pubkey,
    pub proposal_type: ProposalType,
}
```

---

## Related Documentation

- [Tokenomics](tokenomics.md) - Token distribution and economics
- [Economics](economics.md) - Pricing and fee structure
- [Gateway Operators Guide](../guides/gateway-operators.md) - Earning from fees
- [Full Node Setup](../guides/full-node.md) - Staking and consensus

---

## Source Code

Full source code available at:
- [`programs/vault-token/src/lib.rs`](https://github.com/solanavault/solanavault/tree/main/programs/vault-token)
- [`programs/vault-staking/src/lib.rs`](https://github.com/solanavault/solanavault/tree/main/programs/vault-staking)
- [`programs/vault-rewards/src/lib.rs`](https://github.com/solanavault/solanavault/tree/main/programs/vault-rewards)
- [`programs/vault-governance/src/lib.rs`](https://github.com/solanavault/solanavault/tree/main/programs/vault-governance)
