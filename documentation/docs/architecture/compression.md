# Compression Pipeline

SolanaVault achieves 15-25:1 compression ratios through a three-stage pipeline optimized for Solana blockchain data patterns.

---

## Overview

The compression pipeline exploits blockchain-specific redundancy:

| Data Pattern | Compression Technique |
|--------------|----------------------|
| Repeated program IDs | Dictionary encoding |
| Sequential blockhashes | Delta encoding |
| Common instruction patterns | Template matching |
| Structural redundancy | Entropy coding |

---

## Stage 1: Structural Compression

Stage 1 applies fast, lossless compression to exploit structural patterns.

### Program Clustering

Transactions are grouped by program ID:

```
Before:
  Tx1: Program A, Program B
  Tx2: Program A, Program C
  Tx3: Program A, Program B

After:
  Cluster A: [Tx1, Tx2, Tx3] (reference: 1 byte each)
  Cluster B: [Tx1, Tx3]
  Cluster C: [Tx2]
```

**Compression gain:** 3-5x for program IDs

### Account Dictionary

Frequently used accounts are assigned short codes:

```
Dictionary:
  0x01 = 11111111111111111111111111111111 (System Program)
  0x02 = TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA (Token Program)
  0x03 = Vote111111111111111111111111111111111111111 (Vote Program)
  ...

Before: 32 bytes per account reference
After: 1-2 bytes per reference
```

**Compression gain:** 10-30x for common accounts

### Blockhash Delta Encoding

Sequential blockhashes are stored as deltas:

```
Block 245000000: 5KQCz...abc (full hash)
Block 245000001: +47 bytes difference
Block 245000002: +12 bytes difference
...
```

**Compression gain:** 5-10x for blockhash sequences

### Stage 1 Output

- Typical ratio: 3-5:1
- Speed: <1ms per block
- Reversible: Fully lossless

---

## Stage 2: Pattern Recognition

Stage 2 identifies and compresses transaction patterns.

### Transaction Templates

Common transaction types are stored as templates:

```rust
Template: SPL_TOKEN_TRANSFER {
    program: Token Program,
    accounts: [source, destination, owner],
    data: TransferInstruction { amount: u64 }
}

Actual transaction:
  template_id: 0x42
  source: account_ref
  destination: account_ref
  owner: account_ref
  amount: 1000000
```

**Known templates:**
- SPL Token transfers
- SOL transfers
- NFT mints
- Stake operations
- Vote transactions

### Instruction Deduplication

Identical instructions are stored once and referenced:

```
Block contains 500 identical vote instructions
Stored: 1 instruction + 500 references
```

### Signature Optimization

Transaction signatures are compressed using:

- **Aggregation**: When possible, combine signatures
- **Reference**: Link to known signer patterns
- **Delta**: Store differences from expected signatures

### Stage 2 Output

- Typical ratio: 2-3:1 (on Stage 1 output)
- Speed: 1-5ms per block
- Reversible: Fully lossless

---

## Stage 3: ML-Based Optimization

Stage 3 applies advanced compression techniques.

### Neural Predictors

Trained models predict likely data patterns:

```
Input: Transaction context
Prediction: Likely next bytes
Actual: Compressed as difference from prediction
```

### Entropy Coding

Adaptive entropy coding based on data statistics:

- **Arithmetic coding** for high-entropy data
- **Huffman coding** for structured data
- **Range coding** for mixed content

### Final Compression

Apply general-purpose compression:

```rust
// Algorithm selection based on data type
match data_type {
    BlockHeader => zstd_level_19,  // Maximum compression
    Transactions => lz4_hc,        // Balance speed/ratio
    Signatures => zstd_level_10,   // Good compression
}
```

### Stage 3 Output

- Typical ratio: 2-3:1 (on Stage 2 output)
- Speed: 5-50ms per block
- Reversible: Fully lossless

---

## Compression Levels

Users can select compression level:

| Level | Stages | Ratio | Speed | Use Case |
|-------|--------|-------|-------|----------|
| Low | 1 only | 3-5:1 | <1ms | Real-time |
| Medium | 1+2 | 8-12:1 | 1-5ms | Default |
| High | 1+2+3 | 15-25:1 | 10-50ms | Archival |
| Maximum | All + extra | 20-30:1 | 50-100ms | Cold storage |

---

## Decompression

Decompression reverses the pipeline:

```
Compressed Data
      │
      ▼
┌─────────────┐
│  Stage 3    │  Entropy decoding, zstd decompress
│  Decompress │  Speed: 10-30 μs
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Stage 2    │  Template expansion, reference resolution
│  Decompress │  Speed: 5-20 μs
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Stage 1    │  Dictionary lookup, delta reconstruction
│  Decompress │  Speed: 5-15 μs
└──────┬──────┘
       │
       ▼
Original Block
```

**Total decompression time:** 13-85 microseconds per block

---

## Compression API

### Rust API

```rust
use vault_core::compression::{CompressionStrategy, compress, decompress};

// Compress a block
let strategy = CompressionStrategy::High;
let compressed = compress(&block_data, strategy)?;

// Decompress
let original = decompress(&compressed)?;

// Verify integrity
assert_eq!(sha256(&original), sha256(&block_data));
```

### Configuration

```toml
[compression]
# Default compression level
default_level = "high"

# Stage-specific settings
[compression.stage1]
enable_program_clustering = true
dictionary_size = 65536
delta_window = 1000

[compression.stage2]
enable_templates = true
template_cache_size = 10000
dedup_enabled = true

[compression.stage3]
neural_predictor = true
entropy_coder = "arithmetic"
final_algorithm = "zstd"
zstd_level = 19
```

---

## Performance Benchmarks

### Compression Speed

| Block Type | Stage 1 | Stage 2 | Stage 3 | Total |
|------------|---------|---------|---------|-------|
| Standard | 0.5ms | 2ms | 15ms | 17.5ms |
| Vote-heavy | 0.3ms | 1ms | 8ms | 9.3ms |
| DEX trades | 0.8ms | 4ms | 25ms | 29.8ms |
| NFT mints | 0.6ms | 3ms | 20ms | 23.6ms |

### Decompression Speed

| Block Type | Time | Throughput |
|------------|------|------------|
| Standard | 45μs | 22,000 blocks/sec |
| Vote-heavy | 25μs | 40,000 blocks/sec |
| DEX trades | 85μs | 11,700 blocks/sec |
| NFT mints | 60μs | 16,600 blocks/sec |

### Compression Ratios by Data Type

| Data Type | Raw Size | Compressed | Ratio |
|-----------|----------|------------|-------|
| Block headers | 1 KB | 50 bytes | 20:1 |
| Vote txs | 500 KB | 20 KB | 25:1 |
| Token transfers | 200 KB | 12 KB | 17:1 |
| DEX trades | 1 MB | 60 KB | 17:1 |
| Mixed block | 2 MB | 100 KB | 20:1 |

---

## Integrity Verification

All compressed data includes integrity verification:

```
┌────────────────────────────────────────┐
│ Compressed Block                       │
├────────────────────────────────────────┤
│ Header (32 bytes)                      │
│  ├── Original size (8 bytes)           │
│  ├── Compressed size (8 bytes)         │
│  ├── Original SHA-256 (32 bytes... │
│  ├── Compression version (4 bytes)     │
│  └── Flags (4 bytes)                   │
├────────────────────────────────────────┤
│ Compressed payload                     │
├────────────────────────────────────────┤
│ Checksum (CRC32)                       │
└────────────────────────────────────────┘
```

### Verification Steps

1. **CRC32 check**: Verify data integrity
2. **Decompress**: Apply reverse pipeline
3. **SHA-256 compare**: Match original hash
4. **Size verify**: Match expected size

---

## Algorithm Selection

The pipeline automatically selects optimal algorithms:

```rust
fn select_algorithm(data: &[u8]) -> Algorithm {
    let entropy = calculate_entropy(data);
    let structure = detect_structure(data);

    match (entropy, structure) {
        (low, highly_structured) => Algorithm::Dictionary,
        (medium, some_structure) => Algorithm::LZ4,
        (high, no_structure) => Algorithm::Zstd,
        _ => Algorithm::Adaptive,
    }
}
```

---

## Next Steps

- [Network Protocol](network.md) - How compressed data is distributed
- [Architecture Overview](overview.md) - Full system architecture
- [API Reference](../reference/api.md) - Integration details
