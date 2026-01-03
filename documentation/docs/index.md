# SolanaVault

**Decentralized Blockchain Compression & Storage Network for Solana**

SolanaVault is a fully decentralized network that provides **15-25:1 compression ratios** on Solana blockchain data with drop-in RPC compatibility and built-in economic incentives.

---

## Why SolanaVault?

| Challenge | SolanaVault Solution |
|-----------|---------------------|
| Blockchain data grows exponentially | 15-25:1 compression ratios |
| Centralized infrastructure is a single point of failure | Fully decentralized P2P network |
| Running nodes is expensive | Light clients with pay-per-use pricing |
| No economic incentive for operators | Gateway operators earn 95% of fees |

---

## Key Features

### High-Performance Compression
- **15-25:1** typical compression ratios (up to 30:1 on redundant blocks)
- **13-85 microseconds** decompression per block
- Three-stage pipeline optimized for Solana data patterns

### True Decentralization
- Byzantine Fault Tolerant consensus
- Kademlia DHT peer discovery
- No central points of failure
- NNG-based P2P transport with microsecond latency

### Economic Sustainability
- Pay-per-use model for light clients
- Gateway operators earn revenue
- Volume discounts for high-usage applications
- Self-sustaining network economics

### Drop-in Compatibility
- Standard Solana RPC API works unchanged
- Existing tools and workflows continue to work
- No code changes required for applications

---

## Choose Your Path

<div class="grid cards" markdown>

-   :material-rocket-launch:{ .lg .middle } **Light Client**

    ---

    For developers and applications. Pay only for what you use with automatic caching.

    [:octicons-arrow-right-24: Light Client Guide](guides/light-client.md)

-   :material-server:{ .lg .middle } **Gateway Operator**

    ---

    Run gateway nodes and earn revenue by serving light clients.

    [:octicons-arrow-right-24: Gateway Guide](guides/gateway-operators.md)

-   :material-database:{ .lg .middle } **Full Node**

    ---

    Participate in consensus, store data, and earn rewards.

    [:octicons-arrow-right-24: Full Node Setup](guides/full-node.md)

</div>

---

## Quick Start

Get started in under 5 minutes:

=== "Light Client"

    ```bash
    # Install SolanaVault
    cargo install solanavault-cli

    # Start light client with initial balance
    vault-light-client start --balance 50000

    # Use standard Solana CLI - it now connects through SolanaVault
    solana balance
    ```

=== "Gateway Node"

    ```bash
    # Build from source
    git clone https://github.com/solanavault/solanavault
    cd solanavault
    cargo build --release

    # Run decentralized gateway
    ./target/release/vault-rpc-decentralized --gateway-mode
    ```

=== "Full Node"

    ```bash
    # Build and run full node
    cargo build --release
    ./target/release/vault-node --storage-capacity 100GB
    ```

---

## Performance Metrics

| Metric | Value |
|--------|-------|
| Compression Ratio | 15-25:1 typical |
| Decompression Speed | 13-85 microseconds/block |
| Cache Hit Rate | >90% |
| Storage Savings | 95%+ |
| Network Throughput | 1000+ queries/second/node |

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        Applications                              │
│            (Wallets, dApps, Explorers, Analytics)               │
└─────────────────────────┬───────────────────────────────────────┘
                          │ Standard Solana RPC
┌─────────────────────────▼───────────────────────────────────────┐
│                      Light Clients                               │
│              (Pay-per-use, automatic caching)                    │
└─────────────────────────┬───────────────────────────────────────┘
                          │ Micro-payments
┌─────────────────────────▼───────────────────────────────────────┐
│                     Gateway Nodes                                │
│              (Monetized access points, 95% revenue)             │
└─────────────────────────┬───────────────────────────────────────┘
                          │ NNG P2P Protocol
┌─────────────────────────▼───────────────────────────────────────┐
│                  Decentralized Network                           │
│     ┌─────────────┐  ┌─────────────┐  ┌─────────────┐          │
│     │Storage Node │  │Storage Node │  │Storage Node │   ...    │
│     │   (BFT)     │  │   (BFT)     │  │   (BFT)     │          │
│     └─────────────┘  └─────────────┘  └─────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Next Steps

- [Quick Start Guide](guides/quickstart.md) - Get up and running in minutes
- [Installation](guides/installation.md) - Detailed installation instructions
- [Architecture Overview](architecture/overview.md) - Deep dive into how it works
- [Smart Contracts](architecture/smart-contracts.md) - On-chain Solana programs
- [Tokenomics](architecture/tokenomics.md) - Token distribution and economics
- [API Reference](reference/api.md) - Complete API documentation
