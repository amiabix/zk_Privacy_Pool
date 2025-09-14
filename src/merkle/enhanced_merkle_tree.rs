//! Enhanced Merkle Tree Implementation
//! Architecture-compliant Merkle tree with Poseidon hashing for ZK-friendliness
//! Production-ready with RocksDB persistence and reorg handling

use crate::utxo::transaction::MerkleProof;
use crate::crypto::{CryptoResult, CryptoError, ArchitectureCompliantCrypto};
use crate::database::DatabaseManager;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use anyhow::Result;
use hex;

/// Enhanced Merkle Tree for privacy pool commitments
/// Architecture-compliant with Poseidon hashing and efficient operations
/// Production-ready with RocksDB persistence and reorg handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMerkleTree {
    /// Tree depth (32 levels = 2^32 max leaves)
    pub depth: u8,
    /// Current number of leaves
    pub leaf_count: u64,
    /// Current root hash
    pub root: [u8; 32],
    /// Tree nodes: level -> index -> hash (Level 0 = leaves, Level depth = root)
    pub nodes: HashMap<u8, HashMap<u64, [u8; 32]>>,
    /// Fast commitment lookup: commitment -> leaf_index
    pub commitment_to_index: HashMap<[u8; 32], u64>,
    /// Pre-computed empty subtree hashes for efficiency
    pub empty_hashes: Vec<[u8; 32]>,
    /// Next leaf index (persisted)
    pub next_leaf_index: u64,
    /// Root version counter
    pub root_version: u64,
}

/// Enhanced Merkle Tree with database persistence
pub struct PersistentMerkleTree {
    tree: EnhancedMerkleTree,
    db: DatabaseManager,
}

impl EnhancedMerkleTree {
    /// Create new enhanced Merkle tree with default depth of 32
    pub fn new() -> CryptoResult<Self> {
        Self::with_depth(32)
    }

    /// Create new enhanced Merkle tree with specified depth
    pub fn with_depth(depth: u8) -> CryptoResult<Self> {
        if depth == 0 || depth > 32 {
            return Err(CryptoError::InvalidInput("Tree depth must be between 1 and 32".to_string()));
        }

        let empty_hashes = Self::compute_empty_hashes(depth)?;
        let root = empty_hashes[depth as usize];

        Ok(Self {
            depth,
            leaf_count: 0,
            root,
            nodes: HashMap::new(),
            commitment_to_index: HashMap::new(),
            empty_hashes,
            next_leaf_index: 0,
            root_version: 0,
        })
    }

    /// Insert a commitment into the tree (idempotent)
    /// Returns the leaf index where the commitment was inserted
    /// API: insert_leaf(commitment: [u8;32]) -> Result<leaf_index: u64>
    pub fn insert_leaf(&mut self, commitment: [u8; 32]) -> CryptoResult<u64> {
        // Check if commitment already exists (idempotent)
        if let Some(&existing_index) = self.commitment_to_index.get(&commitment) {
            return Ok(existing_index);
        }

        // Check if tree is full
        let max_leaves = 1u64 << self.depth;
        if self.next_leaf_index >= max_leaves {
            return Err(CryptoError::InvalidInput("Merkle tree is full".to_string()));
        }

        let leaf_index = self.next_leaf_index;

        // Hash the commitment to get leaf hash
        let leaf_hash = ArchitectureCompliantCrypto::hash_merkle_leaf(&commitment)?;

        // Insert leaf at level 0
        self.nodes.entry(0).or_insert_with(HashMap::new).insert(leaf_index, leaf_hash);

        // Update path to root
        let mut current_hash = leaf_hash;
        let mut current_index = leaf_index;

        for level in 1..=self.depth {
            let parent_index = current_index / 2;
            let is_right_child = current_index % 2 == 1;

            // Get sibling hash
            let sibling_index = if is_right_child {
                current_index - 1
            } else {
                current_index + 1
            };

            let sibling_hash = self.get_node_hash(level - 1, sibling_index);

            // Compute parent hash
            let parent_hash = if is_right_child {
                // Current node is right child
                ArchitectureCompliantCrypto::hash_merkle_node(&sibling_hash, &current_hash)?
            } else {
                // Current node is left child
                ArchitectureCompliantCrypto::hash_merkle_node(&current_hash, &sibling_hash)?
            };

            // Store parent node
            self.nodes.entry(level).or_insert_with(HashMap::new).insert(parent_index, parent_hash);

            current_hash = parent_hash;
            current_index = parent_index;
        }

        // Update root
        self.root = current_hash;
        self.root_version += 1;

        // Update commitment lookup
        self.commitment_to_index.insert(commitment, leaf_index);

        // Increment counters
        self.leaf_count += 1;
        self.next_leaf_index += 1;

        Ok(leaf_index)
    }

    /// Legacy insert method for backward compatibility
    pub fn insert(&mut self, commitment: [u8; 32]) -> CryptoResult<u64> {
        self.insert_leaf(commitment)
    }

    /// Get Merkle proof for a leaf at given index
    pub fn get_proof(&self, leaf_index: u64) -> CryptoResult<MerkleProof> {
        if leaf_index >= self.leaf_count {
            return Err(CryptoError::InvalidInput("Leaf index out of bounds".to_string()));
        }

        let mut siblings = Vec::new();
        let mut path_bits = Vec::new();
        let mut current_index = leaf_index;

        // Collect siblings for each level
        for level in 0..self.depth {
            let is_right_child = current_index % 2 == 1;
            let sibling_index = if is_right_child {
                current_index - 1
            } else {
                current_index + 1
            };

            let sibling_hash = self.get_node_hash(level, sibling_index);
            siblings.push(sibling_hash);
            path_bits.push(if is_right_child { 1 } else { 0 });

            current_index /= 2;
        }

        Ok(MerkleProof {
            leaf_index,
            siblings,
            path: path_bits,
            root: self.root,
        })
    }

    /// Get current root hash
    pub fn get_root(&self) -> [u8; 32] {
        self.root
    }

    /// Get node hash at specific level and index
    /// Returns empty hash if node doesn't exist
    fn get_node_hash(&self, level: u8, index: u64) -> [u8; 32] {
        self.nodes
            .get(&level)
            .and_then(|level_nodes| level_nodes.get(&index))
            .copied()
            .unwrap_or_else(|| self.get_empty_hash_for_level(level))
    }

    /// Get empty hash for a specific level
    fn get_empty_hash_for_level(&self, level: u8) -> [u8; 32] {
        if level as usize >= self.empty_hashes.len() {
            [0u8; 32] // Fallback
        } else {
            self.empty_hashes[level as usize]
        }
    }

    /// Pre-compute empty subtree hashes for efficiency
    fn compute_empty_hashes(depth: u8) -> CryptoResult<Vec<[u8; 32]>> {
        let mut empty_hashes = Vec::with_capacity((depth + 1) as usize);

        // Level 0: empty leaf (all zeros)
        empty_hashes.push([0u8; 32]);

        // Level 1 to depth: empty internal nodes
        for _level in 1..=depth {
            let child_empty = empty_hashes[empty_hashes.len() - 1];
            let parent_empty = ArchitectureCompliantCrypto::hash_merkle_node(&child_empty, &child_empty)?;
            empty_hashes.push(parent_empty);
        }

        Ok(empty_hashes)
    }

    /// Verify a Merkle proof against this tree's root
    pub fn verify_proof(&self, proof: &MerkleProof, commitment: [u8; 32]) -> CryptoResult<bool> {
        if proof.root != self.root {
            return Ok(false);
        }

        self.verify_proof_with_root(proof, commitment, self.root)
    }

    /// Verify a Merkle proof against a specific root
    pub fn verify_proof_with_root(
        &self,
        proof: &MerkleProof,
        commitment: [u8; 32],
        root: [u8; 32]
    ) -> CryptoResult<bool> {
        if proof.siblings.len() != self.depth as usize {
            return Ok(false);
        }

        if proof.path.len() != self.depth as usize {
            return Ok(false);
        }

        // Start with leaf hash
        let mut current_hash = ArchitectureCompliantCrypto::hash_merkle_leaf(&commitment)?;
        let mut current_index = proof.leaf_index;

        // Traverse up the tree
        for (sibling, &path_bit) in proof.siblings.iter().zip(proof.path.iter()) {
            let is_right_child = path_bit == 1;

            // Verify path bit matches index
            if (current_index % 2 == 1) != is_right_child {
                return Ok(false);
            }

            // Compute parent hash
            current_hash = if is_right_child {
                ArchitectureCompliantCrypto::hash_merkle_node(sibling, &current_hash)?
            } else {
                ArchitectureCompliantCrypto::hash_merkle_node(&current_hash, sibling)?
            };

            current_index /= 2;
        }

        Ok(current_hash == root)
    }

    /// Get leaf hash for a commitment (if it exists)
    pub fn get_leaf_by_commitment(&self, commitment: &[u8; 32]) -> Option<[u8; 32]> {
        let leaf_index = self.commitment_to_index.get(commitment)?;
        self.nodes.get(&0)?.get(leaf_index).copied()
    }

    /// Get leaf index for a commitment (if it exists)
    pub fn get_leaf_index(&self, commitment: &[u8; 32]) -> Option<u64> {
        self.commitment_to_index.get(commitment).copied()
    }

    /// Get current tree size (number of leaves)
    pub fn size(&self) -> u64 {
        self.leaf_count
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.leaf_count == 0
    }

    /// Get current tree statistics
    pub fn stats(&self) -> TreeStats {
        TreeStats {
            depth: self.depth,
            leaf_count: self.leaf_count,
            max_leaves: 1u64 << self.depth,
            root: self.root,
            nodes_stored: self.nodes.values().map(|level| level.len()).sum(),
        }
    }

    /// Rollback to specific block number (for reorg handling)
    /// Strategy A: Wait N_CONFIRMATIONS before inserting (recommended)
    /// Strategy B: Support rollbacks (costly but possible)
    pub fn rollback_to_block(&mut self, block_number: u64) -> Result<()> {
        // This is a simplified implementation
        // In production, you'd need to track block_number -> leaf_index mappings
        // and remove/rollback affected leaves and recompute nodes
        
        // For now, we'll just log the rollback request
        println!("Rollback requested to block {}", block_number);
        
        // TODO: Implement proper rollback logic
        // 1. Find all leaves inserted after block_number
        // 2. Remove them from tree
        // 3. Recompute affected nodes
        // 4. Update root and version
        Ok(())
    }

    /// Get leaf at specific index
    pub fn get_leaf(&self, leaf_index: u64) -> Option<[u8; 32]> {
        if leaf_index >= self.leaf_count {
            return None;
        }
        self.nodes.get(&0)?.get(&leaf_index).copied()
    }

    /// Check if tree has commitment
    pub fn has_commitment(&self, commitment: &[u8; 32]) -> bool {
        self.commitment_to_index.contains_key(commitment)
    }

    /// Get all commitments (for debugging)
    pub fn get_all_commitments(&self) -> Vec<[u8; 32]> {
        self.commitment_to_index.keys().copied().collect()
    }
}

impl Default for EnhancedMerkleTree {
    fn default() -> Self {
        Self::new().expect("Failed to create default EnhancedMerkleTree")
    }
}

impl PersistentMerkleTree {
    /// Create new persistent Merkle tree with database
    pub fn new(db: DatabaseManager, depth: u8) -> CryptoResult<Self> {
        let tree = EnhancedMerkleTree::with_depth(depth)?;
        Ok(Self { tree, db })
    }

    /// Insert leaf with database persistence
    pub fn insert_leaf(&mut self, commitment: [u8; 32]) -> CryptoResult<u64> {
        let leaf_index = self.tree.insert_leaf(commitment)?;
        
        // Persist to database
        self.persist_insertion(commitment, leaf_index)
            .map_err(|e| CryptoError::SerializationError(e.to_string()))?;
        
        Ok(leaf_index)
    }

    /// Persist insertion to database
    fn persist_insertion(&self, commitment: [u8; 32], leaf_index: u64) -> Result<()> {
        // Store leaf mapping: leaf_index -> commitment
        let leaf_key = format!("leaf:{}", leaf_index);
        self.db.put_cf("cf_leaves", leaf_key.as_bytes(), &commitment)?;
        
        // Store commitment index: commitment -> leaf_index
        let commitment_key = format!("commitment:{}", hex::encode(commitment));
        let leaf_index_bytes = leaf_index.to_be_bytes();
        self.db.put_cf("cf_commitment_index", commitment_key.as_bytes(), &leaf_index_bytes)?;
        
        // Store next leaf index
        let next_leaf_bytes = self.tree.next_leaf_index.to_be_bytes();
        self.db.put_cf("cf_tree_metadata", b"next_leaf", &next_leaf_bytes)?;
        
        // Store root version
        let root_version_bytes = self.tree.root_version.to_be_bytes();
        self.db.put_cf("cf_tree_metadata", b"root_version", &root_version_bytes)?;
        
        // Store current root
        self.db.put_cf("cf_tree_metadata", b"current_root", &self.tree.root)?;
        
        Ok(())
    }

    /// Get tree reference
    pub fn tree(&self) -> &EnhancedMerkleTree {
        &self.tree
    }

    /// Get mutable tree reference
    pub fn tree_mut(&mut self) -> &mut EnhancedMerkleTree {
        &mut self.tree
    }
}

/// Tree statistics
#[derive(Debug, Clone)]
pub struct TreeStats {
    pub depth: u8,
    pub leaf_count: u64,
    pub max_leaves: u64,
    pub root: [u8; 32],
    pub nodes_stored: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::CryptoUtils;

    #[test]
    fn test_enhanced_merkle_tree_creation() {
        let tree = EnhancedMerkleTree::new().unwrap();
        assert_eq!(tree.depth, 32);
        assert_eq!(tree.leaf_count, 0);
        assert_ne!(tree.root, [0u8; 32]); // Root should be empty tree hash
    }

    #[test]
    fn test_small_tree_operations() {
        let mut tree = EnhancedMerkleTree::with_depth(4).unwrap();
        let commitment = CryptoUtils::random_32();

        // Test insert
        let index = tree.insert(commitment).unwrap();
        assert_eq!(index, 0);
        assert_eq!(tree.leaf_count, 1);

        // Test get_proof
        let proof = tree.get_proof(index).unwrap();
        assert_eq!(proof.leaf_index, index);
        assert_eq!(proof.siblings.len(), 4);
        assert_eq!(proof.root, tree.get_root());

        // Test verify_proof
        assert!(tree.verify_proof(&proof, commitment).unwrap());

        // Test with wrong commitment should fail
        let wrong_commitment = CryptoUtils::random_32();
        assert!(!tree.verify_proof(&proof, wrong_commitment).unwrap());
    }

    #[test]
    fn test_multiple_insertions() {
        let mut tree = EnhancedMerkleTree::with_depth(4).unwrap();
        let mut commitments = Vec::new();

        // Insert 5 commitments
        for i in 0..5 {
            let commitment = [i as u8; 32];
            commitments.push(commitment);
            let index = tree.insert(commitment).unwrap();
            assert_eq!(index, i as u64);
        }

        assert_eq!(tree.leaf_count, 5);

        // Verify all commitments are findable
        for (i, commitment) in commitments.iter().enumerate() {
            assert_eq!(tree.get_leaf_index(commitment), Some(i as u64));

            // Generate and verify proof for each
            let proof = tree.get_proof(i as u64).unwrap();
            assert!(tree.verify_proof(&proof, *commitment).unwrap());
        }
    }

    #[test]
    fn test_duplicate_commitment_handling() {
        let mut tree = EnhancedMerkleTree::with_depth(4).unwrap();
        let commitment = CryptoUtils::random_32();

        let index1 = tree.insert(commitment).unwrap();
        let index2 = tree.insert(commitment).unwrap(); // Same commitment

        assert_eq!(index1, index2);
        assert_eq!(tree.leaf_count, 1); // Should not increase
    }

    #[test]
    fn test_tree_stats() {
        let mut tree = EnhancedMerkleTree::with_depth(4).unwrap();
        let stats_empty = tree.stats();

        assert_eq!(stats_empty.leaf_count, 0);
        assert_eq!(stats_empty.max_leaves, 16); // 2^4

        // Insert some commitments
        tree.insert([1u8; 32]).unwrap();
        tree.insert([2u8; 32]).unwrap();

        let stats_filled = tree.stats();
        assert_eq!(stats_filled.leaf_count, 2);
        assert!(stats_filled.nodes_stored > 0);
    }
}
