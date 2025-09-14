//! Enhanced Merkle Tree Implementation
//! Provides efficient Merkle tree operations for the privacy pool

use crate::utxo::{UTXO, MerkleProof};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Enhanced Merkle Tree for privacy pool
/// Provides efficient insertion, proof generation, and verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMerkleTree {
    /// Tree depth (maximum 32 levels)
    pub depth: u32,
    /// Current root hash
    pub root: [u8; 32],
    /// Tree leaves (commitments)
    pub leaves: Vec<[u8; 32]>,
    /// Tree nodes for efficient updates
    pub nodes: HashMap<(u32, u32), [u8; 32]>,
    /// Next leaf index
    pub next_leaf_index: u64,
    /// UTXO to leaf index mapping
    pub utxo_to_index: HashMap<[u8; 32], u64>, // commitment -> leaf index
}

impl EnhancedMerkleTree {
    /// Create a new Merkle tree with default depth of 32
    pub fn new() -> Self {
        Self::with_depth(32)
    }

    /// Create a new Merkle tree with specified depth
    pub fn with_depth(depth: u32) -> Self {
        let max_leaves = 2u64.pow(depth);
        let mut tree = Self {
            depth,
            root: [0u8; 32],
            leaves: Vec::with_capacity(max_leaves as usize),
            nodes: HashMap::new(),
            next_leaf_index: 0,
            utxo_to_index: HashMap::new(),
        };
        
        // Initialize with empty leaves
        for _ in 0..max_leaves {
            tree.leaves.push([0u8; 32]);
        }
        
        // Build initial tree
        tree.update_root();
        tree
    }

    /// Insert a commitment into the tree
    pub fn insert_commitment(&mut self, commitment: [u8; 32]) -> Result<u64, String> {
        if self.next_leaf_index >= 2u64.pow(self.depth) {
            return Err("Tree is full".to_string());
        }

        let index = self.next_leaf_index;
        self.leaves[index as usize] = commitment;
        self.utxo_to_index.insert(commitment, index);
        self.next_leaf_index += 1;
        
        // Update tree nodes
        self.update_tree_from_leaf(index);
        
        Ok(index)
    }

    /// Insert a UTXO into the tree
    pub fn insert_utxo(&mut self, utxo: &UTXO) -> Result<u64, String> {
        self.insert_commitment(utxo.commitment)
    }

    /// Update tree from specific leaf
    fn update_tree_from_leaf(&mut self, leaf_index: u64) {
        let mut current_index = leaf_index;
        let mut level = 0;

        // Update leaf node
        self.nodes.insert((level, current_index as u32), self.leaves[current_index as usize]);

        // Update parent nodes
        while level < self.depth {
            let parent_index = current_index / 2;
            let left_child = current_index * 2;
            let right_child = current_index * 2 + 1;

            // Get left and right child hashes
            let left_hash = if left_child < self.next_leaf_index {
                self.leaves[left_child as usize]
            } else {
                [0u8; 32]
            };

            let right_hash = if right_child < self.next_leaf_index {
                self.leaves[right_child as usize]
            } else {
                [0u8; 32]
            };

            // Compute parent hash
            let parent_hash = self.hash_children(left_hash, right_hash);
            self.nodes.insert((level + 1, parent_index as u32), parent_hash);

            current_index = parent_index;
            level += 1;
        }

        // Update root
        self.root = self.nodes.get(&(self.depth, 0)).copied().unwrap_or([0u8; 32]);
    }

    /// Update the entire tree root
    fn update_root(&mut self) {
        if self.next_leaf_index == 0 {
            self.root = [0u8; 32];
            return;
        }

        // Build tree bottom-up
        for level in 0..self.depth {
            let level_size = 2u64.pow(self.depth - level);
            for i in 0..level_size {
                let left_child = i * 2;
                let right_child = i * 2 + 1;

                let left_hash = if level == 0 {
                    // Leaf level
                    if left_child < self.next_leaf_index {
                        self.leaves[left_child as usize]
                    } else {
                        [0u8; 32]
                    }
                } else {
                    // Internal level
                    self.nodes.get(&(level - 1, left_child as u32)).copied().unwrap_or([0u8; 32])
                };

                let right_hash = if level == 0 {
                    // Leaf level
                    if right_child < self.next_leaf_index {
                        self.leaves[right_child as usize]
                    } else {
                        [0u8; 32]
                    }
                } else {
                    // Internal level
                    self.nodes.get(&(level - 1, right_child as u32)).copied().unwrap_or([0u8; 32])
                };

                let parent_hash = self.hash_children(left_hash, right_hash);
                self.nodes.insert((level, i as u32), parent_hash);
            }
        }

        // Set root
        self.root = self.nodes.get(&(self.depth - 1, 0)).copied().unwrap_or([0u8; 32]);
    }

    /// Generate Merkle proof for a commitment
    pub fn generate_proof(&self, commitment: [u8; 32]) -> Option<MerkleProof> {
        let leaf_index = self.utxo_to_index.get(&commitment)?;
        self.generate_proof_by_index(*leaf_index)
    }

    /// Generate Merkle proof by leaf index
    pub fn generate_proof_by_index(&self, leaf_index: u64) -> Option<MerkleProof> {
        if leaf_index >= self.next_leaf_index {
            return None;
        }

        let mut siblings = Vec::new();
        let mut path = Vec::new();
        let mut current_index = leaf_index;
        let mut level = 0;

        // Build proof path
        while level < self.depth {
            let sibling_index = if current_index % 2 == 0 {
                current_index + 1
            } else {
                current_index - 1
            };

            // Get sibling hash
            let sibling_hash = if level == 0 {
                // Leaf level
                if sibling_index < self.next_leaf_index {
                    self.leaves[sibling_index as usize]
                } else {
                    [0u8; 32]
                }
            } else {
                // Internal level
                self.nodes.get(&(level - 1, sibling_index as u32)).copied().unwrap_or([0u8; 32])
            };

            siblings.push(sibling_hash);
            path.push((current_index % 2) as u32);

            current_index /= 2;
            level += 1;
        }

        Some(MerkleProof::new(siblings, path, self.root, leaf_index))
    }

    /// Verify Merkle proof
    pub fn verify_proof(&self, proof: &MerkleProof, leaf: [u8; 32]) -> bool {
        proof.verify(leaf)
    }

    /// Get current root
    pub fn get_root(&self) -> [u8; 32] {
        self.root
    }

    /// Get tree size (number of leaves)
    pub fn size(&self) -> u64 {
        self.next_leaf_index
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.next_leaf_index == 0
    }

    /// Get leaf by index
    pub fn get_leaf(&self, index: u64) -> Option<[u8; 32]> {
        if index < self.next_leaf_index {
            Some(self.leaves[index as usize])
        } else {
            None
        }
    }

    /// Hash two children nodes
    fn hash_children(&self, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
}

impl Default for EnhancedMerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Relayer service for Merkle tree management
#[derive(Debug, Clone)]
pub struct RelayerService {
    /// Merkle tree instance
    pub tree: EnhancedMerkleTree,
    /// Service configuration
    pub config: RelayerConfig,
}

/// Relayer configuration
#[derive(Debug, Clone)]
pub struct RelayerConfig {
    /// Maximum tree depth
    pub max_depth: u32,
    /// Batch size for updates
    pub batch_size: u32,
    /// Update interval in seconds
    pub update_interval: u64,
}

impl RelayerService {
    /// Create a new relayer service
    pub fn new(config: RelayerConfig) -> Self {
        Self {
            tree: EnhancedMerkleTree::with_depth(config.max_depth),
            config,
        }
    }

    /// Add commitment to tree
    pub fn add_commitment(&mut self, commitment: [u8; 32]) -> Result<u64, String> {
        self.tree.insert_commitment(commitment)
    }

    /// Get Merkle proof for commitment
    pub fn get_proof(&self, commitment: [u8; 32]) -> Option<MerkleProof> {
        self.tree.generate_proof(commitment)
    }

    /// Get current root
    pub fn get_root(&self) -> [u8; 32] {
        self.tree.get_root()
    }
}
