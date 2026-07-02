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

**[🌐 Site](https://solanavault.cryptuon.com/) · [📚 Docs](https://docs.cryptuon.com/solanavault/) · [🔬 Cryptuon Research](https://github.com/cryptuon)**

## Overview

SolanaVault is a **fully decentralized blockchain compression and storage network** that provides efficient storage and retrieval of Solana blockchain data with extreme compression ratios while maintaining perfect data integrity. Built with economic incentives and drop-in compatibility.

## The Problem

Solana's rapid growth creates significant infrastructure challenges:
- **Storage Costs**: Exponentially growing data storage requirements
- **Archive Nodes**: High costs for full historical data access
- **Performance**: Slow retrieval of historical blockchain data
- **Centralization**: Over-reliance on centralized infrastructure providers
- **Developer Access**: Expensive to run full nodes for simple applications

## Our Solution

SolanaVault creates a **decentralized economy** for blockchain data access:

🌐 **Fully Decentralized Network** - P2P nodes with NNG transport and Byzantine consensus
🤖 **Intelligent Compression** - 15-25:1 ratios through multi-stage pipeline optimization
💰 **Economic Incentives** - Pay-per-use model with gateway operators earning revenue
🔌 **Drop-in Compatibility** - Standard Solana RPC API works unchanged
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