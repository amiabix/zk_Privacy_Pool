//! Cryptographic Primitives Module
//! 
//! This module provides production-ready cryptographic implementations
//! for the Privacy Pool ZKVM system, including:
//! - Ed25519 signature verification
//! - ECDSA signature verification  
//! - Key derivation (BIP32/HD wallets)
//! - Commitment schemes (Pedersen/Poseidon)
//! - Merkle proof verification
//! - Nullifier generation and verification

use rand::RngCore;
use sha2::Digest;
// Removed unused import: use sha3::Digest as Sha3Digest;

pub mod signatures;
pub mod key_derivation;
pub mod commitments;
pub mod merkle_proofs;
pub mod nullifiers;
pub mod poseidon;
pub mod bn254;
pub mod ecies;

// Re-export main types
pub use signatures::*;
pub use key_derivation::*;
pub use commitments::*;
pub use merkle_proofs::*;
pub use nullifiers::*;
pub use poseidon::*;
pub use bn254::*;
pub use ecies::*;

/// Cryptographic error types
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    
    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),
    
    #[error("Invalid private key: {0}")]
    InvalidPrivateKey(String),
    
    #[error("Key derivation failed: {0}")]
    KeyDerivationFailed(String),
    
    #[error("Commitment generation failed: {0}")]
    CommitmentFailed(String),
    
    #[error("Merkle proof verification failed: {0}")]
    MerkleProofFailed(String),
    
    #[error("Nullifier generation failed: {0}")]
    NullifierFailed(String),
    
    #[error("Hash function error: {0}")]
    HashError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Result type for cryptographic operations
pub type CryptoResult<T> = Result<T, CryptoError>;

/// Cryptographic context for domain separation
#[derive(Debug, Clone)]
pub struct CryptoContext {
    /// Domain separator for this context
    pub domain: [u8; 32],
    /// Context-specific salt
    pub salt: [u8; 32],
}

impl CryptoContext {
    /// Create new cryptographic context
    pub fn new(domain: &str) -> Self {
        let domain_hash = blake2::Blake2s256::digest(domain.as_bytes());
        let mut domain_bytes = [0u8; 32];
        domain_bytes.copy_from_slice(&domain_hash[..32]);
        
        let salt = blake2::Blake2s256::digest(b"privacy-pool-salt");
        let mut salt_bytes = [0u8; 32];
        salt_bytes.copy_from_slice(&salt[..32]);
        
        Self {
            domain: domain_bytes,
            salt: salt_bytes,
        }
    }
    
    /// Create context for UTXO operations
    pub fn utxo_context() -> Self {
        Self::new("privacy-pool-utxo")
    }
    
    /// Create context for Merkle tree operations
    pub fn merkle_context() -> Self {
        Self::new("privacy-pool-merkle")
    }
    
    /// Create context for nullifier operations
    pub fn nullifier_context() -> Self {
        Self::new("privacy-pool-nullifier")
    }
    
    /// Create context for commitment operations
    pub fn commitment_context() -> Self {
        Self::new("privacy-pool-commitment")
    }
}

/// Domain constants for cryptographic operations
pub mod domains {
    /// Domain separator for nullifier generation
    pub const DOMAIN_NULL: &[u8] = b"privacy-pool-nullifier";
    
    /// Domain separator for commitment generation
    pub const DOMAIN_COMMIT: &[u8] = b"privacy-pool-commitment";
    
    /// Domain separator for note encryption
    pub const DOMAIN_NOTE: &[u8] = b"privacy-pool-note";
    
    /// Domain separator for ECDH key derivation
    pub const DOMAIN_ECDH: &[u8] = b"privacy-pool-ecdh";
}

/// Cryptographic utilities
pub struct CryptoUtils;

impl CryptoUtils {
    /// Generate cryptographically secure random bytes
    pub fn random_bytes(length: usize) -> Vec<u8> {
        use rand::RngCore;
        let mut bytes = vec![0u8; length];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
    
    /// Generate random 32-byte array
    pub fn random_32() -> [u8; 32] {
        let mut bytes = [0u8; 32];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
    
    /// Generate random 64-byte array
    pub fn random_64() -> [u8; 64] {
        let mut bytes = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
    
    /// Hash data with Blake2b-256
    pub fn blake2b256(data: &[u8]) -> [u8; 32] {
        blake2::Blake2s256::digest(data).into()
    }
    
    /// Hash data with SHA-256
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        sha2::Sha256::digest(data).into()
    }
    
    /// Hash data with Keccak-256
    pub fn keccak256(data: &[u8]) -> [u8; 32] {
        sha3::Keccak256::digest(data).into()
    }
    
    /// Constant-time comparison of byte arrays
    pub fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
        use subtle::ConstantTimeEq;
        a.ct_eq(b).into()
    }
    
    /// Convert bytes to hex string
    pub fn to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }
    
    /// Convert hex string to bytes
    pub fn from_hex(hex: &str) -> CryptoResult<Vec<u8>> {
        hex::decode(hex).map_err(|e| CryptoError::SerializationError(e.to_string()))
    }
    
    /// Generate random 24 bytes for XChaCha20-Poly1305 nonce
    pub fn random_24() -> [u8; 24] {
        let mut bytes = [0u8; 24];
        rand::thread_rng().fill_bytes(&mut bytes);
        bytes
    }
    
    /// HKDF-SHA256 key derivation
    pub fn hkdf_sha256(ikm: &[u8], salt: &[u8], info: &[u8], length: usize) -> CryptoResult<Vec<u8>> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        type HmacSha256 = Hmac<Sha256>;
        
        let mut hkdf = HmacSha256::new_from_slice(salt)
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("HKDF init failed: {:?}", e)))?;
        
        hkdf.update(ikm);
        let prk = hkdf.finalize().into_bytes();
        
        let mut okm = Vec::new();
        let mut counter = 1u8;
        
        while okm.len() < length {
            let mut hmac = HmacSha256::new_from_slice(&prk)
                .map_err(|e| CryptoError::KeyDerivationFailed(format!("HKDF expand failed: {:?}", e)))?;
            
            if !okm.is_empty() {
                hmac.update(&okm[okm.len() - 32..]);
            }
            hmac.update(info);
            hmac.update(&[counter]);
            
            let t = hmac.finalize().into_bytes();
            okm.extend_from_slice(&t);
            counter += 1;
        }
        
        okm.truncate(length);
        Ok(okm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_context() {
        let ctx = CryptoContext::new("test-domain");
        assert_ne!(ctx.domain, [0u8; 32]);
        assert_ne!(ctx.salt, [0u8; 32]);
    }

    #[test]
    fn test_crypto_utils() {
        let bytes = CryptoUtils::random_32();
        assert_eq!(bytes.len(), 32);
        
        let hash = CryptoUtils::blake2b256(b"test data");
        assert_eq!(hash.len(), 32);
        
        let hex = CryptoUtils::to_hex(&hash);
        let decoded = CryptoUtils::from_hex(&hex).unwrap();
        assert_eq!(hash.to_vec(), decoded);
    }

    #[test]
    fn test_constant_time_comparison() {
        let a = [1u8, 2u8, 3u8];
        let b = [1u8, 2u8, 3u8];
        let c = [1u8, 2u8, 4u8];
        
        assert!(CryptoUtils::constant_time_eq(&a, &b));
        assert!(!CryptoUtils::constant_time_eq(&a, &c));
    }
}
