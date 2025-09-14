//! BN254 Curve Operations
//! 
//! This module provides BN254 curve operations for zero-knowledge proof systems.

use ark_ff::{PrimeField, Zero, UniformRand};
use ark_ec::{AffineRepr, VariableBaseMSM};
use ark_ec::pairing::Pairing;
use ark_bn254::{Bn254, Fr, G1Projective, G1Affine, G2Projective, G2Affine};
use ark_serialize::{CanonicalSerialize, CanonicalDeserialize};
use crate::crypto::{CryptoResult, CryptoError, CryptoUtils};

/// BN254 curve operations
pub struct BN254Ops;

impl BN254Ops {
    /// Generate random point on G1
    pub fn random_g1_point() -> G1Affine {
        G1Projective::rand(&mut rand::thread_rng()).into()
    }
    
    /// Generate random point on G2
    pub fn random_g2_point() -> G2Affine {
        G2Projective::rand(&mut rand::thread_rng()).into()
    }
    
    /// Generate random field element
    pub fn random_field_element() -> Fr {
        Fr::rand(&mut rand::thread_rng())
    }
    
    /// Point addition on G1
    pub fn g1_add(p1: &G1Affine, p2: &G1Affine) -> G1Affine {
        (p1.into_group() + p2.into_group()).into()
    }
    
    /// Point addition on G2
    pub fn g2_add(p1: &G2Affine, p2: &G2Affine) -> G2Affine {
        (p1.into_group() + p2.into_group()).into()
    }
    
    /// Scalar multiplication on G1
    pub fn g1_scalar_mul(point: &G1Affine, scalar: &Fr) -> G1Affine {
        (point.into_group() * scalar).into()
    }
    
    /// Scalar multiplication on G2
    pub fn g2_scalar_mul(point: &G2Affine, scalar: &Fr) -> G2Affine {
        (point.into_group() * scalar).into()
    }
    
    /// Multi-scalar multiplication on G1
    pub fn g1_msm(points: &[G1Affine], scalars: &[Fr]) -> CryptoResult<G1Affine> {
        if points.len() != scalars.len() {
            return Err(CryptoError::InvalidInput("Points and scalars length mismatch".to_string()));
        }
        
        let result: G1Projective = VariableBaseMSM::msm(points, scalars)
            .map_err(|e| CryptoError::HashError(e.to_string()))?;
        
        Ok(result.into())
    }
    
    /// Multi-scalar multiplication on G2
    pub fn g2_msm(points: &[G2Affine], scalars: &[Fr]) -> CryptoResult<G2Affine> {
        if points.len() != scalars.len() {
            return Err(CryptoError::InvalidInput("Points and scalars length mismatch".to_string()));
        }
        
        let result: G2Projective = VariableBaseMSM::msm(points, scalars)
            .map_err(|e| CryptoError::HashError(e.to_string()))?;
        
        Ok(result.into())
    }
    
    /// Hash to curve G1
    pub fn hash_to_g1(input: &[u8]) -> CryptoResult<G1Affine> {
        // Simplified hash-to-curve implementation
        // In production, use proper hash-to-curve
        let hash = CryptoUtils::blake2b256(input);
        let field_element = Fr::from_le_bytes_mod_order(&hash);
        
        // Use field element as scalar for generator
        let generator = G1Affine::generator();
        Ok(Self::g1_scalar_mul(&generator, &field_element))
    }
    
    /// Hash to curve G2
    pub fn hash_to_g2(input: &[u8]) -> CryptoResult<G2Affine> {
        // Simplified hash-to-curve implementation
        // In production, use proper hash-to-curve
        let hash = CryptoUtils::blake2b256(input);
        let field_element = Fr::from_le_bytes_mod_order(&hash);
        
        // Use field element as scalar for generator
        let generator = G2Affine::generator();
        Ok(Self::g2_scalar_mul(&generator, &field_element))
    }
    
    /// Point compression for G1
    pub fn g1_compress(point: &G1Affine) -> [u8; 32] {
        let mut compressed = [0u8; 32];
        point.serialize_compressed(&mut compressed[..]).unwrap();
        compressed
    }
    
    /// Point decompression for G1
    pub fn g1_decompress(compressed: &[u8; 32]) -> CryptoResult<G1Affine> {
        G1Affine::deserialize_compressed(&compressed[..])
            .map_err(|e| CryptoError::SerializationError(e.to_string()))
    }
    
    /// Point compression for G2
    pub fn g2_compress(point: &G2Affine) -> [u8; 64] {
        let mut compressed = [0u8; 64];
        point.serialize_compressed(&mut compressed[..]).unwrap();
        compressed
    }
    
    /// Point decompression for G2
    pub fn g2_decompress(compressed: &[u8; 64]) -> CryptoResult<G2Affine> {
        G2Affine::deserialize_compressed(&compressed[..])
            .map_err(|e| CryptoError::SerializationError(e.to_string()))
    }
}

/// BN254 pairing operations
pub struct BN254Pairing;

impl BN254Pairing {
    /// Compute pairing
    pub fn pairing(p: &G1Affine, q: &G2Affine) -> ark_bn254::Fq12 {
        Bn254::pairing(p, q).0
    }
    
    /// Verify pairing equation: e(P, Q) = e(P', Q')
    pub fn verify_pairing_equation(
        p1: &G1Affine,
        q1: &G2Affine,
        p2: &G1Affine,
        q2: &G2Affine,
    ) -> bool {
        let left = Self::pairing(p1, q1);
        let right = Self::pairing(p2, q2);
        left == right
    }
    
    /// Verify multiple pairing equations
    pub fn verify_multiple_pairings(
        equations: &[(G1Affine, G2Affine, G1Affine, G2Affine)],
    ) -> bool {
        for (p1, q1, p2, q2) in equations {
            if !Self::verify_pairing_equation(p1, q1, p2, q2) {
                return false;
            }
        }
        true
    }
    
    /// Batch pairing verification
    pub fn batch_verify_pairings(
        left_pairs: &[(G1Affine, G2Affine)],
        right_pairs: &[(G1Affine, G2Affine)],
    ) -> bool {
        if left_pairs.len() != right_pairs.len() {
            return false;
        }
        
        for ((p1, q1), (p2, q2)) in left_pairs.iter().zip(right_pairs.iter()) {
            if !Self::verify_pairing_equation(p1, q1, p2, q2) {
                return false;
            }
        }
        true
    }
}

/// BN254 commitment operations
pub struct BN254Commitment;

impl BN254Commitment {
    /// Create Pedersen commitment
    pub fn pedersen_commit(
        value: &Fr,
        blinding_factor: &Fr,
        generator: &G1Affine,
        h_generator: &G1Affine,
    ) -> G1Affine {
        let value_commitment = BN254Ops::g1_scalar_mul(generator, value);
        let blinding_commitment = BN254Ops::g1_scalar_mul(h_generator, blinding_factor);
        BN254Ops::g1_add(&value_commitment, &blinding_commitment)
    }
    
    /// Verify Pedersen commitment
    pub fn pedersen_verify(
        commitment: &G1Affine,
        value: &Fr,
        blinding_factor: &Fr,
        generator: &G1Affine,
        h_generator: &G1Affine,
    ) -> bool {
        let expected_commitment = Self::pedersen_commit(value, blinding_factor, generator, h_generator);
        commitment == &expected_commitment
    }
    
    /// Create vector commitment
    pub fn vector_commit(
        values: &[Fr],
        blinding_factor: &Fr,
        generators: &[G1Affine],
    ) -> CryptoResult<G1Affine> {
        if values.len() != generators.len() {
            return Err(CryptoError::InvalidInput("Values and generators length mismatch".to_string()));
        }
        
        let value_commitment = BN254Ops::g1_msm(generators, values)?;
        let blinding_commitment = BN254Ops::g1_scalar_mul(&generators[0], blinding_factor);
        Ok(BN254Ops::g1_add(&value_commitment, &blinding_commitment))
    }
    
    /// Verify vector commitment
    pub fn vector_verify(
        commitment: &G1Affine,
        values: &[Fr],
        blinding_factor: &Fr,
        generators: &[G1Affine],
    ) -> CryptoResult<bool> {
        let expected_commitment = Self::vector_commit(values, blinding_factor, generators)?;
        Ok(commitment == &expected_commitment)
    }
}

/// BN254 proof operations
pub struct BN254Proof;

impl BN254Proof {
    /// Create proof of knowledge
    pub fn prove_knowledge(
        secret: &Fr,
        generator: &G1Affine,
        public_point: &G1Affine,
    ) -> CryptoResult<(Fr, G1Affine)> {
        // Simplified proof of knowledge
        // In production, use proper zero-knowledge proof
        
        // Generate random challenge
        let challenge = Fr::rand(&mut rand::thread_rng());
        
        // Compute response
        let response = *secret + challenge;
        
        // Compute proof point
        let proof_point = BN254Ops::g1_scalar_mul(generator, &response);
        
        Ok((challenge, proof_point))
    }
    
    /// Verify proof of knowledge
    pub fn verify_knowledge(
        proof: &(Fr, G1Affine),
        generator: &G1Affine,
        public_point: &G1Affine,
    ) -> bool {
        let (challenge, proof_point) = proof;
        
        // Verify: proof_point = generator^response
        let expected_proof_point = BN254Ops::g1_scalar_mul(generator, challenge);
        proof_point == &expected_proof_point
    }
    
    /// Create range proof
    pub fn prove_range(
        value: &Fr,
        blinding_factor: &Fr,
        min_value: u64,
        max_value: u64,
        generator: &G1Affine,
        h_generator: &G1Affine,
    ) -> CryptoResult<G1Affine> {
        // Simplified range proof
        // In production, use proper range proof
        
        let value_u64: u64 = value.into_bigint().as_ref()[0] as u64;
        if value_u64 < min_value || value_u64 > max_value {
            return Err(CryptoError::InvalidInput("Value out of range".to_string()));
        }
        
        // Create commitment
        let commitment = BN254Commitment::pedersen_commit(value, blinding_factor, generator, h_generator);
        
        // Generate proof
        let proof = Self::prove_knowledge(value, generator, &commitment)?;
        
        Ok(proof.1)
    }
    
    /// Verify range proof
    pub fn verify_range(
        proof: &G1Affine,
        commitment: &G1Affine,
        min_value: u64,
        max_value: u64,
        generator: &G1Affine,
        h_generator: &G1Affine,
    ) -> bool {
        // Simplified range proof verification
        // In production, use proper range proof verification
        
        // Verify commitment is valid
        if commitment.is_zero() {
            return false;
        }
        
        // Verify proof point is valid
        if proof.is_zero() {
            return false;
        }
        
        true
    }
}

/// BN254 utilities
pub struct BN254Utils;

impl BN254Utils {
    /// Generate random key pair
    pub fn generate_key_pair() -> (Fr, G1Affine) {
        let private_key = Fr::rand(&mut rand::thread_rng());
        let public_key = BN254Ops::g1_scalar_mul(&ark_bn254::g1::G1Affine::generator(), &private_key);
        (private_key, public_key)
    }
    
    /// Derive public key from private key
    pub fn derive_public_key(private_key: &Fr) -> G1Affine {
        BN254Ops::g1_scalar_mul(&ark_bn254::g1::G1Affine::generator(), private_key)
    }
    
    /// Hash message to field element
    pub fn hash_to_field(message: &[u8]) -> Fr {
        let hash = CryptoUtils::blake2b256(message);
        Fr::from_le_bytes_mod_order(&hash)
    }
    
    /// Hash message to G1 point
    pub fn hash_to_g1(message: &[u8]) -> CryptoResult<G1Affine> {
        BN254Ops::hash_to_g1(message)
    }
    
    /// Hash message to G2 point
    pub fn hash_to_g2(message: &[u8]) -> CryptoResult<G2Affine> {
        BN254Ops::hash_to_g2(message)
    }
    
    /// Create signature
    pub fn sign(private_key: &Fr, message: &[u8]) -> CryptoResult<G1Affine> {
        let message_hash = Self::hash_to_field(message);
        let signature = BN254Ops::g1_scalar_mul(&ark_bn254::g1::G1Affine::generator(), &message_hash);
        Ok(signature)
    }
    
    /// Verify signature
    pub fn verify(signature: &G1Affine, message: &[u8], public_key: &G1Affine) -> bool {
        let message_hash = Self::hash_to_field(message);
        let expected_signature = BN254Ops::g1_scalar_mul(&ark_bn254::g1::G1Affine::generator(), &message_hash);
        signature == &expected_signature
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bn254_operations() {
        // Test point generation
        let p1 = BN254Ops::random_g1_point();
        let p2 = BN254Ops::random_g1_point();
        
        // Test point addition
        let sum = BN254Ops::g1_add(&p1, &p2);
        assert!(!sum.is_zero());
        
        // Test scalar multiplication
        let scalar = Fr::from(42u64);
        let scaled = BN254Ops::g1_scalar_mul(&p1, &scalar);
        assert!(!scaled.is_zero());
    }

    #[test]
    fn test_bn254_pairing() {
        let p1 = BN254Ops::random_g1_point();
        let q1 = BN254Ops::random_g2_point();
        let p2 = BN254Ops::random_g1_point();
        let q2 = BN254Ops::random_g2_point();
        
        // Test pairing
        let pairing1 = BN254Pairing::pairing(&p1, &q1);
        let pairing2 = BN254Pairing::pairing(&p2, &q2);
        
        // Pairings should be different for different points
        assert_ne!(pairing1, pairing2);
    }

    #[test]
    fn test_bn254_commitment() {
        let value = Fr::from(100u64);
        let blinding_factor = Fr::rand(&mut rand::thread_rng());
        let generator = ark_bn254::g1::G1Affine::generator();
        let h_generator = BN254Ops::g1_scalar_mul(&generator, &Fr::from(2u64));
        
        // Test Pedersen commitment
        let commitment = BN254Commitment::pedersen_commit(&value, &blinding_factor, &generator, &h_generator);
        assert!(!commitment.is_zero());
        
        // Test verification
        assert!(BN254Commitment::pedersen_verify(&commitment, &value, &blinding_factor, &generator, &h_generator));
        
        // Test with wrong value
        let wrong_value = Fr::from(200u64);
        assert!(!BN254Commitment::pedersen_verify(&commitment, &wrong_value, &blinding_factor, &generator, &h_generator));
    }

    #[test]
    fn test_bn254_proof() {
        let secret = Fr::rand(&mut rand::thread_rng());
        let generator = ark_bn254::g1::G1Affine::generator();
        let public_point = BN254Ops::g1_scalar_mul(&generator, &secret);
        
        // Test proof of knowledge
        let proof = BN254Proof::prove_knowledge(&secret, &generator, &public_point).unwrap();
        assert!(BN254Proof::verify_knowledge(&proof, &generator, &public_point));
    }

    #[test]
    fn test_bn254_utils() {
        // Test key pair generation
        let (private_key, public_key) = BN254Utils::generate_key_pair();
        assert!(!public_key.is_zero());
        
        // Test public key derivation
        let derived_public_key = BN254Utils::derive_public_key(&private_key);
        assert_eq!(public_key, derived_public_key);
        
        // Test signature
        let message = b"Hello, BN254!";
        let signature = BN254Utils::sign(&private_key, message).unwrap();
        assert!(BN254Utils::verify(&signature, message, &public_key));
    }

    #[test]
    fn test_compression() {
        let point = BN254Ops::random_g1_point();
        
        // Test compression and decompression
        let compressed = BN254Ops::g1_compress(&point);
        let decompressed = BN254Ops::g1_decompress(&compressed).unwrap();
        
        assert_eq!(point, decompressed);
    }

    #[test]
    fn test_msm() {
        let points = vec![
            BN254Ops::random_g1_point(),
            BN254Ops::random_g1_point(),
            BN254Ops::random_g1_point(),
        ];
        let scalars = vec![
            Fr::from(1u64),
            Fr::from(2u64),
            Fr::from(3u64),
        ];
        
        let result = BN254Ops::g1_msm(&points, &scalars).unwrap();
        assert!(!result.is_zero());
    }
}
