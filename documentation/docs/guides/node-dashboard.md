# Node Dashboard

Monitor your SolanaVault node with the built-in Terminal UI (TUI) or Web Dashboard.

---

## Overview

SolanaVault provides two interfaces for monitoring your node:

| Interface | Best For | Access |
|-----------|----------|--------|
| **TUI** | Server terminals, SSH sessions | `--tui` flag |
| **Web Dashboard** | Remote monitoring, multiple nodes | `--dashboard-port PORT` |

Both interfaces share the same underlying `NodeDashboardApi` and display identical metrics.

---

## Building with Dashboard Features

Dashboard features are optional and controlled via Cargo feature flags.

### Build Options

=== "TUI Only"

    ```bash
    cargo build -p vault-node --features tui --release
    ```

=== "Web Dashboard Only"

    ```bash
    cargo build -p vault-node --features dashboard --release
    ```

=== "Both (Full)"

    ```bash
    cargo build -p vault-node --features full --release
    ```

!!! note "Default Build"
    The default build (`cargo build -p vault-node`) does not include dashboard features to minimize binary size.

---

## Terminal UI (TUI)

The TUI provides a full-screen terminal interface for monitoring your node.

### Starting the TUI

```bash
./target/release/vault-node --tui
```

Or with other options:

```bash
./target/release/vault-node \
  --node-id my-node \
  --data-dir /data/vault \
  --capacity 107374182400 \
  --tui
```

### TUI Navigation

| Key | Action |
|-----|--------|
| `Tab` / `→` | Next tab |
| `Shift+Tab` / `←` | Previous tab |
| `1` - `4` | Jump to tab |
| `q` / `Esc` | Quit |

### TUI Tabs

#### Overview Tab

Displays:

- Node info (ID, address, version, uptime, status)
- Storage gauge with usage percentage
- Network summary (peers, messages)
- Activity sparkline (messages over time)

#### Storage Tab

Detailed storage metrics:

- Total/used/available capacity
- Blocks stored
- Cache hit/miss counts and rate
- Compression ratio and space saved
- Historical sparklines

#### Network Tab

Network and consensus metrics:

- Connected/total peers
- Messages sent/received
- Bandwidth usage
- Consensus proposals and votes
- Reputation score

#### Economics Tab

Staking and rewards:

- Own stake and network total
- Pending rewards
- Performance score
- APY (base and effective)
- Gateway revenue (if applicable)

---

## Web Dashboard

The web dashboard provides a browser-based interface with real-time updates via WebSocket.

### Starting the Web Dashboard

```bash
./target/release/vault-node --dashboard-port 3000
```

Access at: `http://localhost:3000`

### Running with Both Node and Dashboard

```bash
./target/release/vault-node \
  --node-id my-node \
  --data-dir /data/vault \
  --dashboard-port 3000
```

The node runs normally while the dashboard serves on the specified port.

### Building the Vue.js Frontend

The web dashboard frontend is built with Vue.js and Tailwind CSS.

```bash
# Navigate to frontend directory
cd crates/vault-node/dashboard-frontend

# Install dependencies
npm install

# Build for production
npm run build

# Development mode (with hot reload)
npm run dev
```

After building, rebuild vault-node:

```bash
cargo build -p vault-node --features dashboard --release
```

!!! tip "Development Workflow"
    During development, run `npm run dev` in the frontend directory. The Vite dev server proxies API requests to `http://localhost:3000`.

---

## Dashboard API

The web dashboard exposes REST and WebSocket endpoints.

### REST Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/health` | GET | Health check |
| `/api/stats` | GET | Full node statistics |
| `/api/storage` | GET | Storage metrics only |
| `/api/network` | GET | Network metrics only |
| `/api/economics` | GET | Economics metrics only |
| `/api/history` | GET | Metrics history for charts |

### Example: Get All Stats

```bash
curl http://localhost:3000/api/stats | jq
```

**Response:**

```json
{
  "timestamp": 1704067200,
  "node_info": {
    "node_id": "my-node",
    "address": "127.0.0.1:8080",
    "version": "0.1.0",
    "uptime_seconds": 3600,
    "status": "Running"
  },
  "storage": {
    "total_capacity": 107374182400,
    "used_capacity": 5368709120,
    "available_capacity": 102005473280,
    "blocks_stored": 150,
    "compression_ratio": 18.5,
    "total_original_bytes": 1073741824,
    "total_compressed_bytes": 58041905,
    "cache_hits": 1250,
    "cache_misses": 87,
    "cache_hit_rate": 0.935
  },
  "network": {
    "total_peers": 50,
    "connected_peers": 45,
    "messages_sent": 12500,
    "messages_received": 14200,
    "bandwidth_in_bytes": 0,
    "bandwidth_out_bytes": 0,
    "average_latency_ms": 0.0
  },
  "economics": {
    "staking": {
      "total_staked": 100000000,
      "own_stake": 10000000,
      "pending_rewards": 50000,
      "performance_score": 1.05,
      "base_apy": 0.12
    },
    "rewards": {
      "total_earned": 250000,
      "distributed_this_epoch": 5000,
      "epochs_completed": 50
    },
    "gateway": null
  },
  "consensus": {
    "active_proposals": 2,
    "votes_cast": 500,
    "proposals_accepted": 480,
    "proposals_rejected": 15,
    "reputation_score": 0.98
  }
}
```

### WebSocket Real-time Updates

Connect to `/ws` for real-time metric updates (1-second interval).

**JavaScript Example:**

```javascript
const ws = new WebSocket('ws://localhost:3000/ws');

ws.onopen = () => {
  console.log('Connected to dashboard');
};

ws.onmessage = (event) => {
  const snapshot = JSON.parse(event.data);
  console.log('Storage used:', snapshot.storage.used_capacity);
  console.log('Compression ratio:', snapshot.storage.compression_ratio);
};

ws.onclose = () => {
  console.log('Disconnected');
};
```

---

## Metrics Reference

### Node Info

| Metric | Type | Description |
|--------|------|-------------|
| `node_id` | string | Unique node identifier |
| `address` | string | Node network address |
| `version` | string | Software version |
| `uptime_seconds` | u64 | Time since start |
| `status` | enum | Starting, Running, Syncing, Degraded, Stopped |

### Storage Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `total_capacity` | u64 | Total storage in bytes |
| `used_capacity` | u64 | Used storage in bytes |
| `available_capacity` | u64 | Free storage in bytes |
| `blocks_stored` | u64 | Number of blocks |
| `compression_ratio` | f64 | Average compression ratio |
| `total_original_bytes` | u64 | Pre-compression bytes |
| `total_compressed_bytes` | u64 | Post-compression bytes |
| `cache_hits` | u64 | Cache hit count |
| `cache_misses` | u64 | Cache miss count |
| `cache_hit_rate` | f64 | Hit rate (0.0-1.0) |

### Network Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `total_peers` | usize | Known peer count |
| `connected_peers` | usize | Active connections |
| `messages_sent` | u64 | Outbound messages |
| `messages_received` | u64 | Inbound messages |
| `bandwidth_in_bytes` | u64 | Inbound bandwidth |
| `bandwidth_out_bytes` | u64 | Outbound bandwidth |
| `average_latency_ms` | f64 | Average RTT |

### Economics Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `total_staked` | u64 | Network total stake |
| `own_stake` | u64 | This node's stake |
| `pending_rewards` | u64 | Unclaimed rewards |
| `performance_score` | f64 | Performance multiplier (0.0-2.0) |
| `base_apy` | f64 | Base annual yield |
| `total_earned` | u64 | Lifetime rewards |

### Consensus Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `active_proposals` | usize | Open proposals |
| `votes_cast` | u64 | Total votes |
| `proposals_accepted` | u64 | Accepted count |
| `proposals_rejected` | u64 | Rejected count |
| `reputation_score` | f64 | Node reputation (0.0-1.0+) |

---

## Architecture

```
vault-node binary
    |
    +---> --tui flag ---------> TUI (ratatui)
    |                              |
    +---> --dashboard-port ---> Web Dashboard (Axum + Vue.js)
    |                              |
    +------------------------------+
               |
       NodeDashboardApi (vault-core)
               |
    +----------+----------+
    |          |          |
 Storage   Network   Economics
```

### Key Components

| Component | Location | Description |
|-----------|----------|-------------|
| `NodeDashboardApi` | `vault-core/src/dashboard/api.rs` | Shared API |
| `DashboardSnapshot` | `vault-core/src/dashboard/metrics.rs` | Metric types |
| `MetricsHistory` | `vault-core/src/dashboard/history.rs` | Time-series |
| `TuiApp` | `vault-node/src/tui/app.rs` | TUI application |
| `WebDashboard` | `vault-node/src/dashboard/server.rs` | HTTP server |

---

## Customization

### Extending the Dashboard API

Add custom metrics by implementing `NetworkStatsProvider`:

```rust
use vault_core::dashboard::{NetworkStatsProvider, NetworkMetrics};
use async_trait::async_trait;

struct MyNetworkProvider {
    // Your network tracking state
}

#[async_trait]
impl NetworkStatsProvider for MyNetworkProvider {
    async fn get_network_stats(&self) -> NetworkMetrics {
        NetworkMetrics {
            total_peers: 100,
            connected_peers: 85,
            messages_sent: 50000,
            messages_received: 48000,
            bandwidth_in_bytes: 1024 * 1024,
            bandwidth_out_bytes: 512 * 1024,
            average_latency_ms: 15.5,
        }
    }
}
```

### Custom Frontend Components

Add Vue.js components in `dashboard-frontend/src/components/`:

```vue
<template>
  <div class="card">
    <h2 class="text-lg font-semibold text-vault-cyan mb-4">Custom Metric</h2>
    <div class="text-4xl font-bold text-vault-green">
      {{ value }}
    </div>
  </div>
</template>

<script setup>
import { computed } from 'vue'
const props = defineProps(['snapshot'])
const value = computed(() => props.snapshot?.custom_field || 0)
</script>
```

---

## Troubleshooting

### TUI Not Rendering Correctly

```bash
# Check terminal type
echo $TERM

# Try forcing 256 colors
export TERM=xterm-256color
./target/release/vault-node --tui
```

### Web Dashboard Connection Refused

```bash
# Check if port is in use
netstat -tlnp | grep 3000

# Try different port
./target/release/vault-node --dashboard-port 8080
```

### WebSocket Disconnects

The dashboard automatically reconnects after 2 seconds. If issues persist:

```bash
# Check for firewall issues
sudo ufw status

# Enable WebSocket port
sudo ufw allow 3000/tcp
```

### Frontend Not Loading

If you see the placeholder page:

```bash
# Build the frontend
cd crates/vault-node/dashboard-frontend
npm install
npm run build

# Rebuild vault-node
cargo build -p vault-node --features dashboard --release
```

---

## Next Steps

- [Full Node Setup](full-node.md) - Complete node configuration
- [API Reference](../reference/api.md) - All API endpoints
- [Configuration](../reference/configuration.md) - Node options
- [Architecture Overview](../architecture/overview.md) - System design
