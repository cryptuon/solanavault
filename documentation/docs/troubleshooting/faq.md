# Frequently Asked Questions

Common questions about SolanaVault.

---

## General

### What is SolanaVault?

SolanaVault is a decentralized blockchain compression and storage network for Solana. It achieves 15-25:1 compression ratios on blockchain data while maintaining full compatibility with standard Solana RPC APIs.

### How does SolanaVault differ from running my own Solana node?

| Aspect | Solana Node | SolanaVault Light Client |
|--------|-------------|--------------------------|
| Storage | ~2TB+ | <1GB (cached) |
| RAM | 128GB+ | 4GB |
| Cost | High (hardware) | Pay-per-use |
| Setup | Complex | Simple |
| Maintenance | Required | None |

### Is SolanaVault open source?

Yes, SolanaVault is open source under the [MIT License](https://github.com/solanavault/solanavault/blob/main/LICENSE).

### What compression ratios can I expect?

| Data Type | Typical Ratio |
|-----------|---------------|
| Vote transactions | 20-25:1 |
| Token transfers | 15-20:1 |
| DEX trades | 15-18:1 |
| NFT mints | 18-22:1 |
| Mixed blocks | 15-20:1 |

---

## Light Client

### How much does it cost to use the light client?

Pricing is based on usage:

| Operation | Approximate Cost |
|-----------|------------------|
| getSlot | 100 micro-tokens |
| getBlock (avg) | 5,000 micro-tokens |
| getTransaction | 500 micro-tokens |

Volume discounts of 10-25% are available.

### Can I use existing Solana tools with SolanaVault?

Yes! SolanaVault provides a drop-in compatible RPC endpoint. All existing tools work:

- Solana CLI
- @solana/web3.js
- solana-py
- Any Solana RPC client

### How do I reduce my costs?

1. **Enable caching** - Repeated requests cost 50% less
2. **Use historical data** - 20% cheaper than real-time
3. **Batch requests** - Reduces per-request overhead
4. **Build volume** - Discounts start at 10,000 requests/month

### Is my data secure?

Yes:
- All communications are encrypted (TLS 1.3)
- No private keys or sensitive data are transmitted
- Light client only reads public blockchain data

---

## Gateway Operators

### How much can I earn as a gateway operator?

Revenue depends on:
- Request volume
- Average request size
- Competition from other gateways

Example: 10M requests/month at 500μ average = ~4.75M tokens/month (after 5% network fee).

### What are the hardware requirements?

| Resource | Minimum | Recommended |
|----------|---------|-------------|
| CPU | 4 cores | 8+ cores |
| RAM | 8GB | 32GB |
| Storage | 100GB SSD | 500GB NVMe |
| Network | 100 Mbps | 1 Gbps |

### How do I attract more clients?

1. **Competitive pricing** - Lower fees attract volume
2. **High availability** - 99.9%+ uptime
3. **Low latency** - Fast responses
4. **Geographic presence** - Serve local clients

### Can I set my own prices?

Yes, gateways control their pricing within market parameters. You can adjust:
- Base fees
- Data fees
- Priority multipliers
- Volume discounts

---

## Full Nodes

### What's the difference between a gateway and a full node?

| Aspect | Gateway | Full Node |
|--------|---------|-----------|
| Primary role | Serve client requests | Store data, consensus |
| Revenue | Request fees (95%) | Consensus rewards |
| Storage | Cache only | Full compressed chain |
| Consensus | No | Yes |

### How much stake do I need?

| Tier | Stake | Benefits |
|------|-------|----------|
| Minimum | 1,000 tokens | Basic participation |
| Silver | 10,000 tokens | 1.2x rewards |
| Gold | 100,000 tokens | 1.5x rewards |

### What happens if my node goes offline?

- **Short outage (<1 hour)**: No penalty
- **Extended outage (1-24 hours)**: Reduced reputation
- **Long outage (>24 hours)**: 1%/day stake slashing

### Can I run both a gateway and a full node?

Yes! Many operators run both to maximize revenue:
- Gateway earns request fees
- Full node earns consensus rewards

---

## Compression

### Is compression lossless?

Yes, all compression is completely lossless. Decompressed data is identical to the original, verified by cryptographic hash comparison.

### How fast is decompression?

| Block Type | Decompression Time |
|------------|-------------------|
| Typical block | 30-50 microseconds |
| Vote-heavy | 20-30 microseconds |
| Complex DEX | 60-85 microseconds |

### Can I compress my own data?

Yes, use the CLI or Rust library:

```bash
vault-cli compress-demo --blocks 245000000:245001000
```

```rust
use vault_core::compression::{compress, CompressionStrategy};
let compressed = compress(&data, CompressionStrategy::High)?;
```

---

## Security

### How does consensus work?

SolanaVault uses Byzantine Fault Tolerant (BFT) consensus:
- 2/3 majority required for agreement
- Tolerates up to 1/3 malicious nodes
- Single-round finality (no forks)

### What prevents data corruption?

Multiple layers of protection:
1. **Cryptographic hashes** - Verify data integrity
2. **BFT consensus** - 2/3 nodes must agree
3. **Replication** - Data stored on multiple nodes
4. **Slashing** - Economic penalties for corruption

### Is my application data private?

SolanaVault only stores public blockchain data. It does not:
- Store private keys
- Have access to your wallet
- Track application-level data

---

## Economics

### Where does the 5% network fee go?

| Allocation | Percentage |
|------------|------------|
| Protocol development | 40% |
| Bug bounties | 20% |
| Ecosystem grants | 30% |
| Emergency reserve | 10% |

### How do staking rewards work?

```
Daily Reward = Base Reward × Stake Weight × Uptime × Reputation
```

Rewards are distributed daily and can be claimed anytime after a 24-hour cooldown.

### What are the slashing conditions?

| Violation | Penalty |
|-----------|---------|
| Double voting | 10% of stake |
| Data corruption | 20% of stake |
| Extended downtime | 1%/day |

---

## Technical

### What protocols does SolanaVault use?

| Layer | Protocol |
|-------|----------|
| Transport | NNG (nanomsg-next-generation) |
| Discovery | Kademlia DHT |
| Consensus | BFT (Byzantine Fault Tolerant) |
| Serialization | Bincode |
| Encryption | TLS 1.3 |

### What are the network requirements?

| Component | Ports | Protocol |
|-----------|-------|----------|
| P2P | 4040 | TCP |
| Client | 5050 | TCP |
| HTTP RPC | 3030 | TCP |
| Consensus | 4041 | TCP |

### Can I run SolanaVault in Docker?

Yes:

```bash
docker run -d \
  -p 3030:3030 \
  -p 4040:4040 \
  -v /data/vault:/data \
  solanavault/gateway
```

### Is there a testnet?

Yes, connect to the testnet:

```toml
[network]
bootstrap_nodes = [
    "tcp://testnet-bootstrap1.solanavault.com:4040"
]
```

---

## Troubleshooting

### Where can I find logs?

```bash
# Systemd
journalctl -u solanavault-node -f

# Debug logging
RUST_LOG=debug vault-node ...
```

### How do I report a bug?

1. Search existing issues: [GitHub Issues](https://github.com/solanavault/solanavault/issues)
2. If new, open an issue with:
   - Version (`vault-cli --version`)
   - OS and hardware
   - Steps to reproduce
   - Logs and error messages

### Where can I get help?

- **Documentation**: You're here!
- **GitHub Issues**: Technical problems
- **Discord**: Real-time community help
- **Twitter**: Announcements and updates

---

## Future Plans

### What features are planned?

- Cross-chain support (Ethereum, etc.)
- Mobile light client SDKs
- Advanced compression algorithms
- Governance system
- Layer 2 integration

### How can I contribute?

1. **Code**: Submit PRs on GitHub
2. **Documentation**: Improve docs
3. **Testing**: Report bugs
4. **Community**: Help others

See [Contributing Guide](https://github.com/solanavault/solanavault/blob/main/CONTRIBUTING.md).

---

## Still have questions?

- Check [Common Issues](common-issues.md) for troubleshooting
- Browse the [full documentation](../index.md)
- Ask on GitHub or Discord
