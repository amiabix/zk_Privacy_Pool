//! Commitment Schemes Implementation
//! 
//! This module provides Pedersen and Poseidon commitment schemes
//! for privacy-preserving cryptographic operations.

use ark_ff::PrimeField;
use ark_ec::AffineRepr;
use ark_bn254::{Fr, G1Affine};
use ark_std::UniformRand;
use crate::crypto::{CryptoResult, CryptoError, CryptoContext, CryptoUtils};

/// Pedersen commitment implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PedersenCommitment {
    /// The commitment value
    pub commitment: G1Affine,
    /// The blinding factor used
    pub blinding_factor: Fr,
}

/// Poseidon commitment implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoseidonCommitment {
    /// The commitment hash
    pub commitment: [u8; 32],
    /// The blinding factor used
    pub blinding_factor: [u8; 32],
}

/// Commitment scheme trait
pub trait CommitmentScheme {
    type Commitment;
    type BlindingFactor;
    type Value;
    
    /// Create a commitment
    fn commit(value: &Self::Value, blinding_factor: &Self::BlindingFactor) -> CryptoResult<Self::Commitment>;
    
    /// Verify a commitment
    fn verify(commitment: &Self::Commitment, value: &Self::Value, blinding_factor: &Self::BlindingFactor) -> CryptoResult<bool>;
    
    /// Create a random blinding factor
    fn random_blinding_factor() -> Self::BlindingFactor;
}

/// Pedersen commitment scheme implementation
pub struct PedersenCommitmentScheme {
    /// Generator point for values
    pub g: G1Affine,
    /// Generator point for blinding factors
    pub h: G1Affine,
}

impl PedersenCommitmentScheme {
    /// Create new Pedersen commitment scheme
    pub fn new() -> Self {
        // Use standard generators for BN254
        let g = ark_bn254::g1::G1Affine::generator();
        let h = ark_bn254::g1::G1Affine::generator() * Fr::from(2u64);
        
        Self { g, h: h.into() }
    }
    
    /// Create commitment with context
    pub fn commit_with_context(
        &self,
        value: &Fr,
        blinding_factor: &Fr,
        context: &CryptoContext,
    ) -> CryptoResult<PedersenCommitment> {
        // Add context to the commitment
        let context_field = Fr::from_le_bytes_mod_order(&context.domain);
        let value_with_context = *value + context_field;
        
        // Compute commitment: C = g^value * h^blinding_factor
        let commitment = self.g * value_with_context + self.h * blinding_factor;
        
        Ok(PedersenCommitment {
            commitment: commitment.into(),
            blinding_factor: *blinding_factor,
        })
    }
    
    /// Verify commitment with context
    pub fn verify_with_context(
        &self,
        commitment: &PedersenCommitment,
        value: &Fr,
        blinding_factor: &Fr,
        context: &CryptoContext,
    ) -> CryptoResult<bool> {
        // Add context to the value
        let context_field = Fr::from_le_bytes_mod_order(&context.domain);
        let value_with_context = *value + context_field;
        
        // Recompute commitment
        let expected_commitment = self.g * value_with_context + self.h * blinding_factor;
        
        Ok(commitment.commitment == expected_commitment)
    }
    
    /// Batch verify multiple commitments
    pub fn batch_verify(
        &self,
        commitments: &[(PedersenCommitment, Fr, Fr)],
        context: &CryptoContext,
    ) -> CryptoResult<bool> {
        for (commitment, value, blinding_factor) in commitments {
            if !self.verify_with_context(commitment, value, blinding_factor, context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl CommitmentScheme for PedersenCommitmentScheme {
    type Commitment = PedersenCommitment;
    type BlindingFactor = Fr;
    type Value = Fr;
    
    fn commit(value: &Self::Value, blinding_factor: &Self::BlindingFactor) -> CryptoResult<Self::Commitment> {
        let scheme = Self::new();
        let context = CryptoContext::commitment_context();
        scheme.commit_with_context(value, blinding_factor, &context)
    }
    
    fn verify(commitment: &Self::Commitment, value: &Self::Value, blinding_factor: &Self::BlindingFactor) -> CryptoResult<bool> {
        let scheme = Self::new();
        let context = CryptoContext::commitment_context();
        scheme.verify_with_context(commitment, value, blinding_factor, &context)
    }
    
    fn random_blinding_factor() -> Self::BlindingFactor {
        Fr::rand(&mut rand::thread_rng())
    }
}

/// Poseidon commitment scheme implementation
pub struct PoseidonCommitmentScheme {
    /// Poseidon hash instance
    pub hasher: crate::crypto::poseidon::PoseidonHash,
}

impl PoseidonCommitmentScheme {
    /// Create new Poseidon commitment scheme
    pub fn new() -> Self {
        Self {
            hasher: crate::crypto::poseidon::PoseidonHash::new(),
        }
    }
    
    /// Create commitment with context
    pub fn commit_with_context(
        &self,
        value: &[u8; 32],
        blinding_factor: &[u8; 32],
        context: &CryptoContext,
    ) -> CryptoResult<PoseidonCommitment> {
        // Combine value, blinding factor, and context
        let mut input = Vec::new();
        input.extend_from_slice(&context.domain);
        input.extend_from_slice(&context.salt);
        input.extend_from_slice(value);
        input.extend_from_slice(blinding_factor);
        
        // Hash using Poseidon
        let hash = self.hasher.hash(&input)?;
        
        Ok(PoseidonCommitment {
            commitment: hash,
            blinding_factor: *blinding_factor,
        })
    }
    
    /// Verify commitment with context
    pub fn verify_with_context(
        &self,
        commitment: &PoseidonCommitment,
        value: &[u8; 32],
        blinding_factor: &[u8; 32],
        context: &CryptoContext,
    ) -> CryptoResult<bool> {
        // Recompute commitment
        let expected_commitment = self.commit_with_context(value, blinding_factor, context)?;
        Ok(commitment.commitment == expected_commitment.commitment)
    }
    
    /// Batch verify multiple commitments
    pub fn batch_verify(
        &self,
        commitments: &[(PoseidonCommitment, [u8; 32], [u8; 32])],
        context: &CryptoContext,
    ) -> CryptoResult<bool> {
        for (commitment, value, blinding_factor) in commitments {
            if !self.verify_with_context(commitment, value, blinding_factor, context)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

impl CommitmentScheme for PoseidonCommitmentScheme {
    type Commitment = PoseidonCommitment;
    type BlindingFactor = [u8; 32];
    type Value = [u8; 32];
    
    fn commit(value: &Self::Value, blinding_factor: &Self::BlindingFactor) -> CryptoResult<Self::Commitment> {
        let scheme = Self::new();
        let context = CryptoContext::commitment_context();
        scheme.commit_with_context(value, blinding_factor, &context)
    }
    
    fn verify(commitment: &Self::Commitment, value: &Self::Value, blinding_factor: &Self::BlindingFactor) -> CryptoResult<bool> {
        let scheme = Self::new();
        let context = CryptoContext::commitment_context();
        scheme.verify_with_context(commitment, value, blinding_factor, &context)
    }
    
    fn random_blinding_factor() -> Self::BlindingFactor {
        CryptoUtils::random_32()
    }
}

/// UTXO commitment implementation
pub struct UTXOCommitment;

impl UTXOCommitment {
    /// Create commitment for UTXO value
    pub fn commit_value(value: u64, blinding_factor: &[u8; 32]) -> CryptoResult<PoseidonCommitment> {
        let value_bytes = value.to_le_bytes();
        let mut value_32 = [0u8; 32];
        value_32[0..8].copy_from_slice(&value_bytes);
        
        let scheme = PoseidonCommitmentScheme::new();
        let context = CryptoContext::utxo_context();
        scheme.commit_with_context(&value_32, blinding_factor, &context)
    }
    
    /// Create commitment for UTXO owner
    pub fn commit_owner(owner: &[u8; 32], blinding_factor: &[u8; 32]) -> CryptoResult<PoseidonCommitment> {
        let scheme = PoseidonCommitmentScheme::new();
        let context = CryptoContext::utxo_context();
        scheme.commit_with_context(owner, blinding_factor, &context)
    }
    
    /// Create combined commitment for UTXO
    pub fn commit_utxo(
        value: u64,
        owner: &[u8; 32],
        value_blinding: &[u8; 32],
        owner_blinding: &[u8; 32],
    ) -> CryptoResult<PoseidonCommitment> {
        // Combine value and owner commitments
        let value_commitment = Self::commit_value(value, value_blinding)?;
        let owner_commitment = Self::commit_owner(owner, owner_blinding)?;
        
        // Create combined commitment
        let mut combined_input = Vec::new();
        combined_input.extend_from_slice(&value_commitment.commitment);
        combined_input.extend_from_slice(&owner_commitment.commitment);
        combined_input.extend_from_slice(value_blinding);
        combined_input.extend_from_slice(owner_blinding);
        
        let hasher = crate::crypto::poseidon::PoseidonHash::new();
        let hash = hasher.hash(&combined_input)?;
        
        Ok(PoseidonCommitment {
            commitment: hash,
            blinding_factor: *value_blinding, // Use value blinding as primary
        })
    }
    
    /// Verify UTXO commitment
    pub fn verify_utxo(
        commitment: &PoseidonCommitment,
        value: u64,
        owner: &[u8; 32],
        value_blinding: &[u8; 32],
        owner_blinding: &[u8; 32],
    ) -> CryptoResult<bool> {
        let expected_commitment = Self::commit_utxo(value, owner, value_blinding, owner_blinding)?;
        Ok(commitment.commitment == expected_commitment.commitment)
    }
}

/// Range proof commitment for value ranges
pub struct RangeProofCommitment;

impl RangeProofCommitment {
    /// Create commitment for value in range [0, 2^n)
    pub fn commit_range(
        value: u64,
        blinding_factor: &[u8; 32],
        bit_length: usize,
    ) -> CryptoResult<Vec<PoseidonCommitment>> {
        if bit_length > 64 {
            return Err(CryptoError::InvalidInput("Bit length too large".to_string()));
        }
        
        let mut commitments = Vec::new();
        let mut remaining_value = value;
        
        for i in 0..bit_length {
            let bit = (remaining_value >> i) & 1;
            let bit_bytes = [bit as u8; 32];
            
            let commitment = PoseidonCommitmentScheme::commit(&bit_bytes, blinding_factor)?;
            commitments.push(commitment);
            
            remaining_value >>= 1;
        }
        
        Ok(commitments)
    }
    
    /// Verify range proof commitment
    pub fn verify_range(
        commitments: &[PoseidonCommitment],
        value: u64,
        blinding_factor: &[u8; 32],
        bit_length: usize,
    ) -> CryptoResult<bool> {
        if commitments.len() != bit_length {
            return Ok(false);
        }
        
        let mut remaining_value = value;
        
        for (i, commitment) in commitments.iter().enumerate() {
            let bit = (remaining_value >> i) & 1;
            let bit_bytes = [bit as u8; 32];
            
            if !PoseidonCommitmentScheme::verify(commitment, &bit_bytes, blinding_factor)? {
                return Ok(false);
            }
            
            remaining_value >>= 1;
        }
        
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pedersen_commitment() {
        let value = Fr::from(42u64);
        let blinding_factor = Fr::rand(&mut rand::thread_rng());
        
        let commitment = PedersenCommitmentScheme::commit(&value, &blinding_factor).unwrap();
        assert!(PedersenCommitmentScheme::verify(&commitment, &value, &blinding_factor).unwrap());
        
        // Test with wrong value
        let wrong_value = Fr::from(43u64);
        assert!(!PedersenCommitmentScheme::verify(&commitment, &wrong_value, &blinding_factor).unwrap());
    }

    #[test]
    fn test_poseidon_commitment() {
        let value = CryptoUtils::random_32();
        let blinding_factor = CryptoUtils::random_32();
        
        let commitment = PoseidonCommitmentScheme::commit(&value, &blinding_factor).unwrap();
        assert!(PoseidonCommitmentScheme::verify(&commitment, &value, &blinding_factor).unwrap());
        
        // Test with wrong value
        let wrong_value = CryptoUtils::random_32();
        assert!(!PoseidonCommitmentScheme::verify(&commitment, &wrong_value, &blinding_factor).unwrap());
    }

    #[test]
    fn test_utxo_commitment() {
        let value = 1000000000000000000u64; // 1 ETH in wei
        let owner = CryptoUtils::random_32();
        let value_blinding = CryptoUtils::random_32();
        let owner_blinding = CryptoUtils::random_32();
        
        let commitment = UTXOCommitment::commit_utxo(value, &owner, &value_blinding, &owner_blinding).unwrap();
        assert!(UTXOCommitment::verify_utxo(&commitment, value, &owner, &value_blinding, &owner_blinding).unwrap());
        
        // Test with wrong value
        let wrong_value = 2000000000000000000u64;
        assert!(!UTXOCommitment::verify_utxo(&commitment, wrong_value, &owner, &value_blinding, &owner_blinding).unwrap());
    }

    #[test]
    fn test_range_proof_commitment() {
        let value = 42u64;
        let blinding_factor = CryptoUtils::random_32();
        let bit_length = 8;
        
        let commitments = RangeProofCommitment::commit_range(value, &blinding_factor, bit_length).unwrap();
        assert_eq!(commitments.len(), bit_length);
        
        assert!(RangeProofCommitment::verify_range(&commitments, value, &blinding_factor, bit_length).unwrap());
        
        // Test with wrong value
        let wrong_value = 43u64;
        assert!(!RangeProofCommitment::verify_range(&commitments, wrong_value, &blinding_factor, bit_length).unwrap());
    }

    #[test]
    fn test_batch_verification() {
        let context = CryptoContext::commitment_context();
        let scheme = PoseidonCommitmentScheme::new();
        
        let mut commitments = Vec::new();
        for _ in 0..5 {
            let value = CryptoUtils::random_32();
            let blinding_factor = CryptoUtils::random_32();
            let commitment = scheme.commit_with_context(&value, &blinding_factor, &context).unwrap();
            commitments.push((commitment, value, blinding_factor));
        }
        
        assert!(scheme.batch_verify(&commitments, &context).unwrap());
    }
}
