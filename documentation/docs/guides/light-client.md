# Light Client Guide

The light client is the recommended way for applications to interact with SolanaVault. It provides pay-per-use access to compressed blockchain data without running a full node.

---

## Overview

The light client:

- **Connects to gateway nodes** in the decentralized network
- **Pays micro-tokens** for data requests
- **Caches responses** to minimize costs
- **Provides standard Solana RPC** interface

---

## Quick Start

```bash
# Start light client with initial balance
./target/release/vault-light-client start --balance 50000

# The client now provides a local RPC endpoint
# Default: http://localhost:8899
```

---

## Configuration

### Command Line Options

```bash
vault-light-client start [OPTIONS]

Options:
  --balance <TOKENS>      Initial token balance (default: 10000)
  --gateway <URL>         Gateway endpoint (default: auto-discover)
  --port <PORT>           Local RPC port (default: 8899)
  --cache-size <MB>       Cache size in MB (default: 512)
  --ipc-path <PATH>       IPC socket path for local apps
  --config <FILE>         Configuration file path
```

### Configuration File

Create `~/.solanavault/light-client.toml`:

```toml
[client]
initial_balance = 50000
local_port = 8899
cache_size_mb = 512

[network]
# Auto-discover or specify gateways
gateway_endpoints = [
    "tcp://gateway1.solanavault.com:5050",
    "tcp://gateway2.solanavault.com:5050"
]
auto_reconnect = true
connection_timeout_ms = 5000

[cache]
enabled = true
max_entries = 100000
ttl_seconds = 3600
persistent = true
persistent_path = "~/.solanavault/cache"

[pricing]
max_price_per_request = 500  # Reject if price exceeds this
prefer_volume_discounts = true
```

---

## Pricing Model

The light client pays micro-tokens for each request:

### Base Fees

| Request Type | Base Fee | Data Fee |
|-------------|----------|----------|
| getSlot | 100μ | - |
| getBlock | 100μ | 50μ/KB |
| getTransaction | 100μ | 50μ/KB |
| getAccountInfo | 100μ | 50μ/KB |

### Modifiers

| Modifier | Multiplier |
|----------|------------|
| Priority request | 1.5x |
| Real-time data | 2.0x |
| Historical data (>1 day) | 0.8x |

### Volume Discounts

| Monthly Volume | Discount |
|---------------|----------|
| 0 - 10,000 requests | 0% |
| 10,001 - 100,000 | 10% |
| 100,001 - 1,000,000 | 20% |
| 1,000,000+ | 25% |

---

## Usage Examples

### Basic RPC Calls

```bash
# Start the light client
./target/release/vault-light-client start --balance 50000

# In another terminal, use standard Solana RPC
curl http://localhost:8899 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}'

# Get block
curl http://localhost:8899 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getBlock","params":[245000000]}'
```

### Using Solana CLI

```bash
# Configure Solana CLI to use light client
solana config set --url http://localhost:8899

# Now all commands use SolanaVault
solana balance
solana block 245000000
solana slot
```

### Programmatic Access (Rust)

```rust
use solana_client::rpc_client::RpcClient;

fn main() {
    // Connect to local light client
    let client = RpcClient::new("http://localhost:8899");

    // Use standard Solana SDK methods
    let slot = client.get_slot().unwrap();
    println!("Current slot: {}", slot);

    let block = client.get_block(245000000).unwrap();
    println!("Block has {} transactions", block.transactions.len());
}
```

### Programmatic Access (JavaScript)

```javascript
import { Connection } from '@solana/web3.js';

// Connect to local light client
const connection = new Connection('http://localhost:8899');

// Use standard @solana/web3.js methods
const slot = await connection.getSlot();
console.log('Current slot:', slot);

const block = await connection.getBlock(245000000);
console.log('Block transactions:', block.transactions.length);
```

---

## Monitoring

### Check Balance

```bash
# Via CLI
vault-light-client balance

# Via RPC
curl http://localhost:8899/client/balance
```

### View Statistics

```bash
# Request statistics
curl http://localhost:8899/client/stats
```

**Response:**

```json
{
  "total_requests": 1523,
  "cache_hits": 1201,
  "cache_hit_rate": 0.789,
  "total_spent": 45690,
  "average_cost_per_request": 30,
  "connected_gateways": 3,
  "uptime_seconds": 3600
}
```

### View Cache Status

```bash
curl http://localhost:8899/client/cache
```

---

## Cost Optimization

### Enable Caching

Caching dramatically reduces costs for repeated requests:

```toml
[cache]
enabled = true
max_entries = 100000
ttl_seconds = 3600
```

### Batch Requests

Group multiple requests when possible:

```bash
# Instead of multiple calls, batch them
curl http://localhost:8899 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '[
    {"jsonrpc":"2.0","id":1,"method":"getSlot"},
    {"jsonrpc":"2.0","id":2,"method":"getBlockHeight"},
    {"jsonrpc":"2.0","id":3,"method":"getEpochInfo"}
  ]'
```

### Use Historical Data

Historical data costs less than real-time:

```toml
[pricing]
prefer_historical = true  # When possible, use cached historical data
```

### Monitor Spending

Set spending limits:

```toml
[pricing]
max_daily_spend = 100000  # Stop requests if limit exceeded
alert_threshold = 80000   # Alert at 80% of limit
```

---

## Troubleshooting

### Connection Failed

```
Error: Failed to connect to gateway
```

**Solutions:**

1. Check network connectivity
2. Verify gateway endpoints are correct
3. Try alternative gateways

```bash
# List available gateways
vault-light-client gateways list

# Test specific gateway
vault-light-client gateways test tcp://gateway1.solanavault.com:5050
```

### Insufficient Balance

```
Error: Insufficient balance for request
```

**Solutions:**

1. Top up your balance
2. Enable caching to reduce costs
3. Use historical data endpoints

```bash
# Add more tokens
vault-light-client balance add 50000
```

### High Latency

**Solutions:**

1. Connect to geographically closer gateways
2. Increase cache size
3. Use persistent cache

```toml
[network]
prefer_low_latency = true
max_latency_ms = 100
```

---

## Security Considerations

### Token Security

- Tokens are stored locally in `~/.solanavault/wallet`
- Use file permissions to protect the wallet file
- Consider using hardware security modules for large balances

### Network Security

- All communications are encrypted
- Gateway authenticity is verified via signatures
- Use trusted gateway lists in production

```toml
[security]
verify_gateway_signatures = true
trusted_gateways = [
    "gateway1.solanavault.com",
    "gateway2.solanavault.com"
]
```

---

## Next Steps

- [Configuration Reference](../reference/configuration.md) - All configuration options
- [API Reference](../reference/api.md) - Complete API documentation
- [Gateway Operators Guide](gateway-operators.md) - Run your own gateway
- [Troubleshooting](../troubleshooting/common-issues.md) - Solve common problems
