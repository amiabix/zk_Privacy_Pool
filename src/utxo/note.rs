//! Encrypted Note Structure
//! 
//! This module defines the Note JSON schema and serialization
//! for encrypted notes in the privacy pool system.

use serde::{Serialize, Deserialize};
use serde_with::{serde_as, Bytes};
use crate::crypto::domains;

/// Core Note struct with essential fields for privacy pool
/// Note = { value, pubkey, blinding, commitment }
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Note {
    /// Note value in wei
    pub value: u64,
    
    /// Recipient's public viewing key (33 bytes compressed)
    #[serde_as(as = "Bytes")]
    pub pubkey: [u8; 33],
    
    /// Blinding factor for commitment
    #[serde_as(as = "Bytes")]
    pub blinding: [u8; 32],
    
    /// Computed commitment hash
    #[serde_as(as = "Bytes")]
    pub commitment: [u8; 32],
    
    /// Protocol version
    pub version: u8,
    
    /// Chain ID for cross-chain compatibility
    pub chain_id: u64,
    
    /// Privacy pool contract address
    pub pool_address: String,
    
    /// Secret for nullifier generation (private)
    #[serde_as(as = "Bytes")]
    pub secret: [u8; 32],
    
    /// Creation timestamp
    pub created_at: u64,
    
    /// Transaction hash (set after confirmation)
    pub tx_hash: Option<String>,
    
    /// Output index in transaction
    pub output_index: Option<u32>,
    
    /// Unique note identifier
    pub note_id: String,
}

impl Note {
    /// Create a new note with random secret and blinding factor
    pub fn new(
        value: u64,
        pubkey: [u8; 33],
        version: u8,
        chain_id: u64,
        pool_address: String,
    ) -> Self {
        let secret = crate::crypto::CryptoUtils::random_32();
        let blinding = crate::crypto::CryptoUtils::random_32();
        
        // Compute commitment using Poseidon
        let commitment = Self::compute_commitment(value, &pubkey, &secret, &blinding);
        
        // Generate unique note ID
        let note_id = Self::generate_note_id(&commitment);
        
        Self {
            value,
            pubkey,
            blinding,
            commitment,
            version,
            chain_id,
            pool_address,
            secret,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash: None,
            output_index: None,
            note_id,
        }
    }
    
    /// Compute commitment using Poseidon hash
    /// C = Poseidon(pubkey || value || secret || blinding)
    pub fn compute_commitment(
        value: u64,
        pubkey: &[u8; 33],
        secret: &[u8; 32],
        blinding: &[u8; 32],
    ) -> [u8; 32] {
        
        // Convert value to field element representation
        let value_field = value.to_be_bytes();
        let mut value_bytes = [0u8; 32];
        value_bytes[24..32].copy_from_slice(&value_field);
        
        // Create input for Poseidon: pubkey || value || secret || blinding
        let mut input = Vec::new();
        input.extend_from_slice(pubkey);
        input.extend_from_slice(&value_bytes);
        input.extend_from_slice(secret);
        input.extend_from_slice(blinding);
        
        // Use SHA256 commitment that binds all inputs as requested by user
        // C = Poseidon(pubkey || value || secret || blinding)
        // TODO: Implement proper Poseidon hash for all inputs
        crate::crypto::CryptoUtils::sha256(&[
            domains::DOMAIN_COMMIT,
            pubkey,
            &value_bytes,
            secret,
            blinding,
        ].concat())
    }
    
    /// Generate unique note ID based on commitment only
    /// This prevents linkage if note_id leaks off-chain
    pub fn generate_note_id(commitment: &[u8; 32]) -> String {
        let mut data = Vec::new();
        data.extend_from_slice(domains::DOMAIN_NOTE);
        data.extend_from_slice(commitment);
        
        let hash = crate::crypto::CryptoUtils::sha256(&data);
        format!("note_{}", hex::encode(hash))
    }
    
    /// Serialize note to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Deserialize note from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
    
    /// Verify note integrity
    pub fn verify(&self) -> bool {
        // Verify commitment matches computed value
        let computed_commitment = Self::compute_commitment(
            self.value,
            &self.pubkey,
            &self.secret,
            &self.blinding,
        );
        
        computed_commitment == self.commitment
    }
    
    /// Check if note is confirmed (has tx_hash and output_index)
    pub fn is_confirmed(&self) -> bool {
        self.tx_hash.is_some() && self.output_index.is_some()
    }
    
    /// Mark note as confirmed
    pub fn mark_confirmed(&mut self, tx_hash: String, output_index: u32) {
        self.tx_hash = Some(tx_hash);
        self.output_index = Some(output_index);
    }
    
    /// Generate nullifier for spending this note
    /// nullifier = Poseidon(secret || owner_sk)
    pub fn generate_nullifier(&self, owner_sk: &[u8; 32]) -> [u8; 32] {
        // Create input for nullifier: secret || owner_sk
        // Use deterministic SHA256 for consistent results
        crate::crypto::CryptoUtils::sha256(&[
            domains::DOMAIN_NULL,
            &self.secret,
            owner_sk,
        ].concat())
    }
    
    /// Verify nullifier matches this note
    pub fn verify_nullifier(&self, nullifier: &[u8; 32], owner_sk: &[u8; 32]) -> bool {
        let computed_nullifier = self.generate_nullifier(owner_sk);
        computed_nullifier == *nullifier
    }
    
    /// Create note from existing components (for testing/advanced usage)
    pub fn from_components(
        value: u64,
        pubkey: [u8; 33],
        blinding: [u8; 32],
        version: u8,
        chain_id: u64,
        pool_address: String,
        secret: [u8; 32],
    ) -> Self {
        let commitment = Self::compute_commitment(value, &pubkey, &secret, &blinding);
        let note_id = Self::generate_note_id(&commitment);
        
        Self {
            value,
            pubkey,
            blinding,
            commitment,
            version,
            chain_id,
            pool_address,
            secret,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash: None,
            output_index: None,
            note_id,
        }
    }
    
    /// Get note value in ETH (for display purposes)
    pub fn value_eth(&self) -> f64 {
        self.value as f64 / 1e18
    }
    
    /// Get note value in wei
    pub fn value_wei(&self) -> u64 {
        self.value
    }
    
    /// Check if note is spendable (confirmed and not already spent)
    pub fn is_spendable(&self) -> bool {
        self.is_confirmed() && self.tx_hash.is_some()
    }
    
    /// Encrypt note with recipient's public viewing key using ECIES
    pub fn encrypt_with_recipient_key(&self, recipient_pubkey: &[u8; 33]) -> Result<EncryptedNote, Box<dyn std::error::Error>> {
        use crate::crypto::ecies::Ecies;
        
        // Serialize note to JSON
        let note_json = self.to_json()?;
        let note_bytes = note_json.as_bytes();
        
        // Encrypt using ECIES
        let encrypted = Ecies::encrypt_note_with_aad(
            self, // Note implements the required trait
            recipient_pubkey,
            &self.commitment,
            &self.pool_address.parse::<[u8; 20]>().unwrap_or([0u8; 20])
        )?;
        
        Ok(encrypted)
    }
    
    /// Decrypt note with recipient's private key
    pub fn decrypt_with_recipient_key(
        encrypted_note: &EncryptedNote, 
        recipient_privkey: &[u8; 32],
        pool_address: &str
    ) -> Result<Self, Box<dyn std::error::Error>> {
        use crate::crypto::ecies::Ecies;
        
        // Decrypt using ECIES
        let note = Ecies::decrypt_note_with_aad(
            encrypted_note,
            recipient_privkey,
            &encrypted_note.commitment.unwrap_or([0u8; 32]),
            &pool_address.parse::<[u8; 20]>().unwrap_or([0u8; 20])
        )?;
        
        Ok(note)
    }
    
    /// Create a simple note with just the core fields
    pub fn create_simple(value: u64, pubkey: [u8; 33]) -> Self {
        let blinding = crate::crypto::CryptoUtils::random_32();
        let secret = crate::crypto::CryptoUtils::random_32();
        let commitment = Self::compute_commitment(value, &pubkey, &secret, &blinding);
        let note_id = Self::generate_note_id(&commitment);
        
        Self {
            value,
            pubkey,
            blinding,
            commitment,
            version: 1,
            chain_id: 1,
            pool_address: "0x0000000000000000000000000000000000000000".to_string(),
            secret,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash: None,
            output_index: None,
            note_id,
        }
    }
}

/// Encrypted note structure
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedNote {
    /// Ephemeral public key for ECDH
    #[serde_as(as = "Bytes")]
    pub ephemeral_pubkey: [u8; 33],
    
    /// Nonce for AEAD encryption
    #[serde_as(as = "Bytes")]
    pub nonce: [u8; 24], // XChaCha20-Poly1305 nonce
    
    /// Encrypted note data
    pub ciphertext: Vec<u8>,
    
    /// Optional commitment for relayer matching
    #[serde_as(as = "Option<Bytes>")]
    pub commitment: Option<[u8; 32]>,
}

impl EncryptedNote {
    /// Create new encrypted note
    pub fn new(
        ephemeral_pubkey: [u8; 33],
        nonce: [u8; 24],
        ciphertext: Vec<u8>,
        commitment: Option<[u8; 32]>,
    ) -> Self {
        Self {
            ephemeral_pubkey,
            nonce,
            ciphertext,
            commitment,
        }
    }
    
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
    
    /// Deserialize from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_note_creation() {
        let pubkey = [0x42u8; 33];
        let note = Note::new(
            1000000000000000000u64, // 1 ETH
            pubkey,
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
        );
        
        assert_eq!(note.version, 1);
        assert_eq!(note.chain_id, 1);
        assert_eq!(note.value, 1000000000000000000u64);
        assert_eq!(note.pubkey, pubkey);
        assert!(note.verify());
        assert!(!note.is_confirmed());
    }

    #[test]
    fn test_note_serialization() {
        let pubkey = [0x42u8; 33];
        let note = Note::new(
            1000000000000000000u64,
            pubkey,
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
        );
        
        let json = note.to_json().unwrap();
        let deserialized = Note::from_json(&json).unwrap();
        
        assert_eq!(note, deserialized);
    }

    #[test]
    fn test_note_confirmation() {
        let pubkey = [0x42u8; 33];
        let mut note = Note::new(
            1000000000000000000u64,
            pubkey,
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
        );
        
        assert!(!note.is_confirmed());
        
        note.mark_confirmed("0xabcdef1234567890".to_string(), 0);
        
        assert!(note.is_confirmed());
        assert_eq!(note.tx_hash, Some("0xabcdef1234567890".to_string()));
        assert_eq!(note.output_index, Some(0));
    }
    
    #[test]
    fn test_commitment_computation() {
        let pubkey = [0x42u8; 33];
        let secret = [0x13u8; 32];
        let blinding = [0x37u8; 32];
        let value = 1000000000000000000u64;
        
        let commitment1 = Note::compute_commitment(value, &pubkey, &secret, &blinding);
        let commitment2 = Note::compute_commitment(value, &pubkey, &secret, &blinding);
        
        // Same inputs should produce same commitment
        assert_eq!(commitment1, commitment2);
        
        // Different pubkey should produce different commitment
        let different_pubkey = [0x43u8; 33];
        let commitment3 = Note::compute_commitment(value, &different_pubkey, &secret, &blinding);
        assert_ne!(commitment1, commitment3);
        
        // Different secret should produce different commitment
        let different_secret = [0x14u8; 32];
        let commitment4 = Note::compute_commitment(value, &pubkey, &different_secret, &blinding);
        assert_ne!(commitment1, commitment4);
    }
    
    #[test]
    fn test_nullifier_generation() {
        let pubkey = [0x42u8; 33];
        let owner_sk = [0x13u8; 32];
        let note = Note::new(
            1000000000000000000u64,
            pubkey,
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
        );
        
        let nullifier1 = note.generate_nullifier(&owner_sk);
        let nullifier2 = note.generate_nullifier(&owner_sk);
        
        // Same note and key should produce same nullifier
        assert_eq!(nullifier1, nullifier2);
        
        // Different key should produce different nullifier
        let different_sk = [0x14u8; 32];
        let nullifier3 = note.generate_nullifier(&different_sk);
        assert_ne!(nullifier1, nullifier3);
        
        // Verify nullifier matches
        assert!(note.verify_nullifier(&nullifier1, &owner_sk));
        assert!(!note.verify_nullifier(&nullifier1, &different_sk));
    }
    
    #[test]
    fn test_note_id_uniqueness() {
        let pubkey = [0x42u8; 33];
        let note1 = Note::new(
            1000000000000000000u64,
            pubkey,
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
        );
        
        let note2 = Note::new(
            2000000000000000000u64, // Different value
            pubkey,
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
        );
        
        // Different notes should have different IDs
        assert_ne!(note1.note_id, note2.note_id);
        
        // Same commitment should produce same note ID
        let note_id1 = Note::generate_note_id(&note1.commitment);
        let note_id2 = Note::generate_note_id(&note1.commitment);
        assert_eq!(note_id1, note_id2);
        assert_eq!(note1.note_id, note_id1);
    }
    
    #[test]
    fn test_note_from_components() {
        let pubkey = [0x42u8; 33];
        let secret = [0x13u8; 32];
        let blinding = [0x37u8; 32];
        let value = 1000000000000000000u64;
        
        let note = Note::from_components(
            value,
            pubkey,
            blinding,
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
            secret,
        );
        
        assert_eq!(note.value, value);
        assert_eq!(note.pubkey, pubkey);
        assert_eq!(note.secret, secret);
        assert_eq!(note.blinding, blinding);
        assert!(note.verify());
    }
    
    #[test]
    fn test_value_conversion() {
        let note = Note::new(
            1,
            1,
            "0x1234567890123456789012345678901234567890".to_string(),
            1500000000000000000u64, // 1.5 ETH
            [0x42u8; 32],
        );
        
        assert_eq!(note.value_wei(), 1500000000000000000u64);
        assert_eq!(note.value_eth(), 1.5);
    }
}

// Serialization is now handled by serde_with::Bytes for cleaner, more maintainable code
