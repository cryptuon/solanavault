# SolanaVault Smart Contracts

On-chain Solana programs implementing the VAULT tokenomics.

## Programs

| Program | Description | Key Features |
|---------|-------------|--------------|
| **vault-token** | VAULT SPL Token | 1B supply cap, emission schedule, burning |
| **vault-staking** | Staking & Unbonding | Tiers, 14-day unbonding, slashing |
| **vault-rewards** | Epoch Rewards | 24h epochs, performance-weighted, fee distribution |
| **vault-governance** | DAO Voting | Stake-weighted, time multiplier, timelocked execution |

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

## Quick Start

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

### Test

```bash
anchor test
```

### Deploy

```bash
# Devnet
anchor deploy --provider.cluster devnet

# Mainnet (requires funded wallet)
anchor deploy --provider.cluster mainnet
```

## Token Economics

### Supply

| Metric | Value |
|--------|-------|
| Total Supply | 1,000,000,000 VAULT |
| Decimals | 9 |
| Network | Solana |

### Emission Schedule

| Year | Rate | Tokens |
|------|------|--------|
| 1 | 10% | 100,000,000 |
| 2 | 8% | 80,000,000 |
| 3 | 6% | 60,000,000 |
| 4+ | 4% | 40,000,000/year |

### Staking Tiers

| Tier | Minimum | Reward Multiplier |
|------|---------|-------------------|
| Bronze | 1,000 VAULT | 1.0x |
| Silver | 10,000 VAULT | 1.2x |
| Gold | 100,000 VAULT | 1.5x |
| Platinum | 1,000,000 VAULT | 2.0x |

### Slashing

| Offense | Slash |
|---------|-------|
| Extended Downtime | 5% |
| Invalid Proposal | 15% |
| Data Corruption | 30% |
| Malicious Behavior | 50% |

### Governance

| Proposal Type | Quorum | Approval | Timelock |
|---------------|--------|----------|----------|
| Parameter Change | 10% | 50% | 48 hours |
| Treasury Allocation | 20% | 66% | 7 days |
| Protocol Upgrade | 30% | 75% | 14 days |
| Emergency Action | 5% | 90% | 6 hours |

## Program IDs

> **Note:** These are placeholder IDs. Replace with actual deployed addresses after running `anchor deploy`.

| Program | Placeholder | Deployed |
|---------|-------------|----------|
| vault-token | `11111111111111111111111111111112` | TBD |
| vault-staking | `11111111111111111111111111111113` | TBD |
| vault-rewards | `11111111111111111111111111111114` | TBD |
| vault-governance | `11111111111111111111111111111115` | TBD |

### Updating Program IDs

After deployment, update the program IDs:

1. Run `anchor keys list` to get deployed addresses
2. Update each `declare_id!()` in `src/lib.rs` files
3. Update `Anchor.toml` with new addresses
4. Rebuild: `anchor build`

## Development

### Project Structure

```
programs/
├── Anchor.toml           # Anchor configuration
├── Cargo.toml            # Workspace config
├── vault-token/          # Token program
│   ├── Cargo.toml
│   └── src/lib.rs
├── vault-staking/        # Staking program
│   ├── Cargo.toml
│   └── src/lib.rs
├── vault-rewards/        # Rewards program
│   ├── Cargo.toml
│   └── src/lib.rs
└── vault-governance/     # Governance program
    ├── Cargo.toml
    └── src/lib.rs
```

### Key Instructions

#### Token Program

```rust
// Initialize token
initialize(emission_authority: Pubkey)

// Mint tokens (emission authority only)
mint_tokens(amount: u64)

// Burn tokens
burn_tokens(amount: u64)
```

#### Staking Program

```rust
// Stake tokens
stake(amount: u64)

// Request unstake (starts 14-day unbonding)
request_unstake(amount: u64)

// Complete unstake (after unbonding)
complete_unstake()

// Claim rewards
claim_rewards()
```

#### Rewards Program

```rust
// Advance epoch (every 24 hours)
advance_epoch()

// Record fee (called by gateways)
record_fee(amount: u64, gateway: Pubkey)

// Distribute staker reward
distribute_staker_reward(epoch: u64)
```

#### Governance Program

```rust
// Create proposal
create_proposal(
    proposal_type: ProposalType,
    title: String,
    description: String,
    actions: Vec<ProposalAction>
)

// Cast vote
cast_vote(vote_type: VoteType)

// Finalize proposal (after voting)
finalize_proposal()

// Execute proposal (after timelock)
execute_proposal()
```

## Security

### Audits

- [ ] Internal review
- [ ] External audit (TBD)
- [ ] Bug bounty program

### Key Security Features

1. **PDA Authority**: All sensitive operations use PDA-based authorities
2. **Timelock**: Governance actions have mandatory waiting periods
3. **Slashing**: Malicious behavior results in stake loss
4. **Emission Cap**: Hard cap of 1B tokens enforced on-chain
5. **Quorum Requirements**: Minimum participation for governance

## License

MIT License - see [LICENSE](../LICENSE)
