# Getting Started with SolanaVault

This guide will help you quickly set up and start using SolanaVault for Solana blockchain data compression and storage.

## Prerequisites

### System Requirements
- **Operating System**: Linux, macOS, or Windows with WSL2
- **Rust**: Version 1.70.0 or later
- **Memory**: Minimum 4GB RAM (8GB+ recommended for production)
- **Storage**: 10GB+ free space (SSD recommended)
- **Network**: Stable internet connection

### Installing Rust
If you don't have Rust installed:

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
rustc --version
cargo --version
```

## Installation

### 1. Clone the Repository
```bash
git clone https://github.com/your-org/solanavault.git
cd solanavault
```

### 2. Build the Project
```bash
# Build all components in release mode
cargo build --release

# This creates binaries in target/release/:
# - vault-cli
# - vault-node
# - vault-rpc-proxy
```

### 3. Verify Installation
```bash
# Check that binaries were created
ls -la target/release/

# Test basic functionality
./target/release/vault-cli --help
```

## Quick Start Examples

### Example 1: Basic Compression Demo

Test compression on historical Solana blocks:

```bash
# Compress a range of historical blocks
./target/release/vault-cli compress-demo --blocks 245000000:245001000

# Expected output:
# 🚀 Processing 1000 blocks...
# ✅ Compressed: 1,264,000 bytes → 146,000 bytes (8.66:1 ratio)
# 💰 Storage savings: 88.5%
# ⚡ Processing time: 2.3 seconds
```

### Example 2: Start RPC Proxy

Launch the RPC proxy for transparent Solana API access:

```bash
# Start the RPC proxy server
./target/release/vault-rpc-proxy

# Expected output:
# 🚀 SolanaVault RPC Proxy - Production Network Starting...
# ✅ Blockchain compression initialized
# ✅ Historical block storage ready
# 🌐 Server listening on http://127.0.0.1:3030
```

### Example 3: Test with Standard Solana Tools

Use the RPC proxy with existing Solana tooling:

```bash
# In another terminal, test with curl
curl -X POST http://localhost:3030 \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getConfirmedBlock",
    "params": [245000000]
  }'

# Test with Solana CLI (if installed)
solana -u http://localhost:3030 block 245000000
```

## Understanding the Components

### 1. vault-cli
Command-line interface for interacting with SolanaVault:

```bash
# Available commands
./target/release/vault-cli --help

# Compression demo
./target/release/vault-cli compress-demo --help

# Cost analysis
./target/release/vault-cli cost-analysis --help

# Deploy to vault network
./target/release/vault-cli deploy-to-vault --help
```

### 2. vault-rpc-proxy
Drop-in replacement for Solana RPC that automatically routes requests:

```bash
# Start with default configuration
./target/release/vault-rpc-proxy

# Start with custom configuration
./target/release/vault-rpc-proxy --port 8080 --upstream https://api.mainnet-beta.solana.com
```

**Routing Logic:**
- **Historical blocks** (>1000 slots old): Served from compressed storage
- **Recent blocks**: Proxied to upstream Solana RPC
- **Other requests**: Passed through to upstream RPC

### 3. vault-node
Storage node for the distributed SolanaVault network:

```bash
# Start a storage node
./target/release/vault-node

# Start with custom configuration
./target/release/vault-node --data-dir ./my-vault-data --port 4040
```

## Configuration

### Environment Variables
```bash
# Optional: Set custom configuration
export VAULT_DATA_DIR="./vault-data"
export VAULT_LOG_LEVEL="info"
export VAULT_RPC_PORT="3030"
export VAULT_UPSTREAM_RPC="https://api.mainnet-beta.solana.com"
```

### Configuration Files
SolanaVault uses sensible defaults, but you can customize behavior:

```toml
# vault-config.toml (optional)
[storage]
data_dir = "./vault-data"
max_cache_size = "256MB"
compression_level = "high"

[network]
rpc_port = 3030
upstream_rpc = "https://api.mainnet-beta.solana.com"
max_connections = 1000

[compression]
target_ratio = 20.0
max_compression_time = "5s"
enable_ml_optimization = true
```

## Development Workflow

### 1. Running Tests
```bash
# Run all tests
cargo test --workspace

# Run specific test suites
cargo test -p vault-core
cargo test -p vault-rpc-proxy

# Run with output
cargo test --workspace -- --nocapture
```

### 2. Development Mode
```bash
# Build in development mode (faster compilation, less optimization)
cargo build

# Run with debug logging
RUST_LOG=debug ./target/debug/vault-rpc-proxy

# Run specific components
cargo run -p vault-cli -- compress-demo --blocks 245000000:245000010
cargo run -p vault-rpc-proxy
cargo run -p vault-node
```

### 3. Code Formatting and Linting
```bash
# Format code
cargo fmt

# Run clippy for linting
cargo clippy -- -D warnings

# Check for issues
cargo check
```

## Monitoring and Debugging

### 1. Logging
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/vault-rpc-proxy

# Enable specific module logging
RUST_LOG=vault_core::compression=debug ./target/release/vault-rpc-proxy

# Log to file
RUST_LOG=info ./target/release/vault-rpc-proxy 2>&1 | tee vault.log
```

### 2. Performance Monitoring
```bash
# View compression statistics
curl http://localhost:3030/stats

# Monitor system resources
htop
iostat -x 1
```

### 3. Troubleshooting

#### Common Issues

**Build Errors:**
```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
cargo build --release
```

**Network Connection Issues:**
```bash
# Test upstream RPC connectivity
curl -X POST https://api.mainnet-beta.solana.com \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getVersion"}'

# Check port availability
netstat -tulpn | grep 3030
```

**Memory Issues:**
```bash
# Monitor memory usage
free -h
ps aux | grep vault

# Reduce memory usage in configuration
export VAULT_MAX_CACHE_SIZE="128MB"
```

## Next Steps

### Production Deployment
- **[Configuration Guide](configuration.md)** - Advanced configuration options
- **[Deployment Guide](../deployment/)** - Production deployment strategies
- **[Monitoring Guide](../monitoring/)** - Production monitoring setup

### Development
- **[Contributing Guide](../development/contributing.md)** - How to contribute
- **[Architecture Overview](../architecture/overview.md)** - Understanding the system
- **[API Reference](../api/core.md)** - Detailed API documentation

### Advanced Usage
- **[Demo Guide](demo.md)** - Interactive demonstrations
- **[Performance Tuning](performance-tuning.md)** - Optimization techniques
- **[Network Operations](network-operations.md)** - Multi-node deployment

## Support

- **Issues**: [GitHub Issues](https://github.com/your-org/solanavault/issues)
- **Documentation**: [docs/](../)
- **Examples**: [examples/](../../examples/)

---

**Next**: [Demo Guide](demo.md)