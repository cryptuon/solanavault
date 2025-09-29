//! # Storage Node
//!
//! Implementation of a storage node in the Vault network with persistent storage.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::workflows::{CompressionWorkflow};
use crate::workflows::compression_workflow::CompressedBlock;

/// Metadata for a stored block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    /// Block slot number
    pub slot: u64,
    /// File path where the block is stored
    pub file_path: PathBuf,
    /// Original size in bytes
    pub original_size: usize,
    /// Compressed size in bytes
    pub compressed_size: usize,
    /// Compression ratio achieved
    pub compression_ratio: f64,
    /// Block hash for integrity
    pub block_hash: String,
    /// Timestamp when stored
    pub stored_at: u64,
}

/// Represents a storage node in the Vault network
#[derive(Debug)]
pub struct StorageNode {
    /// Unique identifier for the node
    pub id: String,

    /// Node's network address
    pub address: String,

    /// Available storage capacity in bytes
    pub capacity: u64,

    /// Used storage in bytes
    pub used: u64,

    /// Node's reputation score
    pub reputation: f64,

    /// Data directory for storing blocks
    pub data_dir: PathBuf,

    /// Compression workflow for processing blocks
    pub compression_workflow: CompressionWorkflow,

    /// In-memory index of stored blocks for quick lookup
    pub block_index: HashMap<u64, BlockMetadata>,
}

impl StorageNode {
    /// Create a new storage node
    pub fn new(id: String, address: String, capacity: u64) -> Self {
        Self {
            id,
            address,
            capacity,
            used: 0,
            reputation: 1.0,
            data_dir: PathBuf::from("./vault-data"),
            compression_workflow: CompressionWorkflow::new(),
            block_index: HashMap::new(),
        }
    }

    /// Create a new storage node with custom data directory
    pub fn new_with_data_dir(id: String, address: String, capacity: u64, data_dir: PathBuf) -> Self {
        Self {
            id,
            address,
            capacity,
            used: 0,
            reputation: 1.0,
            data_dir,
            compression_workflow: CompressionWorkflow::new(),
            block_index: HashMap::new(),
        }
    }

    /// Initialize the storage node (create directories, load existing data)
    pub async fn initialize(&mut self) -> Result<(), StorageError> {
        println!("🔧 Initializing storage node {} at {}", self.id, self.data_dir.display());

        // Create directory structure
        let blocks_dir = self.data_dir.join("blocks");
        let metadata_dir = self.data_dir.join("metadata");
        let index_file = self.data_dir.join("index.json");

        fs::create_dir_all(&blocks_dir).await?;
        fs::create_dir_all(&metadata_dir).await?;

        // Load existing block index if it exists
        if index_file.exists() {
            match self.load_block_index().await {
                Ok(count) => println!("✅ Loaded {} existing blocks from index", count),
                Err(e) => println!("⚠️  Could not load existing index: {}", e),
            }
        }

        println!("✅ Storage node initialized with {:.2} GB capacity",
                 self.capacity as f64 / 1_000_000_000.0);

        Ok(())
    }
    
    /// Check if the node has enough space for data of given size
    pub fn has_capacity(&self, size: u64) -> bool {
        self.used + size <= self.capacity
    }
    
    /// Store a compressed block on this node
    pub async fn store_block(&mut self, slot: u64, block_data: &[u8]) -> Result<BlockMetadata, StorageError> {
        println!("📦 Storing block {} ({} bytes) on node {}", slot, block_data.len(), self.id);

        // Check if we already have this block
        if self.block_index.contains_key(&slot) {
            return Err(StorageError::BlockAlreadyExists(slot));
        }

        // Process block through compression workflow
        let compressed_block = self.compression_workflow
            .process_block(slot, block_data)
            .await
            .map_err(|e| StorageError::CompressionFailed(format!("Compression failed: {}", e)))?;

        // Check capacity
        if !self.has_capacity(compressed_block.compressed_size as u64) {
            return Err(StorageError::InsufficientCapacity);
        }

        // Generate file path
        let block_file = self.data_dir.join("blocks").join(format!("block_{}.vault", slot));

        // Serialize and store compressed block to disk
        let serialized_block = serde_json::to_vec(&compressed_block)
            .map_err(|e| StorageError::SerializationFailed(format!("Serialization failed: {}", e)))?;

        let mut file = fs::File::create(&block_file).await?;
        file.write_all(&serialized_block).await?;
        file.sync_all().await?;

        // Create metadata
        let metadata = BlockMetadata {
            slot,
            file_path: block_file,
            original_size: compressed_block.original_size,
            compressed_size: compressed_block.compressed_size,
            compression_ratio: compressed_block.compression_ratio,
            block_hash: compressed_block.block_hash,
            stored_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        // Update storage tracking
        self.used += compressed_block.compressed_size as u64;
        self.block_index.insert(slot, metadata.clone());

        // Persist index
        self.save_block_index().await?;

        println!("✅ Block {} stored with {:.2}:1 compression ratio", slot, compressed_block.compression_ratio);

        Ok(metadata)
    }

    /// Retrieve a block from storage
    pub async fn retrieve_block(&mut self, slot: u64) -> Result<Vec<u8>, StorageError> {
        println!("🔍 Retrieving block {} from node {}", slot, self.id);

        // Check if we have this block
        let metadata = self.block_index.get(&slot)
            .ok_or_else(|| StorageError::BlockNotFound(slot))?;

        // Try compression workflow cache first
        match self.compression_workflow.retrieve_block(slot).await {
            Ok(block_data) => {
                println!("📦 Retrieved block {} from compression workflow cache", slot);
                return Ok(block_data);
            }
            Err(_) => {
                // Fall back to disk storage
                println!("💾 Loading block {} from disk storage", slot);
            }
        }

        // Read compressed block from disk
        let mut file = fs::File::open(&metadata.file_path).await?;
        let mut serialized_data = Vec::new();
        file.read_to_end(&mut serialized_data).await?;

        // Deserialize compressed block
        let compressed_block: CompressedBlock = serde_json::from_slice(&serialized_data)
            .map_err(|e| StorageError::DeserializationFailed(format!("Deserialization failed: {}", e)))?;

        // Decompress block data using the CompressionStrategy trait
        let decompressed_data = {
            use crate::compression::CompressionStrategy;
            self.compression_workflow
                .get_compressor()
                .decompress(&compressed_block.compressed_data)
                .map_err(|e| StorageError::DecompressionFailed(format!("Decompression failed: {}", e)))?
        };

        println!("✅ Block {} retrieved and decompressed ({} bytes)", slot, decompressed_data.len());

        Ok(decompressed_data)
    }

    /// Store raw data on this node (legacy method for compatibility)
    pub fn store_data(&mut self, size: u64) -> Result<(), StorageError> {
        if !self.has_capacity(size) {
            return Err(StorageError::InsufficientCapacity);
        }

        self.used += size;
        Ok(())
    }
    
    /// Get available capacity
    pub fn available_capacity(&self) -> u64 {
        self.capacity - self.used
    }

    /// List all stored blocks
    pub fn list_blocks(&self) -> Vec<&BlockMetadata> {
        self.block_index.values().collect()
    }

    /// Get storage statistics
    pub fn get_storage_stats(&self) -> StorageStats {
        let compression_metrics = self.compression_workflow.get_metrics();

        StorageStats {
            total_capacity: self.capacity,
            used_capacity: self.used,
            available_capacity: self.available_capacity(),
            blocks_stored: self.block_index.len() as u64,
            total_compression_ratio: compression_metrics.average_compression_ratio,
            total_original_bytes: compression_metrics.total_original_bytes,
            total_compressed_bytes: compression_metrics.total_compressed_bytes,
        }
    }

    /// Load block index from disk
    async fn load_block_index(&mut self) -> Result<usize, StorageError> {
        let index_file = self.data_dir.join("index.json");

        if !index_file.exists() {
            return Ok(0);
        }

        let mut file = fs::File::open(&index_file).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let loaded_index: HashMap<u64, BlockMetadata> = serde_json::from_str(&contents)
            .map_err(|e| StorageError::DeserializationFailed(format!("Failed to load index: {}", e)))?;

        // Update used space calculation
        let mut total_used = 0u64;
        for metadata in loaded_index.values() {
            total_used += metadata.compressed_size as u64;
        }

        self.used = total_used;
        let count = loaded_index.len();
        self.block_index = loaded_index;

        Ok(count)
    }

    /// Save block index to disk
    async fn save_block_index(&self) -> Result<(), StorageError> {
        let index_file = self.data_dir.join("index.json");

        let serialized = serde_json::to_string_pretty(&self.block_index)
            .map_err(|e| StorageError::SerializationFailed(format!("Failed to serialize index: {}", e)))?;

        let mut file = fs::File::create(&index_file).await?;
        file.write_all(serialized.as_bytes()).await?;
        file.sync_all().await?;

        Ok(())
    }

    /// Demonstrate storage functionality
    pub async fn demonstrate_storage(&mut self) -> Result<(), StorageError> {
        println!("🎬 SolanaVault Storage Node Demo");
        println!("================================");

        // Create test blocks
        let test_blocks = vec![
            (244000000, self.create_test_block_data(244000000, 50)),
            (244000001, self.create_test_block_data(244000001, 75)),
            (244000002, self.create_test_block_data(244000002, 100)),
        ];

        println!("📝 Created {} test blocks", test_blocks.len());

        // Store all blocks
        for (slot, block_data) in &test_blocks {
            let metadata = self.store_block(*slot, block_data).await?;
            println!("   Stored block {}: {:.2}:1 compression", slot, metadata.compression_ratio);
        }

        // Retrieve and verify all blocks
        println!("\n🔍 Verification Phase:");
        for (slot, original_data) in &test_blocks {
            let retrieved_data = self.retrieve_block(*slot).await?;

            if retrieved_data != *original_data {
                return Err(StorageError::IntegrityCheckFailed(*slot));
            }

            println!("✅ Block {} verified: perfect data integrity", slot);
        }

        // Show storage statistics
        println!("\n📊 Storage Statistics:");
        let stats = self.get_storage_stats();
        println!("   Blocks stored: {}", stats.blocks_stored);
        println!("   Storage used: {:.2} MB ({:.1}%)",
                 stats.used_capacity as f64 / 1_000_000.0,
                 (stats.used_capacity as f64 / stats.total_capacity as f64) * 100.0);
        println!("   Average compression: {:.2}:1", stats.total_compression_ratio);
        println!("   Space saved: {:.2} MB",
                 (stats.total_original_bytes - stats.total_compressed_bytes) as f64 / 1_000_000.0);

        println!("\n🎉 Storage demonstration completed successfully!");

        Ok(())
    }

    fn create_test_block_data(&self, slot: u64, tx_count: usize) -> Vec<u8> {
        let mut block_data = Vec::new();

        // Block header
        block_data.extend_from_slice(b"SOLANA_BLOCK_V1");
        block_data.extend_from_slice(&slot.to_le_bytes());
        block_data.extend_from_slice(&[0; 32]); // Previous blockhash
        block_data.extend_from_slice(&1234567890u64.to_le_bytes()); // Timestamp

        // Transactions with realistic Solana patterns
        for i in 0..tx_count {
            // Transaction signature
            block_data.extend_from_slice(&[0x01; 64]);

            // Common Solana program IDs
            match i % 4 {
                0 => block_data.extend_from_slice("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".as_bytes()),
                1 => block_data.extend_from_slice("11111111111111111111111111111112".as_bytes()),
                2 => block_data.extend_from_slice("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".as_bytes()),
                _ => block_data.extend_from_slice("9WzDXwBbmkg8ZTbNMqUxvQRAyrZzDsGYdLVL9zYtAWWM".as_bytes()),
            }

            // Common transaction amounts
            match i % 3 {
                0 => block_data.extend_from_slice(&1_000_000_000u64.to_le_bytes()), // 1 SOL
                1 => block_data.extend_from_slice(&100_000_000u64.to_le_bytes()),   // 0.1 SOL
                _ => block_data.extend_from_slice(&10_000_000u64.to_le_bytes()),    // 0.01 SOL
            }

            // Instruction data
            block_data.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
            block_data.extend_from_slice(&(i as u32).to_le_bytes());
        }

        block_data
    }
}

/// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    /// Total storage capacity
    pub total_capacity: u64,
    /// Used storage capacity
    pub used_capacity: u64,
    /// Available storage capacity
    pub available_capacity: u64,
    /// Number of blocks stored
    pub blocks_stored: u64,
    /// Total compression ratio achieved
    pub total_compression_ratio: f64,
    /// Total original bytes stored
    pub total_original_bytes: u64,
    /// Total compressed bytes stored
    pub total_compressed_bytes: u64,
}

/// Error types for storage operations
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Insufficient capacity on storage node")]
    InsufficientCapacity,

    #[error("Node not found: {0}")]
    NodeNotFound(String),

    #[error("Block {0} not found")]
    BlockNotFound(u64),

    #[error("Block {0} already exists")]
    BlockAlreadyExists(u64),

    #[error("Compression failed: {0}")]
    CompressionFailed(String),

    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),

    #[error("Serialization failed: {0}")]
    SerializationFailed(String),

    #[error("Deserialization failed: {0}")]
    DeserializationFailed(String),

    #[error("Integrity check failed for block {0}")]
    IntegrityCheckFailed(u64),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}