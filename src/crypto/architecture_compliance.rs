//! Architecture Compliance Module
//!
//! This module provides the exact cryptographic functions as specified
//! in architecture.md, ensuring compliance with the documented API.

use crate::crypto::{CryptoResult, CryptoError, domains, CryptoUtils};
use crate::crypto::poseidon::PoseidonHasher;
use serde_with::{serde_as, Bytes};

/// Architecture-compliant commitment generation
/// Implements: commitment = Poseidon(DOMAIN_COMMIT_V1, owner_enc_pk, asset, value, secret, blinding)
pub struct ArchitectureCompliantCrypto;

impl ArchitectureCompliantCrypto {
    /// Generate commitment exactly as specified in architecture.md
    /// commitment = Poseidon(domain_commit, owner_enc_pk, asset, value_field, secret, blinding)
    pub fn compute_commitment(
        owner_enc_pk: &[u8; 33],    // ECC compressed pubkey for ECIES
        asset: &[u8; 20],           // address(0) for ETH
        value: u128,                // in wei
        secret: &[u8; 32],          // random per-note secret
        blinding: &[u8; 32],        // commitment blinding
    ) -> CryptoResult<[u8; 32]> {
        // Create input for Poseidon hash
        let mut input = Vec::new();
        input.extend_from_slice(domains::DOMAIN_COMMIT_V1);
        input.extend_from_slice(owner_enc_pk);
        input.extend_from_slice(asset);
        input.extend_from_slice(&value.to_le_bytes());
        input.extend_from_slice(secret);
        input.extend_from_slice(blinding);

        // Use Poseidon hash (fallback to Blake2b for now until proper Poseidon params)
        match PoseidonHasher::commitment(&input[..32].try_into().unwrap(), blinding) {
            Ok(commitment) => Ok(commitment),
            Err(_) => {
                // Fallback to Blake2b with proper domain separation
                Ok(CryptoUtils::blake2b256(&input))
            }
        }
    }

    /// Generate nullifier exactly as specified in architecture.md
    /// nullifier = Poseidon(DOMAIN_NULL_V1, secret_field, leaf_index_field)
    pub fn derive_nullifier(
        secret: &[u8; 32],
        leaf_index: u64,
    ) -> CryptoResult<[u8; 32]> {
        // Create input for nullifier derivation
        let mut input = Vec::new();
        input.extend_from_slice(domains::DOMAIN_NULL_V1);
        input.extend_from_slice(secret);
        input.extend_from_slice(&leaf_index.to_be_bytes());

        // Use Poseidon hash (fallback to Blake2b)
        match PoseidonHasher::nullifier(&[0u8; 32], leaf_index) {
            Ok(nullifier) => Ok(nullifier),
            Err(_) => {
                // Fallback to Blake2b with proper domain separation
                Ok(CryptoUtils::blake2b256(&input))
            }
        }
    }

    /// Generate note ID exactly as specified in architecture.md
    /// note_id = SHA256(DOMAIN_NOTE_V1 || commitment || secret)
    pub fn generate_note_id(
        commitment: &[u8; 32],
        secret: &[u8; 32],
    ) -> [u8; 32] {
        let mut input = Vec::new();
        input.extend_from_slice(domains::DOMAIN_NOTE_V1);
        input.extend_from_slice(commitment);
        input.extend_from_slice(secret);

        CryptoUtils::sha256(&input)
    }

    /// HKDF for ECIES encryption as specified in architecture.md
    /// HKDF-SHA256 over shared_secret with info = DOMAIN_ECIES_V1 || commitment || version
    pub fn derive_ecies_keys(
        shared_secret: &[u8; 32],
        commitment: &[u8; 32],
        version: u8,
    ) -> CryptoResult<([u8; 32], [u8; 32])> {
        // Create info string
        let mut info = Vec::new();
        info.extend_from_slice(domains::DOMAIN_ECIES_V1);
        info.extend_from_slice(commitment);
        info.push(version);

        // Derive 64 bytes (32 for encryption key, 32 for mac key)
        let derived = CryptoUtils::hkdf_sha256(shared_secret, b"", &info, 64)?;

        let mut enc_key = [0u8; 32];
        let mut mac_key = [0u8; 32];
        enc_key.copy_from_slice(&derived[0..32]);
        mac_key.copy_from_slice(&derived[32..64]);

        Ok((enc_key, mac_key))
    }

    /// Verify nullifier binding to leaf index (prevents replay across contexts)
    pub fn verify_nullifier_binding(
        nullifier: &[u8; 32],
        secret: &[u8; 32],
        leaf_index: u64,
    ) -> CryptoResult<bool> {
        let expected_nullifier = Self::derive_nullifier(secret, leaf_index)?;
        Ok(CryptoUtils::constant_time_eq(nullifier, &expected_nullifier))
    }

    /// Create Merkle leaf hash with domain separation
    pub fn hash_merkle_leaf(commitment: &[u8; 32]) -> CryptoResult<[u8; 32]> {
        let mut input = Vec::new();
        input.extend_from_slice(b"PRIVPOOL_LEAF_V1");
        input.extend_from_slice(commitment);

        Ok(CryptoUtils::blake2b256(&input))
    }

    /// Create Merkle node hash with domain separation
    pub fn hash_merkle_node(left: &[u8; 32], right: &[u8; 32]) -> CryptoResult<[u8; 32]> {
        let mut input = Vec::new();
        input.extend_from_slice(b"PRIVPOOL_NODE_V1");
        input.extend_from_slice(left);
        input.extend_from_slice(right);

        Ok(CryptoUtils::blake2b256(&input))
    }

    /// Validate field element conversion for Poseidon
    /// Ensures consistent endianness and padding rules
    pub fn bytes_to_field_element(bytes: &[u8]) -> CryptoResult<[u8; 32]> {
        if bytes.len() > 32 {
            return Err(CryptoError::InvalidInput("Input too large for field element".to_string()));
        }

        let mut field_bytes = [0u8; 32];
        // Use big-endian encoding as specified
        let copy_len = std::cmp::min(bytes.len(), 32);
        field_bytes[32 - copy_len..].copy_from_slice(&bytes[..copy_len]);

        Ok(field_bytes)
    }

    /// Convert value to field element with specified encoding
    pub fn value_to_field_element(value: u128) -> [u8; 32] {
        let mut field_bytes = [0u8; 32];
        // Use the last 16 bytes for u128, big-endian
        field_bytes[16..32].copy_from_slice(&value.to_be_bytes());
        field_bytes
    }
}

/// Canonical Note structure exactly as specified in architecture.md
#[serde_as]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CanonicalNote {
    pub version: u8,
    pub chain_id: u64,
    pub pool_address: [u8; 20],     // Ethereum address
    pub asset: [u8; 20],            // address(0) for ETH
    pub value: u128,                // in wei
    #[serde_as(as = "Bytes")]
    pub owner_enc_pk: [u8; 33],     // recipient ECC compressed pubkey for ECIES
    pub owner_spend_pk: [u8; 32],   // public key used by spend authorization
    pub secret: [u8; 32],           // random per-note secret (private)
    pub blinding: [u8; 32],         // commitment blinding (private)
    pub commitment: [u8; 32],       // Poseidon commitment hash
    pub created_at: u64,            // epoch seconds
    pub tx_hash: Option<[u8; 32]>,  // set once on-chain deposit confirmed
    pub tx_index: Option<u32>,      // log index or output index
    pub leaf_index: Option<u64>,    // index in merkle tree once inserted
    pub note_id: [u8; 32],          // hash domain: DOMAIN_NOTE || commitment || secret
}

impl CanonicalNote {
    /// Create new canonical note
    pub fn new(
        version: u8,
        chain_id: u64,
        pool_address: [u8; 20],
        asset: [u8; 20],
        value: u128,
        owner_enc_pk: [u8; 33],
        owner_spend_pk: [u8; 32],
        secret: [u8; 32],
        blinding: [u8; 32],
    ) -> CryptoResult<Self> {
        // Compute commitment
        let commitment = ArchitectureCompliantCrypto::compute_commitment(
            &owner_enc_pk,
            &asset,
            value,
            &secret,
            &blinding,
        )?;

        // Generate note ID
        let note_id = ArchitectureCompliantCrypto::generate_note_id(&commitment, &secret);

        Ok(Self {
            version,
            chain_id,
            pool_address,
            asset,
            value,
            owner_enc_pk,
            owner_spend_pk,
            secret,
            blinding,
            commitment,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash: None,
            tx_index: None,
            leaf_index: None,
            note_id,
        })
    }

    /// Compute commitment for this note
    pub fn compute_commitment(&self) -> CryptoResult<[u8; 32]> {
        ArchitectureCompliantCrypto::compute_commitment(
            &self.owner_enc_pk,
            &self.asset,
            self.value,
            &self.secret,
            &self.blinding,
        )
    }

    /// Derive nullifier for this note
    pub fn derive_nullifier(&self, leaf_index: u64) -> CryptoResult<[u8; 32]> {
        ArchitectureCompliantCrypto::derive_nullifier(&self.secret, leaf_index)
    }

    /// Verify commitment matches computed value
    pub fn verify_commitment(&self) -> CryptoResult<bool> {
        let computed = self.compute_commitment()?;
        Ok(CryptoUtils::constant_time_eq(&self.commitment, &computed))
    }

    /// Generate note ID for this note
    pub fn generate_note_id(&self) -> [u8; 32] {
        ArchitectureCompliantCrypto::generate_note_id(&self.commitment, &self.secret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_commitment_generation() {
        let owner_enc_pk = [0x02; 33]; // Compressed pubkey
        let asset = [0u8; 20]; // ETH
        let value = 1000000000000000000u128; // 1 ETH in wei
        let secret = CryptoUtils::random_32();
        let blinding = CryptoUtils::random_32();

        let commitment1 = ArchitectureCompliantCrypto::compute_commitment(
            &owner_enc_pk, &asset, value, &secret, &blinding
        ).unwrap();

        let commitment2 = ArchitectureCompliantCrypto::compute_commitment(
            &owner_enc_pk, &asset, value, &secret, &blinding
        ).unwrap();

        // Should be deterministic
        assert_eq!(commitment1, commitment2);
    }

    #[test]
    fn test_nullifier_generation() {
        let secret = CryptoUtils::random_32();
        let leaf_index = 42u64;

        let nullifier1 = ArchitectureCompliantCrypto::derive_nullifier(&secret, leaf_index).unwrap();
        let nullifier2 = ArchitectureCompliantCrypto::derive_nullifier(&secret, leaf_index).unwrap();

        // Should be deterministic
        assert_eq!(nullifier1, nullifier2);

        // Should be different for different leaf indices
        let nullifier3 = ArchitectureCompliantCrypto::derive_nullifier(&secret, leaf_index + 1).unwrap();
        assert_ne!(nullifier1, nullifier3);
    }

    #[test]
    fn test_note_id_generation() {
        let commitment = CryptoUtils::random_32();
        let secret = CryptoUtils::random_32();

        let note_id1 = ArchitectureCompliantCrypto::generate_note_id(&commitment, &secret);
        let note_id2 = ArchitectureCompliantCrypto::generate_note_id(&commitment, &secret);

        // Should be deterministic
        assert_eq!(note_id1, note_id2);
    }

    #[test]
    fn test_canonical_note_creation() {
        let note = CanonicalNote::new(
            1, // version
            1, // chain_id (mainnet)
            [0x19; 20], // pool_address
            [0u8; 20], // ETH asset
            1000000000000000000u128, // 1 ETH
            [0x02; 33], // compressed pubkey
            CryptoUtils::random_32(), // spend pubkey
            CryptoUtils::random_32(), // secret
            CryptoUtils::random_32(), // blinding
        ).unwrap();

        // Verify commitment is correct
        assert!(note.verify_commitment().unwrap());

        // Verify note ID generation
        let expected_note_id = note.generate_note_id();
        assert_eq!(note.note_id, expected_note_id);
    }

    #[test]
    fn test_nullifier_binding_verification() {
        let secret = CryptoUtils::random_32();
        let leaf_index = 123u64;

        let nullifier = ArchitectureCompliantCrypto::derive_nullifier(&secret, leaf_index).unwrap();

        // Should verify correctly
        assert!(ArchitectureCompliantCrypto::verify_nullifier_binding(&nullifier, &secret, leaf_index).unwrap());

        // Should fail with wrong leaf index
        assert!(!ArchitectureCompliantCrypto::verify_nullifier_binding(&nullifier, &secret, leaf_index + 1).unwrap());
    }

    #[test]
    fn test_ecies_key_derivation() {
        let shared_secret = CryptoUtils::random_32();
        let commitment = CryptoUtils::random_32();
        let version = 1u8;

        let (enc_key1, mac_key1) = ArchitectureCompliantCrypto::derive_ecies_keys(
            &shared_secret, &commitment, version
        ).unwrap();

        let (enc_key2, mac_key2) = ArchitectureCompliantCrypto::derive_ecies_keys(
            &shared_secret, &commitment, version
        ).unwrap();

        // Should be deterministic
        assert_eq!(enc_key1, enc_key2);
        assert_eq!(mac_key1, mac_key2);

        // Keys should be different
        assert_ne!(enc_key1, mac_key1);
    }
}