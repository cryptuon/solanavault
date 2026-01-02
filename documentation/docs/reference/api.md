# API Reference

Complete API reference for SolanaVault, covering RPC endpoints, Rust library, and WebSocket interfaces.

---

## RPC API

SolanaVault provides a drop-in compatible Solana JSON-RPC API.

### Endpoint

```
http://localhost:8899   (Light Client)
http://localhost:3030   (Gateway/Proxy)
```

### Request Format

```bash
curl http://localhost:8899 \
  -X POST \
  -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "METHOD_NAME",
    "params": [...]
  }'
```

---

## Supported Methods

### Slot & Block Information

#### getSlot

Returns the current slot.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getSlot"}'
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "result": 245000000,
  "id": 1
}
```

#### getBlockHeight

Returns the current block height.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getBlockHeight"}'
```

#### getBlock

Returns block information for a specific slot.

**Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| slot | u64 | Slot number |
| config | object | Optional configuration |

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getBlock",
    "params": [
      245000000,
      {
        "encoding": "json",
        "transactionDetails": "full",
        "rewards": true
      }
    ]
  }'
```

#### getBlocks

Returns a list of confirmed blocks between two slots.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getBlocks",
    "params": [245000000, 245000100]
  }'
```

---

### Transaction Methods

#### getTransaction

Returns transaction details.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getTransaction",
    "params": [
      "TRANSACTION_SIGNATURE",
      {"encoding": "json"}
    ]
  }'
```

#### getSignaturesForAddress

Returns signatures for transactions involving an address.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getSignaturesForAddress",
    "params": [
      "ADDRESS",
      {"limit": 10}
    ]
  }'
```

---

### Account Methods

#### getAccountInfo

Returns account information.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getAccountInfo",
    "params": [
      "ACCOUNT_ADDRESS",
      {"encoding": "base64"}
    ]
  }'
```

#### getBalance

Returns the balance of an account.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getBalance",
    "params": ["ACCOUNT_ADDRESS"]
  }'
```

#### getMultipleAccounts

Returns information for multiple accounts.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{
    "jsonrpc": "2.0",
    "id": 1,
    "method": "getMultipleAccounts",
    "params": [
      ["ADDRESS_1", "ADDRESS_2", "ADDRESS_3"],
      {"encoding": "base64"}
    ]
  }'
```

---

### Epoch & Cluster Information

#### getEpochInfo

Returns epoch information.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getEpochInfo"}'
```

**Response:**

```json
{
  "jsonrpc": "2.0",
  "result": {
    "epoch": 550,
    "slotIndex": 123456,
    "slotsInEpoch": 432000,
    "absoluteSlot": 245000000
  },
  "id": 1
}
```

#### getClusterNodes

Returns information about cluster nodes.

```bash
curl http://localhost:8899 -X POST -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"getClusterNodes"}'
```

---

## SolanaVault Extensions

### /stats

Returns SolanaVault-specific statistics.

```bash
curl http://localhost:3030/stats
```

**Response:**

```json
{
  "compression_ratio": 18.5,
  "cache_hit_rate": 0.92,
  "requests_per_second": 156,
  "connected_peers": 45,
  "storage_used_gb": 234.5,
  "uptime_seconds": 86400
}
```

### /gateway/status

Returns gateway status (gateway nodes only).

```bash
curl http://localhost:3030/gateway/status
```

### /client/balance

Returns light client balance.

```bash
curl http://localhost:8899/client/balance
```

### /client/stats

Returns light client statistics.

```bash
curl http://localhost:8899/client/stats
```

---

## Rust Library API

### Compression

```rust
use vault_core::compression::{
    CompressionStrategy,
    compress,
    decompress,
    BlockchainCompressionAdapter,
};

// Compress data
let strategy = CompressionStrategy::High;
let compressed = compress(&data, strategy)?;

// Decompress data
let original = decompress(&compressed)?;

// Use adapter for blockchain-specific compression
let adapter = BlockchainCompressionAdapter::new();
let compressed = adapter.compress_block(&block)?;
let block = adapter.decompress_block(&compressed)?;
```

### CompressionStrategy

```rust
pub enum CompressionStrategy {
    /// Stage 1 only - fast, ~3-5:1 ratio
    Low,

    /// Stage 1+2 - balanced, ~8-12:1 ratio
    Medium,

    /// Stage 1+2+3 - slow, ~15-25:1 ratio
    High,

    /// Maximum compression - slowest, ~20-30:1 ratio
    Maximum,
}
```

### Storage

```rust
use vault_core::storage::StorageNode;

// Create storage node
let storage = StorageNode::new(config)?;

// Store data
let key = storage.store(&data).await?;

// Retrieve data
let data = storage.get(&key).await?;

// Delete data
storage.delete(&key).await?;
```

### Data Access

```rust
use vault_core::data::{SolanaBlockClient, BlockCache};

// Create block client
let client = SolanaBlockClient::new(rpc_url)?;

// Fetch block
let block = client.get_block(245000000).await?;

// Use cache
let cache = BlockCache::new(cache_config);
let cached_block = cache.get_or_fetch(245000000, &client).await?;
```

### Memory Management

```rust
use vault_core::memory::{
    VaultStorageEngine,
    VaultCacheManager,
    VaultMemoryPool,
};

// Storage engine
let storage = VaultStorageEngine::new(path)?;
storage.put(key, value)?;
let value = storage.get(key)?;

// Cache manager
let cache = VaultCacheManager::new(config)?;
cache.insert(key, value, ttl)?;
let value = cache.get(&key)?;

// Memory pool
let pool = VaultMemoryPool::new(size)?;
let buffer = pool.allocate(1024)?;
```

### Workflows

```rust
use vault_core::workflows::CompressionWorkflow;

// Create workflow
let workflow = CompressionWorkflow::new(config)?;

// Process blocks
let result = workflow.compress_range(start_slot, end_slot).await?;
println!("Compressed {} blocks", result.block_count);
println!("Compression ratio: {:.1}:1", result.ratio);
```

---

## WebSocket API

### Connection

```javascript
const ws = new WebSocket('ws://localhost:8899');
```

### Subscribe to Slot Updates

```javascript
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  id: 1,
  method: 'slotSubscribe'
}));
```

### Subscribe to Account Updates

```javascript
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  id: 1,
  method: 'accountSubscribe',
  params: [
    'ACCOUNT_ADDRESS',
    { encoding: 'base64' }
  ]
}));
```

### Unsubscribe

```javascript
ws.send(JSON.stringify({
  jsonrpc: '2.0',
  id: 1,
  method: 'slotUnsubscribe',
  params: [subscriptionId]
}));
```

---

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| -32600 | Invalid Request | Malformed JSON |
| -32601 | Method not found | Unknown method |
| -32602 | Invalid params | Bad parameters |
| -32603 | Internal error | Server error |
| -32001 | Block not found | Block doesn't exist |
| -32002 | Transaction not found | Tx doesn't exist |
| -32003 | Insufficient balance | Not enough tokens |
| -32004 | Rate limited | Too many requests |

---

## Rate Limits

| Tier | Requests/Second | Burst |
|------|-----------------|-------|
| Free | 10 | 20 |
| Standard | 100 | 200 |
| Premium | 1000 | 2000 |
| Enterprise | Unlimited | - |

---

## SDK Examples

### JavaScript/TypeScript

```typescript
import { Connection } from '@solana/web3.js';

// Connect to SolanaVault light client
const connection = new Connection('http://localhost:8899');

// Get slot
const slot = await connection.getSlot();

// Get block
const block = await connection.getBlock(slot);

// Get account info
const account = await connection.getAccountInfo(address);
```

### Python

```python
from solana.rpc.api import Client

# Connect to SolanaVault
client = Client("http://localhost:8899")

# Get slot
slot = client.get_slot()

# Get block
block = client.get_block(slot.value)
```

### Go

```go
import "github.com/gagliardetto/solana-go/rpc"

// Connect to SolanaVault
client := rpc.New("http://localhost:8899")

// Get slot
slot, err := client.GetSlot(context.Background(), rpc.CommitmentFinalized)

// Get block
block, err := client.GetBlock(context.Background(), slot)
```

---

## Next Steps

- [Configuration Reference](configuration.md) - All configuration options
- [CLI Commands](cli.md) - Command-line reference
- [Light Client Guide](../guides/light-client.md) - Client setup
- [Troubleshooting](../troubleshooting/common-issues.md) - Solve problems
