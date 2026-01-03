# SolanaVault Documentation

Welcome to SolanaVault - a **fully decentralized blockchain compression and storage network** with economic incentives, designed specifically for the Solana ecosystem.

## 🌐 What's New: Decentralized Network

SolanaVault is now a **complete decentralized network** with:
- **Economic incentives** for network operators
- **Light clients** for developers who don't want to run full nodes
- **Drop-in compatibility** with existing Solana tools
- **Byzantine consensus** for data integrity
- **Pay-per-use model** that's cost-effective for all participants

## 📖 Documentation Structure

### 🚀 Getting Started
- **[Quick Start Guide](guides/getting-started.md)** - Get up and running in 5 minutes
- **[Demo Guide](guides/demo.md)** - See the decentralized network in action
- **[Economics Demo](../examples/economics_demo.rs)** - Understand the economic model

### 🌐 Network Access
- **[Light Client Guide](guides/light-client.md)** - For developers (recommended)
- **[Gateway Operations](guides/gateway-operations.md)** - For infrastructure providers
- **[Full Node Setup](guides/full-node-setup.md)** - For network participants

### 🏗️ Architecture
- **[Decentralized Architecture](architecture/decentralized.md)** - P2P network design
- **[Economic Model](architecture/economics.md)** - Incentives and payment flows
- **[Consensus Protocol](architecture/consensus.md)** - Byzantine Fault Tolerance
- **[Memory Management](architecture/memory-management.md)** - Advanced memory systems
- **[Compression Pipeline](architecture/compression.md)** - Multi-stage compression

### 🔧 API Reference
- **[Light Client API](api/light-client.md)** - Client library interfaces
- **[Gateway API](api/gateway.md)** - Gateway node operations
- **[RPC Proxy](api/rpc-proxy.md)** - Standard Solana RPC endpoints
- **[Network Protocol](api/network-protocol.md)** - P2P message formats

### 👨‍💻 Development
- **[Contributing](development/contributing.md)** - How to contribute to SolanaVault
- **[Network Development](development/network-development.md)** - P2P and consensus development
- **[Economics Development](development/economics-development.md)** - Payment and incentive systems
- **[Testing](development/testing.md)** - Testing the decentralized network

### 🔬 Research & Technical Papers
- **[Compression Algorithms](research/compression-algorithms.md)** - Technical details on compression algorithms
- **[Economics Model](research/economics.md)** - Tokenomics and incentive mechanisms
- **[Tokenomics](../documentation/docs/architecture/tokenomics.md)** - Complete token distribution and vesting
- **[Whitepaper](research/whitepaper/README.md)** - Academic whitepaper and research

## 💰 Economic Model

SolanaVault operates on a sustainable **pay-per-use model** powered by the **VAULT token**:

> **Token Details**: 1B total supply, 40% for network rewards, 14-day unbonding period. See [complete tokenomics](../documentation/docs/architecture/tokenomics.md).

### For Developers
- **Light clients** pay micro-tokens for network access
- **Automatic caching** reduces costs for repeated requests
- **Volume discounts** reward high-usage applications
- **No infrastructure** required - just use standard Solana APIs

### For Operators
- **Gateway nodes** earn 95% of client payments
- **Storage nodes** earn consensus rewards
- **Reputation bonuses** for reliable service
- **Dynamic pricing** balances supply and demand

### Cost Example
```
getBlock() request:
├─ Base fee: 100 micro-tokens
├─ Data fee: 500 micro-tokens (10KB compressed block)
├─ Total cost: 600 micro-tokens (~$0.0006)
└─ vs. Running full node: $200+/month
```

## 🎯 Access Patterns

### 1. Light Client (Recommended for Developers)
```bash
# Start local light client daemon
vault-light-client start --balance 50000

# Use existing Solana libraries unchanged
const connection = new Connection("http://localhost:8899");
```

### 2. Gateway Access (Direct Payment)
```bash
# Connect to paid gateway nodes
curl -H "Authorization: Bearer vault_token" \
  https://gateway1.solanavault.com/getBlock/245000000
```

### 3. Full Node (Earn Revenue)
```bash
# Participate in network and earn rewards
./target/release/vault-node --storage-capacity 100GB
./target/release/vault-rpc-decentralized --gateway-mode
```

## 🚀 Performance Highlights

**Network Performance**:
- **NNG Transport** - Microsecond-level P2P messaging
- **DHT Discovery** - Automatic peer finding and content routing
- **Byzantine Consensus** - Data integrity with 2/3 majority agreement

**Compression Performance**:
- **15-25:1 Compression Ratio** - Real-world Solana block compression
- **Sub-millisecond Decompression** - Lightning-fast data retrieval
- **95% Cost Reduction** - vs. running full archive nodes
- **100% Data Integrity** - Cryptographically verified

## System Architecture

```
Applications (unchanged)
    ↓
Light Client (pay micro-tokens)
    ↓
Gateway Nodes (earn revenue)
    ↓
Decentralized SolanaVault Network
    ├─ NNG P2P Transport
    ├─ Kademlia DHT Discovery
    ├─ Byzantine Consensus
    └─ Distributed Storage Nodes
```

## 🎯 Quick Navigation

### For Users
- [Light Client Setup](guides/light-client.md)
- [Getting Started](guides/getting-started.md)
- [Economics Demo](../examples/economics_demo.rs)

### For Developers
- [Decentralized Architecture](architecture/decentralized.md)
- [Contributing Guide](development/contributing.md)
- [Network Development](development/network-development.md)

### For Operators
- [Gateway Operations](guides/gateway-operations.md)
- [Full Node Setup](guides/full-node-setup.md)
- [Economics Model](architecture/economics.md)

### For Researchers
- [Whitepaper](research/whitepaper/README.md)
- [Compression Algorithms](research/compression-algorithms.md)
- [Consensus Protocol](architecture/consensus.md)

## Use Cases

### For Developers
- **No full nodes required** - Light client handles everything
- **Standard APIs work** - Drop-in replacement for Solana RPC
- **Pay only for usage** - No fixed infrastructure costs
- **Automatic optimization** - Caching and routing handled transparently

### For Infrastructure Providers
- **Gateway operations** - Earn revenue serving light clients
- **Storage participation** - Earn consensus rewards for data storage
- **Economic sustainability** - Clear revenue model with growing demand

### For the Ecosystem
- **Reduced centralization** - Distributed network vs. few large RPC providers
- **Lower barriers to entry** - Developers don't need expensive infrastructure
- **Economic incentives** - Self-sustaining network with operator rewards

## 🛠️ System Requirements

- **Rust**: 1.70.0 or later
- **Memory**: 4GB+ recommended for gateway/storage nodes
- **Storage**: SSD recommended for storage nodes
- **Network**: Stable internet connection for P2P networking
- **Light Clients**: Minimal requirements (can run on mobile devices)

## 📞 Support

- **Economics Demo**: `cargo run --example economics_demo`
- **Network Issues**: Report P2P or consensus problems on GitHub
- **Payment Questions**: See the economics documentation
- **Integration Help**: Light client and gateway setup guides

---

**Ready to join the decentralized network?** Start with our **[Light Client Guide](guides/light-client.md)** or explore the **[Economics Demo](../examples/economics_demo.rs)** to understand the full system.

**Last Updated**: 2025-09-29
**Version**: 0.1.0-decentralized