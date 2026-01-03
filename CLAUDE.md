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

# Build decentralized binaries
cargo build --release --bin vault-rpc-proxy          # Centralized proxy
cargo build --release --bin vault-rpc-decentralized  # Decentralized proxy

# Run tests for all workspace crates
cargo test --workspace

# Run specific test suites
cargo test -p vault-core
cargo test compression_integration
cargo test --test compression_integration

# Run with debug logging
RUST_LOG=debug cargo test test_name -- --nocapture
RUST_BACKTRACE=1 cargo test test_name -- --nocapture

# Economics demo
cargo run --example economics_demo
```

## Running Components

```bash
# Decentralized Network (Recommended)
./target/release/vault-rpc-decentralized    # Full decentralized proxy
./target/release/vault-node                 # Storage node
./target/release/vault-cli compress-demo    # Demo compression

# Legacy Centralized Mode
./target/release/vault-rpc-proxy            # Centralized proxy (legacy)

# Light Client for Applications
./target/release/vault-light-client start --balance 100000

# Development
RUST_LOG=debug ./target/release/vault-rpc-decentralized
./demo.sh  # Complete demo sequence
```

## Node UI (TUI & Web Dashboard)

The vault-node supports both a Terminal User Interface (TUI) and a Web Dashboard, driven by the same core `NodeDashboardApi`.

### Building with UI Features

```bash
# Build with TUI only
cargo build -p vault-node --features tui --release

# Build with Web Dashboard only
cargo build -p vault-node --features dashboard --release

# Build with both TUI and Web Dashboard
cargo build -p vault-node --features full --release
```

### Running with UI

```bash
# Run with TUI (terminal interface)
./target/release/vault-node --tui

# Run with Web Dashboard on port 3000
./target/release/vault-node --dashboard-port 3000

# Both can run simultaneously
./target/release/vault-node --dashboard-port 3000  # Then access http://localhost:3000
```

### Building the Vue.js Frontend

The web dashboard uses Vue.js + Tailwind CSS. To build:

```bash
cd crates/vault-node/dashboard-frontend
npm install
npm run build
```

Then rebuild vault-node with `cargo build -p vault-node --features dashboard --release`.

### Dashboard API Endpoints

When running with `--dashboard-port`:

- `GET /api/stats` - Full node statistics
- `GET /api/storage` - Storage metrics only
- `GET /api/network` - Network metrics only
- `GET /api/economics` - Economics metrics only
- `GET /api/history` - Metrics history for charts
- `GET /api/health` - Health check
- `WS /ws` - WebSocket for real-time updates

### UI Architecture

```
vault-node binary
    |
    +---> --tui flag ---------> TUI (ratatui)
    |                              |
    +---> --dashboard-port ---> Web Dashboard (axum + Vue.js)
    |                              |
    +------------------------------+
               |
       NodeDashboardApi (shared in vault-core)
               |
    +----------+----------+
    |          |          |
 Storage   Network   Economics
```

### TUI Features

- 4 tabs: Overview, Storage, Network, Economics
- Real-time 1-second refresh
- Keyboard navigation: Tab/Arrow keys, 1-4 number keys, q to quit
- Sparkline charts for time-series data
- Gauges for capacity visualization

### Web Dashboard Features

- Real-time updates via WebSocket
- Responsive grid layout with Tailwind CSS
- Dark theme optimized for monitoring
- Interactive charts and gauges

## Architecture Overview

SolanaVault is a **fully decentralized network** with economic incentives:

### Core Components

- **vault-core**: Central library with decentralized networking
  - `network/`: Complete P2P networking stack
    - `transport.rs`: NNG-based high-performance transport
    - `discovery.rs`: Kademlia DHT for peer discovery
    - `consensus.rs`: Byzantine Fault Tolerant consensus
    - `decentralized.rs`: Coordinated network manager
    - `light_client.rs`: Lightweight client for non-node users
    - `gateway.rs`: Monetized network access points
  - `compression/`: Multi-stage compression (15-25:1 ratios)
  - `memory/`: Advanced caching and storage
  - `economics/`: Staking, rewards, and payment systems

- **vault-rpc-proxy**: Two modes of operation
  - `main.rs`: Legacy centralized proxy
  - `decentralized_main.rs`: **New decentralized proxy**
- **vault-cli**: Network tools and demos
- **vault-node**: Full network participant with storage

### Decentralized Network Architecture

**Transport Layer**: NNG (nanomsg-next-generation) for P2P communication
- Binary message serialization (not JSON)
- Publisher/Subscriber patterns
- Direct peer connections
- Microsecond-level latency

**Discovery Layer**: Kademlia DHT
- Automatic peer discovery
- Content-based routing
- Bootstrap node support
- Geographic distribution

**Consensus Layer**: Byzantine Fault Tolerant
- Data integrity verification
- 2/3 majority requirements
- Reputation-based slashing
- Automatic conflict resolution

**Economic Layer**: Pay-per-use model
- Light clients pay micro-tokens
- Gateway operators earn revenue
- Network fees fund infrastructure
- Volume discounts and surge pricing

### Client Access Patterns

**1. Light Client (Recommended for most users)**
```bash
# Install and run locally
vault-light-client start --wallet-balance 50000
# Use standard Solana APIs that now connect to decentralized network
```

**2. Direct Gateway Access**
```bash
# Connect directly to gateway nodes for paid access
curl -H "Authorization: Bearer vault_token" https://gateway1.solanavault.com
```

**3. Full Node Participation**
```bash
# Run complete node and earn consensus rewards
./target/release/vault-node --storage-capacity 100GB
```

### Economic Model

**Cost Structure (micro-tokens)**:
- Base fee: 100μ per request
- Data fee: 50μ per KB
- Priority: 1.5x multiplier
- Volume discounts: Up to 25% off

**Revenue Distribution**:
- 95% to gateway operators
- 5% to network development fund
- Consensus rewards for storage nodes
- Reputation bonuses for reliability

## Testing Patterns

The codebase has comprehensive testing including decentralized network tests:

- Integration tests: `crates/vault-core/tests/`
- Network tests: P2P, consensus, and economic simulations
- Economics demo: `examples/economics_demo.rs`
- Real blockchain data testing

## Environment Variables

```bash
# Networking
RUST_LOG=debug                                    # General debug
RUST_LOG=vault_core::network=debug               # Network-specific
RUST_LOG=vault_core::network::consensus=debug    # Consensus only

# Development
RUST_BACKTRACE=1                                 # Error traces
VAULT_BOOTSTRAP_NODES=tcp://node1:4040,tcp://node2:4040  # Bootstrap peers
VAULT_GATEWAY_ENDPOINT=https://gateway.solanavault.com   # Gateway URL
```

## Key Dependencies

- **NNG**: High-performance networking transport
- **Blockchain-compression**: Core compression algorithms at `../blockchain-compression`
- **Solana SDK**: v1.18 for blockchain compatibility
- **Tokio**: Async runtime for network operations
- **Serde**: Message serialization
- **UUID, SHA2**: Cryptographic operations

## Development Workflow

### For Network Development:
1. Start with `cargo run --example economics_demo`
2. Test decentralized proxy: `./target/release/vault-rpc-decentralized`
3. Test light client integration
4. Deploy gateway nodes for production

### For Compression Development:
1. Test specific algorithms: `cargo test compression_integration`
2. Benchmark: `cargo run -p vault-cli -- compress-demo`
3. Verify integrity: Check round-trip compression

### For Economic Development:
1. Modify pricing in `network/gateway.rs`
2. Test with `economics_demo.rs`
3. Verify payment flows and incentives

## Production Deployment

**Gateway Node**:
```bash
./target/release/vault-rpc-decentralized \
  --gateway-mode \
  --pricing-config pricing.json \
  --storage-capacity 1TB
```

**Light Client**:
```bash
./target/release/vault-light-client \
  --ipc-path /tmp/vault.sock \
  --gateway tcp://gateway.solanavault.com:5050
```

**Storage Node**:
```bash
./target/release/vault-node \
  --storage-path /data/vault \
  --consensus-participation true \
  --bootstrap tcp://bootstrap.solanavault.com:4040
```

## Performance Notes

- **Network**: NNG provides superior performance vs HTTP/JSON-RPC
- **Compression**: 15-25:1 ratios require `--release` builds
- **Economics**: Micro-payments require payment channel optimization
- **Consensus**: Byzantine agreement adds latency but ensures integrity
- **Caching**: Light clients cache aggressively to minimize costs

## Key Concepts

- **Drop-in Compatibility**: Standard Solana RPC API works unchanged
- **Economic Sustainability**: Users pay for usage, operators earn revenue
- **True Decentralization**: No central points of failure or control
- **Automatic Optimization**: Intelligent routing, caching, and pricing
- **Developer-Friendly**: Existing tools and workflows continue to work