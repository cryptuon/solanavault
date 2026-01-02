# Installation

Detailed installation instructions for all SolanaVault components.

---

## System Requirements

### Minimum Requirements

| Component | Light Client | Gateway Node | Full Node |
|-----------|-------------|--------------|-----------|
| CPU | 2 cores | 4 cores | 8 cores |
| RAM | 4 GB | 8 GB | 32 GB |
| Storage | 1 GB | 100 GB SSD | 1 TB NVMe |
| Network | 10 Mbps | 100 Mbps | 1 Gbps |

### Supported Platforms

- **Linux**: Ubuntu 20.04+, Debian 11+, CentOS 8+, Fedora 35+
- **macOS**: 12.0+ (Monterey and later)
- **Windows**: WSL2 recommended

---

## Prerequisites

### Rust Toolchain

Install Rust 1.70 or later:

```bash
# Install Rust via rustup
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Reload shell environment
source ~/.cargo/env

# Verify installation
rustc --version  # Should be 1.70.0 or later
cargo --version
```

### System Dependencies

=== "Ubuntu/Debian"

    ```bash
    sudo apt-get update
    sudo apt-get install -y \
        build-essential \
        pkg-config \
        libssl-dev \
        libclang-dev \
        cmake \
        git
    ```

=== "macOS"

    ```bash
    # Install Xcode command line tools
    xcode-select --install

    # Install dependencies via Homebrew
    brew install openssl pkg-config cmake
    ```

=== "Fedora/RHEL"

    ```bash
    sudo dnf install -y \
        gcc \
        gcc-c++ \
        openssl-devel \
        pkgconfig \
        clang-devel \
        cmake \
        git
    ```

=== "Arch Linux"

    ```bash
    sudo pacman -S \
        base-devel \
        openssl \
        pkg-config \
        clang \
        cmake \
        git
    ```

---

## Installation Methods

### Method 1: Build from Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/solanavault/solanavault.git
cd solanavault

# Build all components in release mode
cargo build --release

# Binaries are located in ./target/release/
ls -la ./target/release/vault-*
```

**Available binaries:**

| Binary | Description |
|--------|-------------|
| `vault-cli` | Command-line tools and demos |
| `vault-node` | Full network node |
| `vault-rpc-proxy` | Centralized RPC proxy |
| `vault-rpc-decentralized` | Decentralized RPC proxy |
| `vault-light-client` | Lightweight client |

### Method 2: Install Specific Components

Build only what you need:

```bash
# Light client only
cargo build --release --bin vault-light-client

# CLI tools only
cargo build --release --bin vault-cli

# Gateway/proxy only
cargo build --release --bin vault-rpc-decentralized

# Full node only
cargo build --release --bin vault-node
```

### Method 3: Development Build

For development with debug symbols:

```bash
# Debug build (faster compilation, slower runtime)
cargo build

# Run tests
cargo test --workspace

# Run with debug logging
RUST_LOG=debug ./target/debug/vault-cli compress-demo
```

---

## Post-Installation Setup

### Add to PATH

```bash
# Add to your shell profile (~/.bashrc, ~/.zshrc, etc.)
export PATH="$PATH:/path/to/solanavault/target/release"

# Reload shell
source ~/.bashrc  # or ~/.zshrc
```

### Verify Installation

```bash
# Check CLI
vault-cli --help

# Run compression demo
vault-cli compress-demo

# Check version
vault-cli --version
```

### Create Configuration Directory

```bash
# Create config directory
mkdir -p ~/.solanavault

# Example configuration
cat > ~/.solanavault/config.toml << 'EOF'
[network]
bootstrap_nodes = [
    "tcp://bootstrap1.solanavault.com:4040",
    "tcp://bootstrap2.solanavault.com:4040"
]

[light_client]
cache_size_mb = 512
auto_reconnect = true

[gateway]
pricing_strategy = "dynamic"
volume_discount_threshold = 10000
EOF
```

---

## Environment Variables

Configure SolanaVault behavior via environment variables:

```bash
# Logging
export RUST_LOG=info                           # Log level: trace, debug, info, warn, error
export RUST_LOG=vault_core::network=debug      # Component-specific logging

# Network
export VAULT_BOOTSTRAP_NODES="tcp://node1:4040,tcp://node2:4040"
export VAULT_GATEWAY_ENDPOINT="https://gateway.solanavault.com"

# Development
export RUST_BACKTRACE=1                        # Enable backtraces
```

---

## Updating

### Update from Source

```bash
cd solanavault

# Fetch latest changes
git pull origin main

# Rebuild
cargo build --release
```

### Clean Build

If you encounter issues after updating:

```bash
# Clean build artifacts
cargo clean

# Rebuild from scratch
cargo build --release
```

---

## Uninstallation

### Remove Binaries

```bash
# Remove built binaries
rm -rf /path/to/solanavault/target

# Or remove specific binaries from PATH location
rm /usr/local/bin/vault-*
```

### Remove Configuration

```bash
# Remove config and data
rm -rf ~/.solanavault
```

---

## Next Steps

- [Quick Start](quickstart.md) - Get running quickly
- [Light Client Guide](light-client.md) - Configure the light client
- [Gateway Guide](gateway-operators.md) - Set up a gateway node
- [Full Node Setup](full-node.md) - Run a full network node
