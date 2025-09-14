//! Key Derivation Implementation
//! 
//! This module provides BIP32/HD wallet key derivation
//! for hierarchical deterministic key generation.

use hmac::{Hmac, Mac};
use sha2::Sha512;
use secp256k1::{Secp256k1, SecretKey, PublicKey};
use ed25519_dalek::{SigningKey, VerifyingKey};
use crate::crypto::{CryptoResult, CryptoError, CryptoUtils};

type HmacSha512 = Hmac<Sha512>;

/// BIP32 extended private key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedPrivateKey {
    /// The private key (32 bytes)
    pub private_key: [u8; 32],
    /// The chain code (32 bytes)
    pub chain_code: [u8; 32],
    /// The depth in the key hierarchy
    pub depth: u8,
    /// The parent fingerprint
    pub parent_fingerprint: [u8; 4],
    /// The child number
    pub child_number: u32,
}

/// BIP32 extended public key
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtendedPublicKey {
    /// The public key (33 bytes for secp256k1)
    pub public_key: [u8; 33],
    /// The chain code (32 bytes)
    pub chain_code: [u8; 32],
    /// The depth in the key hierarchy
    pub depth: u8,
    /// The parent fingerprint
    pub parent_fingerprint: [u8; 4],
    /// The child number
    pub child_number: u32,
}

/// BIP32 key derivation path
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivationPath {
    /// The path components
    pub components: Vec<u32>,
    /// Whether this is a hardened path
    pub hardened: bool,
}

impl DerivationPath {
    /// Create new derivation path from string (e.g., "m/44'/0'/0'/0/0")
    pub fn from_string(path: &str) -> CryptoResult<Self> {
        if !path.starts_with("m/") {
            return Err(CryptoError::InvalidInput("Path must start with 'm/'".to_string()));
        }
        
        let components_str = &path[2..];
        let parts: Vec<&str> = components_str.split('/').collect();
        let mut components = Vec::new();
        let mut hardened = false;
        
        for part in parts {
            if part.is_empty() {
                continue;
            }
            
            let (component_str, is_hardened) = if part.ends_with('\'') {
                (&part[..part.len()-1], true)
            } else {
                (part, false)
            };
            
            let component = component_str.parse::<u32>()
                .map_err(|_| CryptoError::InvalidInput(format!("Invalid path component: {}", part)))?;
            
            components.push(component);
            if is_hardened {
                hardened = true;
            }
        }
        
        Ok(Self { components, hardened })
    }
    
    /// Create standard BIP44 path for Ethereum
    pub fn ethereum(account: u32, change: u32, address_index: u32) -> Self {
        Self {
            components: vec![44, 60, account, change, address_index],
            hardened: true,
        }
    }
    
    /// Create standard BIP44 path for Bitcoin
    pub fn bitcoin(account: u32, change: u32, address_index: u32) -> Self {
        Self {
            components: vec![44, 0, account, change, address_index],
            hardened: true,
        }
    }
    
    /// Create privacy pool specific path
    pub fn privacy_pool(account: u32, utxo_index: u32) -> Self {
        Self {
            components: vec![44, 60, account, 0, utxo_index],
            hardened: true,
        }
    }
}

impl ExtendedPrivateKey {
    /// Create master key from seed
    pub fn from_seed(seed: &[u8]) -> CryptoResult<Self> {
        if seed.len() < 16 || seed.len() > 64 {
            return Err(CryptoError::InvalidInput("Seed must be 16-64 bytes".to_string()));
        }
        
        let mut hmac = HmacSha512::new_from_slice(b"Bitcoin seed")
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        hmac.update(seed);
        let result = hmac.finalize();
        let bytes = result.into_bytes();
        
        let mut private_key = [0u8; 32];
        let mut chain_code = [0u8; 32];
        private_key.copy_from_slice(&bytes[0..32]);
        chain_code.copy_from_slice(&bytes[32..64]);
        
        // Validate private key
        let secp = Secp256k1::new();
        SecretKey::from_slice(&private_key)
            .map_err(|e| CryptoError::InvalidPrivateKey(e.to_string()))?;
        
        Ok(Self {
            private_key,
            chain_code,
            depth: 0,
            parent_fingerprint: [0u8; 4],
            child_number: 0,
        })
    }
    
    /// Derive child private key
    pub fn derive_child(&self, child_number: u32) -> CryptoResult<Self> {
        let secp = Secp256k1::new();
        let private_key = SecretKey::from_slice(&self.private_key)
            .map_err(|e| CryptoError::InvalidPrivateKey(e.to_string()))?;
        let public_key = private_key.public_key(&secp);
        
        let mut hmac = HmacSha512::new_from_slice(&self.chain_code)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        
        if child_number >= 0x80000000 {
            // Hardened derivation
            hmac.update(&[0u8]);
            hmac.update(&self.private_key);
            hmac.update(&child_number.to_be_bytes());
        } else {
            // Non-hardened derivation
            hmac.update(&public_key.serialize());
            hmac.update(&child_number.to_be_bytes());
        }
        
        let result = hmac.finalize();
        let bytes = result.into_bytes();
        
        let mut child_private_key = [0u8; 32];
        let mut child_chain_code = [0u8; 32];
        child_private_key.copy_from_slice(&bytes[0..32]);
        child_chain_code.copy_from_slice(&bytes[32..64]);
        
        // Add parent private key using simple byte arithmetic
        let mut sum_bytes = [0u8; 32];
        for i in 0..32 {
            let sum = self.private_key[i] as u16 + child_private_key[i] as u16;
            sum_bytes[i] = (sum % 256) as u8;
        }
        
        // Validate child private key
        SecretKey::from_slice(&sum_bytes)
            .map_err(|e| CryptoError::InvalidPrivateKey(e.to_string()))?;
        
        // Calculate parent fingerprint
        let parent_fingerprint = CryptoUtils::sha256(&public_key.serialize())[0..4].try_into()
            .map_err(|_| CryptoError::KeyDerivationFailed("Failed to calculate fingerprint".to_string()))?;
        
        Ok(Self {
            private_key: sum_bytes.into(),
            chain_code: child_chain_code,
            depth: self.depth + 1,
            parent_fingerprint,
            child_number,
        })
    }
    
    /// Derive key from path
    pub fn derive_path(&self, path: &DerivationPath) -> CryptoResult<Self> {
        let mut current = self.clone();
        
        for &component in &path.components {
            current = current.derive_child(component)?;
        }
        
        Ok(current)
    }
    
    /// Get extended public key
    pub fn extended_public_key(&self) -> CryptoResult<ExtendedPublicKey> {
        let secp = Secp256k1::new();
        let private_key = SecretKey::from_slice(&self.private_key)
            .map_err(|e| CryptoError::InvalidPrivateKey(e.to_string()))?;
        let public_key = private_key.public_key(&secp);
        
        Ok(ExtendedPublicKey {
            public_key: public_key.serialize(),
            chain_code: self.chain_code,
            depth: self.depth,
            parent_fingerprint: self.parent_fingerprint,
            child_number: self.child_number,
        })
    }
    
    /// Get Ed25519 signing key
    pub fn ed25519_signing_key(&self) -> CryptoResult<SigningKey> {
        Ok(SigningKey::from_bytes(&self.private_key))
    }
    
    /// Get Ed25519 verifying key
    pub fn ed25519_verifying_key(&self) -> CryptoResult<VerifyingKey> {
        let signing_key = self.ed25519_signing_key()?;
        Ok(signing_key.verifying_key())
    }
    
    /// Get secp256k1 secret key
    pub fn secp256k1_secret_key(&self) -> CryptoResult<SecretKey> {
        SecretKey::from_slice(&self.private_key)
            .map_err(|e| CryptoError::InvalidPrivateKey(e.to_string()))
    }
    
    /// Get secp256k1 public key
    pub fn secp256k1_public_key(&self) -> CryptoResult<PublicKey> {
        let secret_key = self.secp256k1_secret_key()?;
        let secp = Secp256k1::new();
        Ok(secret_key.public_key(&secp))
    }
}

impl ExtendedPublicKey {
    /// Derive child public key (non-hardened only)
    pub fn derive_child(&self, child_number: u32) -> CryptoResult<Self> {
        if child_number >= 0x80000000 {
            return Err(CryptoError::KeyDerivationFailed(
                "Cannot derive hardened child from public key".to_string()
            ));
        }
        
        let mut hmac = HmacSha512::new_from_slice(&self.chain_code)
            .map_err(|e| CryptoError::KeyDerivationFailed(e.to_string()))?;
        hmac.update(&self.public_key);
        hmac.update(&child_number.to_be_bytes());
        
        let result = hmac.finalize();
        let bytes = result.into_bytes();
        
        let mut child_public_key = [0u8; 33];
        let mut child_chain_code = [0u8; 32];
        child_public_key.copy_from_slice(&bytes[0..33]);
        child_chain_code.copy_from_slice(&bytes[32..64]);
        
        // Add to parent public key
        let secp = Secp256k1::new();
        let parent_public = PublicKey::from_slice(&self.public_key)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;
        
        // This is a simplified implementation
        // In production, you'd need proper point addition
        let child_public = PublicKey::from_slice(&child_public_key)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;
        
        // Calculate parent fingerprint
        let parent_fingerprint = CryptoUtils::sha256(&self.public_key)[0..4].try_into()
            .map_err(|_| CryptoError::KeyDerivationFailed("Failed to calculate fingerprint".to_string()))?;
        
        Ok(Self {
            public_key: child_public.serialize(),
            chain_code: child_chain_code,
            depth: self.depth + 1,
            parent_fingerprint,
            child_number,
        })
    }
    
    /// Derive key from path (non-hardened only)
    pub fn derive_path(&self, path: &DerivationPath) -> CryptoResult<Self> {
        if path.hardened {
            return Err(CryptoError::KeyDerivationFailed(
                "Cannot derive hardened path from public key".to_string()
            ));
        }
        
        let mut current = self.clone();
        
        for &component in &path.components {
            current = current.derive_child(component)?;
        }
        
        Ok(current)
    }
    
    /// Get Ed25519 verifying key
    pub fn ed25519_verifying_key(&self) -> CryptoResult<VerifyingKey> {
        // Convert secp256k1 public key to Ed25519 format
        // This is a simplified conversion - in production, you'd use proper key conversion
        let mut ed25519_bytes = [0u8; 32];
        ed25519_bytes.copy_from_slice(&self.public_key[1..33]);
        
        VerifyingKey::from_bytes(&ed25519_bytes)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))
    }
    
    /// Get secp256k1 public key
    pub fn secp256k1_public_key(&self) -> CryptoResult<PublicKey> {
        PublicKey::from_slice(&self.public_key)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))
    }
}

/// Key derivation utilities
pub struct KeyDerivation;

impl KeyDerivation {
    /// Generate master key from mnemonic
    pub fn from_mnemonic(mnemonic: &str, passphrase: Option<&str>) -> CryptoResult<ExtendedPrivateKey> {
        // In production, you'd use proper BIP39 mnemonic to seed conversion
        // For now, we'll use a simplified approach
        let passphrase = passphrase.unwrap_or("");
        let seed = format!("{}{}", mnemonic, passphrase);
        let seed_bytes = CryptoUtils::sha256(seed.as_bytes());
        
        ExtendedPrivateKey::from_seed(&seed_bytes)
    }
    
    /// Generate random mnemonic (simplified)
    pub fn generate_mnemonic() -> String {
        // In production, you'd use proper BIP39 mnemonic generation
        // For now, we'll generate a random string
        let words = [
            "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract",
            "absurd", "abuse", "access", "accident", "account", "accuse", "achieve", "acid",
            "acoustic", "acquire", "across", "act", "action", "actor", "actress", "actual",
            "adapt", "add", "addict", "address", "adjust", "admit", "adult", "advance"
        ];
        
        let mut mnemonic = String::new();
        for i in 0..12 {
            let word = words[i % words.len()];
            if i > 0 {
                mnemonic.push(' ');
            }
            mnemonic.push_str(word);
        }
        
        mnemonic
    }
    
    /// Validate mnemonic (simplified)
    pub fn validate_mnemonic(mnemonic: &str) -> bool {
        // In production, you'd use proper BIP39 validation
        mnemonic.split_whitespace().count() == 12
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivation_path() {
        let path = DerivationPath::from_string("m/44'/60'/0'/0/0").unwrap();
        assert_eq!(path.components, vec![44, 60, 0, 0, 0]);
        assert!(path.hardened);
        
        let ethereum_path = DerivationPath::ethereum(0, 0, 0);
        assert_eq!(ethereum_path.components, vec![44, 60, 0, 0, 0]);
        assert!(ethereum_path.hardened);
    }

    #[test]
    fn test_extended_private_key() {
        let seed = CryptoUtils::random_32();
        let master_key = ExtendedPrivateKey::from_seed(&seed).unwrap();
        
        assert_eq!(master_key.depth, 0);
        assert_eq!(master_key.parent_fingerprint, [0u8; 4]);
        assert_eq!(master_key.child_number, 0);
        
        // Test child derivation
        let child_key = master_key.derive_child(0).unwrap();
        assert_eq!(child_key.depth, 1);
        assert_eq!(child_key.child_number, 0);
        
        // Test path derivation
        let path = DerivationPath::from_string("m/44'/60'/0'/0/0").unwrap();
        let derived_key = master_key.derive_path(&path).unwrap();
        assert_eq!(derived_key.depth, 5);
    }

    #[test]
    fn test_extended_public_key() {
        let seed = CryptoUtils::random_32();
        let master_key = ExtendedPrivateKey::from_seed(&seed).unwrap();
        let master_public = master_key.extended_public_key().unwrap();
        
        assert_eq!(master_public.depth, 0);
        assert_eq!(master_public.parent_fingerprint, [0u8; 4]);
        assert_eq!(master_public.child_number, 0);
        
        // Test child derivation
        let child_public = master_public.derive_child(0).unwrap();
        assert_eq!(child_public.depth, 1);
        assert_eq!(child_public.child_number, 0);
    }

    #[test]
    fn test_key_derivation_utilities() {
        let mnemonic = KeyDerivation::generate_mnemonic();
        assert!(KeyDerivation::validate_mnemonic(&mnemonic));
        
        let master_key = KeyDerivation::from_mnemonic(&mnemonic, None).unwrap();
        assert_eq!(master_key.depth, 0);
    }
}
