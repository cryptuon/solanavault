# Economics

SolanaVault implements a sustainable economic model that incentivizes network participation while keeping costs low for users.

---

## Overview

The economic model has three key participants:

| Participant | Role | Incentive |
|-------------|------|-----------|
| Light Clients | Consume data | Pay for access |
| Gateway Operators | Serve requests | Earn fees (95%) |
| Storage Nodes | Store data, consensus | Earn rewards |

```
┌─────────────────────────────────────────────────────────────────┐
│                         Token Flow                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   Light Clients                                                  │
│        │                                                         │
│        │ Pay micro-tokens                                        │
│        ▼                                                         │
│   ┌─────────────────────┐                                       │
│   │   Payment Pool      │                                       │
│   └─────────┬───────────┘                                       │
│             │                                                    │
│    ┌────────┴────────┐                                          │
│    │                 │                                          │
│    ▼                 ▼                                          │
│   95%               5%                                          │
│ Gateway          Network                                        │
│ Revenue          Fund                                           │
│    │                 │                                          │
│    │                 └──► Protocol development                  │
│    │                 └──► Bug bounties                          │
│    │                 └──► Grants                                │
│    │                                                            │
│    └──► Operator earnings                                       │
│                                                                  │
│   Storage Nodes ◄──── Consensus Rewards (separate pool)        │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## Pricing Model

### Base Fees

All fees are denominated in micro-tokens (μ):

| Request Type | Base Fee | Data Fee |
|-------------|----------|----------|
| getSlot | 100μ | - |
| getBlock | 100μ | 50μ/KB |
| getTransaction | 100μ | 50μ/KB |
| getAccountInfo | 100μ | 50μ/KB |
| getBlockHeight | 100μ | - |
| getEpochInfo | 100μ | - |

### Price Calculation

```
Total Cost = Base Fee + (Data Size × Data Fee) × Modifiers
```

Example:

```
Request: getBlock (2 MB response)
Base Fee: 100μ
Data Fee: 50μ × 2048 KB = 102,400μ
Total: 102,500μ
```

### Modifiers

| Modifier | Multiplier | Condition |
|----------|------------|-----------|
| Priority | 1.5x | Request flagged as priority |
| Real-time | 2.0x | Latest block data |
| Historical | 0.8x | Data older than 24 hours |
| Cached | 0.5x | Served from cache |

### Volume Discounts

High-volume users receive discounts:

| Monthly Requests | Discount |
|-----------------|----------|
| 0 - 10,000 | 0% |
| 10,001 - 100,000 | 10% |
| 100,001 - 1,000,000 | 20% |
| 1,000,000+ | 25% |

### Surge Pricing

During high demand, prices increase:

```rust
fn calculate_surge_multiplier(utilization: f64) -> f64 {
    match utilization {
        u if u < 0.7 => 1.0,      // Normal
        u if u < 0.8 => 1.2,      // Elevated
        u if u < 0.9 => 1.5,      // High
        _ => 2.0,                  // Critical
    }
}
```

---

## Revenue Distribution

### Gateway Revenue

Gateways receive 95% of all fees they serve:

```
Client pays: 1,000μ for request
├── Gateway receives: 950μ (95%)
└── Network fund: 50μ (5%)
```

### Network Fund (5%)

The network fund supports:

| Allocation | Percentage | Purpose |
|------------|------------|---------|
| Protocol Development | 40% | Core team, infrastructure |
| Bug Bounties | 20% | Security researchers |
| Ecosystem Grants | 30% | Community projects |
| Emergency Reserve | 10% | Unexpected costs |

---

## Staking and Consensus Rewards

### Staking

Storage nodes stake tokens to participate:

| Stake Tier | Minimum | Benefits |
|------------|---------|----------|
| Bronze | 1,000 | Basic participation |
| Silver | 10,000 | 1.2x rewards |
| Gold | 100,000 | 1.5x rewards, priority routing |
| Platinum | 1,000,000 | 2x rewards, governance |

### Consensus Rewards

Nodes earn rewards for honest consensus participation:

```
Daily Reward = Base Reward × Stake Weight × Uptime × Reputation

Where:
  Base Reward = Network emission / Active validators
  Stake Weight = Your stake / Total stake
  Uptime = Hours online / 24
  Reputation = 0.0 to 1.0 based on history
```

Example:

```
Base Reward: 10,000 tokens/day shared among validators
Your Stake: 50,000 tokens
Total Stake: 5,000,000 tokens
Uptime: 99.5%
Reputation: 0.95

Daily Reward = 10,000 × (50,000/5,000,000) × 0.995 × 0.95
             = 10,000 × 0.01 × 0.995 × 0.95
             = 94.5 tokens/day
```

### Slashing

Malicious behavior results in stake slashing:

| Violation | Slash Amount | Cooldown |
|-----------|--------------|----------|
| Double voting | 10% | 7 days |
| Data corruption | 20% | 30 days |
| Extended downtime (>24h) | 1%/day | None |
| Invalid proposals | 5% | 3 days |

---

## Light Client Economics

### Balance Management

Light clients maintain a token balance:

```bash
# Check balance
vault-light-client balance

# Add tokens
vault-light-client balance add 50000

# Set low balance alert
vault-light-client balance alert --threshold 10000
```

### Cost Optimization

Tips for minimizing costs:

1. **Enable caching**: Repeated requests cost 50% less
2. **Use historical data**: 20% cheaper than real-time
3. **Batch requests**: Reduces overhead
4. **Monitor spending**: Set daily limits

```toml
[light_client.cost_optimization]
cache_enabled = true
prefer_historical = true
max_daily_spend = 100000
```

### Estimating Costs

Typical costs per operation:

| Use Case | Requests/Day | Est. Daily Cost |
|----------|--------------|-----------------|
| Personal wallet | 100 | 10,000μ |
| Small dApp | 10,000 | 500,000μ |
| Block explorer | 100,000 | 5,000,000μ |
| Analytics platform | 1,000,000 | 40,000,000μ |

---

## Gateway Economics

### Profitability Analysis

Gateway profitability depends on:

| Factor | Impact |
|--------|--------|
| Request volume | More requests = more revenue |
| Average fee | Higher fees (larger responses) = more revenue |
| Operating costs | Server, bandwidth, maintenance |
| Competition | Pricing pressure from other gateways |

### Example P&L

```
Monthly Revenue:
  Requests: 10,000,000
  Avg fee: 500μ
  Gross: 5,000,000,000μ
  Gateway share (95%): 4,750,000,000μ

Monthly Costs:
  Server: 500,000,000μ
  Bandwidth: 200,000,000μ
  Maintenance: 100,000,000μ
  Total: 800,000,000μ

Net Profit: 3,950,000,000μ
```

### Competitive Strategies

1. **Lower prices**: Attract volume
2. **Better performance**: Faster = preferred
3. **Higher reliability**: More uptime = more trust
4. **Geographic advantage**: Lower latency for local clients

---

## Storage Node Economics

### Revenue Streams

Storage nodes earn from:

| Source | Description |
|--------|-------------|
| Consensus rewards | Participation in BFT |
| Storage fees | Storing data for the network |
| Serving fees | Responding to data requests |

### Cost Structure

| Cost | Monthly Estimate |
|------|-----------------|
| Server (8 core, 32GB) | 200,000,000μ |
| Storage (1TB NVMe) | 100,000,000μ |
| Bandwidth (1TB) | 50,000,000μ |
| Electricity | 30,000,000μ |
| **Total** | **380,000,000μ** |

### Break-even Analysis

```
Required daily revenue = Monthly costs / 30
                      = 380,000,000 / 30
                      = 12,666,667μ/day

At 0.001 tokens/day/stake reward rate:
Required stake = 12,666,667 / 1000
               = 12,667 tokens (minimum viable stake)
```

---

## Token Metrics

### Supply

| Metric | Value |
|--------|-------|
| Total Supply | 1,000,000,000 tokens |
| Circulating | TBD |
| Locked (team) | TBD |
| Ecosystem fund | TBD |

### Emission Schedule

```
Year 1: 10% of total supply for rewards
Year 2: 8% of total supply
Year 3: 6% of total supply
Year 4+: 4% of total supply (steady state)
```

### Deflationary Mechanisms

| Mechanism | Effect |
|-----------|--------|
| Fee burning | 1% of all fees burned |
| Slashing | Slashed stakes are burned |
| Unused funds | Returned to treasury |

---

## Economic Security

### Attack Cost Analysis

| Attack | Required Resources | Cost |
|--------|-------------------|------|
| 51% consensus | 51% of stake | Very high |
| Sybil (1000 nodes) | 1000 × min stake | High |
| Eclipse single node | Network presence | Medium |

### Incentive Alignment

The economic model ensures:

1. **Honest behavior is profitable**: Rewards exceed attack gains
2. **Attacks are expensive**: Slashing makes attacks costly
3. **Long-term alignment**: Stake lockups prevent hit-and-run

---

## Configuration

### Gateway Pricing

```toml
[economics.gateway]
base_fee = 100
data_fee_per_kb = 50
priority_multiplier = 1.5
surge_enabled = true
volume_discounts = [
    { threshold = 10000, discount = 0.10 },
    { threshold = 100000, discount = 0.20 },
    { threshold = 1000000, discount = 0.25 },
]
```

### Node Staking

```toml
[economics.staking]
minimum_stake = 1000
unbonding_period_days = 14
reward_claim_cooldown_hours = 24
```

---

## Next Steps

- [Network Protocol](network.md) - How payments flow through the network
- [Gateway Operators Guide](../guides/gateway-operators.md) - Start earning
- [Full Node Setup](../guides/full-node.md) - Run a storage node
- [Light Client Guide](../guides/light-client.md) - Optimize spending
