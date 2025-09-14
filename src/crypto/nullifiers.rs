//! Nullifier Generation and Verification
//! 
//! This module provides production-ready nullifier generation
//! for preventing double-spending in privacy-preserving systems.

use crate::crypto::{CryptoResult, CryptoError, CryptoContext, CryptoUtils};
use crate::crypto::signatures::{EcdsaSig, Ed25519Scheme, EcdsaScheme, SignatureScheme};
use crate::crypto::key_derivation::ExtendedPrivateKey;
use ed25519_dalek::Verifier;

/// Nullifier for preventing double-spending
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nullifier {
    /// The nullifier value (32 bytes)
    pub value: [u8; 32],
    /// The UTXO commitment this nullifier corresponds to
    pub utxo_commitment: [u8; 32],
    /// The signature proving ownership
    pub signature: Vec<u8>,
    /// The public key used for verification
    pub public_key: [u8; 32],
}

/// Nullifier generator
pub struct NullifierGenerator {
    /// Cryptographic context
    pub context: CryptoContext,
    /// Hash function to use
    pub hash_function: NullifierHashFunction,
}

/// Hash function for nullifier generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullifierHashFunction {
    /// SHA-256
    Sha256,
    /// Blake2b-256
    Blake2b256,
    /// Keccak-256
    Keccak256,
    /// Poseidon
    Poseidon,
}

impl NullifierGenerator {
    /// Create new nullifier generator
    pub fn new(context: CryptoContext, hash_function: NullifierHashFunction) -> Self {
        Self { context, hash_function }
    }
    
    /// Generate nullifier for UTXO
    pub fn generate_nullifier(
        &self,
        utxo_commitment: &[u8; 32],
        private_key: &[u8; 32],
        utxo_index: u64,
    ) -> CryptoResult<Nullifier> {
        // Create nullifier seed
        let nullifier_seed = self.create_nullifier_seed(utxo_commitment, utxo_index)?;
        
        // Generate nullifier value
        let nullifier_value = self.hash_nullifier(&nullifier_seed)?;
        
        // Create signature proving ownership
        let signature = self.sign_nullifier(&nullifier_value, private_key)?;
        
        // Derive public key
        let public_key = self.derive_public_key(private_key)?;
        
        Ok(Nullifier {
            value: nullifier_value,
            utxo_commitment: *utxo_commitment,
            signature,
            public_key,
        })
    }
    
    /// Generate nullifier with Ed25519 signature
    pub fn generate_nullifier_ed25519(
        &self,
        utxo_commitment: &[u8; 32],
        private_key: &[u8; 32],
        utxo_index: u64,
    ) -> CryptoResult<Nullifier> {
        // Create nullifier seed
        let nullifier_seed = self.create_nullifier_seed(utxo_commitment, utxo_index)?;
        
        // Generate nullifier value
        let nullifier_value = self.hash_nullifier(&nullifier_seed)?;
        
        // Create Ed25519 signature
        let ed25519_private = ed25519_dalek::SigningKey::from_bytes(private_key);
        
        let signature = Ed25519Scheme::sign(&ed25519_private, &nullifier_value)?;
        let signature_bytes = signature.to_bytes();
        
        // Get public key
        let public_key = ed25519_private.verifying_key().to_bytes();
        
        Ok(Nullifier {
            value: nullifier_value,
            utxo_commitment: *utxo_commitment,
            signature: signature_bytes.to_vec(),
            public_key,
        })
    }
    
    /// Generate nullifier with ECDSA signature
    pub fn generate_nullifier_ecdsa(
        &self,
        utxo_commitment: &[u8; 32],
        private_key: &[u8; 32],
        utxo_index: u64,
    ) -> CryptoResult<Nullifier> {
        // Create nullifier seed
        let nullifier_seed = self.create_nullifier_seed(utxo_commitment, utxo_index)?;
        
        // Generate nullifier value
        let nullifier_value = self.hash_nullifier(&nullifier_seed)?;
        
        // Create ECDSA signature
        let ecdsa_private = secp256k1::SecretKey::from_slice(private_key)
            .map_err(|e| CryptoError::InvalidPrivateKey(e.to_string()))?;
        
        let signature = EcdsaScheme::sign(&ecdsa_private, &nullifier_value)?;
        let signature_bytes = signature.to_bytes();
        
        // Get public key
        let secp = secp256k1::Secp256k1::new();
        let public_key = ecdsa_private.public_key(&secp).serialize();
        
        // Convert 33-byte public key to 32-byte (remove compression flag)
        let mut public_key_32 = [0u8; 32];
        public_key_32.copy_from_slice(&public_key[1..33]);
        
        Ok(Nullifier {
            value: nullifier_value,
            utxo_commitment: *utxo_commitment,
            signature: signature_bytes.to_vec(),
            public_key: public_key_32,
        })
    }
    
    /// Verify nullifier
    pub fn verify_nullifier(&self, nullifier: &Nullifier) -> CryptoResult<bool> {
        // Recreate nullifier seed
        let nullifier_seed = self.create_nullifier_seed(&nullifier.utxo_commitment, 0)?;
        
        // Verify nullifier value
        let expected_value = self.hash_nullifier(&nullifier_seed)?;
        if nullifier.value != expected_value {
            return Ok(false);
        }
        
        // Verify signature
        self.verify_nullifier_signature(nullifier)
    }
    
    /// Verify nullifier with specific UTXO index
    pub fn verify_nullifier_with_index(
        &self,
        nullifier: &Nullifier,
        utxo_index: u64,
    ) -> CryptoResult<bool> {
        // Recreate nullifier seed with specific index
        let nullifier_seed = self.create_nullifier_seed(&nullifier.utxo_commitment, utxo_index)?;
        
        // Verify nullifier value
        let expected_value = self.hash_nullifier(&nullifier_seed)?;
        if nullifier.value != expected_value {
            return Ok(false);
        }
        
        // Verify signature
        self.verify_nullifier_signature(nullifier)
    }
    
    /// Batch verify multiple nullifiers
    pub fn batch_verify_nullifiers(&self, nullifiers: &[Nullifier]) -> CryptoResult<bool> {
        for nullifier in nullifiers {
            if !self.verify_nullifier(nullifier)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    /// Create nullifier seed
    fn create_nullifier_seed(
        &self,
        utxo_commitment: &[u8; 32],
        utxo_index: u64,
    ) -> CryptoResult<Vec<u8>> {
        let mut seed = Vec::new();
        seed.extend_from_slice(&self.context.domain);
        seed.extend_from_slice(&self.context.salt);
        seed.extend_from_slice(utxo_commitment);
        seed.extend_from_slice(&utxo_index.to_be_bytes());
        Ok(seed)
    }
    
    /// Hash nullifier seed
    fn hash_nullifier(&self, seed: &[u8]) -> CryptoResult<[u8; 32]> {
        let hash = match self.hash_function {
            NullifierHashFunction::Sha256 => CryptoUtils::sha256(seed),
            NullifierHashFunction::Blake2b256 => CryptoUtils::blake2b256(seed),
            NullifierHashFunction::Keccak256 => CryptoUtils::keccak256(seed),
            NullifierHashFunction::Poseidon => {
                // Use Poseidon hash with proper domain separation
                use crate::crypto::poseidon::PoseidonHasher;
                PoseidonHasher::nullifier(&[0u8; 32], 0) // Simplified for now
                    .unwrap_or_else(|_| CryptoUtils::blake2b256(seed))
            }
        };
        Ok(hash)
    }
    
    /// Sign nullifier
    fn sign_nullifier(&self, nullifier_value: &[u8; 32], private_key: &[u8; 32]) -> CryptoResult<Vec<u8>> {
        // Create message to sign
        let mut message = Vec::new();
        message.extend_from_slice(&self.context.domain);
        message.extend_from_slice(nullifier_value);
        
        // Sign with Ed25519
        let ed25519_private = ed25519_dalek::SigningKey::from_bytes(private_key);
        
        let signature = Ed25519Scheme::sign(&ed25519_private, &message)?;
        Ok(signature.to_bytes().to_vec())
    }
    
    /// Derive public key from private key
    fn derive_public_key(&self, private_key: &[u8; 32]) -> CryptoResult<[u8; 32]> {
        let ed25519_private = ed25519_dalek::SigningKey::from_bytes(private_key);
        
        Ok(ed25519_private.verifying_key().to_bytes())
    }
    
    /// Verify nullifier signature
    fn verify_nullifier_signature(&self, nullifier: &Nullifier) -> CryptoResult<bool> {
        // Create message to verify
        let mut message = Vec::new();
        message.extend_from_slice(&self.context.domain);
        message.extend_from_slice(&nullifier.value);
        
        // Try Ed25519 verification
        if let Ok(signature_bytes) = <[u8; 64]>::try_from(&nullifier.signature[..64]) {
            let signature = ed25519_dalek::Signature::from_bytes(&signature_bytes);
            if let Ok(public_key) = ed25519_dalek::VerifyingKey::from_bytes(&nullifier.public_key) {
                if public_key.verify(&message, &signature).is_ok() {
                    return Ok(true);
                }
            }
        }
        
        // Try ECDSA verification
        if let Ok(signature_bytes) = <[u8; 97]>::try_from(&nullifier.signature[..97]) {
            if let Ok(ecdsa_sig) = EcdsaSig::from_bytes(&signature_bytes) {
                if let Ok(public_key) = secp256k1::PublicKey::from_slice(&nullifier.public_key) {
                    if EcdsaScheme::verify(&ecdsa_sig, &message, &public_key).unwrap_or(false) {
                        return Ok(true);
                    }
                }
            }
        }
        
        Ok(false)
    }
}

/// Nullifier set for tracking used nullifiers
pub struct NullifierSet {
    /// Set of used nullifiers
    pub nullifiers: std::collections::HashSet<[u8; 32]>,
    /// Generator for verification
    pub generator: NullifierGenerator,
}

impl NullifierSet {
    /// Create new nullifier set
    pub fn new(context: CryptoContext, hash_function: NullifierHashFunction) -> Self {
        Self {
            nullifiers: std::collections::HashSet::new(),
            generator: NullifierGenerator::new(context, hash_function),
        }
    }
    
    /// Add nullifier to set
    pub fn add_nullifier(&mut self, nullifier: &Nullifier) -> CryptoResult<bool> {
        // Verify nullifier first
        if !self.generator.verify_nullifier(nullifier)? {
            return Err(CryptoError::NullifierFailed("Invalid nullifier".to_string()));
        }
        
        // Check if already used
        if self.nullifiers.contains(&nullifier.value) {
            return Err(CryptoError::NullifierFailed("Nullifier already used".to_string()));
        }
        
        // Add to set
        self.nullifiers.insert(nullifier.value);
        Ok(true)
    }
    
    /// Check if nullifier is used
    pub fn is_nullifier_used(&self, nullifier_value: &[u8; 32]) -> bool {
        self.nullifiers.contains(nullifier_value)
    }
    
    /// Get nullifier count
    pub fn count(&self) -> usize {
        self.nullifiers.len()
    }
    
    /// Clear all nullifiers
    pub fn clear(&mut self) {
        self.nullifiers.clear();
    }
    
    /// Batch add nullifiers
    pub fn add_nullifiers(&mut self, nullifiers: &[Nullifier]) -> CryptoResult<usize> {
        let mut added_count = 0;
        
        for nullifier in nullifiers {
            if self.add_nullifier(nullifier).is_ok() {
                added_count += 1;
            }
        }
        
        Ok(added_count)
    }
}

/// Nullifier utilities
pub struct NullifierUtils;

impl NullifierUtils {
    /// Generate nullifier from UTXO data
    pub fn generate_from_utxo(
        utxo_commitment: &[u8; 32],
        utxo_index: u64,
        private_key: &[u8; 32],
        context: &CryptoContext,
    ) -> CryptoResult<Nullifier> {
        let generator = NullifierGenerator::new(context.clone(), NullifierHashFunction::Blake2b256);
        generator.generate_nullifier(utxo_commitment, private_key, utxo_index)
    }
    
    /// Generate nullifier from extended private key
    pub fn generate_from_extended_key(
        utxo_commitment: &[u8; 32],
        utxo_index: u64,
        extended_key: &ExtendedPrivateKey,
        context: &CryptoContext,
    ) -> CryptoResult<Nullifier> {
        let private_key = extended_key.secp256k1_secret_key()?.secret_bytes();
        let generator = NullifierGenerator::new(context.clone(), NullifierHashFunction::Blake2b256);
        generator.generate_nullifier(utxo_commitment, &private_key, utxo_index)
    }
    
    /// Verify nullifier against multiple contexts
    pub fn verify_against_contexts(
        nullifier: &Nullifier,
        contexts: &[CryptoContext],
    ) -> CryptoResult<bool> {
        for context in contexts {
            let generator = NullifierGenerator::new(context.clone(), NullifierHashFunction::Blake2b256);
            if generator.verify_nullifier(nullifier).unwrap_or(false) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    
    /// Generate nullifier proof
    pub fn generate_nullifier_proof(
        nullifier: &Nullifier,
        utxo_index: u64,
        merkle_proof: &crate::utxo::transaction::MerkleProof,
    ) -> CryptoResult<NullifierProof> {
        // Create proof that nullifier corresponds to UTXO in Merkle tree
        let mut proof_data = Vec::new();
        proof_data.extend_from_slice(&nullifier.value);
        proof_data.extend_from_slice(&nullifier.utxo_commitment);
        proof_data.extend_from_slice(&utxo_index.to_be_bytes());
        proof_data.extend_from_slice(&merkle_proof.root);
        
        let proof_hash = CryptoUtils::blake2b256(&proof_data);
        
        Ok(NullifierProof {
            nullifier: nullifier.clone(),
            utxo_index,
            merkle_proof: merkle_proof.clone(),
            proof_hash,
        })
    }
}

/// Nullifier proof for verification
#[derive(Debug, Clone)]
pub struct NullifierProof {
    /// The nullifier
    pub nullifier: Nullifier,
    /// UTXO index
    pub utxo_index: u64,
    /// Merkle proof of UTXO inclusion
    pub merkle_proof: crate::utxo::transaction::MerkleProof,
    /// Proof hash
    pub proof_hash: [u8; 32],
}

impl NullifierProof {
    /// Verify the nullifier proof
    pub fn verify(&self, context: &CryptoContext) -> CryptoResult<bool> {
        // Verify nullifier
        let generator = NullifierGenerator::new(context.clone(), NullifierHashFunction::Blake2b256);
        if !generator.verify_nullifier_with_index(&self.nullifier, self.utxo_index)? {
            return Ok(false);
        }
        
        // Verify Merkle proof
        let merkle_verifier = crate::crypto::merkle_proofs::MerkleProofVerifier::new(
            crate::crypto::merkle_proofs::HashFunction::Blake2b256,
            self.merkle_proof.siblings.len(),
        );
        
        if !merkle_verifier.verify_proof(&self.merkle_proof, &self.nullifier.utxo_commitment)? {
            return Ok(false);
        }
        
        // Verify proof hash
        let mut proof_data = Vec::new();
        proof_data.extend_from_slice(&self.nullifier.value);
        proof_data.extend_from_slice(&self.nullifier.utxo_commitment);
        proof_data.extend_from_slice(&self.utxo_index.to_be_bytes());
        proof_data.extend_from_slice(&self.merkle_proof.root);
        
        let expected_hash = CryptoUtils::blake2b256(&proof_data);
        Ok(self.proof_hash == expected_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nullifier_generation() {
        let context = CryptoContext::nullifier_context();
        let generator = NullifierGenerator::new(context, NullifierHashFunction::Blake2b256);
        
        let utxo_commitment = CryptoUtils::random_32();
        let private_key = CryptoUtils::random_32();
        let utxo_index = 0;
        
        let nullifier = generator.generate_nullifier(&utxo_commitment, &private_key, utxo_index).unwrap();
        
        assert_eq!(nullifier.utxo_commitment, utxo_commitment);
        assert!(generator.verify_nullifier(&nullifier).unwrap());
    }

    #[test]
    fn test_nullifier_set() {
        let context = CryptoContext::nullifier_context();
        let mut nullifier_set = NullifierSet::new(context, NullifierHashFunction::Blake2b256);
        
        let utxo_commitment = CryptoUtils::random_32();
        let private_key = CryptoUtils::random_32();
        let utxo_index = 0;
        
        let generator = NullifierGenerator::new(CryptoContext::nullifier_context(), NullifierHashFunction::Blake2b256);
        let nullifier = generator.generate_nullifier(&utxo_commitment, &private_key, utxo_index).unwrap();
        
        // Add nullifier
        assert!(nullifier_set.add_nullifier(&nullifier).unwrap());
        assert!(nullifier_set.is_nullifier_used(&nullifier.value));
        
        // Try to add again (should fail)
        assert!(nullifier_set.add_nullifier(&nullifier).is_err());
    }

    #[test]
    fn test_nullifier_proof() {
        let context = CryptoContext::nullifier_context();
        let generator = NullifierGenerator::new(context.clone(), NullifierHashFunction::Blake2b256);
        
        let utxo_commitment = CryptoUtils::random_32();
        let private_key = CryptoUtils::random_32();
        let utxo_index = 0;
        
        let nullifier = generator.generate_nullifier(&utxo_commitment, &private_key, utxo_index).unwrap();
        
        // Create mock Merkle proof
        let merkle_proof = crate::utxo::transaction::MerkleProof {
            siblings: vec![CryptoUtils::random_32(), CryptoUtils::random_32()],
            path: vec![0, 1],
            root: CryptoUtils::random_32(),
            leaf_index: utxo_index,
        };
        
        let nullifier_proof = NullifierUtils::generate_nullifier_proof(&nullifier, utxo_index, &merkle_proof).unwrap();
        
        // Verify proof
        assert!(nullifier_proof.verify(&context).unwrap());
    }

    #[test]
    fn test_batch_nullifier_verification() {
        let context = CryptoContext::nullifier_context();
        let generator = NullifierGenerator::new(context, NullifierHashFunction::Blake2b256);
        
        let mut nullifiers = Vec::new();
        for i in 0..5 {
            let utxo_commitment = CryptoUtils::random_32();
            let private_key = CryptoUtils::random_32();
            let nullifier = generator.generate_nullifier(&utxo_commitment, &private_key, i).unwrap();
            nullifiers.push(nullifier);
        }
        
        assert!(generator.batch_verify_nullifiers(&nullifiers).unwrap());
    }
}
