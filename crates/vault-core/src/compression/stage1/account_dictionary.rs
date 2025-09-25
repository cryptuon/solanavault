//! # Account Dictionary Compression
//!
//! Compresses 32-byte Solana account addresses to 2-byte dictionary indices.
//! This is one of the most effective compression techniques for Solana data.

use super::super::traits::CompressionError;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

/// Account dictionary for compressing addresses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDictionary {
    /// Map from pubkey to dictionary index
    address_to_index: HashMap<Pubkey, u16>,
    /// Map from index back to pubkey
    index_to_address: HashMap<u16, Pubkey>,
    /// Next available index
    next_index: u16,
    /// Common Solana system addresses pre-populated
    system_addresses: HashMap<Pubkey, u16>,
}

impl AccountDictionary {
    /// Create a new account dictionary with common system addresses
    pub fn new() -> Self {
        let mut dict = Self {
            address_to_index: HashMap::new(),
            index_to_address: HashMap::new(),
            next_index: 100, // Reserve 0-99 for system addresses
            system_addresses: HashMap::new(),
        };

        // Pre-populate with common Solana system addresses
        dict.add_system_addresses();
        dict
    }

    /// Add common Solana system addresses that appear in most blocks
    fn add_system_addresses(&mut self) {
        let common_addresses = [
            // System Program
            ("11111111111111111111111111111111", 0),
            // Token Program
            ("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", 1),
            // Associated Token Program
            ("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", 2),
            // Memo Program
            ("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr", 3),
            // Compute Budget Program
            ("ComputeBudget111111111111111111111111111111", 4),
            // Address Lookup Table Program
            ("AddressLookupTab1e1111111111111111111111111", 5),
            // BPF Loader
            ("BPFLoaderUpgradeab1e11111111111111111111111", 6),
            // Serum DEX
            ("9xQeWvG816bUx9EPjHmaT23yvVM2ZWbrrpZb9PusVFin", 7),
            // Raydium AMM
            ("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", 8),
            // SPL Token-2022
            ("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb", 9),
        ];

        for (address_str, index) in common_addresses {
            if let Ok(pubkey) = address_str.parse::<Pubkey>() {
                self.system_addresses.insert(pubkey, index);
                self.address_to_index.insert(pubkey, index);
                self.index_to_address.insert(index, pubkey);
            }
        }
    }

    /// Add an address to the dictionary and return its index
    pub fn add_address(&mut self, address: Pubkey) -> u16 {
        if let Some(&existing_index) = self.address_to_index.get(&address) {
            return existing_index;
        }

        let index = self.next_index;
        self.address_to_index.insert(address, index);
        self.index_to_address.insert(index, address);
        self.next_index += 1;

        index
    }

    /// Get the index for an address, adding it if not present
    pub fn get_or_add_index(&mut self, address: Pubkey) -> u16 {
        self.add_address(address)
    }

    /// Get address from index
    pub fn get_address(&self, index: u16) -> Option<Pubkey> {
        self.index_to_address.get(&index).copied()
    }

    /// Get index from address (without adding)
    pub fn get_index(&self, address: &Pubkey) -> Option<u16> {
        self.address_to_index.get(address).copied()
    }

    /// Compress raw data by finding and replacing Pubkey patterns
    pub fn compress_data(&mut self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        if data.len() < 32 {
            return Ok(data.to_vec());
        }

        let mut compressed = Vec::new();
        let mut i = 0;

        while i < data.len() {
            // Look for 32-byte sequences that could be pubkeys
            if i + 32 <= data.len() {
                // Try to parse as pubkey
                if let Ok(bytes) = <[u8; 32]>::try_from(&data[i..i + 32]) {
                    let pubkey = Pubkey::from(bytes);

                    // Check if this looks like a valid pubkey (basic validation)
                    if self.is_likely_pubkey(&pubkey) {
                        let index = self.get_or_add_index(pubkey);

                        // Write compressed marker (0xFF) followed by 2-byte index
                        compressed.push(0xFF);
                        compressed.extend_from_slice(&index.to_le_bytes());
                        i += 32;
                        continue;
                    }
                }
            }

            // If not a pubkey, copy the byte as-is
            compressed.push(data[i]);
            i += 1;
        }

        Ok(compressed)
    }

    /// Decompress data by replacing compressed indices with full addresses
    pub fn decompress_data(&self, compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        let mut decompressed = Vec::new();
        let mut i = 0;

        while i < compressed_data.len() {
            if compressed_data[i] == 0xFF && i + 2 < compressed_data.len() {
                // Read 2-byte index
                let index_bytes = [compressed_data[i + 1], compressed_data[i + 2]];
                let index = u16::from_le_bytes(index_bytes);

                // Look up address
                if let Some(address) = self.get_address(index) {
                    decompressed.extend_from_slice(address.as_ref());
                    i += 3;
                } else {
                    return Err(CompressionError::InvalidFormat);
                }
            } else {
                decompressed.push(compressed_data[i]);
                i += 1;
            }
        }

        Ok(decompressed)
    }

    /// Simple heuristic to check if bytes look like a valid pubkey
    fn is_likely_pubkey(&self, pubkey: &Pubkey) -> bool {
        let bytes = pubkey.as_ref();

        // Check if it's a known system address
        if self.system_addresses.contains_key(pubkey) {
            return true;
        }

        // Basic heuristics for valid pubkeys:
        // 1. Not all zeros
        if bytes.iter().all(|&b| b == 0) {
            return false;
        }

        // 2. Not all 0xFF
        if bytes.iter().all(|&b| b == 0xFF) {
            return false;
        }

        // 3. Some entropy (not too many repeated bytes)
        let mut byte_counts = [0u8; 256];
        for &byte in bytes {
            byte_counts[byte as usize] += 1;
        }

        // If any byte appears more than 50% of the time, probably not a pubkey
        let max_count = byte_counts.iter().max().unwrap_or(&0);
        if *max_count > 16 {
            return false;
        }

        true
    }

    /// Get the number of entries in the dictionary
    pub fn entry_count(&self) -> usize {
        self.address_to_index.len()
    }

    /// Serialize dictionary for storage
    pub fn serialize(&self) -> Result<Vec<u8>, CompressionError> {
        let serializable = SerializableDictionary {
            addresses: self.index_to_address.clone(),
            next_index: self.next_index,
        };

        bincode::serialize(&serializable)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))
    }

    /// Deserialize dictionary from storage
    pub fn deserialize(data: &[u8]) -> Result<Self, CompressionError> {
        let serializable: SerializableDictionary = bincode::deserialize(data)
            .map_err(|e| CompressionError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;

        let mut dict = Self::new();
        dict.index_to_address = serializable.addresses;
        dict.next_index = serializable.next_index;

        // Rebuild address_to_index map
        for (&index, &address) in &dict.index_to_address {
            dict.address_to_index.insert(address, index);
        }

        Ok(dict)
    }

    /// Calculate compression ratio for the dictionary
    pub fn compression_ratio(&self) -> f64 {
        if self.entry_count() == 0 {
            return 1.0;
        }

        // Each address saves 30 bytes (32 - 2)
        let bytes_saved = self.entry_count() * 30;
        let dictionary_overhead = self.entry_count() * 32; // Store full addresses in dict

        if dictionary_overhead > bytes_saved {
            1.0 // No compression benefit
        } else {
            (bytes_saved - dictionary_overhead) as f64 / (self.entry_count() * 32) as f64
        }
    }
}

impl Default for AccountDictionary {
    fn default() -> Self {
        Self::new()
    }
}

/// Serializable version of the dictionary for storage
#[derive(Serialize, Deserialize)]
struct SerializableDictionary {
    addresses: HashMap<u16, Pubkey>,
    next_index: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_dictionary_basic() {
        let mut dict = AccountDictionary::new();

        let addr1 = Pubkey::new_unique();
        let addr2 = Pubkey::new_unique();

        let idx1 = dict.add_address(addr1);
        let idx2 = dict.add_address(addr2);

        assert_ne!(idx1, idx2);
        assert_eq!(dict.get_address(idx1), Some(addr1));
        assert_eq!(dict.get_address(idx2), Some(addr2));
    }

    #[test]
    fn test_system_addresses_preloaded() {
        let dict = AccountDictionary::new();

        // System program should be pre-loaded
        let system_program = "11111111111111111111111111111111".parse::<Pubkey>().unwrap();
        assert!(dict.get_index(&system_program).is_some());
    }

    #[test]
    fn test_compression_decompression() {
        let mut dict = AccountDictionary::new();

        // Create test data with a pubkey
        let test_pubkey = Pubkey::new_unique();
        let mut test_data = vec![1, 2, 3, 4];
        test_data.extend_from_slice(test_pubkey.as_ref());
        test_data.extend_from_slice(&[5, 6, 7, 8]);

        let compressed = dict.compress_data(&test_data).unwrap();
        let decompressed = dict.decompress_data(&compressed).unwrap();

        assert_eq!(test_data, decompressed);
        assert!(compressed.len() < test_data.len()); // Should be smaller
    }
}