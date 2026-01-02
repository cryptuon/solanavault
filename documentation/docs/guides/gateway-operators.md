# Gateway Operators Guide

Gateway operators run access points to the SolanaVault network and earn revenue by serving light clients. This guide covers setup, configuration, and optimization for gateway operators.

---

## Overview

As a gateway operator, you:

- **Serve light client requests** from the decentralized network
- **Earn 95% of fees** paid by light clients
- **Contribute to network decentralization**
- **Set your own pricing** within market parameters

---

## Requirements

### Hardware Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8 GB | 32 GB |
| Storage | 100 GB SSD | 500 GB NVMe |
| Network | 100 Mbps | 1 Gbps |
| Uptime | 95% | 99.9% |

### Network Requirements

- Public IP address or domain name
- Ports 4040 (P2P), 5050 (client), and 3030 (HTTP) open
- Low-latency connection to Solana RPC endpoints

---

## Quick Start

### Step 1: Build the Gateway

```bash
git clone https://github.com/solanavault/solanavault
cd solanavault
cargo build --release --bin vault-rpc-decentralized
```

### Step 2: Configure the Gateway

Create `~/.solanavault/gateway.toml`:

```toml
[gateway]
name = "my-gateway"
public_address = "tcp://your-ip:5050"
http_port = 3030

[pricing]
base_fee = 100           # Base fee per request (micro-tokens)
data_fee_per_kb = 50     # Additional fee per KB of data
priority_multiplier = 1.5
volume_discount = true

[network]
p2p_port = 4040
bootstrap_nodes = [
    "tcp://bootstrap1.solanavault.com:4040",
    "tcp://bootstrap2.solanavault.com:4040"
]

[storage]
cache_size_gb = 50
data_path = "/data/solanavault"
```

### Step 3: Start the Gateway

```bash
./target/release/vault-rpc-decentralized \
  --gateway-mode \
  --config ~/.solanavault/gateway.toml
```

### Step 4: Verify Operation

```bash
# Check gateway status
curl http://localhost:3030/gateway/status

# Test client request
curl http://localhost:3030 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}'
```

---

## Configuration Reference

### Gateway Settings

```toml
[gateway]
# Display name for your gateway
name = "my-gateway"

# Public address that clients connect to
public_address = "tcp://your-domain.com:5050"

# HTTP RPC port for client requests
http_port = 3030

# Maximum concurrent connections
max_connections = 1000

# Connection timeout in milliseconds
connection_timeout_ms = 30000

# Enable gateway mode (required)
enabled = true
```

### Pricing Configuration

```toml
[pricing]
# Pricing strategy: "fixed", "dynamic", or "market"
strategy = "dynamic"

# Base fee per request (micro-tokens)
base_fee = 100

# Additional fee per KB of data returned
data_fee_per_kb = 50

# Multiplier for priority requests
priority_multiplier = 1.5

# Enable volume discounts
volume_discount = true

# Volume discount tiers
[[pricing.discount_tiers]]
threshold = 10000
discount = 0.10

[[pricing.discount_tiers]]
threshold = 100000
discount = 0.20

[[pricing.discount_tiers]]
threshold = 1000000
discount = 0.25

# Surge pricing (during high demand)
[pricing.surge]
enabled = true
threshold_utilization = 0.80
max_multiplier = 2.0
```

### Network Configuration

```toml
[network]
# P2P port for node communication
p2p_port = 4040

# Bootstrap nodes for network discovery
bootstrap_nodes = [
    "tcp://bootstrap1.solanavault.com:4040",
    "tcp://bootstrap2.solanavault.com:4040"
]

# Maximum peer connections
max_peers = 50

# Peer discovery interval (seconds)
discovery_interval = 60

# Enable geographic peer preference
prefer_local_peers = true
```

### Storage Configuration

```toml
[storage]
# Local cache size in GB
cache_size_gb = 50

# Data storage path
data_path = "/data/solanavault"

# Enable persistent cache
persistent_cache = true

# Cache eviction policy: "lru", "lfu", or "adaptive"
eviction_policy = "adaptive"
```

---

## Revenue Model

### Fee Structure

As a gateway operator, you receive **95%** of all fees paid by clients:

```
Client pays: 150 micro-tokens
├── Gateway receives: 142.5 micro-tokens (95%)
└── Network fund: 7.5 micro-tokens (5%)
```

### Example Revenue

| Daily Requests | Avg Fee | Daily Revenue |
|---------------|---------|---------------|
| 10,000 | 100μ | 950,000μ |
| 100,000 | 100μ | 9,500,000μ |
| 1,000,000 | 100μ | 95,000,000μ |

### Revenue Optimization

1. **Competitive Pricing**: Lower prices attract more clients
2. **High Availability**: Better uptime = more requests
3. **Low Latency**: Faster responses = preferred gateway
4. **Geographic Distribution**: Serve local clients

---

## Monitoring

### Gateway Dashboard

Access the built-in dashboard:

```bash
# Open in browser
http://localhost:3030/dashboard
```

### Metrics Endpoint

```bash
curl http://localhost:3030/gateway/metrics
```

**Response:**

```json
{
  "uptime_seconds": 86400,
  "total_requests": 1523456,
  "requests_per_second": 17.6,
  "total_revenue": 152345600,
  "connected_clients": 234,
  "cache_hit_rate": 0.85,
  "avg_latency_ms": 12,
  "peer_connections": 45,
  "storage_used_gb": 42.3
}
```

### Prometheus Integration

Enable Prometheus metrics:

```toml
[monitoring]
prometheus_enabled = true
prometheus_port = 9090
```

Scrape config:

```yaml
scrape_configs:
  - job_name: 'solanavault-gateway'
    static_configs:
      - targets: ['localhost:9090']
```

### Alerting

Set up alerts for critical metrics:

```toml
[alerting]
enabled = true
webhook_url = "https://your-alerting-service/webhook"

[[alerting.rules]]
metric = "error_rate"
threshold = 0.05
severity = "critical"

[[alerting.rules]]
metric = "latency_p99_ms"
threshold = 500
severity = "warning"
```

---

## High Availability

### Load Balancing

Run multiple gateway instances behind a load balancer:

```bash
# Instance 1
./target/release/vault-rpc-decentralized \
  --gateway-mode \
  --http-port 3031

# Instance 2
./target/release/vault-rpc-decentralized \
  --gateway-mode \
  --http-port 3032
```

Nginx configuration:

```nginx
upstream solanavault_gateway {
    least_conn;
    server localhost:3031;
    server localhost:3032;
}

server {
    listen 3030;
    location / {
        proxy_pass http://solanavault_gateway;
    }
}
```

### Automatic Failover

Configure automatic failover:

```toml
[high_availability]
enabled = true
health_check_interval_ms = 5000
failover_threshold = 3
```

---

## Security

### TLS Configuration

Enable TLS for client connections:

```toml
[tls]
enabled = true
cert_path = "/etc/solanavault/cert.pem"
key_path = "/etc/solanavault/key.pem"
```

### Rate Limiting

Protect against abuse:

```toml
[rate_limiting]
enabled = true
requests_per_second = 100
burst_size = 200
ban_duration_seconds = 300
```

### Firewall Rules

```bash
# Allow P2P traffic
ufw allow 4040/tcp

# Allow client traffic
ufw allow 5050/tcp

# Allow HTTP RPC
ufw allow 3030/tcp

# Restrict to specific IPs if needed
ufw allow from 10.0.0.0/8 to any port 3030
```

---

## Production Deployment

### Systemd Service

Create `/etc/systemd/system/solanavault-gateway.service`:

```ini
[Unit]
Description=SolanaVault Gateway
After=network.target

[Service]
Type=simple
User=solanavault
ExecStart=/opt/solanavault/vault-rpc-decentralized --gateway-mode --config /etc/solanavault/gateway.toml
Restart=always
RestartSec=10
LimitNOFILE=65535

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable solanavault-gateway
sudo systemctl start solanavault-gateway
```

### Docker Deployment

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin vault-rpc-decentralized

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/vault-rpc-decentralized /usr/local/bin/
EXPOSE 3030 4040 5050
CMD ["vault-rpc-decentralized", "--gateway-mode"]
```

```bash
docker build -t solanavault-gateway .
docker run -d -p 3030:3030 -p 4040:4040 -p 5050:5050 \
  -v /data/solanavault:/data \
  solanavault-gateway
```

---

## Troubleshooting

### No Client Connections

1. Verify public address is reachable
2. Check firewall rules
3. Ensure bootstrap nodes are accessible

### Low Revenue

1. Compare pricing with other gateways
2. Improve latency and availability
3. Increase cache size for better hit rates

### High Latency

1. Use NVMe storage for cache
2. Increase RAM for larger in-memory cache
3. Optimize network routing

---

## Next Steps

- [Full Node Setup](full-node.md) - Also participate in consensus
- [Configuration Reference](../reference/configuration.md) - All options
- [Economics](../architecture/economics.md) - Deep dive into the economic model
- [Network Architecture](../architecture/network.md) - How the network works
