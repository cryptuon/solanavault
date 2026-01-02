# Configuration Reference

Complete reference for all SolanaVault configuration options.

---

## Configuration Files

SolanaVault uses TOML configuration files:

| Component | Default Path |
|-----------|-------------|
| Light Client | `~/.solanavault/light-client.toml` |
| Gateway | `~/.solanavault/gateway.toml` |
| Full Node | `~/.solanavault/node.toml` |

Override with `--config <path>` flag.

---

## Light Client Configuration

```toml
#
# Light Client Configuration
#

[client]
# Initial token balance
initial_balance = 50000

# Local RPC port
local_port = 8899

# IPC socket path (optional)
ipc_path = "/tmp/solanavault.sock"

# Log level: trace, debug, info, warn, error
log_level = "info"


[network]
# Gateway endpoints (auto-discover if empty)
gateway_endpoints = [
    "tcp://gateway1.solanavault.com:5050",
    "tcp://gateway2.solanavault.com:5050"
]

# Auto-reconnect on disconnect
auto_reconnect = true

# Connection timeout (ms)
connection_timeout_ms = 5000

# Request timeout (ms)
request_timeout_ms = 30000

# Maximum retries per request
max_retries = 3


[cache]
# Enable response caching
enabled = true

# Maximum cache entries
max_entries = 100000

# Cache entry TTL (seconds)
ttl_seconds = 3600

# Enable persistent cache
persistent = true

# Persistent cache path
persistent_path = "~/.solanavault/cache"

# Cache size limit (MB)
max_size_mb = 512


[pricing]
# Maximum price per request (reject if higher)
max_price_per_request = 500

# Prefer volume discounts
prefer_volume_discounts = true

# Maximum daily spend (0 = unlimited)
max_daily_spend = 0

# Alert when balance falls below
low_balance_alert = 10000


[security]
# Verify gateway signatures
verify_gateway_signatures = true

# Trusted gateway list (empty = trust all)
trusted_gateways = []

# Enable TLS
tls_enabled = true
```

---

## Gateway Configuration

```toml
#
# Gateway Node Configuration
#

[gateway]
# Gateway display name
name = "my-gateway"

# Public address for clients
public_address = "tcp://your-ip:5050"

# HTTP RPC port
http_port = 3030

# Maximum concurrent connections
max_connections = 1000

# Connection timeout (ms)
connection_timeout_ms = 30000

# Enable gateway mode
enabled = true


[pricing]
# Pricing strategy: "fixed", "dynamic", "market"
strategy = "dynamic"

# Base fee per request (micro-tokens)
base_fee = 100

# Data fee per KB (micro-tokens)
data_fee_per_kb = 50

# Priority request multiplier
priority_multiplier = 1.5

# Enable volume discounts
volume_discount = true

# Discount tiers
[[pricing.discount_tiers]]
threshold = 10000
discount = 0.10

[[pricing.discount_tiers]]
threshold = 100000
discount = 0.20

[[pricing.discount_tiers]]
threshold = 1000000
discount = 0.25


[pricing.surge]
# Enable surge pricing
enabled = true

# Utilization threshold to trigger
threshold_utilization = 0.80

# Maximum surge multiplier
max_multiplier = 2.0


[network]
# P2P port
p2p_port = 4040

# Bootstrap nodes
bootstrap_nodes = [
    "tcp://bootstrap1.solanavault.com:4040",
    "tcp://bootstrap2.solanavault.com:4040"
]

# Maximum peer connections
max_peers = 50

# Discovery interval (seconds)
discovery_interval = 60

# Prefer geographically close peers
prefer_local_peers = true


[storage]
# Cache size (GB)
cache_size_gb = 50

# Data path
data_path = "/data/solanavault"

# Enable persistent cache
persistent_cache = true

# Cache eviction policy: "lru", "lfu", "adaptive"
eviction_policy = "adaptive"


[rate_limiting]
# Enable rate limiting
enabled = true

# Requests per second per client
requests_per_second = 100

# Burst allowance
burst_size = 200

# Ban duration for abusive clients (seconds)
ban_duration_seconds = 300


[tls]
# Enable TLS
enabled = true

# Certificate path
cert_path = "/etc/solanavault/cert.pem"

# Private key path
key_path = "/etc/solanavault/key.pem"


[monitoring]
# Enable Prometheus metrics
prometheus_enabled = true

# Prometheus port
prometheus_port = 9090

# Enable dashboard
dashboard_enabled = true


[alerting]
# Enable alerts
enabled = true

# Webhook URL for alerts
webhook_url = "https://your-alerting-service/webhook"

# Alert rules
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

## Full Node Configuration

```toml
#
# Full Node Configuration
#

[node]
# Node identifier
name = "my-fullnode"

# Data storage path
data_path = "/data/solanavault"

# Maximum storage capacity (GB)
storage_capacity_gb = 1000

# Enable block verification
verify_blocks = true

# Log level
log_level = "info"


[network]
# P2P port
p2p_port = 4040

# Consensus port
consensus_port = 4041

# Admin API port
admin_port = 4042

# Public address
public_address = "tcp://your-ip:4040"

# Bootstrap nodes
bootstrap_nodes = [
    "tcp://bootstrap1.solanavault.com:4040",
    "tcp://bootstrap2.solanavault.com:4040"
]

# Maximum peer connections
max_peers = 100

# Connection timeout (ms)
connection_timeout_ms = 10000


[consensus]
# Enable consensus participation
participation = true

# Stake amount
stake_amount = 10000

# Minimum stake required
min_stake = 1000

# Enable slashing protection
enable_slashing_protection = true

# Vote timeout (ms)
vote_timeout_ms = 5000

# Maximum rounds per consensus
max_rounds = 10


[storage]
# Compression level: "low", "medium", "high", "maximum"
compression_level = "high"

# Replication factor
replication_factor = 3

# Local cache size (MB)
cache_size_mb = 8192

# Verify data on read
verify_on_read = true

# Garbage collection interval (hours)
gc_interval_hours = 24


[sync]
# Sync mode: "full", "fast", "warp"
mode = "fast"

# Verify checkpoints
checkpoint_verification = true

# Parallel downloads
parallel_downloads = 8


[performance]
# Worker thread count
worker_threads = 16

# I/O buffer size (KB)
io_buffer_kb = 256

# Connection pool size
connection_pool = 100

# Enable memory mapping
mmap_enabled = true
```

---

## Compression Configuration

```toml
#
# Compression Settings (used by all components)
#

[compression]
# Default compression level: "low", "medium", "high", "maximum"
default_level = "high"


[compression.stage1]
# Enable program clustering
enable_program_clustering = true

# Dictionary size for account encoding
dictionary_size = 65536

# Delta encoding window
delta_window = 1000


[compression.stage2]
# Enable template matching
enable_templates = true

# Template cache size
template_cache_size = 10000

# Enable instruction deduplication
dedup_enabled = true


[compression.stage3]
# Enable neural predictors
neural_predictor = true

# Entropy coder: "arithmetic", "huffman", "range"
entropy_coder = "arithmetic"

# Final compression algorithm: "zstd", "lz4", "brotli"
final_algorithm = "zstd"

# Zstd compression level (1-22)
zstd_level = 19
```

---

## Network Common Settings

```toml
#
# Network Settings (common to all components)
#

[network.buffers]
# Send buffer size (bytes)
send_buffer_size = 16777216  # 16 MB

# Receive buffer size (bytes)
recv_buffer_size = 16777216  # 16 MB

# Message queue size
message_queue_size = 10000


[network.timeouts]
# Connection timeout (ms)
connect_timeout_ms = 5000

# Request timeout (ms)
request_timeout_ms = 30000

# Keepalive interval (ms)
keepalive_interval_ms = 30000


[network.concurrency]
# I/O threads
io_threads = 4

# Worker threads
worker_threads = 16

# Maximum concurrent requests
max_concurrent_requests = 1000
```

---

## Environment Variables

All configuration options can be overridden via environment variables:

```bash
# Format: VAULT_SECTION_KEY
# Nested: VAULT_SECTION_SUBSECTION_KEY

# Examples:
export VAULT_CLIENT_LOCAL_PORT=9000
export VAULT_NETWORK_MAX_PEERS=200
export VAULT_COMPRESSION_DEFAULT_LEVEL=maximum
export VAULT_GATEWAY_PRICING_BASE_FEE=150

# Bootstrap nodes (comma-separated)
export VAULT_BOOTSTRAP_NODES="tcp://node1:4040,tcp://node2:4040"

# Logging
export RUST_LOG=info
export RUST_LOG=vault_core::network=debug
export RUST_BACKTRACE=1
```

---

## Command Line Flags

Common flags available for all binaries:

| Flag | Description |
|------|-------------|
| `--config <path>` | Configuration file path |
| `--log-level <level>` | Log level override |
| `--data-path <path>` | Data directory override |
| `-v, --verbose` | Increase verbosity |
| `-q, --quiet` | Decrease verbosity |
| `--version` | Show version |
| `--help` | Show help |

---

## Configuration Validation

Validate configuration before starting:

```bash
# Validate config file
vault-cli config validate ~/.solanavault/node.toml

# Show effective configuration
vault-cli config show --resolved
```

---

## Next Steps

- [CLI Commands](cli.md) - Command-line reference
- [API Reference](api.md) - API documentation
- [Light Client Guide](../guides/light-client.md) - Configure light client
- [Gateway Guide](../guides/gateway-operators.md) - Configure gateway
