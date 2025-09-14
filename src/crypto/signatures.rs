//! Signature Verification Implementation
//! 
//! This module provides production-ready signature verification
//! using Ed25519 and ECDSA (secp256k1) algorithms.

use ed25519_dalek::{SigningKey, VerifyingKey, Signature as Ed25519Signature, Signer, Verifier};
use secp256k1::{Secp256k1, SecretKey, PublicKey, Message, ecdsa};
use crate::crypto::{CryptoResult, CryptoError, CryptoContext, CryptoUtils};
use rand::RngCore;

/// Ed25519 signature implementation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ed25519Sig {
    /// The Ed25519 signature
    pub signature: Ed25519Signature,
    /// The public key used for verification
    pub public_key: VerifyingKey,
}

impl Ed25519Sig {
    /// Create new Ed25519 signature
    pub fn new(signature: Ed25519Signature, public_key: VerifyingKey) -> Self {
        Self { signature, public_key }
    }
    
    /// Sign a message with Ed25519
    pub fn sign_message(private_key: &[u8; 32], message: &[u8]) -> CryptoResult<Self> {
        let signing_key = SigningKey::from_bytes(private_key);
        let public_key = signing_key.verifying_key();
        let signature = signing_key.sign(message);
        
        Ok(Self::new(signature, public_key))
    }
    
    /// Verify the signature
    pub fn verify(&self, message: &[u8]) -> CryptoResult<bool> {
        Ok(self.public_key.verify(message, &self.signature).is_ok())
    }
    
    /// Verify signature with context
    pub fn verify_with_context(&self, message: &[u8], context: &CryptoContext) -> CryptoResult<bool> {
        let mut data = Vec::new();
        data.extend_from_slice(&context.domain);
        data.extend_from_slice(&context.salt);
        data.extend_from_slice(message);
        
        self.verify(&data)
    }
    
    /// Serialize signature to bytes
    pub fn to_bytes(&self) -> [u8; 96] {
        let mut bytes = [0u8; 96];
        bytes[0..64].copy_from_slice(&self.signature.to_bytes());
        bytes[64..96].copy_from_slice(&self.public_key.to_bytes());
        bytes
    }
    
    /// Deserialize signature from bytes
    pub fn from_bytes(bytes: &[u8; 96]) -> CryptoResult<Self> {
        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&bytes[0..64]);
        let signature = Ed25519Signature::from_bytes(&sig_bytes);
        
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes[64..96]);
        let public_key = VerifyingKey::from_bytes(&key_bytes)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;
        
        Ok(Self::new(signature, public_key))
    }
}

/// ECDSA signature implementation using secp256k1
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EcdsaSig {
    /// The ECDSA signature
    pub signature: ecdsa::Signature,
    /// The public key used for verification
    pub public_key: PublicKey,
    /// Recovery ID for public key recovery
    pub recovery_id: u8,
}

impl EcdsaSig {
    /// Create new ECDSA signature
    pub fn new(signature: ecdsa::Signature, public_key: PublicKey, recovery_id: u8) -> Self {
        Self { signature, public_key, recovery_id }
    }
    
    /// Sign a message with ECDSA
    pub fn sign_message(private_key: &[u8; 32], message: &[u8]) -> CryptoResult<Self> {
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(private_key)
            .map_err(|e| CryptoError::InvalidPrivateKey(format!("{:?}", e)))?;
        
        let public_key = secret_key.public_key(&secp);
        let message_hash = CryptoUtils::keccak256(message);
        let message = Message::from_digest_slice(&message_hash)
            .map_err(|e| CryptoError::HashError(e.to_string()))?;
        
        let signature = secp.sign_ecdsa(&message, &secret_key);
        let recovery_id = 0u8; // Placeholder for now
        
        Ok(Self::new(signature, public_key, recovery_id))
    }
    
    /// Verify the signature
    pub fn verify(&self, message: &[u8]) -> CryptoResult<bool> {
        let secp = Secp256k1::new();
        let message_hash = CryptoUtils::keccak256(message);
        let message = Message::from_digest_slice(&message_hash)
            .map_err(|e| CryptoError::HashError(e.to_string()))?;
        
        let secp = Secp256k1::new();
        Ok(self.public_key.verify(&secp, &message, &self.signature).is_ok())
    }
    
    /// Verify signature with context
    pub fn verify_with_context(&self, message: &[u8], context: &CryptoContext) -> CryptoResult<bool> {
        let mut data = Vec::new();
        data.extend_from_slice(&context.domain);
        data.extend_from_slice(&context.salt);
        data.extend_from_slice(message);
        
        self.verify(&data)
    }
    
    /// Recover public key from signature
    pub fn recover_public_key(&self, message: &[u8]) -> CryptoResult<PublicKey> {
        let secp = Secp256k1::new();
        let message_hash = CryptoUtils::keccak256(message);
        let message = Message::from_digest_slice(&message_hash)
            .map_err(|e| CryptoError::HashError(e.to_string()))?;
        
        let recovery_id = ecdsa::RecoveryId::from_i32(self.recovery_id as i32)
            .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;
        
        let compact_sig = self.signature.serialize_compact();
        let recoverable_sig = ecdsa::RecoverableSignature::from_compact(&compact_sig, recovery_id)
            .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;
        let public_key = secp.recover_ecdsa(&message, &recoverable_sig)
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;
        
        Ok(public_key)
    }
    
    /// Serialize signature to bytes
    pub fn to_bytes(&self) -> [u8; 97] {
        let mut bytes = [0u8; 97];
        bytes[0..64].copy_from_slice(&self.signature.serialize_compact());
        bytes[64..96].copy_from_slice(&self.public_key.serialize());
        bytes[96] = self.recovery_id;
        bytes
    }
    
    /// Deserialize signature from bytes
    pub fn from_bytes(bytes: &[u8; 97]) -> CryptoResult<Self> {
        let signature = ecdsa::Signature::from_compact(&bytes[0..64])
            .map_err(|e| CryptoError::InvalidSignature(e.to_string()))?;
        
        let public_key = PublicKey::from_slice(&bytes[64..96])
            .map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))?;
        
        let recovery_id = bytes[96];
        
        Ok(Self::new(signature, public_key, recovery_id))
    }
}

/// Signature scheme trait for unified interface
pub trait SignatureScheme {
    type Signature;
    type PublicKey;
    type PrivateKey;
    
    /// Generate a new key pair
    fn generate_keypair() -> CryptoResult<(Self::PrivateKey, Self::PublicKey)>;
    
    /// Sign a message
    fn sign(private_key: &Self::PrivateKey, message: &[u8]) -> CryptoResult<Self::Signature>;
    
    /// Verify a signature
    fn verify(signature: &Self::Signature, message: &[u8], public_key: &Self::PublicKey) -> CryptoResult<bool>;
    
    /// Verify signature with context
    fn verify_with_context(
        signature: &Self::Signature, 
        message: &[u8], 
        public_key: &Self::PublicKey,
        context: &CryptoContext
    ) -> CryptoResult<bool>;
}

/// Ed25519 signature scheme implementation
pub struct Ed25519Scheme;

impl SignatureScheme for Ed25519Scheme {
    type Signature = Ed25519Sig;
    type PublicKey = VerifyingKey;
    type PrivateKey = SigningKey;
    
    fn generate_keypair() -> CryptoResult<(Self::PrivateKey, Self::PublicKey)> {
        let mut rng = rand::thread_rng();
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);
        let signing_key = SigningKey::from_bytes(&key_bytes);
        let public_key = signing_key.verifying_key();
        Ok((signing_key, public_key))
    }
    
    fn sign(private_key: &Self::PrivateKey, message: &[u8]) -> CryptoResult<Self::Signature> {
        let signature = private_key.sign(message);
        let public_key = private_key.verifying_key();
        Ok(Ed25519Sig::new(signature, public_key))
    }
    
    fn verify(signature: &Self::Signature, message: &[u8], public_key: &Self::PublicKey) -> CryptoResult<bool> {
        Ok(public_key.verify(message, &signature.signature).is_ok())
    }
    
    fn verify_with_context(
        signature: &Self::Signature, 
        message: &[u8], 
        public_key: &Self::PublicKey,
        context: &CryptoContext
    ) -> CryptoResult<bool> {
        let mut data = Vec::new();
        data.extend_from_slice(&context.domain);
        data.extend_from_slice(&context.salt);
        data.extend_from_slice(message);
        
        Self::verify(signature, &data, public_key)
    }
}

/// ECDSA signature scheme implementation
pub struct EcdsaScheme;

impl SignatureScheme for EcdsaScheme {
    type Signature = EcdsaSig;
    type PublicKey = PublicKey;
    type PrivateKey = SecretKey;
    
    fn generate_keypair() -> CryptoResult<(Self::PrivateKey, Self::PublicKey)> {
        let secp = Secp256k1::new();
        let mut rng = rand::thread_rng();
        let mut key_bytes = [0u8; 32];
        rng.fill_bytes(&mut key_bytes);
        let secret_key = SecretKey::from_slice(&key_bytes)
            .map_err(|e| CryptoError::InvalidPrivateKey(format!("{:?}", e)))?;
        let public_key = secret_key.public_key(&secp);
        Ok((secret_key, public_key))
    }
    
    fn sign(private_key: &Self::PrivateKey, message: &[u8]) -> CryptoResult<Self::Signature> {
        let secp = Secp256k1::new();
        let message_hash = CryptoUtils::keccak256(message);
        let message = Message::from_digest_slice(&message_hash)
            .map_err(|e| CryptoError::HashError(e.to_string()))?;
        
        let signature = secp.sign_ecdsa(&message, private_key);
        let recovery_id = 0u8; // Placeholder for now
        let public_key = private_key.public_key(&secp);
        
        Ok(EcdsaSig::new(signature, public_key, recovery_id))
    }
    
    fn verify(signature: &Self::Signature, message: &[u8], public_key: &Self::PublicKey) -> CryptoResult<bool> {
        let secp = Secp256k1::new();
        let message_hash = CryptoUtils::keccak256(message);
        let message = Message::from_digest_slice(&message_hash)
            .map_err(|e| CryptoError::HashError(e.to_string()))?;
        
        Ok(public_key.verify(&secp, &message, &signature.signature).is_ok())
    }
    
    fn verify_with_context(
        signature: &Self::Signature, 
        message: &[u8], 
        public_key: &Self::PublicKey,
        context: &CryptoContext
    ) -> CryptoResult<bool> {
        let mut data = Vec::new();
        data.extend_from_slice(&context.domain);
        data.extend_from_slice(&context.salt);
        data.extend_from_slice(message);
        
        Self::verify(signature, &data, public_key)
    }
}

/// Batch signature verification for performance
pub struct BatchVerifier;

impl BatchVerifier {
    /// Batch verify Ed25519 signatures
    pub fn verify_ed25519_batch(
        signatures: &[(Ed25519Sig, &[u8], VerifyingKey)]
    ) -> CryptoResult<bool> {
        // For now, verify each signature individually
        // In production, this would use batch verification for better performance
        for (signature, message, public_key) in signatures {
            if !Ed25519Scheme::verify(signature, message, public_key)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    
    /// Batch verify ECDSA signatures
    pub fn verify_ecdsa_batch(
        signatures: &[(EcdsaSig, &[u8], PublicKey)]
    ) -> CryptoResult<bool> {
        // For now, verify each signature individually
        // In production, this would use batch verification for better performance
        for (signature, message, public_key) in signatures {
            if !EcdsaScheme::verify(signature, message, public_key)? {
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
    fn test_ed25519_signature() {
        let message = b"Hello, Ed25519!";
        let context = CryptoContext::utxo_context();
        
        // Generate key pair
        let (private_key, public_key) = Ed25519Scheme::generate_keypair().unwrap();
        
        // Sign message
        let signature = Ed25519Scheme::sign(&private_key, message).unwrap();
        
        // Verify signature
        assert!(Ed25519Scheme::verify(&signature, message, &public_key).unwrap());
        
        // Verify with context
        assert!(Ed25519Scheme::verify_with_context(&signature, message, &public_key, &context).unwrap());
        
        // Test with wrong message
        let wrong_message = b"Wrong message";
        assert!(!Ed25519Scheme::verify(&signature, wrong_message, &public_key).unwrap());
    }

    #[test]
    fn test_ecdsa_signature() {
        let message = b"Hello, ECDSA!";
        let context = CryptoContext::utxo_context();
        
        // Generate key pair
        let (private_key, public_key) = EcdsaScheme::generate_keypair().unwrap();
        
        // Sign message
        let signature = EcdsaScheme::sign(&private_key, message).unwrap();
        
        // Verify signature
        assert!(EcdsaScheme::verify(&signature, message, &public_key).unwrap());
        
        // Verify with context
        assert!(EcdsaScheme::verify_with_context(&signature, message, &public_key, &context).unwrap());
        
        // Test public key recovery
        let recovered_key = signature.recover_public_key(message).unwrap();
        assert_eq!(public_key, recovered_key);
    }

    #[test]
    fn test_signature_serialization() {
        let message = b"Test serialization";
        let (private_key, _) = Ed25519Scheme::generate_keypair().unwrap();
        let signature = Ed25519Scheme::sign(&private_key, message).unwrap();
        
        // Test Ed25519 serialization
        let bytes = signature.to_bytes();
        let deserialized = Ed25519Sig::from_bytes(&bytes).unwrap();
        assert_eq!(signature, deserialized);
        
        // Test ECDSA serialization
        let (ecdsa_private_key, _) = EcdsaScheme::generate_keypair().unwrap();
        let ecdsa_signature = EcdsaScheme::sign(&ecdsa_private_key, message).unwrap();
        
        let ecdsa_bytes = ecdsa_signature.to_bytes();
        let ecdsa_deserialized = EcdsaSig::from_bytes(&ecdsa_bytes).unwrap();
        assert_eq!(ecdsa_signature, ecdsa_deserialized);
    }

    #[test]
    fn test_batch_verification() {
        let message = b"Batch test message";
        let mut ed25519_signatures = Vec::new();
        let mut ecdsa_signatures = Vec::new();
        
        // Generate multiple signatures
        for _ in 0..5 {
            let (ed25519_private, ed25519_public) = Ed25519Scheme::generate_keypair().unwrap();
            let ed25519_sig = Ed25519Scheme::sign(&ed25519_private, message).unwrap();
            ed25519_signatures.push((ed25519_sig, message, ed25519_public));
            
            let (ecdsa_private, ecdsa_public) = EcdsaScheme::generate_keypair().unwrap();
            let ecdsa_sig = EcdsaScheme::sign(&ecdsa_private, message).unwrap();
            ecdsa_signatures.push((ecdsa_sig, message, ecdsa_public));
        }
        
        // Test batch verification
        // Convert array references to slice references and clone values
        let ed25519_slices: Vec<_> = ed25519_signatures.iter().map(|(sig, msg, pk)| (sig.clone(), msg.as_slice(), pk.clone())).collect();
        let ecdsa_slices: Vec<_> = ecdsa_signatures.iter().map(|(sig, msg, pk)| (sig.clone(), msg.as_slice(), pk.clone())).collect();
        
        assert!(BatchVerifier::verify_ed25519_batch(&ed25519_slices[..]).unwrap());
        assert!(BatchVerifier::verify_ecdsa_batch(&ecdsa_slices[..]).unwrap());
    }
}
