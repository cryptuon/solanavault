# API Reference

This document describes the APIs provided by SolanaVault.

## RPC Proxy API

The RPC proxy provides a drop-in replacement for the standard Solana RPC API.

### Methods

#### getConfirmedBlock

Retrieves a confirmed block, automatically routing to the appropriate data source.

```typescript
async getConfirmedBlock(slot: number): Promise<ConfirmedBlock>
```

Parameters:
- `slot`: The slot number of the block to retrieve

Returns:
- `ConfirmedBlock`: The requested block data

#### getBlocksWithLimit

Retrieves a list of confirmed blocks within a range.

```typescript
async getBlocksWithLimit(startSlot: number, limit: number): Promise<number[]>
```

Parameters:
- `startSlot`: The starting slot number
- `limit`: The maximum number of blocks to return

Returns:
- `number[]`: Array of slot numbers

## Node API

The node API allows interaction with storage nodes directly.

### storeBlock

Stores a compressed block on the network.

```rust
fn store_block(&mut self, block: CompressedBlock) -> Result<StorageReceipt, StorageError>
```

### retrieveBlock

Retrieves a block from local storage or the network.

```rust
fn retrieve_block(&self, slot: u64) -> Result<ConfirmedBlock, RetrievalError>
```

## Economic API

API for interacting with the staking and reward system.

### stakeTokens

Stakes tokens to participate in the storage network.

```rust
fn stake_tokens(&mut self, amount: u64) -> Result<(), StakingError>
```

### withdrawRewards

Withdraws accumulated rewards.

```rust
fn withdraw_rewards(&mut self) -> Result<u64, WithdrawalError>
```