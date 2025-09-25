# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

SolanaVault is a revolutionary blockchain data compression and storage system that achieves 1271:1 compression ratios on Solana data, reducing storage costs by 96%. The system implements a multi-layer architecture with compression algorithms, distributed storage, P2P networking, and token economics.

## Build & Development Commands

### Core Commands
```bash
# Build entire workspace (recommended for development)
cargo build --release

# Quick syntax check without building
cargo check --workspace

# Run all tests
cargo test --workspace

# Run specific test module
cargo test compression_tests

# Run integration tests (requires network access)
cargo test --test compression_tests -- --ignored

# Run a specific test
cargo test test_v1_compression_basic
```

### Demo Commands
```bash
# Full compression demo (demonstrates 1271:1 ratio)
./target/release/vault-cli compress-demo --blocks 245000000:245001000

# Deploy compressed data to storage network
./target/release/vault-cli deploy-to-vault --compressed-blocks compressed_blocks.vault

# Cost analysis comparison
./target/release/vault-cli cost-analysis --blocks 1000000

# Start storage node
./target/release/vault-node --node-id test-node --debug

# Start RPC proxy (drop-in Solana RPC replacement)
./target/release/vault-rpc-proxy
```

### Compression Algorithm Testing
```bash
# Test production compression (PracticalMaxCompression)
cargo test test_practical_max_compression -- --nocapture

# Test comprehensive compression performance
cargo test test_comprehensive_compression_performance -- --nocapture

# Debug compression stage by stage
cargo test test_debug_stage_by_stage -- --nocapture
```

## Architecture Overview

### Core Components Structure
- **vault-core**: Central library containing all core algorithms and systems
- **vault-cli**: Command-line interface for network interaction and demos
- **vault-node**: Storage node implementation for the distributed network
- **vault-rpc-proxy**: Drop-in replacement for Solana RPC with historical data support

### vault-core Architecture
The core library is organized into 5 main layers:

1. **Compression Layer** (`src/compression/`)
   - **Production Algorithm**: `production_v3.rs` - 1271:1 compression using PracticalMaxCompression
   - **Versioned Algorithms**: `v1.rs` (10:1), `v2.rs` (25:1), `v3.rs` (47:1) for compatibility
   - **Multi-Stage Pipeline**: `stage1/` (structural), `stage2/` (bot intelligence), `stage3/` (ML)
   - **Key Type**: `ProductionCompressor = PracticalMaxCompression`

2. **Storage Layer** (`src/storage/`)
   - **StorageNode**: Individual node with capacity and reputation tracking
   - **StorageNetwork**: Manages 3+ node distributed storage with replication
   - **ReplicationStrategy**: 3x replication with 2/3 availability requirement

3. **Network Layer** (`src/network/`)
   - **P2P System**: `p2p.rs` - Peer discovery, mesh networking, reputation system
   - **Bootstrap Support**: Network discovery through bootstrap nodes

4. **Economics Layer** (`src/economics/`)
   - **Staking System**: `staking.rs` - Token staking, performance scoring, slashing
   - **Reward Distribution**: `rewards.rs` - Performance-based reward calculation
   - **APY**: 8-15% based on performance metrics

5. **Data Layer** (`src/data/`)
   - **SolanaBlockClient**: Interfaces with Solana RPC for block data
   - **BlockCache**: Caching layer for frequently accessed blocks

### Compression Algorithm Details

The system uses a 5-stage compression pipeline in `production_v3.rs`:

1. **Stage 1 (85% compression)**: Solana-specific structural patterns
   - Account dictionary: 32 bytes → 2 bytes mapping
   - Program clustering: Common program deduplication
   - Blockhash delta compression

2. **Stage 2 (additional 80%)**: Bot intelligence
   - Arbitrage bot pattern detection
   - Signature clustering using VAE
   - Predictable failure pattern compression

3. **Stage 3 (additional 70%)**: Temporal analysis
   - Account state evolution tracking
   - Zero-value transfer bitmap optimization
   - Cross-block correlation patterns

4. **Stage 4 (additional 60%)**: Neural compression
   - Instruction VAE for latent patterns
   - Context-aware dictionaries
   - Adaptive Huffman encoding

5. **Stage 5 (additional 40%)**: Entropy optimization
   - Context Tree Weighting (CTW)
   - DEFLATE final compression
   - Cryptographic integrity verification

## Key Implementation Notes

### Compression Strategy Pattern
All compression algorithms implement the `CompressionStrategy` trait:
```rust
pub trait CompressionStrategy {
    fn compress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn decompress(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError>;
    fn version(&self) -> CompressionVersion;
}
```

### Production Algorithm Usage
Use `ProductionCompressor` (alias for `PracticalMaxCompression`) for new code:
```rust
use vault_core::compression::ProductionCompressor;
let compressor = ProductionCompressor::new();
```

### Economic System Integration
The economics system requires performance metrics collection:
- **Uptime tracking**: Node availability percentage
- **Response time monitoring**: Average retrieval latency
- **Storage proofs**: Cryptographic proof of data possession
- **Success rate tracking**: Data retrieval success percentage

### Error Handling
The system uses `CompressionError` for all compression operations and `thiserror` for structured error handling throughout the codebase.

### Testing Strategy
- **Unit tests**: Embedded in each module with `#[cfg(test)]`
- **Integration tests**: In `crates/tests/` directory
- **Real data testing**: Tests marked with `#[ignore]` require network access
- **Performance benchmarks**: Use `-- --nocapture` to see detailed output

### Network Protocol
The P2P layer uses:
- **Peer discovery**: DHT-based routing with Kademlia-style lookups
- **Reputation system**: Performance-based peer scoring
- **Bootstrap nodes**: Initial network entry points
- **Mesh topology**: Full connectivity between storage nodes

This architecture enables SolanaVault to achieve its revolutionary compression ratios while maintaining data integrity and economic sustainability through carefully designed incentive mechanisms.