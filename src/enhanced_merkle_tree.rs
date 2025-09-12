//! Enhanced Merkle Tree Implementation for Privacy Pool
//! Implements commitment insertion and UTXO binding to owners

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use anyhow::{Result, anyhow};
use std::collections::HashMap;

/// Enhanced Merkle Tree with commitment tracking and owner binding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedMerkleTree {
    /// All UTXOs in the tree
    pub utxos: Vec<UTXO>,
    /// Merkle tree root hash
    pub root: [u8; 32],
    /// Tree depth (log2 of number of leaves)
    pub depth: usize,
    /// Mapping from commitment hash to UTXO index
    pub commitment_to_index: HashMap<[u8; 32], usize>,
    /// Mapping from owner public key to UTXO indices
    pub owner_to_utxos: HashMap<[u8; 32], Vec<usize>>,
    /// Next available leaf index
    pub next_leaf_index: u64,
    /// Nullifier registry to prevent double spending
    pub used_nullifiers: HashMap<[u8; 32], bool>,
    /// Spent UTXOs (by leaf index)
    pub spent_utxos: HashMap<u64, bool>,
}

/// UTXO with enhanced owner binding
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub struct UTXO {
    /// Commitment hash (includes owner public key)
    pub commitment: [u8; 32],
    /// Value in wei
    pub value: u64,
    /// Secret key for spending
    pub secret: [u8; 32],
    /// Nullifier for double-spend prevention
    pub nullifier: [u8; 32],
    /// Owner's public key (32 bytes)
    pub owner: [u8; 32],
    /// Leaf index in Merkle tree
    pub leaf_index: u64,
    /// Height when UTXO was created
    pub height: u32,
}

impl UTXO {
    /// Create a new UTXO with proper commitment binding to owner
    pub fn new(
        value: u64,
        secret: [u8; 32],
        nullifier: [u8; 32],
        owner: [u8; 32],
        height: u32,
    ) -> Self {
        let commitment = Self::compute_commitment(value, &secret, &nullifier, &owner);
        Self {
            commitment,
            value,
            secret,
            nullifier,
            owner,
            leaf_index: 0, // Will be set when added to tree
            height,
        }
    }
    
    /// Compute commitment hash that includes owner public key
    /// commitment = hash(value || secret || nullifier || owner_pubkey)
    pub fn compute_commitment(
        value: u64,
        secret: &[u8; 32],
        nullifier: &[u8; 32],
        owner: &[u8; 32],
    ) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&value.to_le_bytes());
        hasher.update(secret);
        hasher.update(nullifier);
        hasher.update(owner);
        hasher.finalize().into()
    }
    
    /// Verify that the UTXO belongs to the given owner
    pub fn verify_ownership(&self, owner: &[u8; 32]) -> bool {
        self.owner == *owner
    }
    
    /// Generate nullifier hash for spending
    pub fn compute_nullifier_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.nullifier);
        hasher.update(&self.leaf_index.to_le_bytes());
        hasher.finalize().into()
    }
}

/// Merkle proof for UTXO inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Sibling hashes along the path to root
    pub siblings: Vec<[u8; 32]>,
    /// Path indices (true = right, false = left)
    pub path_indices: Vec<bool>,
    /// Root hash at time of proof generation
    pub root: [u8; 32],
    /// Leaf index of the UTXO
    pub leaf_index: u64,
}

impl EnhancedMerkleTree {
    /// Create a new empty Merkle tree
    pub fn new() -> Self {
        Self {
            utxos: Vec::new(),
            root: [0u8; 32],
            depth: 0,
            commitment_to_index: HashMap::new(),
            owner_to_utxos: HashMap::new(),
            next_leaf_index: 0,
            used_nullifiers: HashMap::new(),
            spent_utxos: HashMap::new(),
        }
    }
    
    /// Insert a commitment into the Merkle tree and bind to owner
    pub fn insert_commitment(
        &mut self,
        value: u64,
        secret: [u8; 32],
        nullifier: [u8; 32],
        owner: [u8; 32],
        height: u32,
    ) -> Result<usize> {
        // Create UTXO with proper commitment binding
        let mut utxo = UTXO::new(value, secret, nullifier, owner, height);
        utxo.leaf_index = self.next_leaf_index;
        
        // Check for duplicate commitment
        if self.commitment_to_index.contains_key(&utxo.commitment) {
            return Err(anyhow!("Commitment already exists in tree"));
        }
        
        let index = self.utxos.len();
        
        // Add to UTXO list
        self.utxos.push(utxo);
        
        // Update mappings
        self.commitment_to_index.insert(utxo.commitment, index);
        self.owner_to_utxos.entry(utxo.owner).or_insert_with(Vec::new).push(index);
        
        // Update Merkle tree
        self.update_root()?;
        
        // Increment next leaf index
        self.next_leaf_index += 1;
        
        Ok(index)
    }
    
    /// Get UTXO by commitment hash
    pub fn get_utxo_by_commitment(&self, commitment: &[u8; 32]) -> Option<&UTXO> {
        self.commitment_to_index
            .get(commitment)
            .and_then(|&index| self.utxos.get(index))
    }
    
    /// Get all UTXOs owned by a specific public key
    pub fn get_utxos_by_owner(&self, owner: &[u8; 32]) -> Vec<&UTXO> {
        self.owner_to_utxos
            .get(owner)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&index| self.utxos.get(index))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// Generate Merkle proof for a UTXO
    pub fn generate_proof(&self, leaf_index: u64) -> Result<MerkleProof> {
        let index = leaf_index as usize;
        if index >= self.utxos.len() {
            return Err(anyhow!("Leaf index out of bounds"));
        }
        
        let mut siblings = Vec::new();
        let mut path_indices = Vec::new();
        let mut current_index = index;
        
        // Build the complete tree structure exactly like update_root does
        let mut current_level = self.utxos
            .iter()
            .map(|utxo| self.hash_utxo(utxo))
            .collect::<Vec<_>>();
        
        // Generate proof by traversing tree levels
        while current_level.len() > 1 {
            let is_right = current_index % 2 == 1;
            let sibling_index = if is_right { current_index - 1 } else { current_index + 1 };
            
            // Find sibling
            if sibling_index < current_level.len() {
                siblings.push(current_level[sibling_index]);
            } else {
                // For odd number of nodes, the sibling is the same node (duplication)
                siblings.push(current_level[current_index]);
            }
            
            path_indices.push(is_right);
            
            // Build next level
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = current_level[i];
                let right = if i + 1 < current_level.len() {
                    current_level[i + 1]
                } else {
                    // Duplicate last element if odd number
                    current_level[i]
                };
                next_level.push(self.hash_pair(left, right));
            }
            
            current_level = next_level;
            current_index /= 2;
        }
        
        Ok(MerkleProof {
            siblings,
            path_indices,
            root: self.root,
            leaf_index,
        })
    }
    
    /// Verify Merkle proof
    pub fn verify_proof(&self, proof: &MerkleProof, leaf: &[u8; 32]) -> bool {
        let mut current = *leaf;
        for (sibling, is_right) in proof.siblings.iter().zip(proof.path_indices.iter()) {
            current = if *is_right {
                // Current node is on the right, sibling on the left
                self.hash_pair(*sibling, current)
            } else {
                // Current node is on the left, sibling on the right
                self.hash_pair(current, *sibling)
            };
        }
        // Verify against the proof's root
        current == proof.root
    }
    
    /// Get current tree root
    pub fn get_root(&self) -> [u8; 32] {
        self.root
    }
    
    /// Get tree statistics
    pub fn get_stats(&self) -> TreeStats {
        TreeStats {
            total_utxos: self.utxos.len(),
            tree_depth: self.depth,
            root_hash: self.root,
            next_leaf_index: self.next_leaf_index,
        }
    }
    
    /// Update Merkle tree root after insertion
    fn update_root(&mut self) -> Result<()> {
        if self.utxos.is_empty() {
            self.root = [0u8; 32];
            self.depth = 0;
            return Ok(());
        }
        
        if self.utxos.len() == 1 {
            self.root = self.hash_utxo(&self.utxos[0]);
            self.depth = 1;
            return Ok(());
        }
        
        // Build tree bottom-up
        let mut current_level = self.utxos
            .iter()
            .map(|utxo| self.hash_utxo(utxo))
            .collect::<Vec<_>>();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = current_level[i];
                let right = if i + 1 < current_level.len() {
                    current_level[i + 1]
                } else {
                    current_level[i] // Duplicate last element if odd number
                };
                
                let parent = self.hash_pair(left, right);
                next_level.push(parent);
            }
            current_level = next_level;
        }
        
        self.root = current_level[0];
        self.depth = (self.utxos.len() as f64).log2().ceil() as usize;
        Ok(())
    }
    
    /// Hash a UTXO for Merkle tree
    pub fn hash_utxo(&self, utxo: &UTXO) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&utxo.commitment);
        hasher.update(&utxo.value.to_le_bytes());
        hasher.update(&utxo.owner);
        hasher.update(&utxo.leaf_index.to_le_bytes());
        hasher.finalize().into()
    }
    
    /// Hash a pair of nodes
    pub fn hash_pair(&self, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&left);
        hasher.update(&right);
        hasher.finalize().into()
    }
    
    /// Spend a UTXO using its nullifier (prevents double spending)
    pub fn spend_utxo(&mut self, nullifier: [u8; 32], leaf_index: u64) -> Result<()> {
        // Check if nullifier has been used
        if self.used_nullifiers.contains_key(&nullifier) {
            return Err(anyhow!("Nullifier has already been used (double spend attempt)"));
        }
        
        // Check if UTXO exists and is not already spent
        if leaf_index >= self.next_leaf_index {
            return Err(anyhow!("Invalid leaf index"));
        }
        
        if self.spent_utxos.contains_key(&leaf_index) {
            return Err(anyhow!("UTXO has already been spent"));
        }
        
        // Mark nullifier as used
        self.used_nullifiers.insert(nullifier, true);
        
        // Mark UTXO as spent
        self.spent_utxos.insert(leaf_index, true);
        
        Ok(())
    }
    
    /// Check if a nullifier has been used
    pub fn is_nullifier_used(&self, nullifier: &[u8; 32]) -> bool {
        self.used_nullifiers.contains_key(nullifier)
    }
    
    /// Check if a UTXO has been spent
    pub fn is_utxo_spent(&self, leaf_index: u64) -> bool {
        self.spent_utxos.contains_key(&leaf_index)
    }
    
    /// Get all unspent UTXOs for an owner
    pub fn get_unspent_utxos_by_owner(&self, owner: &[u8; 32]) -> Vec<&UTXO> {
        self.owner_to_utxos
            .get(owner)
            .map(|indices| {
                indices
                    .iter()
                    .filter_map(|&index| {
                        let utxo = self.utxos.get(index)?;
                        if !self.is_utxo_spent(utxo.leaf_index) {
                            Some(utxo)
                        } else {
                            None
                        }
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Tree statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeStats {
    pub total_utxos: usize,
    pub tree_depth: usize,
    pub root_hash: [u8; 32],
    pub next_leaf_index: u64,
}

/// Relayer service for handling deposit events and tree updates
pub struct RelayerService {
    tree: EnhancedMerkleTree,
    last_processed_block: u64,
}

impl RelayerService {
    pub fn new() -> Self {
        Self {
            tree: EnhancedMerkleTree::new(),
            last_processed_block: 0,
        }
    }
    
    /// Process a deposit event and insert commitment into tree
    pub fn process_deposit_event(
        &mut self,
        depositor: [u8; 32],
        value: u64,
        commitment: [u8; 32],
        block_number: u64,
    ) -> Result<usize> {
        // Generate secret and nullifier for the deposit
        let secret = self.generate_secret(block_number, value);
        let nullifier = self.generate_nullifier(&secret, self.tree.next_leaf_index);
        
        // Insert commitment into tree
        let index = self.tree.insert_commitment(
            value,
            secret,
            nullifier,
            depositor,
            block_number as u32,
        )?;
        
        self.last_processed_block = block_number;
        Ok(index)
    }
    
    /// Get Merkle proof for a commitment
    pub fn get_merkle_proof(&self, commitment: &[u8; 32]) -> Result<MerkleProof> {
        let utxo = self.tree.get_utxo_by_commitment(commitment)
            .ok_or_else(|| anyhow!("UTXO not found for commitment"))?;
        
        self.tree.generate_proof(utxo.leaf_index)
    }
    
    /// Get all UTXOs for an owner
    pub fn get_owner_utxos(&self, owner: &[u8; 32]) -> Vec<&UTXO> {
        self.tree.get_utxos_by_owner(owner)
    }
    
    /// Get tree statistics
    pub fn get_tree_stats(&self) -> TreeStats {
        self.tree.get_stats()
    }
    
    /// Generate secret for UTXO
    fn generate_secret(&self, block_number: u64, value: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&block_number.to_le_bytes());
        hasher.update(&value.to_le_bytes());
        hasher.update(&self.tree.next_leaf_index.to_le_bytes());
        hasher.update(b"privacy_pool_secret");
        hasher.finalize().into()
    }
    
    /// Generate nullifier for UTXO
    fn generate_nullifier(&self, secret: &[u8; 32], leaf_index: u64) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(secret);
        hasher.update(&leaf_index.to_le_bytes());
        hasher.update(b"privacy_pool_nullifier");
        hasher.finalize().into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_commitment_insertion() {
        let mut tree = EnhancedMerkleTree::new();
        
        // Create test UTXO
        let owner = [0x03; 32]; // Public key hash
        let value = 1000000000000000000; // 1 ETH
        let secret = [0x42; 32];
        let nullifier = [0x43; 32];
        let height = 1000;
        
        // Insert commitment
        let index = tree.insert_commitment(value, secret, nullifier, owner, height).unwrap();
        
        // Verify insertion
        assert_eq!(index, 0);
        assert_eq!(tree.utxos.len(), 1);
        assert_eq!(tree.next_leaf_index, 1);
        
        // Verify owner binding
        let owner_utxos = tree.get_utxos_by_owner(&owner);
        assert_eq!(owner_utxos.len(), 1);
        assert_eq!(owner_utxos[0].value, value);
        assert!(owner_utxos[0].verify_ownership(&owner));
    }
    
    #[test]
    fn test_merkle_proof_generation() {
        let mut tree = EnhancedMerkleTree::new();
        
        // Insert multiple UTXOs
        for i in 0..4 {
            let owner = [i as u8; 32];
            let value = (i + 1) * 1000000000000000000;
            let secret = [i as u8; 32];
            let nullifier = [(i + 1) as u8; 32];
            let height = 1000 + i as u32;
            
            tree.insert_commitment(value, secret, nullifier, owner, height).unwrap();
        }
        
        // Generate proof for first UTXO
        let proof = tree.generate_proof(0).unwrap();
        
        // Verify proof
        let leaf_hash = tree.hash_utxo(&tree.utxos[0]);
        assert!(tree.verify_proof(&proof, &leaf_hash));
    }
    
    #[test]
    fn test_relayer_service() {
        let mut relayer = RelayerService::new();
        
        // Process deposit event
        let depositor = [0x03; 32];
        let value = 2000000000000000000; // 2 ETH
        let commitment = [0x44; 32];
        let block_number = 1001;
        
        let index = relayer.process_deposit_event(depositor, value, commitment, block_number).unwrap();
        
        // Verify processing
        assert_eq!(index, 0);
        assert_eq!(relayer.last_processed_block, block_number);
        
        // Verify owner can access UTXOs
        let owner_utxos = relayer.get_owner_utxos(&depositor);
        assert_eq!(owner_utxos.len(), 1);
        assert_eq!(owner_utxos[0].value, value);
    }
    
    #[test]
    fn test_nullifier_registry_and_double_spend_prevention() {
        let mut tree = EnhancedMerkleTree::new();
        
        // Create and insert UTXO
        let owner = [0x05; 32];
        let value = 1000000000000000000;
        let secret = [0x50; 32];
        let nullifier = [0x51; 32];
        let height = 1000;
        
        let index = tree.insert_commitment(value, secret, nullifier, owner, height).unwrap();
        
        // Get UTXO data before spending
        let utxo_leaf_index = tree.utxos[index].leaf_index;
        let nullifier_hash = tree.utxos[index].compute_nullifier_hash();
        
        // First spend should succeed
        assert!(tree.spend_utxo(nullifier_hash, utxo_leaf_index).is_ok());
        
        // Verify nullifier is marked as used
        assert!(tree.is_nullifier_used(&nullifier_hash));
        assert!(tree.is_utxo_spent(utxo_leaf_index));
        
        // Second spend attempt should fail (double spend)
        assert!(tree.spend_utxo(nullifier_hash, utxo_leaf_index).is_err());
        
        // Verify unspent UTXOs doesn't include spent ones
        let unspent_utxos = tree.get_unspent_utxos_by_owner(&owner);
        assert_eq!(unspent_utxos.len(), 0);
        
        // But all UTXOs still includes it
        let all_utxos = tree.get_utxos_by_owner(&owner);
        assert_eq!(all_utxos.len(), 1);
    }
    
    #[test]
    fn test_multiple_utxos_and_selective_spending() {
        let mut tree = EnhancedMerkleTree::new();
        
        let owner = [0x06; 32];
        
        // Insert multiple UTXOs for the same owner
        let mut utxos = Vec::new();
        for i in 0..3 {
            let value = (i + 1) as u64 * 1000000000000000000;
            let secret = [(i + 0x60) as u8; 32];
            let nullifier = [(i + 0x70) as u8; 32];
            let height = 1000 + i;
            
            let index = tree.insert_commitment(value, secret, nullifier, owner, height as u32).unwrap();
            utxos.push(tree.utxos[index]);
        }
        
        // Verify all UTXOs are unspent initially
        let unspent = tree.get_unspent_utxos_by_owner(&owner);
        assert_eq!(unspent.len(), 3);
        
        // Spend the middle UTXO
        let middle_utxo = &utxos[1];
        let nullifier_hash = middle_utxo.compute_nullifier_hash();
        assert!(tree.spend_utxo(nullifier_hash, middle_utxo.leaf_index).is_ok());
        
        // Verify only 2 unspent UTXOs remain
        let unspent = tree.get_unspent_utxos_by_owner(&owner);
        assert_eq!(unspent.len(), 2);
        
        // Verify the correct UTXOs are unspent
        let unspent_values: Vec<u64> = unspent.iter().map(|utxo| utxo.value).collect();
        assert!(unspent_values.contains(&(1000000000000000000))); // First UTXO
        assert!(unspent_values.contains(&(3000000000000000000))); // Third UTXO
        assert!(!unspent_values.contains(&(2000000000000000000))); // Second UTXO (spent)
    }
}
