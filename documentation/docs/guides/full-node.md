# Full Node Setup

Run a full SolanaVault node to participate in consensus, store compressed blockchain data, and earn rewards.

---

## Overview

A full node:

- **Stores compressed blockchain data** on the network
- **Participates in BFT consensus** for data integrity
- **Earns consensus rewards** for honest participation
- **Serves data requests** to other network participants

---

## Requirements

### Hardware Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| CPU | 8 cores | 16+ cores |
| RAM | 32 GB | 64 GB |
| Storage | 1 TB NVMe | 4 TB NVMe |
| Network | 1 Gbps | 10 Gbps |
| Uptime | 99% | 99.9% |

### Network Requirements

- Static public IP address
- Ports 4040 (P2P) and 4041 (consensus) open
- Low-latency connections to other nodes

---

## Quick Start

### Step 1: Build the Node

```bash
git clone https://github.com/solanavault/solanavault
cd solanavault
cargo build --release --bin vault-node
```

### Step 2: Initialize Storage

```bash
# Create data directory
sudo mkdir -p /data/solanavault
sudo chown $USER:$USER /data/solanavault

# Initialize node
./target/release/vault-node init --data-path /data/solanavault
```

### Step 3: Configure the Node

Create `~/.solanavault/node.toml`:

```toml
[node]
name = "my-fullnode"
data_path = "/data/solanavault"
storage_capacity_gb = 1000

[network]
p2p_port = 4040
consensus_port = 4041
public_address = "tcp://your-ip:4040"
bootstrap_nodes = [
    "tcp://bootstrap1.solanavault.com:4040",
    "tcp://bootstrap2.solanavault.com:4040"
]

[consensus]
participation = true
stake_amount = 10000

[storage]
compression_level = "high"
replication_factor = 3
```

### Step 4: Start the Node

```bash
./target/release/vault-node \
  --config ~/.solanavault/node.toml \
  --storage-capacity 1000GB
```

### Step 5: Verify Operation

```bash
# Check node status
curl http://localhost:4042/status

# View sync progress
curl http://localhost:4042/sync
```

---

## Configuration Reference

### Node Settings

```toml
[node]
# Node identifier (auto-generated if not specified)
name = "my-fullnode"

# Data storage path
data_path = "/data/solanavault"

# Maximum storage capacity in GB
storage_capacity_gb = 1000

# Enable block verification
verify_blocks = true

# Log level: trace, debug, info, warn, error
log_level = "info"
```

### Network Settings

```toml
[network]
# P2P communication port
p2p_port = 4040

# Consensus protocol port
consensus_port = 4041

# Admin API port
admin_port = 4042

# Your public address for other nodes to connect
public_address = "tcp://your-ip:4040"

# Bootstrap nodes for initial discovery
bootstrap_nodes = [
    "tcp://bootstrap1.solanavault.com:4040",
    "tcp://bootstrap2.solanavault.com:4040"
]

# Maximum peer connections
max_peers = 100

# Connection timeout (ms)
connection_timeout_ms = 10000
```

### Consensus Settings

```toml
[consensus]
# Enable consensus participation
participation = true

# Stake amount (affects voting weight)
stake_amount = 10000

# Minimum stake required
min_stake = 1000

# Slashing protection
enable_slashing_protection = true

# Vote timeout (ms)
vote_timeout_ms = 5000

# Maximum rounds per consensus
max_rounds = 10
```

### Storage Settings

```toml
[storage]
# Compression level: "low", "medium", "high", "maximum"
compression_level = "high"

# Number of copies across network
replication_factor = 3

# Local cache size (MB)
cache_size_mb = 8192

# Enable data verification
verify_on_read = true

# Garbage collection interval (hours)
gc_interval_hours = 24
```

---

## Consensus Participation

### How Consensus Works

SolanaVault uses Byzantine Fault Tolerant (BFT) consensus:

1. **Data Proposal**: A node proposes new compressed data
2. **Verification**: Other nodes verify the compression
3. **Voting**: Nodes vote on data validity (2/3 majority required)
4. **Commitment**: Verified data is committed to the network

### Staking

Stake tokens to participate in consensus:

```bash
# Deposit stake
vault-node stake deposit 10000

# Check stake status
vault-node stake status

# Withdraw stake (after unbonding period)
vault-node stake withdraw
```

### Reputation System

Your node builds reputation through:

- **Uptime**: Consistent availability
- **Correct Votes**: Voting with the majority
- **Data Serving**: Successfully serving requests
- **Fast Responses**: Low-latency participation

Check your reputation:

```bash
vault-node reputation
```

### Slashing Conditions

Stakes can be slashed for:

| Violation | Penalty |
|-----------|---------|
| Double voting | 10% of stake |
| Data corruption | 20% of stake |
| Extended downtime | 1% per day |
| Invalid proposals | 5% of stake |

---

## Rewards

### Reward Types

| Reward Type | Description |
|-------------|-------------|
| Consensus Rewards | Earned for participating in voting |
| Storage Rewards | Earned for storing and serving data |
| Availability Bonus | Bonus for high uptime |
| Early Adopter | Bonus for early network participants |

### Reward Calculation

```
Daily Reward = (Stake × Uptime × Reputation × Network_Factor)
```

Example:

```
Stake: 10,000 tokens
Uptime: 99.5%
Reputation: 0.95
Network Factor: 0.001

Daily Reward = 10,000 × 0.995 × 0.95 × 0.001 = 9.45 tokens
```

### Claiming Rewards

```bash
# View pending rewards
vault-node rewards pending

# Claim rewards
vault-node rewards claim
```

---

## Data Synchronization

### Initial Sync

When first starting, your node syncs with the network:

```bash
# Check sync status
vault-node sync status
```

**Output:**

```
Sync Status: In Progress
  Current Height: 245,123,456
  Network Height: 250,000,000
  Progress: 98.05%
  ETA: 2 hours 15 minutes
  Speed: 1,234 blocks/second
```

### Sync Modes

| Mode | Description | Speed |
|------|-------------|-------|
| Full | Download all data | Slowest |
| Fast | Download headers + recent data | Medium |
| Warp | Trust checkpoints, download recent | Fastest |

Configure sync mode:

```toml
[sync]
mode = "fast"  # "full", "fast", or "warp"
checkpoint_verification = true
parallel_downloads = 8
```

---

## Monitoring

SolanaVault provides multiple monitoring options:

- **TUI Dashboard**: Terminal-based interface with `--tui` flag
- **Web Dashboard**: Browser-based interface with `--dashboard-port PORT`
- **REST API**: Programmatic access to node metrics

See the [Node Dashboard Guide](node-dashboard.md) for complete dashboard documentation.

### Status Endpoint

```bash
curl http://localhost:4042/status
```

**Response:**

```json
{
  "node_id": "abc123...",
  "version": "1.0.0",
  "uptime_seconds": 86400,
  "sync_status": "synced",
  "current_height": 250000000,
  "peer_count": 45,
  "consensus_participation": true,
  "stake": 10000,
  "reputation": 0.95,
  "storage_used_gb": 456.7,
  "storage_capacity_gb": 1000
}
```

### Metrics

```bash
curl http://localhost:4042/metrics
```

### Logs

```bash
# View logs
journalctl -u solanavault-node -f

# Debug logging
RUST_LOG=debug ./target/release/vault-node ...
```

---

## High Availability

### Redundant Setup

Run multiple nodes for redundancy:

```bash
# Node 1
./target/release/vault-node --config node1.toml

# Node 2 (different machine)
./target/release/vault-node --config node2.toml
```

### Backup and Recovery

```bash
# Backup node data
vault-node backup create /backup/solanavault-backup.tar.gz

# Restore from backup
vault-node backup restore /backup/solanavault-backup.tar.gz
```

---

## Production Deployment

### Systemd Service

Create `/etc/systemd/system/solanavault-node.service`:

```ini
[Unit]
Description=SolanaVault Full Node
After=network.target

[Service]
Type=simple
User=solanavault
ExecStart=/opt/solanavault/vault-node --config /etc/solanavault/node.toml
Restart=always
RestartSec=10
LimitNOFILE=65535
LimitNPROC=65535

[Install]
WantedBy=multi-user.target
```

### Performance Tuning

```toml
[performance]
# Thread pool size
worker_threads = 16

# I/O buffer size (KB)
io_buffer_kb = 256

# Connection pool size
connection_pool = 100

# Enable memory mapping
mmap_enabled = true
```

### System Tuning

```bash
# Increase file descriptor limit
echo "solanavault soft nofile 65535" >> /etc/security/limits.conf
echo "solanavault hard nofile 65535" >> /etc/security/limits.conf

# Optimize network settings
echo "net.core.rmem_max = 134217728" >> /etc/sysctl.conf
echo "net.core.wmem_max = 134217728" >> /etc/sysctl.conf
sysctl -p
```

---

## Troubleshooting

### Node Won't Start

```bash
# Check for port conflicts
netstat -tlnp | grep -E "4040|4041|4042"

# Verify permissions
ls -la /data/solanavault

# Check logs
RUST_LOG=debug vault-node --config ~/.solanavault/node.toml
```

### Sync Stalled

```bash
# Check peer connections
vault-node peers list

# Force resync from checkpoint
vault-node sync reset --from-checkpoint
```

### Low Reputation

1. Improve uptime (aim for 99.9%+)
2. Ensure correct consensus voting
3. Verify data integrity
4. Reduce response latency

---

## Next Steps

- [Node Dashboard](node-dashboard.md) - Monitor your node with TUI or Web Dashboard
- [Gateway Operators Guide](gateway-operators.md) - Also run a gateway
- [Network Architecture](../architecture/network.md) - Understand the protocol
- [Economics](../architecture/economics.md) - Learn about rewards
- [Configuration Reference](../reference/configuration.md) - All options
