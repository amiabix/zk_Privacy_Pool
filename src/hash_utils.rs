//! Hash utility functions for privacy pool
//! Supports both SHA-256 (for testing/compatibility) and Poseidon (for ZK circuits)

use sha2::{Digest, Sha256};
use anyhow::Result;

/// Hash function type selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HashType {
    Sha256,
    Poseidon,
}

/// Unified hasher that supports both SHA-256 and Poseidon
pub struct UnifiedHasher {
    hash_type: HashType,
    // Note: Poseidon doesn't implement Clone/Debug, so we'll create instances as needed
}

impl UnifiedHasher {
    /// Create a new hasher with specified type
    pub fn new(hash_type: HashType) -> Result<Self> {
        Ok(Self {
            hash_type,
        })
    }

    /// Hash two 32-byte values
    pub fn hash_pair(&self, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        match self.hash_type {
            HashType::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
            HashType::Poseidon => {
                // For now, fall back to SHA-256 until Poseidon is fully integrated
                // TODO: Implement proper Poseidon hashing for ZK circuits
                let mut hasher = Sha256::new();
                hasher.update(b"poseidon_prefix"); // Distinguish from regular SHA-256
                hasher.update(&left);
                hasher.update(&right);
                hasher.finalize().into()
            }
        }
    }

    /// Hash multiple values together
    pub fn hash_multi(&self, values: &[&[u8]]) -> [u8; 32] {
        match self.hash_type {
            HashType::Sha256 => {
                let mut hasher = Sha256::new();
                for value in values {
                    hasher.update(value);
                }
                hasher.finalize().into()
            }
            HashType::Poseidon => {
                // For now, fall back to SHA-256 until Poseidon is fully integrated  
                // TODO: Implement proper Poseidon hashing for ZK circuits
                let mut hasher = Sha256::new();
                hasher.update(b"poseidon_multi_prefix"); // Distinguish from regular SHA-256
                for value in values {
                    hasher.update(value);
                }
                hasher.finalize().into()
            }
        }
    }

    /// Hash UTXO data
    pub fn hash_utxo(&self, commitment: &[u8; 32], value: u64, owner: &[u8; 32], leaf_index: u64) -> [u8; 32] {
        let value_bytes = value.to_le_bytes();
        let index_bytes = leaf_index.to_le_bytes();
        
        self.hash_multi(&[commitment, &value_bytes, owner, &index_bytes])
    }

    /// Compute UTXO commitment
    pub fn compute_commitment(&self, value: u64, secret: &[u8; 32], nullifier: &[u8; 32], owner: &[u8; 32]) -> [u8; 32] {
        let value_bytes = value.to_le_bytes();
        self.hash_multi(&[&value_bytes, secret, nullifier, owner])
    }
}

// TODO: Implement proper Poseidon field element conversions when ZK integration is complete

/// Default hasher instance (SHA-256 for compatibility)
pub fn default_hasher() -> UnifiedHasher {
    UnifiedHasher::new(HashType::Sha256).unwrap()
}

/// Poseidon hasher instance 
pub fn poseidon_hasher() -> Result<UnifiedHasher> {
    UnifiedHasher::new(HashType::Poseidon)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_hasher() {
        let hasher = UnifiedHasher::new(HashType::Sha256).unwrap();
        
        let left = [1u8; 32];
        let right = [2u8; 32];
        
        let result = hasher.hash_pair(left, right);
        
        // Verify it produces the same result as direct SHA-256
        let mut sha_hasher = Sha256::new();
        sha_hasher.update(&left);
        sha_hasher.update(&right);
        let expected: [u8; 32] = sha_hasher.finalize().into();
        
        assert_eq!(result, expected);
    }

    #[test]
    fn test_poseidon_hasher() {
        let hasher = UnifiedHasher::new(HashType::Poseidon).unwrap();
        
        let left = [1u8; 32];
        let right = [2u8; 32];
        
        let result1 = hasher.hash_pair(left, right);
        let result2 = hasher.hash_pair(left, right);
        
        // Should be deterministic
        assert_eq!(result1, result2);
        
        // Should be different from different inputs
        let result3 = hasher.hash_pair(right, left);
        assert_ne!(result1, result3);
    }

    #[test]
    fn test_commitment_computation() {
        let hasher = default_hasher();
        
        let value = 1000000000000000000u64; // 1 ETH
        let secret = [0x42u8; 32];
        let nullifier = [0x43u8; 32];
        let owner = [0x44u8; 32];
        
        let commitment = hasher.compute_commitment(value, &secret, &nullifier, &owner);
        
        // Should be deterministic
        let commitment2 = hasher.compute_commitment(value, &secret, &nullifier, &owner);
        assert_eq!(commitment, commitment2);
        
        // Should be different for different values
        let commitment3 = hasher.compute_commitment(value + 1, &secret, &nullifier, &owner);
        assert_ne!(commitment, commitment3);
    }

    #[test] 
    fn test_utxo_hashing() {
        let hasher = default_hasher();
        
        let commitment = [0x11u8; 32];
        let value = 1000000000000000000u64;
        let owner = [0x22u8; 32];
        let leaf_index = 42u64;
        
        let hash1 = hasher.hash_utxo(&commitment, value, &owner, leaf_index);
        let hash2 = hasher.hash_utxo(&commitment, value, &owner, leaf_index);
        
        // Should be deterministic
        assert_eq!(hash1, hash2);
        
        // Should be different for different leaf indices
        let hash3 = hasher.hash_utxo(&commitment, value, &owner, leaf_index + 1);
        assert_ne!(hash1, hash3);
    }
}