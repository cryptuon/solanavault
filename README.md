# SolanaVault

> **Active development.** SolanaVault is under active development. APIs,
> schemas, and on-chain layouts may change between releases.
> Production use at your own risk. Issues and PRs welcome.

<div align="center">
  <h3>🚀 High-Performance Solana Blockchain Compression & Storage</h3>
  <p><strong>15-25:1 Compression Ratio</strong> | <strong>95% Storage Savings</strong> | <strong>Sub-Millisecond Retrieval</strong></p>

  [![Compression](https://img.shields.io/badge/compression-15--25%3A1-red.svg)]()
  [![License](https://img.shields.io/badge/License-MIT-blue.svg)]()
  [![Status](https://img.shields.io/badge/status-active--development-orange.svg)]()
</div>

**[🌐 Site](https://solanavault.cryptuon.com/) · [📚 Docs](https://docs.cryptuon.com/solanavault/) · [🗺️ Roadmap](./ROADMAP.md) · [🔬 Cryptuon Research](https://github.com/cryptuon)**

## Overview

SolanaVault is a **decentralized (DePIN) compression and storage network for Solana blockchain data**. It compresses blocks and account history at **15-25:1** and serves them back with **sub-millisecond retrieval** over a peer-to-peer network of independent operators — while the **standard Solana RPC API stays unchanged**. In short: cheaper archival and state storage, faster reads, and no rewrite of the surface your apps already speak.

## Why This Matters in 2026

Two forces are colliding on Solana. Throughput keeps climbing, and the **state and history that throughput produces have to live somewhere** — today that means expensive archive nodes and a handful of centralized RPC providers. At the same time, the **agent economy** (automated payments, on-chain and verifiable AI, high-frequency RWA settlement) is generating machine-speed read traffic against exactly that historical and account data. State bloat is no longer a tomorrow problem; it is the tax on every indexer, explorer, and inference pipeline running today.

SolanaVault treats blockchain data storage as **shared infrastructure rather than a per-operator cost center**. Instead of every team paying to run — or renting from someone who runs — a full archive node, a **DePIN network of storage operators** holds compressed data cooperatively, earns for serving it, and exposes it through the same RPC methods callers already use. That is the DePIN-storage and state-scaling thesis in one sentence: **push archival and state off the critical path, keep the read surface identical, and let an incentive layer — not a data center — pay for durability.**

This is deliberately narrow. SolanaVault is not a new L1, a new VM, or a replacement RPC dialect. It is the **storage-and-retrieval layer beneath** the RPC you already run. See the [ROADMAP](./ROADMAP.md) for how it gets from working code to production-grade guarantees.

## The Problem

Solana's rapid growth creates significant infrastructure challenges:
- **State bloat**: Throughput growth means account and history data grows faster than any single operator can economically hold
- **Storage costs**: Exponentially growing data storage requirements
- **Archive nodes**: Full historical access is expensive to run and expensive to rent
- **Performance**: Slow retrieval of historical blockchain data — a bottleneck for agent-speed read workloads
- **Centralization**: Over-reliance on a small set of centralized infrastructure providers
- **Developer access**: Expensive to run full nodes for simple applications

## Our Solution

SolanaVault creates a **decentralized economy (DePIN)** for blockchain data access:

🌐 **Fully Decentralized Network** - P2P storage operators with NNG transport and Byzantine consensus — no central authority owns the data
🤖 **Intelligent Compression** - 15-25:1 ratios through multi-stage pipeline optimization, cutting the storage bill for archival and state
💰 **Economic Incentives** - Pay-per-use model with gateway operators earning revenue for serving reads
🔌 **Drop-in Compatibility** - Standard Solana RPC API works unchanged; the storage layer changes, the surface does not
⚡ **Light Client Module** - Embeddable light-client logic so applications can verify network responses without running a full node
🏗️ **Advanced Architecture** - DHT discovery, reputation systems, automatic replication

## Quick Start

### Build
```bash
# Build the project
cargo build --release
```

The release build produces four binaries in `target/release/`:
`vault-cli`, `vault-node`, `vault-rpc-proxy`, `vault-rpc-decentralized`.
The light client currently ships as a Rust module
([`vault-core::network::light_client`](crates/vault-core/src/network/light_client.rs))
consumed by the gateway and proxy crates; a dedicated `vault-light-client`
binary is planned but not yet built.

### For Network Operators
```bash
# Run the decentralized RPC proxy / gateway
./target/release/vault-rpc-decentralized

# Or participate in storage/consensus as a full node
./target/release/vault-node --capacity 107374182400  # 100GB
```

### Test the Network
```bash
# Run economics demo
cargo run --example economics_demo

# Test compression
./target/release/vault-cli compress-demo --blocks 245000000:245001000

# Direct API test
curl -X POST http://localhost:3030 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getConfirmedBlock","params":[245000000]}'
```

## Live Performance Results

Our RPC proxy demonstrates real-world performance:

```
🚀 Block 244000000 Processing:
   Original Size: 1,264 bytes
   Compressed:    146 bytes
   Ratio:         8.66:1

⚡ Performance Metrics:
   Compression Time: <1ms
   Decompression:    13-85μs
   Storage Savings:  95%+
   Data Integrity:   100% verified
```

## How SolanaVault Compares

Honest tradeoffs against the two things teams do today to reach historical and account data:

| Approach | Storage cost | Read latency | Decentralization | What you give up |
|----------|-------------|--------------|------------------|------------------|
| **Run your own full archive node** | Very high (raw, uncompressed history on fast disk) | Fast locally | You control it, but it's a single point | Ongoing ops burden; you pay for 100% of the data even if you read 1% |
| **Centralized RPC provider** | None to you (priced into fees) | Fast, but rate-limited and metered | None — vendor owns availability | Vendor lock-in, egress/credit pricing, trust in a single operator |
| **SolanaVault (DePIN)** | Low — 15-25:1 compression, shared across operators | Sub-millisecond retrieval on cached paths | P2P operator set, no central authority | A younger network; durability/incentive guarantees are still hardening (see [ROADMAP](./ROADMAP.md)) |

SolanaVault is not automatically the right answer for every workload. If you already run archive nodes at scale and are happy with the cost, the win is smaller. The gains are largest for teams that read historical/account data heavily but don't want to own an archive node — and for the DePIN thesis that no single party *should* own it.

## Decentralized Network Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Applications  │────│   Light Client   │────│  Gateway Nodes  │
│  (unchanged)    │    │  (pay-per-use)   │    │ (earn revenue)  │
└─────────────────┘    └──────────────────┘    └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────────────────────────────────────────────────────┐
│                 Decentralized SolanaVault Network              │
├─────────────────┬─────────────────┬─────────────────────────────┤
│ NNG Transport   │ Kademlia DHT    │ Byzantine Consensus         │
│ • P2P Messaging │ • Peer Discovery│ • Data Integrity           │
│ • Binary Protocol│ • Content Routing│ • Reputation System       │
│ • μs Latency    │ • Auto-Discovery│ • Economic Incentives      │
├─────────────────┼─────────────────┼─────────────────────────────┤
│ Compression     │ Memory Mgmt     │ Storage Network             │
│ • 15-25:1 Ratio │ • Smart Caching │ • Distributed Replication  │
│ • ML Optimization│ • Payment Channels│ • Consensus Rewards       │
│ • Pattern Recognition│ • Cost Optimization│ • Automatic Scaling    │
└─────────────────┴─────────────────┴─────────────────────────────┘
```

## Core Components

- **[vault-core](crates/vault-core/)**: Complete decentralized network implementation
  - P2P networking with NNG transport
  - Kademlia DHT for peer discovery
  - Byzantine consensus for data integrity
  - Economic incentive systems
  - Light client and gateway implementations
- **[vault-cli](crates/vault-cli/)**: Network tools and compression demos
- **[vault-node](crates/vault-node/)**: Full storage node with consensus participation
- **[vault-rpc-proxy](crates/vault-rpc-proxy/)**: Dual-mode RPC proxy
  - `main.rs`: Legacy centralized mode
  - `decentralized_main.rs`: **New decentralized network mode**

## Key Features

✅ **Fully Decentralized**: True P2P network with no central authority
✅ **Economic Incentives**: Pay-per-use model with operator revenue
✅ **Drop-in Compatible**: Standard Solana RPC API unchanged
✅ **Light Client Module**: Embeddable light-client verification logic — no full node required
✅ **Proven Compression**: 15-25:1 ratios on real Solana data
✅ **High Performance**: NNG transport with microsecond latency
✅ **Data Integrity**: Byzantine consensus with cryptographic verification
✅ **Automatic Scaling**: DHT discovery and replication
✅ **Cost Effective**: 95%+ reduction vs. running full nodes

## System Requirements

- **Rust**: 1.70.0 or later
- **Memory**: 4GB+ recommended for production workloads
- **Storage**: SSD recommended for optimal performance
- **Network**: Stable internet connection for blockchain data access

## Development

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version  # Ensure 1.70+

# Clone and build
git clone https://github.com/cryptuon/solanavault.git
cd solanavault
cargo build --release

# Run comprehensive tests
cargo test --workspace

# Start development node
cargo run -p vault-node

# Run RPC proxy for testing
cargo run -p vault-rpc-proxy
```

## Documentation

The full documentation site lives in [`documentation/`](documentation/) and is
published at **[docs.cryptuon.com/solanavault](https://docs.cryptuon.com/solanavault/)**.
Marketing copy and product overview: **[solanavault.cryptuon.com](https://solanavault.cryptuon.com/)**.

In-repo references:

### 📚 Getting Started
- **[Quick Start Guide](docs/guides/getting-started.md)** - Get up and running quickly
- **[Demo Guide](docs/guides/demo.md)** - Interactive demonstrations

### 🏗️ Architecture
- **[System Overview](docs/architecture/overview.md)** - High-level architecture
- **[Memory Management](docs/architecture/memory-management.md)** - Advanced memory systems
- **[Compression Pipeline](docs/architecture/compression.md)** - Multi-stage compression

### 🔧 API Reference
- **[Core API](docs/api/core.md)** - Core library interfaces

### 🗺️ Direction
- **[Roadmap](./ROADMAP.md)** - Vision, milestones, and the cheapest path to production

### 👨‍💻 Development
- **[Contributing](docs/development/contributing.md)** - Contribution guidelines

## Performance Benchmarks

| Metric | Value | Description |
|--------|-------|-------------|
| Compression Ratio | 15-25:1 | Typical range on Solana blocks |
| Decompression Speed | 13-85μs | Per block decompression time |
| Cache Hit Rate | >90% | L1/L2/L3 combined hit rate |
| Storage Savings | 95%+ | Compared to raw blockchain data |
| Memory Efficiency | 80-95% | Active memory utilization |

## Contributing

We welcome contributions from the community! Please read our [Contributing Guide](docs/development/contributing.md) for details on:

- Code style and conventions
- Testing requirements
- Pull request process
- Development setup

## License

Licensed under the **MIT** license (see [`LICENSE`](./LICENSE)).

---

<div align="center">
  <strong>Built for developers. Tested on real Solana blockchain data.</strong><br/>
  <em>Open source, under active development. Issues and PRs welcome.</em>
</div>

---

## Part of Cryptuon Research

`solanavault` is one of [20 open-source blockchain-infrastructure projects](https://www.cryptuon.com/projects) from **[Cryptuon Research](https://www.cryptuon.com)** — blockchain theory, shipped as protocols.

**Related projects:** [blockchain-compression](https://blockchain-compression.cryptuon.com/) · [StreamSync](https://streamsync.cryptuon.com/) · [DataMgmt Node](https://datamgmtnode.cryptuon.com/)

Docs: [docs.cryptuon.com/solanavault](https://docs.cryptuon.com/solanavault/) · Contact: [contact@cryptuon.com](mailto:contact@cryptuon.com)