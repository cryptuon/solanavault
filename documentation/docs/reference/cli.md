# CLI Commands

Complete reference for SolanaVault command-line tools.

---

## vault-cli

General-purpose CLI for SolanaVault operations.

### compress-demo

Run compression demonstration.

```bash
vault-cli compress-demo [OPTIONS]

Options:
  --blocks <RANGE>     Block range (e.g., 245000000:245001000)
  --output <FILE>      Output file for compressed data
  --level <LEVEL>      Compression level: low, medium, high, maximum
  --verbose            Show detailed statistics
```

**Examples:**

```bash
# Basic demo
vault-cli compress-demo

# Specific block range
vault-cli compress-demo --blocks 245000000:245001000

# Maximum compression
vault-cli compress-demo --blocks 245000000:245000100 --level maximum

# Save output
vault-cli compress-demo --blocks 245000000:245000100 --output blocks.vault
```

### deploy-to-vault

Deploy compressed blocks to the network.

```bash
vault-cli deploy-to-vault [OPTIONS]

Options:
  --compressed-blocks <FILE>    Compressed blocks file
  --replication <N>             Replication factor (default: 3)
  --verify                      Verify after deployment
```

**Example:**

```bash
vault-cli deploy-to-vault --compressed-blocks blocks.vault --verify
```

### cost-analysis

Analyze storage and network costs.

```bash
vault-cli cost-analysis [OPTIONS]

Options:
  --blocks <RANGE>     Block range to analyze
  --output <FORMAT>    Output format: text, json, csv
```

**Example:**

```bash
vault-cli cost-analysis --blocks 245000000:246000000 --output json
```

### config

Manage configuration.

```bash
vault-cli config <SUBCOMMAND>

Subcommands:
  validate <FILE>      Validate configuration file
  show                 Show current configuration
  init                 Initialize default configuration
```

**Examples:**

```bash
# Validate config
vault-cli config validate ~/.solanavault/node.toml

# Show resolved config
vault-cli config show --resolved

# Initialize new config
vault-cli config init --type gateway
```

---

## vault-light-client

Light client for application access.

### start

Start the light client.

```bash
vault-light-client start [OPTIONS]

Options:
  --balance <TOKENS>    Initial token balance
  --port <PORT>         Local RPC port (default: 8899)
  --gateway <URL>       Gateway endpoint
  --config <FILE>       Configuration file
  --background          Run in background
```

**Examples:**

```bash
# Basic start
vault-light-client start --balance 50000

# Custom port
vault-light-client start --balance 50000 --port 9000

# Specific gateway
vault-light-client start --gateway tcp://gateway.example.com:5050
```

### stop

Stop the running light client.

```bash
vault-light-client stop
```

### status

Check light client status.

```bash
vault-light-client status
```

**Output:**

```
Light Client Status
==================
Status: Running
Local RPC: http://localhost:8899
Connected Gateway: gateway1.solanavault.com
Balance: 45,230 tokens
Cache Hit Rate: 89%
Uptime: 2h 34m
```

### balance

Manage token balance.

```bash
vault-light-client balance [SUBCOMMAND]

Subcommands:
  show                 Show current balance
  add <AMOUNT>         Add tokens
  history              Show spending history
  alert <THRESHOLD>    Set low balance alert
```

**Examples:**

```bash
# Check balance
vault-light-client balance

# Add tokens
vault-light-client balance add 50000

# View history
vault-light-client balance history --days 7

# Set alert
vault-light-client balance alert --threshold 10000
```

### gateways

Manage gateway connections.

```bash
vault-light-client gateways <SUBCOMMAND>

Subcommands:
  list                 List known gateways
  test <URL>           Test gateway connection
  add <URL>            Add gateway manually
  remove <URL>         Remove gateway
```

**Examples:**

```bash
# List gateways
vault-light-client gateways list

# Test specific gateway
vault-light-client gateways test tcp://gateway.example.com:5050
```

---

## vault-node

Full network node.

### init

Initialize node data directory.

```bash
vault-node init [OPTIONS]

Options:
  --data-path <PATH>    Data directory path
  --force               Overwrite existing data
```

**Example:**

```bash
vault-node init --data-path /data/solanavault
```

### start

Start the full node.

```bash
vault-node [OPTIONS]

Options:
  --config <FILE>            Configuration file
  --storage-capacity <GB>    Storage capacity
  --data-path <PATH>         Data directory
  --bootstrap <NODES>        Bootstrap nodes (comma-separated)
```

**Examples:**

```bash
# Basic start
vault-node --storage-capacity 1000GB

# With config
vault-node --config ~/.solanavault/node.toml

# Specify bootstrap
vault-node --bootstrap tcp://node1:4040,tcp://node2:4040
```

### stake

Manage staking.

```bash
vault-node stake <SUBCOMMAND>

Subcommands:
  deposit <AMOUNT>     Deposit stake
  withdraw [AMOUNT]    Withdraw stake
  status               Show stake status
```

**Examples:**

```bash
# Deposit stake
vault-node stake deposit 10000

# Check status
vault-node stake status

# Withdraw
vault-node stake withdraw
```

### rewards

Manage rewards.

```bash
vault-node rewards <SUBCOMMAND>

Subcommands:
  pending              Show pending rewards
  claim                Claim rewards
  history              Show reward history
```

**Examples:**

```bash
# Check pending
vault-node rewards pending

# Claim all
vault-node rewards claim

# View history
vault-node rewards history --days 30
```

### sync

Manage synchronization.

```bash
vault-node sync <SUBCOMMAND>

Subcommands:
  status               Show sync status
  reset                Reset and resync
```

**Examples:**

```bash
# Check sync status
vault-node sync status

# Force resync
vault-node sync reset --from-checkpoint
```

### peers

Manage peer connections.

```bash
vault-node peers <SUBCOMMAND>

Subcommands:
  list                 List connected peers
  ban <ID>             Ban a peer
  unban <ID>           Unban a peer
```

### backup

Backup and restore.

```bash
vault-node backup <SUBCOMMAND>

Subcommands:
  create <FILE>        Create backup
  restore <FILE>       Restore from backup
  verify <FILE>        Verify backup integrity
```

**Examples:**

```bash
# Create backup
vault-node backup create /backup/node-backup.tar.gz

# Restore
vault-node backup restore /backup/node-backup.tar.gz
```

---

## vault-rpc-decentralized

Decentralized RPC proxy/gateway.

### Running

```bash
vault-rpc-decentralized [OPTIONS]

Options:
  --gateway-mode         Run as gateway (serve clients)
  --config <FILE>        Configuration file
  --http-port <PORT>     HTTP RPC port (default: 3030)
  --p2p-port <PORT>      P2P port (default: 4040)
  --pricing-config <F>   Pricing configuration file
```

**Examples:**

```bash
# Run as gateway
vault-rpc-decentralized --gateway-mode

# Custom ports
vault-rpc-decentralized --gateway-mode --http-port 8080 --p2p-port 5000

# With pricing config
vault-rpc-decentralized --gateway-mode --pricing-config pricing.toml
```

---

## vault-rpc-proxy

Centralized RPC proxy (legacy).

```bash
vault-rpc-proxy [OPTIONS]

Options:
  --port <PORT>          HTTP port (default: 3030)
  --upstream <URL>       Upstream Solana RPC
  --cache-size <MB>      Cache size in MB
```

**Example:**

```bash
vault-rpc-proxy --port 3030 --cache-size 1024
```

---

## Global Options

Available for all commands:

| Option | Description |
|--------|-------------|
| `--config <FILE>` | Configuration file |
| `--log-level <LEVEL>` | Log level: trace, debug, info, warn, error |
| `-v, --verbose` | Increase verbosity |
| `-q, --quiet` | Decrease verbosity |
| `--json` | Output in JSON format |
| `--version` | Show version |
| `--help` | Show help |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Configuration error |
| 4 | Network error |
| 5 | Storage error |

---

## Examples

### Complete Workflow

```bash
# 1. Initialize node
vault-node init --data-path /data/solanavault

# 2. Start full node
vault-node --storage-capacity 1000GB &

# 3. Wait for sync
vault-node sync status  # Wait until 100%

# 4. Stake tokens
vault-node stake deposit 10000

# 5. Run gateway
vault-rpc-decentralized --gateway-mode &

# 6. Start light client
vault-light-client start --balance 50000

# 7. Use the network
curl http://localhost:8899 -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}'
```

---

## Next Steps

- [Configuration Reference](configuration.md) - Configuration options
- [API Reference](api.md) - API documentation
- [Quick Start](../guides/quickstart.md) - Get started guide
