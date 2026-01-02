# Quick Start

Get SolanaVault running in under 5 minutes.

---

## Prerequisites

- **Rust** 1.70+ (for building from source)
- **Git** (for cloning the repository)
- **8GB RAM** minimum (16GB recommended)

---

## Option 1: Light Client (Recommended)

The light client is the easiest way to use SolanaVault. It connects to the decentralized network and handles compression/decompression automatically.

### Step 1: Install

```bash
# Clone the repository
git clone https://github.com/solanavault/solanavault
cd solanavault

# Build in release mode
cargo build --release
```

### Step 2: Start the Light Client

```bash
# Start with initial token balance
./target/release/vault-light-client start --balance 50000
```

### Step 3: Use Standard Solana Tools

Your existing Solana tools now route through SolanaVault automatically:

```bash
# Check balance (uses SolanaVault network)
solana balance

# Get block information
solana block 245000000

# Any standard Solana RPC call works
curl http://localhost:8899 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}'
```

---

## Option 2: Run the Demo

Experience SolanaVault's compression capabilities:

### Compression Demo

```bash
# Run the compression demo
./target/release/vault-cli compress-demo

# Compress specific block range
./target/release/vault-cli compress-demo --blocks 245000000:245001000
```

**Expected output:**

```
SolanaVault Compression Demo
============================
Fetching blocks 245000000 to 245001000...
Original size: 1,234,567 bytes
Compressed size: 61,728 bytes
Compression ratio: 20.0:1
Decompression time: 45 microseconds/block
```

### Economics Demo

```bash
# See the economic model in action
cargo run --example economics_demo
```

### Cost Analysis

```bash
# Analyze storage costs
./target/release/vault-cli cost-analysis
```

---

## Option 3: Run RPC Proxy

For development and testing, run the RPC proxy locally:

### Centralized Mode (Simple)

```bash
# Start centralized proxy
./target/release/vault-rpc-proxy

# Proxy is now available at http://localhost:3030
```

### Decentralized Mode (Production)

```bash
# Start decentralized proxy
./target/release/vault-rpc-decentralized

# Connects to the full P2P network
```

### Test the Proxy

```bash
# Get latest slot
curl http://localhost:3030 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}'

# Get block
curl http://localhost:3030 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getBlock","params":[245000000]}'

# Check proxy stats
curl http://localhost:3030/stats
```

---

## Verify Installation

Run the test suite to verify everything works:

```bash
# Run all tests
cargo test --workspace

# Run compression tests specifically
cargo test compression_integration

# Run with debug output
RUST_LOG=debug cargo test test_name -- --nocapture
```

---

## What's Next?

Now that you're up and running:

1. **Light Client Users**: See [Light Client Guide](light-client.md) for configuration options
2. **Infrastructure Operators**: See [Gateway Guide](gateway-operators.md) to earn revenue
3. **Node Operators**: See [Full Node Setup](full-node.md) to participate in consensus
4. **Developers**: See [API Reference](../reference/api.md) for integration details

---

## Troubleshooting

### Build fails with missing dependencies

```bash
# Install required system packages (Ubuntu/Debian)
sudo apt-get install build-essential pkg-config libssl-dev

# macOS
brew install openssl pkg-config
```

### Connection timeout

Ensure you have internet connectivity and can reach Solana RPC endpoints:

```bash
curl https://api.mainnet-beta.solana.com -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}'
```

### Out of memory

Increase available memory or use smaller block ranges:

```bash
# Process fewer blocks at once
./target/release/vault-cli compress-demo --blocks 245000000:245000100
```

See [Common Issues](../troubleshooting/common-issues.md) for more solutions.
