# Architecture Overview

SolanaVault is a fully decentralized blockchain compression and storage network. This document explains the system architecture and how components work together.

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           Applications Layer                             │
│              (Wallets, dApps, Block Explorers, Analytics)               │
└─────────────────────────────────┬───────────────────────────────────────┘
                                  │ Standard Solana JSON-RPC
┌─────────────────────────────────▼───────────────────────────────────────┐
│                           Light Client Layer                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                     │
│  │ Light Client│  │ Light Client│  │ Light Client│  ...                │
│  │  (App 1)    │  │  (App 2)    │  │  (App 3)    │                     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                     │
└─────────┼────────────────┼────────────────┼─────────────────────────────┘
          │                │                │ Micro-payments
┌─────────▼────────────────▼────────────────▼─────────────────────────────┐
│                          Gateway Layer                                   │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                     │
│  │   Gateway   │  │   Gateway   │  │   Gateway   │  ...                │
│  │   Node 1    │  │   Node 2    │  │   Node 3    │                     │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                     │
└─────────┼────────────────┼────────────────┼─────────────────────────────┘
          │                │                │ NNG P2P Protocol
┌─────────▼────────────────▼────────────────▼─────────────────────────────┐
│                       Decentralized Network                              │
│                                                                          │
│   ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐        │
│   │ Storage  │◄──►│ Storage  │◄──►│ Storage  │◄──►│ Storage  │        │
│   │  Node 1  │    │  Node 2  │    │  Node 3  │    │  Node N  │        │
│   └────┬─────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘        │
│        │               │               │               │               │
│        └───────────────┴───────┬───────┴───────────────┘               │
│                                │                                        │
│                    ┌───────────▼───────────┐                           │
│                    │   BFT Consensus       │                           │
│                    │   (Data Integrity)    │                           │
│                    └───────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Core Components

### 1. vault-core

The central library containing all core functionality:

| Module | Purpose |
|--------|---------|
| `compression/` | Multi-stage compression pipeline |
| `network/` | P2P networking, consensus, discovery |
| `economics/` | Staking, rewards, payments |
| `memory/` | Caching and storage management |
| `storage/` | Distributed storage interface |
| `data/` | Solana blockchain data access |

### 2. vault-light-client

Lightweight client for applications:

- Provides standard Solana RPC interface
- Handles micro-payments to gateways
- Caches responses locally
- Auto-discovers network gateways

### 3. vault-rpc-decentralized

Gateway node for serving clients:

- Monetized access to the network
- Load balancing and request routing
- Pricing and rate limiting
- Revenue collection (95% of fees)

### 4. vault-node

Full network participant:

- Stores compressed blockchain data
- Participates in BFT consensus
- Earns consensus and storage rewards
- Serves data to the network

### 5. vault-cli

Command-line tools:

- Compression demos
- Network diagnostics
- Cost analysis
- Administration utilities

---

## Network Layers

### Transport Layer (NNG)

SolanaVault uses NNG (nanomsg-next-generation) for P2P communication:

**Why NNG over HTTP/JSON-RPC?**

| Aspect | NNG | HTTP/JSON-RPC |
|--------|-----|---------------|
| Latency | Microseconds | Milliseconds |
| Overhead | Minimal binary | JSON parsing |
| Patterns | Pub/Sub, Req/Rep | Request only |
| Connections | Persistent | Per-request |

**Message Format:**

```
┌─────────────────────────────────────────┐
│ Header (16 bytes)                       │
│  ├── Message Type (2 bytes)             │
│  ├── Version (1 byte)                   │
│  ├── Flags (1 byte)                     │
│  ├── Sequence (4 bytes)                 │
│  └── Payload Length (8 bytes)           │
├─────────────────────────────────────────┤
│ Payload (variable)                      │
│  └── Bincode-serialized data            │
└─────────────────────────────────────────┘
```

### Discovery Layer (Kademlia DHT)

Nodes discover each other using a Kademlia-based DHT:

1. **Node ID**: SHA-256 hash of public key
2. **Routing Table**: k-buckets for peer discovery
3. **Lookup**: Find nodes closest to target ID
4. **Content Routing**: Locate data by content hash

```
Node Discovery Flow:
1. Connect to bootstrap nodes
2. Perform iterative node lookup
3. Populate routing table
4. Maintain table via periodic refresh
```

### Consensus Layer (BFT)

Byzantine Fault Tolerant consensus ensures data integrity:

**Properties:**

- Tolerates up to f Byzantine nodes (where n ≥ 3f + 1)
- 2/3 majority required for agreement
- Finality after single round (no forks)

**Consensus Round:**

```
┌─────────┐     ┌─────────┐     ┌─────────┐     ┌─────────┐
│ Propose │ ──► │ Pre-vote│ ──► │Pre-commit│ ──► │ Commit  │
└─────────┘     └─────────┘     └─────────┘     └─────────┘
    │               │               │               │
    ▼               ▼               ▼               ▼
 Leader         2/3 votes       2/3 votes       Finalized
 proposes       for proposal    for pre-commit  consensus
```

---

## Data Flow

### Write Path (Compression)

```
1. Fetch raw block from Solana RPC
           │
           ▼
2. Apply Stage 1 compression
   - Program clustering
   - Account dictionary
   - Blockhash delta encoding
           │
           ▼
3. Apply Stage 2 compression
   - Transaction analysis
   - Instruction templates
   - Pattern recognition
           │
           ▼
4. Apply Stage 3 compression
   - ML-based optimization
   - Entropy coding
   - Final compression (zstd)
           │
           ▼
5. BFT consensus verification
           │
           ▼
6. Distribute to storage nodes
   (replication factor: 3)
```

### Read Path (Decompression)

```
1. Application sends RPC request
           │
           ▼
2. Light client checks local cache
   ├── Cache hit  ──► Return cached data
   └── Cache miss ──► Continue
           │
           ▼
3. Route request to gateway
   - Pay micro-tokens
   - Priority routing
           │
           ▼
4. Gateway queries storage network
   - DHT lookup for data location
   - Fetch from nearest node
           │
           ▼
5. Decompress and verify
   - Verify integrity hash
   - Multi-stage decompression
           │
           ▼
6. Return to application
   - Cache result locally
   - Standard RPC response format
```

---

## Performance Characteristics

### Compression Performance

| Stage | Ratio | Speed | Purpose |
|-------|-------|-------|---------|
| Stage 1 | 3-5:1 | Fast | Structural compression |
| Stage 2 | 2-3:1 | Medium | Pattern recognition |
| Stage 3 | 2-3:1 | Slow | Maximum compression |
| **Total** | **15-25:1** | **Varies** | **Combined** |

### Network Performance

| Metric | Value |
|--------|-------|
| Message latency (P2P) | <1 ms |
| Consensus finality | <5 seconds |
| Query throughput | 1000+ qps/node |
| Cache hit rate | >90% |

### Storage Efficiency

| Metric | Value |
|--------|-------|
| Compression ratio | 15-25:1 |
| Decompression speed | 13-85 μs/block |
| Replication factor | 3x |
| Storage savings | 95%+ |

---

## On-Chain Programs

The tokenomics and governance are enforced through Solana smart contracts:

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

| Program | Purpose |
|---------|---------|
| **vault-token** | SPL Token with 1B supply cap, emission schedule |
| **vault-staking** | Tiered staking (Bronze→Platinum), 14-day unbonding |
| **vault-rewards** | 24-hour epochs, 95%/5% fee distribution |
| **vault-governance** | DAO voting with proposal types and timelocks |

See [Smart Contracts](smart-contracts.md) for full implementation details.

---

## Security Model

### Cryptographic Primitives

| Component | Algorithm |
|-----------|-----------|
| Node identity | Ed25519 |
| Data integrity | SHA-256 |
| Transport encryption | TLS 1.3 |
| Consensus signatures | BLS |

### Threat Mitigation

| Threat | Mitigation |
|--------|------------|
| Sybil attacks | Stake-weighted voting |
| Data corruption | BFT consensus verification |
| Eclipse attacks | DHT diversity requirements |
| DoS attacks | Rate limiting, reputation |

---

## Scalability

### Horizontal Scaling

The network scales horizontally:

- **More nodes** = More storage capacity
- **More gateways** = Higher throughput
- **Sharding** = Partition data by block range

### Bottleneck Analysis

| Component | Bottleneck | Mitigation |
|-----------|------------|------------|
| Consensus | Message complexity O(n²) | Hierarchical consensus |
| Storage | Disk I/O | NVMe, caching |
| Network | Bandwidth | Compression, batching |
| Computation | Decompression | Parallel processing |

---

## Next Steps

- [Smart Contracts](smart-contracts.md) - On-chain program details
- [Compression Pipeline](compression.md) - How compression works
- [Network Protocol](network.md) - P2P networking details
- [Tokenomics](tokenomics.md) - Token distribution and economics
- [Economics](economics.md) - Pricing and fee structure
- [API Reference](../reference/api.md) - Integration details
