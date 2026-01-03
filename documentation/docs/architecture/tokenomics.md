# SolanaVault Tokenomics

> Comprehensive token economics derived from the [SolanaVault Whitepaper](../../../docs/research/whitepaper/solanavault_whitepaper.pdf)

---

## Executive Summary

The VAULT token powers the SolanaVault decentralized storage network, enabling:
- **Payment** for data storage and retrieval services
- **Staking** to secure the network and earn rewards
- **Governance** over protocol parameters
- **Incentive alignment** between all network participants

---

## Token Overview

### Basic Information

| Property | Value |
|----------|-------|
| **Token Name** | VAULT |
| **Token Symbol** | VAULT |
| **Blockchain** | Solana (SPL Token) |
| **Total Supply** | 1,000,000,000 VAULT |
| **Decimals** | 9 |
| **Initial Price** | TBD |

### Token Utility

| Utility | Description |
|---------|-------------|
| **Service Payment** | Pay for data storage and retrieval |
| **Staking** | Secure network, earn consensus rewards |
| **Governance** | Vote on protocol parameters |
| **Gateway Operation** | Run gateway nodes, earn fees |
| **Slashing Collateral** | Stake at risk for misbehavior |

---

## Token Distribution

### Allocation

| Category | Allocation | Amount | Vesting |
|----------|------------|--------|---------|
| **Network Rewards** | 40% | 400,000,000 | Released over 10 years |
| **Ecosystem Fund** | 20% | 200,000,000 | 4-year linear vesting |
| **Team & Advisors** | 15% | 150,000,000 | 1-year cliff, 3-year linear |
| **Private Sale** | 10% | 100,000,000 | 6-month cliff, 18-month linear |
| **Public Sale** | 5% | 50,000,000 | 25% at TGE, 75% over 6 months |
| **Treasury** | 7% | 70,000,000 | DAO-controlled |
| **Liquidity** | 3% | 30,000,000 | Immediate for DEX/CEX |

### Distribution Chart

```
                    Token Distribution
    ┌─────────────────────────────────────────────────┐
    │ ████████████████████████████████░░░░░░ 40%     │ Network Rewards
    │ ████████████████░░░░░░░░░░░░░░░░░░░░░░ 20%     │ Ecosystem Fund
    │ ████████████░░░░░░░░░░░░░░░░░░░░░░░░░░ 15%     │ Team & Advisors
    │ ████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 10%     │ Private Sale
    │ ████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  5%     │ Public Sale
    │ ███░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  7%     │ Treasury
    │ ██░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░  3%     │ Liquidity
    └─────────────────────────────────────────────────┘
```

### Vesting Schedule

```
                      Circulating Supply Over Time
    100% ┤                                              ▄▄▄▄▄▄
         │                                          ▄▄▄▀
         │                                      ▄▄▄▀
     75% ┤                                  ▄▄▄▀
         │                              ▄▄▄▀
         │                          ▄▄▄▀
     50% ┤                      ▄▄▄▀
         │                  ▄▄▄▀
         │              ▄▄▄▀
     25% ┤          ▄▄▄▀
         │      ▄▄▄▀
         │  ▄▄▄▀
      0% ┼──────────────────────────────────────────────────────
         TGE    Y1     Y2     Y3     Y4     Y5     ...    Y10
```

---

## Economic Model

*Based on the SolanaVault Whitepaper Section 5: Economic Model*

### Reward Formulas

**Storage Rewards** (from whitepaper):
```
R_storage = α × data_size × storage_duration × network_demand
```

Where:
- `α` = storage reward coefficient (network-determined)
- `data_size` = bytes stored
- `storage_duration` = time stored (epochs)
- `network_demand` = demand multiplier (0.5 - 2.0)

**Retrieval Rewards** (from whitepaper):
```
R_retrieval = β × requests_served × response_quality
```

Where:
- `β` = retrieval reward coefficient
- `requests_served` = number of successful retrievals
- `response_quality` = latency and accuracy score (0.0 - 1.0)

### Consensus Rewards

Daily reward calculation for validators:

```
Daily Reward = Base Reward × Stake Weight × Uptime × Reputation

Where:
  Base Reward  = Network emission / Active validators
  Stake Weight = Your stake / Total stake
  Uptime       = Hours online / 24
  Reputation   = Historical performance score (0.0 - 1.0)
```

**Example Calculation:**
```
Inputs:
  Base Reward:  10,000 VAULT/day (shared pool)
  Your Stake:   50,000 VAULT
  Total Stake:  5,000,000 VAULT
  Uptime:       99.5%
  Reputation:   0.95

Calculation:
  Daily Reward = 10,000 × (50,000/5,000,000) × 0.995 × 0.95
               = 10,000 × 0.01 × 0.995 × 0.95
               = 94.5 VAULT/day
               ≈ 34,493 VAULT/year
               ≈ 69% APY
```

---

## Emission Schedule

### Annual Emission

| Year | Emission Rate | Tokens Released | Cumulative |
|------|--------------|-----------------|------------|
| 1 | 10% | 100,000,000 | 100,000,000 |
| 2 | 8% | 80,000,000 | 180,000,000 |
| 3 | 6% | 60,000,000 | 240,000,000 |
| 4 | 5% | 50,000,000 | 290,000,000 |
| 5 | 4% | 40,000,000 | 330,000,000 |
| 6+ | 4% | 40,000,000/year | → 400,000,000 cap |

### Emission Curve

```
    Emission Rate (% of total supply)
    12% ┤
        │▓▓▓
    10% ┤▓▓▓
        │▓▓▓ ▓▓▓
     8% ┤▓▓▓ ▓▓▓
        │▓▓▓ ▓▓▓ ▓▓▓
     6% ┤▓▓▓ ▓▓▓ ▓▓▓
        │▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓
     4% ┤▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓
        │▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓
     2% ┤▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓
        │▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓ ▓▓▓
     0% ┼─────────────────────────────────────────
         Y1  Y2  Y3  Y4  Y5  Y6  Y7  Y8  Y9  Y10
```

---

## Deflationary Mechanisms

### Fee Burning

| Mechanism | Burn Rate | Description |
|-----------|-----------|-------------|
| Transaction fees | 1% | 1% of all fees permanently burned |
| Slashing | 100% | Slashed stakes fully burned |
| Unused treasury | Quarterly | Unused funds returned and burned |

### Projected Burn

Assuming network activity of 10M requests/day at average 500μ fee:

```
Daily fees:     5,000,000,000μ = 5,000 VAULT
Daily burn:     50 VAULT (1%)
Annual burn:    18,250 VAULT

At scale (100M req/day):
Annual burn:    182,500 VAULT
```

---

## Staking Tiers

### Tier Benefits

| Tier | Minimum Stake | Reward Multiplier | Benefits |
|------|---------------|-------------------|----------|
| **Bronze** | 1,000 VAULT | 1.0x | Basic participation |
| **Silver** | 10,000 VAULT | 1.2x | Priority routing |
| **Gold** | 100,000 VAULT | 1.5x | Governance voting |
| **Platinum** | 1,000,000 VAULT | 2.0x | Protocol proposals |

### Staking Parameters

| Parameter | Value |
|-----------|-------|
| Minimum stake | 1,000 VAULT |
| Unbonding period | 14 days |
| Reward claim cooldown | 24 hours |
| Slashing protection | 7-day grace |

---

## Slashing Schedule

### Offense Categories

| Category | Offense | Slash Amount | Cooldown |
|----------|---------|--------------|----------|
| **Minor** | Extended downtime (>24h) | 1%/day | None |
| **Moderate** | Invalid proposals | 5% | 3 days |
| **Severe** | Double voting | 10% | 7 days |
| **Critical** | Data corruption | 20% | 30 days |
| **Malicious** | Coordinated attack | 50-100% | Permanent |

### Slashing Formula

```
Slash Amount = Base Slash × Severity Multiplier × Repeat Offense Multiplier

Where:
  Repeat Offense Multiplier = 1.5^(offense_count - 1)
```

---

## Gateway Economics

### Revenue Model

Gateway operators earn 95% of fees:

```
Client Payment Flow:
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Client    │────▶│   Gateway   │────▶│   Network   │
│  Pays 100%  │     │ Keeps 95%   │     │  Fund 5%    │
└─────────────┘     └─────────────┘     └─────────────┘
```

### Fee Structure

| Request Type | Base Fee | Data Fee | Example (1MB) |
|-------------|----------|----------|---------------|
| getSlot | 100μ | - | 100μ |
| getBlock | 100μ | 50μ/KB | 51,300μ |
| getTransaction | 100μ | 50μ/KB | Varies |
| getAccountInfo | 100μ | 50μ/KB | Varies |

### Gateway Profitability

**Monthly P&L Example (10M requests):**

```
Revenue:
  Requests:      10,000,000
  Avg fee:       500μ
  Gross:         5,000,000,000μ (5,000 VAULT)
  Gateway (95%): 4,750 VAULT

Costs:
  Server:        500 VAULT/mo
  Bandwidth:     200 VAULT/mo
  Maintenance:   100 VAULT/mo
  Total:         800 VAULT/mo

Net Profit:      3,950 VAULT/mo
```

---

## Network Fund Allocation

The 5% network fund supports ecosystem growth:

| Allocation | Percentage | Purpose |
|------------|------------|---------|
| Protocol Development | 40% | Core team, infrastructure |
| Bug Bounties | 20% | Security researchers |
| Ecosystem Grants | 30% | Community projects |
| Emergency Reserve | 10% | Unexpected costs |

```
    Network Fund Distribution
    ┌────────────────────────────────────────┐
    │████████████████░░░░░░░░░░░░░░░░░░░ 40% │ Development
    │████████████░░░░░░░░░░░░░░░░░░░░░░░ 30% │ Grants
    │████████░░░░░░░░░░░░░░░░░░░░░░░░░░░ 20% │ Bug Bounties
    │████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░ 10% │ Reserve
    └────────────────────────────────────────┘
```

---

## Governance

### Voting Power

Voting power is proportional to staked VAULT:

```
Voting Power = Staked Amount × Time Multiplier

Time Multiplier:
  < 3 months:   1.0x
  3-6 months:   1.25x
  6-12 months:  1.5x
  > 12 months:  2.0x
```

### Proposal Types

| Type | Quorum | Approval | Timelock |
|------|--------|----------|----------|
| Parameter change | 10% | 50%+1 | 48 hours |
| Treasury allocation | 20% | 66% | 7 days |
| Protocol upgrade | 30% | 75% | 14 days |
| Emergency action | 5% | 90% | 6 hours |

### Governance Parameters

| Parameter | Governable | Range |
|-----------|------------|-------|
| Base fee | Yes | 50-500μ |
| Data fee | Yes | 25-100μ/KB |
| Slash percentages | Yes | 1-50% |
| Emission rate | Limited | ±20% |
| Gateway split | Yes | 90-99% |

---

## Economic Security

*From whitepaper Section 6: Security Considerations*

### Attack Cost Analysis

| Attack | Required | Cost Estimate |
|--------|----------|---------------|
| 51% consensus | 51% of staked VAULT | $XXM+ |
| Sybil (1000 nodes) | 1000 × 1000 VAULT | 1,000,000 VAULT |
| Data corruption | Stake at risk | 20%+ slash |

### Security Guarantees

From the whitepaper:
- **Byzantine Fault Tolerance**: Handles up to 33% malicious nodes
- **Stake-based Security**: Economic penalties for malicious behavior
- **Encrypted Communication**: TLS for all network communications

### Incentive Alignment

The economic model ensures:
1. **Honest behavior is profitable** - Rewards exceed attack gains
2. **Attacks are expensive** - Slashing makes attacks costly
3. **Long-term alignment** - Stake lockups prevent hit-and-run

---

## Comparison to Similar Projects

| Feature | SolanaVault | Filecoin | Arweave |
|---------|-------------|----------|---------|
| Focus | Solana data | General storage | Permanent storage |
| Compression | 15-25:1 | None | None |
| Retrieval speed | <1ms | Seconds | Seconds |
| Storage cost | 95% lower | Market rate | One-time fee |
| Token utility | Pay/Stake/Govern | Pay/Stake | Pay only |

---

## Token Metrics Summary

### At Genesis (TGE)

| Metric | Value |
|--------|-------|
| Initial Circulating | ~80,000,000 VAULT (8%) |
| Initial Market Cap | TBD |
| Fully Diluted Value | TBD |
| Initial Staking APY | 50-100% (estimated) |

### Year 1 Projections

| Metric | Conservative | Moderate | Aggressive |
|--------|--------------|----------|------------|
| Circulating Supply | 180M | 180M | 180M |
| Staked Amount | 50M | 100M | 150M |
| Daily Volume | 1M | 5M | 20M |
| Burned (Year 1) | 5,000 | 25,000 | 100,000 |

---

## Smart Contract Addresses

> **Note:** Addresses will be published upon mainnet deployment

| Contract | Address | Purpose |
|----------|---------|---------|
| VAULT Token | TBD | SPL Token mint |
| Staking | TBD | Stake management |
| Governance | TBD | DAO voting |
| Treasury | TBD | Network fund |
| Rewards | TBD | Emission distribution |

---

## Related Documentation

- [Economics Overview](economics.md) - Detailed pricing and fees
- [Network Architecture](network.md) - How payments flow
- [Gateway Operators Guide](../guides/gateway-operators.md) - Start earning
- [Full Node Setup](../guides/full-node.md) - Run a storage node
- [Whitepaper](../../../docs/research/whitepaper/solanavault_whitepaper.pdf) - Academic paper

---

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2024-01-XX | Initial tokenomics document |

---

*This document is derived from the SolanaVault Whitepaper and subject to governance changes.*
