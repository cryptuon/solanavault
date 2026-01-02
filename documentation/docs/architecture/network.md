# Network Protocol

SolanaVault uses a custom P2P networking stack built on NNG (nanomsg-next-generation) with Kademlia DHT discovery and Byzantine Fault Tolerant consensus.

---

## Transport Layer

### NNG Overview

NNG provides high-performance messaging:

| Feature | Benefit |
|---------|---------|
| Zero-copy | Minimal memory overhead |
| Scalability protocols | Built-in patterns |
| Async I/O | Non-blocking operations |
| Reliable | Automatic reconnection |

### Supported Patterns

**Request/Reply (REQ/REP)**

```
Light Client ──────► Gateway
     │                  │
     │    Request       │
     │ ─────────────►   │
     │                  │
     │    Response      │
     │ ◄───────────── │
```

**Publish/Subscribe (PUB/SUB)**

```
Publisher ──────────────────────────►
     │
     ├────► Subscriber 1
     │
     ├────► Subscriber 2
     │
     └────► Subscriber N
```

**Pipeline (PUSH/PULL)**

```
Producer ──► Worker 1 ──►
         ──► Worker 2 ──► Collector
         ──► Worker 3 ──►
```

### Message Format

```
┌─────────────────────────────────────────────┐
│ Frame Header (8 bytes)                      │
│  ├── Version (1 byte): 0x01                 │
│  ├── Type (1 byte): message type            │
│  ├── Flags (2 bytes): compression, priority │
│  └── Length (4 bytes): payload length       │
├─────────────────────────────────────────────┤
│ Message Header (variable)                   │
│  ├── Message ID (16 bytes): UUID            │
│  ├── Sender ID (32 bytes): node pubkey      │
│  ├── Timestamp (8 bytes): unix ms           │
│  └── Signature (64 bytes): Ed25519          │
├─────────────────────────────────────────────┤
│ Payload (variable)                          │
│  └── Bincode-serialized message body        │
└─────────────────────────────────────────────┘
```

### Message Types

| Type | Code | Description |
|------|------|-------------|
| PING | 0x01 | Keepalive |
| PONG | 0x02 | Keepalive response |
| FIND_NODE | 0x10 | DHT node lookup |
| FIND_VALUE | 0x11 | DHT value lookup |
| STORE | 0x12 | DHT store request |
| DATA_REQUEST | 0x20 | Request block data |
| DATA_RESPONSE | 0x21 | Block data response |
| CONSENSUS_PROPOSE | 0x30 | BFT proposal |
| CONSENSUS_VOTE | 0x31 | BFT vote |
| CONSENSUS_COMMIT | 0x32 | BFT commit |

---

## Discovery Layer (Kademlia DHT)

### Node Identity

Each node has a unique 256-bit ID:

```
Node ID = SHA-256(Ed25519 Public Key)
```

### XOR Metric

Distance between nodes is calculated using XOR:

```
distance(A, B) = A XOR B
```

### Routing Table

Nodes maintain k-buckets for efficient routing:

```
┌─────────────────────────────────────────────┐
│ Routing Table                               │
├─────────────────────────────────────────────┤
│ Bucket 0 (distance 2^0 - 2^1):   [node...]  │
│ Bucket 1 (distance 2^1 - 2^2):   [node...]  │
│ Bucket 2 (distance 2^2 - 2^3):   [node...]  │
│ ...                                          │
│ Bucket 255 (distance 2^255):     [node...]  │
└─────────────────────────────────────────────┘
```

Parameters:
- **k = 20**: Bucket size
- **α = 3**: Parallel lookups
- **Refresh**: Every 1 hour

### Lookup Algorithm

```python
def find_node(target_id):
    closest = initial_k_closest_nodes(target_id)
    queried = set()

    while True:
        # Query α closest unqueried nodes
        to_query = closest[:α] - queried
        if not to_query:
            break

        responses = parallel_query(to_query, FIND_NODE, target_id)
        queried.update(to_query)

        # Update closest set
        for response in responses:
            closest = merge_k_closest(closest, response.nodes)

    return closest[:k]
```

### Content Routing

Data is stored at nodes closest to its hash:

```
Block 245000000:
  Content ID = SHA-256("block:245000000")
  Stored at: k closest nodes to Content ID
```

---

## Consensus Layer (BFT)

### Overview

SolanaVault uses a practical Byzantine Fault Tolerant protocol:

- **Safety**: If n ≥ 3f + 1, correct nodes agree
- **Liveness**: Progress guaranteed with honest majority
- **Finality**: Single-round consensus (no forks)

### Consensus Roles

| Role | Count | Responsibility |
|------|-------|----------------|
| Leader | 1 | Proposes values |
| Validators | n-1 | Vote on proposals |
| Observers | any | Watch consensus |

### Protocol Phases

```
┌──────────────────────────────────────────────────────────────────┐
│                      Consensus Round                              │
├──────────────────────────────────────────────────────────────────┤
│                                                                   │
│  Phase 1: PROPOSE                                                │
│  ┌────────┐                                                      │
│  │ Leader │ ─────────► Broadcast proposal to all validators      │
│  └────────┘                                                      │
│                                                                   │
│  Phase 2: PRE-VOTE                                               │
│  ┌────────────┐                                                  │
│  │ Validators │ ─────► Vote for/against proposal                 │
│  └────────────┘        (broadcast to all)                        │
│                                                                   │
│  Phase 3: PRE-COMMIT (if 2/3 pre-votes)                          │
│  ┌────────────┐                                                  │
│  │ Validators │ ─────► Pre-commit vote                           │
│  └────────────┘        (broadcast to all)                        │
│                                                                   │
│  Phase 4: COMMIT (if 2/3 pre-commits)                            │
│  ┌────────────┐                                                  │
│  │    All     │ ─────► Apply committed value                     │
│  └────────────┘                                                  │
│                                                                   │
└──────────────────────────────────────────────────────────────────┘
```

### Vote Messages

```rust
struct Vote {
    round: u64,
    block_hash: [u8; 32],
    voter: PublicKey,
    vote_type: VoteType,  // PreVote or PreCommit
    signature: Signature,
}
```

### Leader Selection

Leaders rotate based on stake weight:

```rust
fn select_leader(round: u64, validators: &[Validator]) -> &Validator {
    let total_stake: u64 = validators.iter().map(|v| v.stake).sum();
    let seed = sha256(&round.to_le_bytes());
    let selection = u64::from_le_bytes(seed[..8]) % total_stake;

    let mut cumulative = 0;
    for validator in validators {
        cumulative += validator.stake;
        if cumulative > selection {
            return validator;
        }
    }
    unreachable!()
}
```

### Slashing Conditions

| Violation | Evidence | Penalty |
|-----------|----------|---------|
| Double vote | Two votes for same round | 10% stake |
| Invalid proposal | Malformed data | 5% stake |
| Equivocation | Conflicting messages | 20% stake |

---

## Connection Management

### Peer Discovery

```
1. Node starts
     │
     ▼
2. Connect to bootstrap nodes
     │
     ▼
3. Perform FIND_NODE for own ID
     │
     ▼
4. Populate routing table
     │
     ▼
5. Periodic refresh (1 hour)
     │
     ▼
6. Maintain connections
```

### Connection Limits

```toml
[network.connections]
max_peers = 100
min_peers = 10
max_inbound = 80
max_outbound = 50
```

### Health Checks

```rust
// Ping every 30 seconds
loop {
    for peer in connected_peers {
        if !peer.ping().await {
            peer.disconnect();
            find_replacement();
        }
    }
    sleep(Duration::from_secs(30));
}
```

---

## Security

### Transport Security

All connections use TLS 1.3:

```toml
[network.tls]
enabled = true
cert_path = "/etc/solanavault/node.crt"
key_path = "/etc/solanavault/node.key"
ca_path = "/etc/solanavault/ca.crt"
```

### Message Authentication

Every message is signed:

```rust
fn send_message(msg: Message, key: &SigningKey) {
    let signature = key.sign(&msg.serialize());
    msg.signature = signature;
    transport.send(msg);
}

fn receive_message(msg: Message) -> Result<Message> {
    let pubkey = lookup_pubkey(&msg.sender_id)?;
    pubkey.verify(&msg.serialize(), &msg.signature)?;
    Ok(msg)
}
```

### Eclipse Attack Prevention

- Minimum peer diversity from different subnets
- Reputation-based peer selection
- Bootstrap node rotation
- Rate limiting on peer discovery

---

## Performance Tuning

### Buffer Sizes

```toml
[network.buffers]
send_buffer_size = 16777216    # 16 MB
recv_buffer_size = 16777216    # 16 MB
message_queue_size = 10000
```

### Timeouts

```toml
[network.timeouts]
connect_timeout_ms = 5000
request_timeout_ms = 30000
keepalive_interval_ms = 30000
```

### Concurrency

```toml
[network.concurrency]
io_threads = 4
worker_threads = 16
max_concurrent_requests = 1000
```

---

## Metrics

Available metrics for monitoring:

| Metric | Description |
|--------|-------------|
| `network_peers_connected` | Current peer count |
| `network_messages_sent` | Total messages sent |
| `network_messages_received` | Total messages received |
| `network_bytes_sent` | Total bytes sent |
| `network_bytes_received` | Total bytes received |
| `network_latency_ms` | Message latency histogram |
| `consensus_rounds` | Completed consensus rounds |
| `consensus_proposals` | Proposals made |
| `consensus_votes` | Votes cast |

---

## Next Steps

- [Economics](economics.md) - Incentive mechanisms
- [Architecture Overview](overview.md) - Full system architecture
- [Configuration Reference](../reference/configuration.md) - Network settings
