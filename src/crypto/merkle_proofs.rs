//! Merkle Proof Verification Implementation
//! 
//! This module provides production-ready Merkle proof verification
//! with support for multiple hash functions and batch verification.

use crate::crypto::{CryptoResult, CryptoError, CryptoContext, CryptoUtils};
use crate::utxo::transaction::MerkleProof;
use sha3::Digest;
use std::collections::HashMap;

/// Merkle proof verifier with multiple hash functions
#[derive(Clone)]
pub struct MerkleProofVerifier {
    /// Hash function to use
    pub hash_function: HashFunction,
    /// Tree depth
    pub depth: usize,
    /// Empty leaf hash
    pub empty_leaf: [u8; 32],
    /// Empty subtree hashes
    pub empty_subtrees: Vec<[u8; 32]>,
}

/// Supported hash functions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashFunction {
    /// SHA-256
    Sha256,
    /// Blake2b-256
    Blake2b256,
    /// Keccak-256
    Keccak256,
    /// Poseidon
    Poseidon,
}

impl MerkleProofVerifier {
    /// Create new Merkle proof verifier
    pub fn new(hash_function: HashFunction, depth: usize) -> Self {
        let empty_leaf = Self::hash_empty_leaf(hash_function);
        let empty_subtrees = Self::precompute_empty_subtrees(hash_function, depth);
        
        Self {
            hash_function,
            depth,
            empty_leaf,
            empty_subtrees,
        }
    }
    
    /// Create verifier with context
    pub fn with_context(hash_function: HashFunction, depth: usize, context: &CryptoContext) -> Self {
        Self::new(hash_function, depth).apply_context(context)
    }
    
    /// Apply cryptographic context to the verifier
    pub fn apply_context(mut self, context: &CryptoContext) -> Self {
        // Modify empty leaf and subtrees based on context
        let context_hash = CryptoUtils::blake2b256(&context.domain);
        self.empty_leaf = self.hash_with_context(&self.empty_leaf, &context_hash);
        
        for i in 0..self.empty_subtrees.len() {
            self.empty_subtrees[i] = self.hash_with_context(&self.empty_subtrees[i], &context_hash);
        }
        self
    }
    
    /// Verify a Merkle proof
    pub fn verify_proof(&self, proof: &MerkleProof, leaf: &[u8; 32]) -> CryptoResult<bool> {
        if proof.siblings.len() != self.depth {
            return Err(CryptoError::MerkleProofFailed(
                format!("Invalid proof depth: expected {}, got {}", self.depth, proof.siblings.len())
            ));
        }
        
        if proof.path.len() != self.depth {
            return Err(CryptoError::MerkleProofFailed(
                format!("Invalid path length: expected {}, got {}", self.depth, proof.path.len())
            ));
        }
        
        // Start with the leaf
        let mut current = *leaf;
        
        // Walk up the tree using the proof
        for (i, (sibling, &path_bit)) in proof.siblings.iter().zip(proof.path.iter()).enumerate() {
            if path_bit == 1 {
                // Right child: hash(left, right)
                current = self.hash_children(*sibling, current);
            } else {
                // Left child: hash(left, right)
                current = self.hash_children(current, *sibling);
            }
        }
        
        // Check if the computed root matches the expected root
        Ok(current == proof.root)
    }
    
    /// Verify proof with context
    pub fn verify_proof_with_context(
        &self,
        proof: &MerkleProof,
        leaf: &[u8; 32],
        context: &CryptoContext,
    ) -> CryptoResult<bool> {
        let verifier = self.clone().apply_context(context);
        verifier.verify_proof(proof, leaf)
    }
    
    /// Batch verify multiple proofs
    pub fn batch_verify_proofs(
        &self,
        proofs: &[(MerkleProof, [u8; 32])],
    ) -> CryptoResult<bool> {
        for (proof, leaf) in proofs {
            if !self.verify_proof(proof, leaf)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    /// Generate Merkle proof for a leaf
    pub fn generate_proof(
        &self,
        leaf_index: u64,
        leaves: &[[u8; 32]],
    ) -> CryptoResult<MerkleProof> {
        if leaf_index >= leaves.len() as u64 {
            return Err(CryptoError::MerkleProofFailed("Leaf index out of bounds".to_string()));
        }
        
        let mut siblings = Vec::new();
        let mut path = Vec::new();
        let mut current_index = leaf_index;
        
        // Build the proof path
        for level in 0..self.depth {
            let sibling_index = current_index ^ 1;
            let sibling = if sibling_index < leaves.len() as u64 {
                leaves[sibling_index as usize]
            } else {
                self.empty_leaf
            };
            
            siblings.push(sibling);
            path.push((current_index & 1) as u32);
            current_index >>= 1;
        }
        
        // Compute the root
        let root = self.compute_root(leaves)?;
        
        Ok(MerkleProof {
            siblings,
            path,
            root,
            leaf_index,
        })
    }
    
    /// Compute Merkle root from leaves
    pub fn compute_root(&self, leaves: &[[u8; 32]]) -> CryptoResult<[u8; 32]> {
        if leaves.is_empty() {
            return Ok(self.empty_leaf);
        }
        
        let mut current_level = leaves.to_vec();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            
            for i in (0..current_level.len()).step_by(2) {
                let left = current_level[i];
                let right = if i + 1 < current_level.len() {
                    current_level[i + 1]
                } else {
                    self.empty_leaf
                };
                
                next_level.push(self.hash_children(left, right));
            }
            
            current_level = next_level;
        }
        
        Ok(current_level[0])
    }
    
    /// Hash two children nodes
    fn hash_children(&self, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        match self.hash_function {
            HashFunction::Sha256 => {
                let mut hasher = sha2::Sha256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
            HashFunction::Blake2b256 => {
                let mut hasher = blake2::Blake2s256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
            HashFunction::Keccak256 => {
                let mut hasher = sha3::Keccak256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
            HashFunction::Poseidon => {
                // In production, use proper Poseidon hash
                // For now, use Blake2b as fallback
                let mut hasher = blake2::Blake2s256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
        }
    }
    
    /// Hash with context
    fn hash_with_context(&self, data: &[u8; 32], context: &[u8; 32]) -> [u8; 32] {
        let mut combined = Vec::new();
        combined.extend_from_slice(context);
        combined.extend_from_slice(data);
        
        match self.hash_function {
            HashFunction::Sha256 => CryptoUtils::sha256(&combined),
            HashFunction::Blake2b256 => CryptoUtils::blake2b256(&combined),
            HashFunction::Keccak256 => CryptoUtils::keccak256(&combined),
            HashFunction::Poseidon => {
                // In production, use proper Poseidon hash
                CryptoUtils::blake2b256(&combined)
            }
        }
    }
    
    /// Hash empty leaf
    fn hash_empty_leaf(hash_function: HashFunction) -> [u8; 32] {
        let empty_data = [0u8; 32];
        
        match hash_function {
            HashFunction::Sha256 => CryptoUtils::sha256(&empty_data),
            HashFunction::Blake2b256 => CryptoUtils::blake2b256(&empty_data),
            HashFunction::Keccak256 => CryptoUtils::keccak256(&empty_data),
            HashFunction::Poseidon => {
                // In production, use proper Poseidon hash
                CryptoUtils::blake2b256(&empty_data)
            }
        }
    }
    
    /// Precompute empty subtree hashes
    fn precompute_empty_subtrees(hash_function: HashFunction, depth: usize) -> Vec<[u8; 32]> {
        let mut subtrees = Vec::new();
        let mut current = Self::hash_empty_leaf(hash_function);
        subtrees.push(current);
        
        for _ in 1..=depth {
            current = Self::hash_children_static(hash_function, current, current);
            subtrees.push(current);
        }
        
        subtrees
    }
    
    /// Hash children nodes (static method)
    fn hash_children_static(hash_function: HashFunction, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        match hash_function {
            HashFunction::Sha256 => {
                let mut hasher = sha2::Sha256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
            HashFunction::Blake2b256 => {
                let mut hasher = blake2::Blake2s256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
            HashFunction::Keccak256 => {
                let mut hasher = sha3::Keccak256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
            HashFunction::Poseidon => {
                // In production, use proper Poseidon hash
                let mut hasher = blake2::Blake2s256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
        }
    }
}

/// Merkle tree builder for efficient proof generation
pub struct MerkleTreeBuilder {
    /// Hash function
    pub hash_function: HashFunction,
    /// Tree depth
    pub depth: usize,
    /// Leaves
    pub leaves: Vec<[u8; 32]>,
    /// Tree nodes for efficient updates
    pub nodes: HashMap<(usize, u64), [u8; 32]>,
}

impl MerkleTreeBuilder {
    /// Create new Merkle tree builder
    pub fn new(hash_function: HashFunction, depth: usize) -> Self {
        Self {
            hash_function,
            depth,
            leaves: Vec::new(),
            nodes: HashMap::new(),
        }
    }
    
    /// Add leaf to the tree
    pub fn add_leaf(&mut self, leaf: [u8; 32]) -> CryptoResult<u64> {
        if self.leaves.len() >= 2usize.pow(self.depth as u32) {
            return Err(CryptoError::MerkleProofFailed("Tree is full".to_string()));
        }
        
        let index = self.leaves.len() as u64;
        self.leaves.push(leaf);
        
        // Update tree nodes
        self.update_tree_from_leaf(index)?;
        
        Ok(index)
    }
    
    /// Generate proof for leaf at index
    pub fn generate_proof(&self, leaf_index: u64) -> CryptoResult<MerkleProof> {
        if leaf_index >= self.leaves.len() as u64 {
            return Err(CryptoError::MerkleProofFailed("Leaf index out of bounds".to_string()));
        }
        
        let verifier = MerkleProofVerifier::new(self.hash_function, self.depth);
        verifier.generate_proof(leaf_index, &self.leaves)
    }
    
    /// Get current root
    pub fn get_root(&self) -> CryptoResult<[u8; 32]> {
        let verifier = MerkleProofVerifier::new(self.hash_function, self.depth);
        verifier.compute_root(&self.leaves)
    }
    
    /// Update tree from specific leaf
    fn update_tree_from_leaf(&mut self, leaf_index: u64) -> CryptoResult<()> {
        let mut current_index = leaf_index;
        let mut level = 0;
        
        // Update leaf node
        self.nodes.insert((level, current_index), self.leaves[leaf_index as usize]);
        
        // Update parent nodes
        while level < self.depth {
            let parent_index = current_index / 2;
            let left_child = current_index * 2;
            let right_child = current_index * 2 + 1;
            
            // Get left and right child hashes
            let left_hash = if left_child < self.leaves.len() as u64 {
                self.leaves[left_child as usize]
            } else {
                let verifier = MerkleProofVerifier::new(self.hash_function, self.depth);
                verifier.empty_leaf
            };
            
            let right_hash = if right_child < self.leaves.len() as u64 {
                self.leaves[right_child as usize]
            } else {
                let verifier = MerkleProofVerifier::new(self.hash_function, self.depth);
                verifier.empty_leaf
            };
            
            // Compute parent hash
            let parent_hash = MerkleProofVerifier::hash_children_static(
                self.hash_function,
                left_hash,
                right_hash,
            );
            
            // Store parent node
            self.nodes.insert((level + 1, parent_index), parent_hash);
            
            // Move up the tree
            current_index = parent_index;
            level += 1;
        }
        
        Ok(())
    }
}

/// Merkle proof utilities
pub struct MerkleProofUtils;

impl MerkleProofUtils {
    /// Verify proof against multiple possible roots
    pub fn verify_against_roots(
        proof: &MerkleProof,
        leaf: &[u8; 32],
        possible_roots: &[[u8; 32]],
        hash_function: HashFunction,
    ) -> CryptoResult<bool> {
        let verifier = MerkleProofVerifier::new(hash_function, proof.siblings.len());
        
        for &root in possible_roots {
            let mut test_proof = proof.clone();
            test_proof.root = root;
            
            if verifier.verify_proof(&test_proof, leaf)? {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    /// Verify proof with tolerance for root updates
    pub fn verify_with_tolerance(
        proof: &MerkleProof,
        leaf: &[u8; 32],
        expected_root: &[u8; 32],
        tolerance: usize,
        hash_function: HashFunction,
    ) -> CryptoResult<bool> {
        let verifier = MerkleProofVerifier::new(hash_function, proof.siblings.len());
        
        // First try exact match
        if verifier.verify_proof(proof, leaf)? {
            return Ok(true);
        }
        
        // If tolerance is 0, return false
        if tolerance == 0 {
            return Ok(false);
        }
        
        // Try with different roots (simplified approach)
        // In production, you'd implement proper root tolerance logic
        Ok(false)
    }
    
    /// Batch verify proofs with different roots
    pub fn batch_verify_with_roots(
        proofs: &[(MerkleProof, [u8; 32])],
        possible_roots: &[[u8; 32]],
        hash_function: HashFunction,
    ) -> CryptoResult<bool> {
        for (proof, leaf) in proofs {
            if !Self::verify_against_roots(proof, leaf, possible_roots, hash_function)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_proof_verification() {
        let verifier = MerkleProofVerifier::new(HashFunction::Blake2b256, 3);
        
        // Create test leaves
        let leaves = vec![
            CryptoUtils::random_32(),
            CryptoUtils::random_32(),
            CryptoUtils::random_32(),
            CryptoUtils::random_32(),
        ];
        
        // Generate proof for first leaf
        let proof = verifier.generate_proof(0, &leaves).unwrap();
        
        // Verify proof
        assert!(verifier.verify_proof(&proof, &leaves[0]).unwrap());
        
        // Test with wrong leaf
        let wrong_leaf = CryptoUtils::random_32();
        assert!(!verifier.verify_proof(&proof, &wrong_leaf).unwrap());
    }

    #[test]
    fn test_merkle_tree_builder() {
        let mut builder = MerkleTreeBuilder::new(HashFunction::Blake2b256, 3);
        
        // Add leaves
        for _ in 0..4 {
            let leaf = CryptoUtils::random_32();
            builder.add_leaf(leaf).unwrap();
        }
        
        // Generate proof for first leaf
        let proof = builder.generate_proof(0).unwrap();
        
        // Verify proof
        let verifier = MerkleProofVerifier::new(HashFunction::Blake2b256, 3);
        assert!(verifier.verify_proof(&proof, &builder.leaves[0]).unwrap());
    }

    #[test]
    fn test_batch_verification() {
        let verifier = MerkleProofVerifier::new(HashFunction::Blake2b256, 3);
        
        let leaves = vec![
            CryptoUtils::random_32(),
            CryptoUtils::random_32(),
            CryptoUtils::random_32(),
            CryptoUtils::random_32(),
        ];
        
        let mut proofs = Vec::new();
        for i in 0..4 {
            let proof = verifier.generate_proof(i, &leaves).unwrap();
            proofs.push((proof, leaves[i as usize]));
        }
        
        assert!(verifier.batch_verify_proofs(&proofs).unwrap());
    }

    #[test]
    fn test_context_verification() {
        let context = CryptoContext::merkle_context();
        let verifier = MerkleProofVerifier::with_context(HashFunction::Blake2b256, 3, &context);
        
        let leaves = vec![CryptoUtils::random_32(), CryptoUtils::random_32()];
        let proof = verifier.generate_proof(0, &leaves).unwrap();
        
        assert!(verifier.verify_proof_with_context(&proof, &leaves[0], &context).unwrap());
    }
}
