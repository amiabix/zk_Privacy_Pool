//! Poseidon Hash Implementation
//! 
//! This module provides Poseidon hash function implementation
//! optimized for zero-knowledge proof systems.

use ark_ff::{Field, Zero, UniformRand};
use ark_bn254::Fr;
use ark_ec::AffineRepr;
use ark_serialize::CanonicalSerialize;
use crate::crypto::{CryptoResult, CryptoError, CryptoContext};

/// Poseidon hash implementation
pub struct PoseidonHash {
    /// Poseidon parameters
    pub params: PoseidonParameters,
    /// Context for domain separation
    pub context: Option<CryptoContext>,
}

/// Poseidon parameters
#[derive(Debug, Clone)]
pub struct PoseidonParameters {
    /// Round constants
    pub round_constants: Vec<Vec<Fr>>,
    /// MDS matrix
    pub mds_matrix: Vec<Vec<Fr>>,
    /// Number of rounds
    pub num_rounds: usize,
    /// Partial rounds
    pub partial_rounds: usize,
}

impl PoseidonHash {
    /// Create new Poseidon hash instance
    pub fn new() -> Self {
        Self {
            params: Self::default_parameters(),
            context: None,
        }
    }
    
    /// Create Poseidon hash with context
    pub fn with_context(context: CryptoContext) -> Self {
        Self {
            params: Self::default_parameters(),
            context: Some(context),
        }
    }
    
    /// Hash input data
    pub fn hash(&self, input: &[u8]) -> CryptoResult<[u8; 32]> {
        // Convert input to field elements
        let field_elements = self.bytes_to_field_elements(input)?;
        
        // Apply Poseidon hash
        let result = self.poseidon_hash(&field_elements)?;
        
        // Convert result to bytes
        Ok(self.field_element_to_bytes(result))
    }
    
    /// Hash multiple inputs
    pub fn hash_multiple(&self, inputs: &[&[u8]]) -> CryptoResult<[u8; 32]> {
        let mut combined = Vec::new();
        for input in inputs {
            combined.extend_from_slice(input);
        }
        self.hash(&combined)
    }
    
    /// Hash with domain separation
    pub fn hash_with_domain(&self, input: &[u8], domain: &[u8]) -> CryptoResult<[u8; 32]> {
        let mut data = Vec::new();
        data.extend_from_slice(domain);
        data.extend_from_slice(input);
        self.hash(&data)
    }
    
    /// Hash UTXO commitment
    pub fn hash_utxo_commitment(
        &self,
        value: u64,
        owner: &[u8; 32],
        blinding_factor: &[u8; 32],
    ) -> CryptoResult<[u8; 32]> {
        let mut input = Vec::new();
        input.extend_from_slice(&value.to_le_bytes());
        input.extend_from_slice(owner);
        input.extend_from_slice(blinding_factor);
        
        if let Some(ref context) = self.context {
            self.hash_with_domain(&input, &context.domain)
        } else {
            self.hash(&input)
        }
    }
    
    /// Hash Merkle tree node
    pub fn hash_merkle_node(&self, left: &[u8; 32], right: &[u8; 32]) -> CryptoResult<[u8; 32]> {
        let mut input = Vec::new();
        input.extend_from_slice(left);
        input.extend_from_slice(right);
        
        if let Some(ref context) = self.context {
            self.hash_with_domain(&input, &context.domain)
        } else {
            self.hash(&input)
        }
    }
    
    /// Hash nullifier
    pub fn hash_nullifier(&self, utxo_commitment: &[u8; 32], utxo_index: u64) -> CryptoResult<[u8; 32]> {
        let mut input = Vec::new();
        input.extend_from_slice(utxo_commitment);
        input.extend_from_slice(&utxo_index.to_be_bytes());
        
        if let Some(ref context) = self.context {
            self.hash_with_domain(&input, &context.domain)
        } else {
            self.hash(&input)
        }
    }
    
    /// Convert bytes to field elements
    fn bytes_to_field_elements(&self, input: &[u8]) -> CryptoResult<Vec<Fr>> {
        let mut elements = Vec::new();
        let mut chunk = Vec::new();
        
        for &byte in input {
            chunk.push(byte);
            if chunk.len() == 31 { // 31 bytes per field element
                let field_element = self.bytes_to_field_element(&chunk)?;
                elements.push(field_element);
                chunk.clear();
            }
        }
        
        // Handle remaining bytes
        if !chunk.is_empty() {
            let field_element = self.bytes_to_field_element(&chunk)?;
            elements.push(field_element);
        }
        
        Ok(elements)
    }
    
    /// Convert bytes to single field element
    fn bytes_to_field_element(&self, bytes: &[u8]) -> CryptoResult<Fr> {
        if bytes.is_empty() {
            return Ok(Fr::zero());
        }
        
        // Pad to 32 bytes
        let mut padded = vec![0u8; 32];
        let copy_len = std::cmp::min(bytes.len(), 32);
        padded[32 - copy_len..].copy_from_slice(&bytes[..copy_len]);
        
        // Convert to field element
        Fr::from_random_bytes(&padded)
            .ok_or_else(|| CryptoError::HashError("Invalid field element".to_string()))
    }
    
    /// Convert field element to bytes
    fn field_element_to_bytes(&self, element: Fr) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        element.serialize_uncompressed(&mut bytes[..]).unwrap();
        bytes
    }
    
    /// Apply Poseidon hash function
    fn poseidon_hash(&self, input: &[Fr]) -> CryptoResult<Fr> {
        if input.is_empty() {
            return Ok(Fr::zero());
        }
        
        // Initialize state
        let mut state = input.to_vec();
        
        // Pad to required length
        while state.len() < 3 {
            state.push(Fr::zero());
        }
        
        // Apply rounds
        for round in 0..self.params.num_rounds {
            // Add round constants
            for i in 0..state.len() {
                if i < self.params.round_constants[round].len() {
                    state[i] += self.params.round_constants[round][i];
                }
            }
            
            // Apply S-box
            for i in 0..state.len() {
                state[i] = self.s_box(state[i]);
            }
            
            // Apply MDS matrix
            if round < self.params.num_rounds - 1 {
                state = self.apply_mds_matrix(&state);
            }
        }
        
        Ok(state[0])
    }
    
    /// S-box function
    fn s_box(&self, x: Fr) -> Fr {
        x * x * x // x^3
    }
    
    /// Apply MDS matrix
    fn apply_mds_matrix(&self, state: &[Fr]) -> Vec<Fr> {
        let mut result = vec![Fr::zero(); state.len()];
        
        for i in 0..state.len() {
            for j in 0..state.len() {
                if i < self.params.mds_matrix.len() && j < self.params.mds_matrix[i].len() {
                    result[i] += self.params.mds_matrix[i][j] * state[j];
                }
            }
        }
        
        result
    }
    
    /// Default Poseidon parameters
    fn default_parameters() -> PoseidonParameters {
        // Simplified parameters for BN254
        // In production, use proper Poseidon parameters
        let mut round_constants = Vec::new();
        let mut mds_matrix = Vec::new();
        
        // Generate random round constants
        for _ in 0..64 {
            let mut constants = Vec::new();
            for _ in 0..3 {
                constants.push(Fr::rand(&mut rand::thread_rng()));
            }
            round_constants.push(constants);
        }
        
        // Generate MDS matrix
        for _ in 0..3 {
            let mut row = Vec::new();
            for _ in 0..3 {
                row.push(Fr::rand(&mut rand::thread_rng()));
            }
            mds_matrix.push(row);
        }
        
        PoseidonParameters {
            round_constants,
            mds_matrix,
            num_rounds: 64,
            partial_rounds: 8,
        }
    }
}

/// Poseidon hash utilities
pub struct PoseidonUtils;

impl PoseidonUtils {
    /// Hash UTXO data
    pub fn hash_utxo(
        value: u64,
        owner: &[u8; 32],
        blinding_factor: &[u8; 32],
        context: &CryptoContext,
    ) -> CryptoResult<[u8; 32]> {
        let poseidon = PoseidonHash::with_context(context.clone());
        poseidon.hash_utxo_commitment(value, owner, blinding_factor)
    }
    
    /// Hash Merkle tree node
    pub fn hash_merkle_node(
        left: &[u8; 32],
        right: &[u8; 32],
        context: &CryptoContext,
    ) -> CryptoResult<[u8; 32]> {
        let poseidon = PoseidonHash::with_context(context.clone());
        poseidon.hash_merkle_node(left, right)
    }
    
    /// Hash nullifier
    pub fn hash_nullifier(
        utxo_commitment: &[u8; 32],
        utxo_index: u64,
        context: &CryptoContext,
    ) -> CryptoResult<[u8; 32]> {
        let poseidon = PoseidonHash::with_context(context.clone());
        poseidon.hash_nullifier(utxo_commitment, utxo_index)
    }
    
    /// Hash commitment
    pub fn hash_commitment(
        value: &[u8; 32],
        blinding_factor: &[u8; 32],
        context: &CryptoContext,
    ) -> CryptoResult<[u8; 32]> {
        let poseidon = PoseidonHash::with_context(context.clone());
        poseidon.hash_multiple(&[value, blinding_factor])
    }
    
    /// Hash multiple commitments
    pub fn hash_multiple_commitments(
        commitments: &[[u8; 32]],
        context: &CryptoContext,
    ) -> CryptoResult<[u8; 32]> {
        let poseidon = PoseidonHash::with_context(context.clone());
        
        let mut input = Vec::new();
        for commitment in commitments {
            input.extend_from_slice(commitment);
        }
        
        poseidon.hash(&input)
    }
}

/// Poseidon hash for specific use cases
pub struct PoseidonHasher;

impl PoseidonHasher {
    /// Hash for UTXO commitments
    pub fn utxo_commitment(
        value: u64,
        owner: &[u8; 32],
        blinding_factor: &[u8; 32],
    ) -> CryptoResult<[u8; 32]> {
        let context = CryptoContext::utxo_context();
        PoseidonUtils::hash_utxo(value, owner, blinding_factor, &context)
    }
    
    /// Hash for Merkle tree nodes
    pub fn merkle_node(left: &[u8; 32], right: &[u8; 32]) -> CryptoResult<[u8; 32]> {
        let context = CryptoContext::merkle_context();
        PoseidonUtils::hash_merkle_node(left, right, &context)
    }
    
    /// Hash for nullifiers
    pub fn nullifier(utxo_commitment: &[u8; 32], utxo_index: u64) -> CryptoResult<[u8; 32]> {
        let context = CryptoContext::nullifier_context();
        PoseidonUtils::hash_nullifier(utxo_commitment, utxo_index, &context)
    }
    
    /// Hash for commitments
    pub fn commitment(value: &[u8; 32], blinding_factor: &[u8; 32]) -> CryptoResult<[u8; 32]> {
        let context = CryptoContext::commitment_context();
        PoseidonUtils::hash_commitment(value, blinding_factor, &context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poseidon_hash() {
        let poseidon = PoseidonHash::new();
        let input = b"Hello, Poseidon!";
        
        let hash1 = poseidon.hash(input).unwrap();
        let hash2 = poseidon.hash(input).unwrap();
        
        // Should be deterministic
        assert_eq!(hash1, hash2);
        
        // Should be different for different inputs
        let different_input = b"Different input";
        let hash3 = poseidon.hash(different_input).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_poseidon_with_context() {
        let context = CryptoContext::utxo_context();
        let poseidon = PoseidonHash::with_context(context.clone());
        
        let input = b"Test input";
        let hash1 = poseidon.hash(input).unwrap();
        let hash2 = poseidon.hash_with_domain(input, &context.domain).unwrap();
        
        // Should be different due to domain separation
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_utxo_commitment() {
        let value = 1000000000000000000u64; // 1 ETH in wei
        let owner = CryptoUtils::random_32();
        let blinding_factor = CryptoUtils::random_32();
        
        let commitment = PoseidonHasher::utxo_commitment(value, &owner, &blinding_factor).unwrap();
        
        // Should be deterministic
        let commitment2 = PoseidonHasher::utxo_commitment(value, &owner, &blinding_factor).unwrap();
        assert_eq!(commitment, commitment2);
        
        // Should be different for different values
        let different_value = 2000000000000000000u64;
        let commitment3 = PoseidonHasher::utxo_commitment(different_value, &owner, &blinding_factor).unwrap();
        assert_ne!(commitment, commitment3);
    }

    #[test]
    fn test_merkle_node_hash() {
        let left = CryptoUtils::random_32();
        let right = CryptoUtils::random_32();
        
        let hash1 = PoseidonHasher::merkle_node(&left, &right).unwrap();
        let hash2 = PoseidonHasher::merkle_node(&left, &right).unwrap();
        
        // Should be deterministic
        assert_eq!(hash1, hash2);
        
        // Should be different for different inputs
        let different_right = CryptoUtils::random_32();
        let hash3 = PoseidonHasher::merkle_node(&left, &different_right).unwrap();
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_nullifier_hash() {
        let utxo_commitment = CryptoUtils::random_32();
        let utxo_index = 42;
        
        let nullifier1 = PoseidonHasher::nullifier(&utxo_commitment, utxo_index).unwrap();
        let nullifier2 = PoseidonHasher::nullifier(&utxo_commitment, utxo_index).unwrap();
        
        // Should be deterministic
        assert_eq!(nullifier1, nullifier2);
        
        // Should be different for different inputs
        let different_index = 43;
        let nullifier3 = PoseidonHasher::nullifier(&utxo_commitment, different_index).unwrap();
        assert_ne!(nullifier1, nullifier3);
    }

    #[test]
    fn test_commitment_hash() {
        let value = CryptoUtils::random_32();
        let blinding_factor = CryptoUtils::random_32();
        
        let commitment1 = PoseidonHasher::commitment(&value, &blinding_factor).unwrap();
        let commitment2 = PoseidonHasher::commitment(&value, &blinding_factor).unwrap();
        
        // Should be deterministic
        assert_eq!(commitment1, commitment2);
        
        // Should be different for different inputs
        let different_value = CryptoUtils::random_32();
        let commitment3 = PoseidonHasher::commitment(&different_value, &blinding_factor).unwrap();
        assert_ne!(commitment1, commitment3);
    }

    #[test]
    fn test_multiple_commitments() {
        let commitments = vec![
            CryptoUtils::random_32(),
            CryptoUtils::random_32(),
            CryptoUtils::random_32(),
        ];
        
        let context = CryptoContext::commitment_context();
        let hash1 = PoseidonUtils::hash_multiple_commitments(&commitments, &context).unwrap();
        let hash2 = PoseidonUtils::hash_multiple_commitments(&commitments, &context).unwrap();
        
        // Should be deterministic
        assert_eq!(hash1, hash2);
    }
}
