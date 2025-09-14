//! Canonical Format Specification
//! 
//! This module defines the exact byte-level formats for all data structures
//! in the privacy pool system. All formats use big-endian encoding and strong
//! domain separation to prevent collisions.

use sha3::{Keccak256, Digest};

/// Domain separators for collision resistance
pub mod domains {
    /// UTXO ID domain separator: "UTOI"
    pub const UTXO_ID: [u8; 4] = [0x55, 0x54, 0x4F, 0x49];
    
    /// Leaf hash domain separator: "LEAF"
    pub const LEAF_HASH: [u8; 4] = [0x4C, 0x45, 0x41, 0x46];
    
    /// Node hash domain separator: "NODE"
    pub const NODE_HASH: [u8; 4] = [0x4E, 0x4F, 0x44, 0x45];
    
    /// Empty leaf domain separator: "EMPT"
    pub const EMPTY_LEAF: [u8; 4] = [0x45, 0x4D, 0x50, 0x54];
    
    /// Tree index domain separator: "INDX"
    pub const TREE_INDEX: [u8; 4] = [0x49, 0x4E, 0x44, 0x58];
    
    /// Transaction ID domain separator: "TXID"
    pub const TRANSACTION_ID: [u8; 4] = [0x54, 0x58, 0x49, 0x44];
}

/// UTXO serialization constants
pub mod utxo_format {
    /// Magic number for UTXO serialization: "UTX0"
    pub const MAGIC: u32 = 0x55545830;
    
    /// Current UTXO format version
    pub const VERSION: u16 = 1;
    
    /// Minimum serialized UTXO size (without lock_data)
    pub const MIN_SIZE: usize = 140;
    
    /// Asset ID for native ETH (20 zero bytes)
    pub const ETH_ASSET_ID: [u8; 20] = [0u8; 20];
}

/// Database column family prefixes
pub mod cf_prefixes {
    pub const UTXOS: u8 = 0x01;
    pub const SMT_LEAVES: u8 = 0x02;
    pub const SMT_NODES: u8 = 0x03;
    pub const OWNER_INDEX: u8 = 0x04;
    pub const ASSET_BALANCES: u8 = 0x05;
    pub const SPENT_TRACKER: u8 = 0x06;
    pub const INPUT_LOCKS: u8 = 0x07;
    pub const MEMPOOL: u8 = 0x08;
    pub const ROOT_HISTORY: u8 = 0x09;
    pub const BLOCK_INDEX: u8 = 0x0A;
    pub const TREE_METADATA: u8 = 0x0B;
}

/// Tree configuration constants
pub mod tree_config {
    /// Default tree depth (32 levels = 2^32 max leaves)
    pub const DEFAULT_DEPTH: u8 = 32;
    
    /// Maximum supported tree depth
    pub const MAX_DEPTH: u8 = 64;
    
    /// Parallel processing threshold for batch updates
    pub const PARALLEL_THRESHOLD: usize = 1000;
}

/// Generate UTXO ID using canonical format
/// 
/// # Arguments
/// * `txid` - Transaction ID (32 bytes)
/// * `vout` - Output index (4 bytes BE)
/// * `created_block` - Block number where UTXO was created (8 bytes BE)
/// * `entropy` - Operator-provided randomness (8 bytes BE)
/// 
/// # Returns
/// * 32-byte UTXO ID
pub fn generate_utxo_id(
    txid: [u8; 32],
    vout: u32,
    created_block: u64,
    entropy: u64,
) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(&domains::UTXO_ID);
    hasher.update(&txid);
    hasher.update(&vout.to_be_bytes());
    hasher.update(&created_block.to_be_bytes());
    hasher.update(&entropy.to_be_bytes());
    hasher.finalize().into()
}

/// Generate leaf hash using canonical format
/// 
/// # Arguments
/// * `serialized_utxo` - Canonical serialized UTXO bytes
/// 
/// # Returns
/// * 32-byte leaf hash
pub fn generate_leaf_hash(serialized_utxo: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(&domains::LEAF_HASH);
    hasher.update(serialized_utxo);
    hasher.finalize().into()
}

/// Generate node hash using canonical format
/// 
/// # Arguments
/// * `left_hash` - Left child hash (32 bytes)
/// * `right_hash` - Right child hash (32 bytes)
/// 
/// # Returns
/// * 32-byte node hash
pub fn generate_node_hash(left_hash: [u8; 32], right_hash: [u8; 32]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(&domains::NODE_HASH);
    hasher.update(&left_hash);
    hasher.update(&right_hash);
    hasher.finalize().into()
}

/// Generate empty leaf hash
/// 
/// # Returns
/// * 32-byte empty leaf hash
pub fn generate_empty_leaf_hash() -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(&domains::EMPTY_LEAF);
    // Create zero-filled serialized UTXO of minimum size
    let zero_utxo = vec![0u8; utxo_format::MIN_SIZE];
    hasher.update(&zero_utxo);
    hasher.finalize().into()
}

/// Generate tree index from UTXO ID
/// 
/// # Arguments
/// * `utxo_id` - UTXO identifier (32 bytes)
/// * `tree_salt` - Per-tree randomization salt (8 bytes)
/// 
/// # Returns
/// * 64-bit tree index (big-endian)
pub fn generate_tree_index(utxo_id: [u8; 32], tree_salt: u64) -> u64 {
    let mut hasher = Keccak256::new();
    hasher.update(&domains::TREE_INDEX);
    hasher.update(&utxo_id);
    hasher.update(&tree_salt.to_be_bytes());
    let hash = hasher.finalize();
    
    // Take first 8 bytes and convert to u64 big-endian
    u64::from_be_bytes([
        hash[0], hash[1], hash[2], hash[3],
        hash[4], hash[5], hash[6], hash[7],
    ])
}

/// Precompute empty subtree hashes up to given depth
/// 
/// # Arguments
/// * `depth` - Maximum tree depth
/// 
/// # Returns
/// * Vector of empty subtree hashes for each level
pub fn precompute_empty_subtrees(depth: u8) -> Vec<[u8; 32]> {
    let mut empty_subtrees = Vec::with_capacity(depth as usize + 1);
    
    // Level 0: empty leaf
    empty_subtrees.push(generate_empty_leaf_hash());
    
    // Levels 1..depth: recursive doubling
    for level in 1..=depth {
        let prev_hash = empty_subtrees[(level - 1) as usize];
        let node_hash = generate_node_hash(prev_hash, prev_hash);
        empty_subtrees.push(node_hash);
    }
    
    empty_subtrees
}

/// Compute full path from leaf to root
/// 
/// # Arguments
/// * `index` - Leaf index
/// * `depth` - Tree depth
/// 
/// # Returns
/// * Vector of (index, level) pairs from leaf to root
pub fn compute_full_path(index: u64, depth: u8) -> Vec<(u64, u8)> {
    let mut path = Vec::with_capacity(depth as usize);
    let mut current = index;
    
    for level in 0..depth {
        path.push((current, level));
        current >>= 1;
    }
    
    path
}

/// Align size to 8-byte boundary
/// 
/// # Arguments
/// * `size` - Size to align
/// 
/// # Returns
/// * Size aligned to next 8-byte boundary
pub fn align8(size: usize) -> usize {
    (size + 7) & !7
}

/// Calculate CRC32 checksum
/// 
/// # Arguments
/// * `data` - Data to checksum
/// 
/// # Returns
/// * CRC32 checksum as big-endian u32
pub fn calculate_crc32(data: &[u8]) -> u32 {
    // Using a simple CRC32 implementation
    // In production, use a proper CRC32 crate like `crc`
    let mut crc: u32 = 0xFFFFFFFF;
    for &byte in data {
        crc ^= byte as u32;
        for _ in 0..8 {
            if crc & 1 != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    (!crc).to_be()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_id_generation() {
        let txid = [1u8; 32];
        let vout = 0u32;
        let created_block = 12345u64;
        let entropy = 67890u64;
        
        let utxo_id = generate_utxo_id(txid, vout, created_block, entropy);
        
        // Should be deterministic
        let utxo_id2 = generate_utxo_id(txid, vout, created_block, entropy);
        assert_eq!(utxo_id, utxo_id2);
        
        // Should be different with different inputs
        let utxo_id3 = generate_utxo_id(txid, vout + 1, created_block, entropy);
        assert_ne!(utxo_id, utxo_id3);
    }

    #[test]
    fn test_hash_functions() {
        let test_data = b"test data";
        let leaf_hash = generate_leaf_hash(test_data);
        
        let left = [1u8; 32];
        let right = [2u8; 32];
        let node_hash = generate_node_hash(left, right);
        
        // Should be deterministic
        assert_eq!(leaf_hash, generate_leaf_hash(test_data));
        assert_eq!(node_hash, generate_node_hash(left, right));
        
        // Should be different with different order
        let node_hash2 = generate_node_hash(right, left);
        assert_ne!(node_hash, node_hash2);
    }

    #[test]
    fn test_empty_subtree_computation() {
        let depth = 4;
        let empty_subtrees = precompute_empty_subtrees(depth);
        
        assert_eq!(empty_subtrees.len(), (depth + 1) as usize);
        
        // Each level should be hash of two copies of previous level
        for i in 1..empty_subtrees.len() {
            let expected = generate_node_hash(empty_subtrees[i-1], empty_subtrees[i-1]);
            assert_eq!(empty_subtrees[i], expected);
        }
    }

    #[test]
    fn test_tree_index_generation() {
        let utxo_id = [1u8; 32];
        let salt = 12345u64;
        
        let index = generate_tree_index(utxo_id, salt);
        
        // Should be deterministic
        assert_eq!(index, generate_tree_index(utxo_id, salt));
        
        // Should be different with different salt
        let index2 = generate_tree_index(utxo_id, salt + 1);
        assert_ne!(index, index2);
    }

    #[test]
    fn test_path_computation() {
        let index = 13u64; // Binary: 1101
        let depth = 4;
        let path = compute_full_path(index, depth);
        
        assert_eq!(path.len(), depth as usize);
        assert_eq!(path[0], (13, 0)); // leaf
        assert_eq!(path[1], (6, 1));  // 13 >> 1 = 6
        assert_eq!(path[2], (3, 2));  // 6 >> 1 = 3
        assert_eq!(path[3], (1, 3));  // 3 >> 1 = 1
    }

    #[test]
    fn test_alignment() {
        assert_eq!(align8(1), 8);
        assert_eq!(align8(8), 8);
        assert_eq!(align8(9), 16);
        assert_eq!(align8(16), 16);
        assert_eq!(align8(17), 24);
    }
}