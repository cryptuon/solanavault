# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build all crates in release mode
cargo build --release

# Build specific crates
cargo build -p vault-core
cargo build -p vault-cli
cargo build -p vault-node
cargo build -p vault-rpc-proxy

# Run tests for all workspace crates
cargo test --workspace

# Run specific test suites
cargo test -p vault-core
cargo test compression_integration
cargo test --test compression_integration

# Run with debug logging
RUST_LOG=debug cargo test test_name -- --nocapture
RUST_BACKTRACE=1 cargo test test_name -- --nocapture

# Run single test function
cargo test test_v2_compression_basic -- --nocapture
```

## Running Components

```bash
# Start RPC proxy (drop-in Solana RPC replacement)
./target/release/vault-rpc-proxy
# or with logging:
RUST_LOG=debug ./target/release/vault-rpc-proxy

# Run compression demo
./target/release/vault-cli compress-demo --blocks 245000000:245001000

# Deploy compressed data to vault network
./target/release/vault-cli deploy-to-vault --compressed-blocks compressed_blocks.vault

# Run cost analysis
./target/release/vault-cli cost-analysis

# Start storage node
./target/release/vault-node

# Run complete demo sequence
./demo.sh
```

## Architecture Overview

SolanaVault is a Rust workspace with 4 main crates:

### Core Components

- **vault-core**: Central library containing all core functionality
  - `compression/`: Multi-stage compression pipeline (v1, v2, v3 algorithms)
  - `memory/`: Advanced memory management (RocksDB storage, multi-level caching, memory pools)
  - `data/`: Solana block data access and caching layer
  - `storage/`: Distributed storage network implementation
  - `network/`: P2P communication protocols
  - `economics/`: Staking and incentive mechanisms
  - `workflows/`: End-to-end operation orchestration

- **vault-rpc-proxy**: Drop-in replacement for Solana RPC that serves compressed data
- **vault-cli**: Command-line tools for network interaction and demos
- **vault-node**: Storage node implementation for the distributed network

### Compression Pipeline

The system uses a 3-stage compression pipeline:

1. **Stage 1**: Program clustering and structural optimization
2. **Stage 2**: Transaction analysis and template extraction
3. **Stage 3**: ML optimization using XGBoost for strategy selection

Compression versions:
- **V1**: Baseline compression (10:1 ratio)
- **V2**: Enhanced compression (25:1 ratio)
- **V3**: Advanced ML-based compression (15-25:1 ratio in production)

### Memory Management

Three-level cache hierarchy implemented:
- **L1 Cache**: Hot data, uncompressed (sub-microsecond access)
- **L2 Cache**: Warm data, compressed (microsecond access)
- **L3 Cache**: Cold data, persistent storage (millisecond access)

Uses RocksDB for persistent storage with intelligent caching strategies.

### External Dependencies

- **blockchain-compression**: Located at `../blockchain-compression`, provides core compression algorithms
- **Solana SDK**: Version 1.18 for blockchain data structures
- **Standard compression**: zstd, lz4, flate2 for baseline algorithms

## Testing Patterns

The codebase has comprehensive testing with 48 files containing tests:

- Integration tests in `crates/vault-core/tests/`
- Unit tests embedded in source files with `#[test]` attributes
- Test utilities in `src/data/test_utils.rs`
- Real Solana block data testing patterns

## Environment Variables

Key environment variables for debugging and configuration:

```bash
RUST_LOG=debug                          # General debug logging
RUST_LOG=vault_core::compression=debug  # Compression-specific logging
RUST_BACKTRACE=1                        # Backtraces on panics
RUST_BACKTRACE=full                     # Full backtraces
```

## Key Configuration

- Workspace uses Rust 2021 edition
- Solana SDK version 1.18 across all crates
- Tokio async runtime with full features
- Production builds should use `--release` flag for optimal compression performance

## Development Workflow

1. Make changes to relevant crate in `crates/`
2. Test specific crate: `cargo test -p crate-name`
3. Build in release mode: `cargo build --release`
4. Test RPC proxy: `./target/release/vault-rpc-proxy`
5. Verify compression: `./target/release/vault-cli compress-demo`

## Performance Notes

- Compression performance is critical - always test with `--release` builds
- Use `RUST_LOG=debug` for detailed compression pipeline analysis
- Memory management is optimized for large block processing
- RPC proxy provides sub-millisecond response times when properly configured