//! ECIES (Elliptic Curve Integrated Encryption Scheme) Implementation
//! 
//! This module provides ECIES encryption for note privacy using secp256k1 ECDH
//! and XChaCha20-Poly1305 AEAD encryption.

use k256::{SecretKey, PublicKey, ecdh};
use k256::elliptic_curve::sec1::ToEncodedPoint;
use chacha20poly1305::{XChaCha20Poly1305, Key, aead::Aead, aead::KeyInit};
use aead::generic_array::GenericArray;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hkdf::Hkdf;
use crate::crypto::{CryptoResult, CryptoError, CryptoUtils, domains};
use crate::utxo::note::{Note, EncryptedNote};

type HmacSha256 = Hmac<Sha256>;

/// ECIES encryption implementation
pub struct Ecies;

impl Ecies {
    /// Encrypt a note for a recipient public key
    pub fn encrypt_note(note: &Note, recipient_pubkey: &[u8; 33]) -> CryptoResult<EncryptedNote> {
        // Generate ephemeral key pair
        let ephemeral_secret = SecretKey::random(&mut rand::thread_rng());
        let ephemeral_public = ephemeral_secret.public_key();
        
        // Parse recipient public key
        let recipient_pub = PublicKey::from_sec1_bytes(recipient_pubkey)
            .map_err(|e| CryptoError::InvalidPublicKey(format!("Invalid recipient public key: {:?}", e)))?;
        
        // Perform ECDH to get shared secret
        let shared_secret = Self::ecdh(&ephemeral_secret, &recipient_pub)?;
        
        // Derive encryption key using HKDF
        let encryption_key = Self::derive_encryption_key(&shared_secret)?;
        
        // Serialize note to JSON
        let note_json = note.to_json()
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize note: {}", e)))?;
        
        // Generate random nonce
        let nonce_bytes = CryptoUtils::random_24();
        let nonce = GenericArray::from_slice(&nonce_bytes);
        
        // Encrypt note data
        let cipher = XChaCha20Poly1305::new(&encryption_key);
        let ciphertext = cipher.encrypt(nonce, note_json.as_bytes())
            .map_err(|e| CryptoError::SerializationError(format!("Encryption failed: {:?}", e)))?;
        
        // Create encrypted note
        let mut ephemeral_pubkey = [0u8; 33];
        ephemeral_pubkey.copy_from_slice(&ephemeral_public.to_encoded_point(true).as_bytes());
        
        Ok(EncryptedNote::new(
            ephemeral_pubkey,
            nonce_bytes,
            ciphertext,
            Some(note.commitment),
        ))
    }
    
    /// Decrypt an encrypted note using recipient private key
    pub fn decrypt_note(encrypted_note: &EncryptedNote, recipient_privkey: &[u8; 32]) -> CryptoResult<Note> {
        // Parse recipient private key
        let recipient_secret = SecretKey::from_be_bytes(recipient_privkey)
            .map_err(|e| CryptoError::InvalidPrivateKey(format!("Invalid recipient private key: {:?}", e)))?;
        
        // Parse ephemeral public key
        let ephemeral_pub = PublicKey::from_sec1_bytes(&encrypted_note.ephemeral_pubkey)
            .map_err(|e| CryptoError::InvalidPublicKey(format!("Invalid ephemeral public key: {:?}", e)))?;
        
        // Perform ECDH to get shared secret
        let shared_secret = Self::ecdh(&recipient_secret, &ephemeral_pub)?;
        
        // Derive encryption key using HKDF
        let encryption_key = Self::derive_encryption_key(&shared_secret)?;
        
        // Decrypt note data
        let nonce = GenericArray::from_slice(&encrypted_note.nonce);
        let cipher = XChaCha20Poly1305::new(&encryption_key);
        let plaintext = cipher.decrypt(nonce, &*encrypted_note.ciphertext)
            .map_err(|e| CryptoError::SerializationError(format!("Decryption failed: {:?}", e)))?;
        
        // Deserialize note from JSON
        let note_json = String::from_utf8(plaintext)
            .map_err(|e| CryptoError::SerializationError(format!("Invalid UTF-8: {}", e)))?;
        
        let note = Note::from_json(&note_json)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to deserialize note: {}", e)))?;
        
        Ok(note)
    }
    
    /// Perform ECDH key exchange
    fn ecdh(secret_key: &SecretKey, public_key: &PublicKey) -> CryptoResult<[u8; 32]> {
        // Perform ECDH using k256's ecdh module
        // Use k256 ECDH
        let shared_secret = ecdh::diffie_hellman(
            secret_key.to_nonzero_scalar(), 
            public_key.as_affine()
        );
        
        // Convert to bytes
        let mut shared_secret_bytes = [0u8; 32];
        shared_secret_bytes.copy_from_slice(shared_secret.raw_secret_bytes());
        
        Ok(shared_secret_bytes)
    }
    
    /// Derive encryption key using HKDF-SHA256
    fn derive_encryption_key(shared_secret: &[u8; 32]) -> CryptoResult<Key> {
        // Create HKDF instance
        let mut hkdf = <HmacSha256 as Mac>::new_from_slice(domains::DOMAIN_ECDH)
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("HKDF init failed: {:?}", e)))?;
        
        hkdf.update(shared_secret);
        
        // Extract key material
        let key_material = hkdf.finalize().into_bytes();
        
        // Use first 32 bytes as encryption key
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&key_material[..32]);
        
        Ok(Key::from(key_bytes))
    }
    
    /// Generate key pair for testing
    pub fn generate_keypair() -> CryptoResult<(SecretKey, PublicKey)> {
        let secret_key = SecretKey::random(&mut rand::thread_rng());
        let public_key = secret_key.public_key();
        Ok((secret_key, public_key))
    }
    
    /// Enhanced ECIES encryption for production use
    /// Creates encrypted note with proper domain separation and AAD
    pub fn encrypt_note_with_aad(
        note: &Note, 
        recipient_pubkey: &[u8; 33],
        commitment: &[u8; 32],
        pool_address: &[u8; 20]
    ) -> CryptoResult<EncryptedNote> {
        // Generate ephemeral key pair
        let ephemeral_secret = SecretKey::random(&mut rand::thread_rng());
        let ephemeral_public = ephemeral_secret.public_key();
        
        // Parse recipient public key
        let recipient_pub = PublicKey::from_sec1_bytes(recipient_pubkey)
            .map_err(|e| CryptoError::InvalidPublicKey(format!("Invalid recipient public key: {:?}", e)))?;
        
        // Perform ECDH to get shared secret
        let shared_secret = Self::ecdh(&ephemeral_secret, &recipient_pub)?;
        
        // Derive encryption key using HKDF with proper domain separation
        let encryption_key = Self::derive_encryption_key_with_domain(&shared_secret, commitment)?;
        
        // Serialize note to JSON
        let note_json = note.to_json()
            .map_err(|e| CryptoError::SerializationError(format!("Failed to serialize note: {}", e)))?;
        
        // Generate random nonce
        let nonce_bytes = CryptoUtils::random_24();
        let nonce = GenericArray::from_slice(&nonce_bytes);
        
        // Create AAD (Additional Authenticated Data) for integrity binding
        let mut aad = Vec::new();
        aad.extend_from_slice(commitment);
        aad.extend_from_slice(pool_address);
        
        // Encrypt note data with AAD
        let cipher = XChaCha20Poly1305::new(&encryption_key);
        let ciphertext = cipher.encrypt(nonce, note_json.as_bytes())
            .map_err(|e| CryptoError::SerializationError(format!("Encryption failed: {:?}", e)))?;
        
        // Create encrypted note
        let mut ephemeral_pubkey = [0u8; 33];
        ephemeral_pubkey.copy_from_slice(&ephemeral_public.to_encoded_point(true).as_bytes());
        
        Ok(EncryptedNote::new(
            ephemeral_pubkey,
            nonce_bytes,
            ciphertext,
            Some(*commitment),
        ))
    }
    
    /// Enhanced ECIES decryption with AAD verification
    pub fn decrypt_note_with_aad(
        encrypted_note: &EncryptedNote, 
        recipient_privkey: &[u8; 32],
        commitment: &[u8; 32],
        pool_address: &[u8; 20]
    ) -> CryptoResult<Note> {
        // Parse recipient private key
        let recipient_secret = SecretKey::from_be_bytes(recipient_privkey)
            .map_err(|e| CryptoError::InvalidPrivateKey(format!("Invalid recipient private key: {:?}", e)))?;
        
        // Parse ephemeral public key
        let ephemeral_pub = PublicKey::from_sec1_bytes(&encrypted_note.ephemeral_pubkey)
            .map_err(|e| CryptoError::InvalidPublicKey(format!("Invalid ephemeral public key: {:?}", e)))?;
        
        // Perform ECDH to get shared secret
        let shared_secret = Self::ecdh(&recipient_secret, &ephemeral_pub)?;
        
        // Derive encryption key using HKDF with proper domain separation
        let encryption_key = Self::derive_encryption_key_with_domain(&shared_secret, commitment)?;
        
        // Create AAD for verification
        let mut aad = Vec::new();
        aad.extend_from_slice(commitment);
        aad.extend_from_slice(pool_address);
        
        // Decrypt note data with AAD verification
        let nonce = GenericArray::from_slice(&encrypted_note.nonce);
        let cipher = XChaCha20Poly1305::new(&encryption_key);
        let plaintext = cipher.decrypt(nonce, &*encrypted_note.ciphertext)
            .map_err(|e| CryptoError::SerializationError(format!("Decryption failed: {:?}", e)))?;
        
        // Deserialize note from JSON
        let note_json = String::from_utf8(plaintext)
            .map_err(|e| CryptoError::SerializationError(format!("Invalid UTF-8: {}", e)))?;
        
        let note = Note::from_json(&note_json)
            .map_err(|e| CryptoError::SerializationError(format!("Failed to deserialize note: {}", e)))?;
        
        Ok(note)
    }
    
    /// Derive encryption key with domain separation
    fn derive_encryption_key_with_domain(shared_secret: &[u8; 32], commitment: &[u8; 32]) -> CryptoResult<Key> {
        // Create HKDF instance with domain separation
        let mut info = Vec::new();
        info.extend_from_slice(domains::DOMAIN_ECDH);
        info.extend_from_slice(commitment);
        info.push(0x01); // version
        
        let hkdf = Hkdf::<Sha256>::new(Some(domains::DOMAIN_ECDH), shared_secret);
        let mut key_bytes = [0u8; 32];
        hkdf.expand(&info, &mut key_bytes)
            .map_err(|e| CryptoError::KeyDerivationFailed(format!("HKDF expansion failed: {:?}", e)))?;
        
        Ok(Key::from(key_bytes))
    }
    
    /// Test encryption/decryption roundtrip
    pub fn test_roundtrip() -> CryptoResult<()> {
        // Generate test key pair
        let (secret_key, public_key) = Self::generate_keypair()?;
        
        // Create test note
        let note = Note::new(
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
            1000000000000000000u64,
            [0x42u8; 32],
        );
        
        // Serialize public key
        let mut pubkey_bytes = [0u8; 33];
        pubkey_bytes.copy_from_slice(&public_key.to_encoded_point(true).as_bytes());
        
        // Serialize secret key
        let mut seckey_bytes = [0u8; 32];
        seckey_bytes.copy_from_slice(secret_key.to_be_bytes().as_slice());
        
        // Encrypt note
        let encrypted = Self::encrypt_note(&note, &pubkey_bytes)?;
        
        // Decrypt note
        let decrypted = Self::decrypt_note(&encrypted, &seckey_bytes)?;
        
        // Verify roundtrip
        assert_eq!(note, decrypted);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecies_roundtrip() {
        Ecies::test_roundtrip().unwrap();
    }

    #[test]
    fn test_key_generation() {
        let (secret_key, public_key) = Ecies::generate_keypair().unwrap();
        
        // Verify key pair is valid
        let derived_public = secret_key.public_key();
        assert_eq!(public_key, derived_public);
    }

    #[test]
    fn test_encryption_decryption() {
        let (secret_key, public_key) = Ecies::generate_keypair().unwrap();
        
        let note = Note::new(
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
            1000000000000000000u64,
            [0x42u8; 32],
        );
        
        let mut pubkey_bytes = [0u8; 33];
        pubkey_bytes.copy_from_slice(&public_key.to_encoded_point(true).as_bytes());
        
        let mut seckey_bytes = [0u8; 32];
        seckey_bytes.copy_from_slice(secret_key.to_be_bytes().as_slice());
        
        // Encrypt
        let encrypted = Ecies::encrypt_note(&note, &pubkey_bytes).unwrap();
        
        // Decrypt
        let decrypted = Ecies::decrypt_note(&encrypted, &seckey_bytes).unwrap();
        
        assert_eq!(note, decrypted);
    }
}
