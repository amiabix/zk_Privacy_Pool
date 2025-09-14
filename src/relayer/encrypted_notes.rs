//! Encrypted Notes Relayer Service
//! 
//! This module handles encrypted note storage, retrieval, and blockchain integration
//! for the privacy pool relayer system.

use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use crate::utxo::note::EncryptedNote;
use crate::database::{DatabaseManager, schema::cf_names};
use crate::merkle::enhanced_merkle_tree::EnhancedMerkleTree;

/// Encrypted note storage entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedNoteEntry {
    /// Unique note ID
    pub note_id: String,
    
    /// Ephemeral public key
    #[serde(serialize_with = "serialize_array33", deserialize_with = "deserialize_array33")]
    pub ephemeral_pubkey: [u8; 33],
    
    /// Nonce for decryption
    pub nonce: [u8; 24],
    
    /// Encrypted note data
    pub ciphertext: Vec<u8>,
    
    /// Optional commitment for matching
    pub commitment: Option<[u8; 32]>,
    
    /// Upload timestamp
    pub uploaded_at: u64,
    
    /// Transaction hash (set after confirmation)
    pub tx_hash: Option<String>,
    
    /// Output index (set after confirmation)
    pub output_index: Option<u32>,
    
    /// Leaf index in Merkle tree (set after confirmation)
    pub leaf_index: Option<u64>,
}

/// Relayer service for encrypted notes
pub struct EncryptedNotesRelayer {
    /// Database manager
    db: DatabaseManager,
    
    /// Merkle tree for commitment tracking
    merkle_tree: EnhancedMerkleTree,
    
    /// Note ID to entry mapping
    note_cache: HashMap<String, EncryptedNoteEntry>,
}

impl EncryptedNotesRelayer {
    /// Create new encrypted notes relayer
    pub fn new(db: DatabaseManager) -> Result<Self> {
        let merkle_tree = EnhancedMerkleTree::new();
        
        Ok(Self {
            db,
            merkle_tree,
            note_cache: HashMap::new(),
        })
    }
    
    /// Upload encrypted note to relayer
    pub fn upload_note(&mut self, encrypted_note: EncryptedNote) -> Result<String> {
        // Generate unique note ID
        let note_id = self.generate_note_id(&encrypted_note);
        
        // Create storage entry
        let entry = EncryptedNoteEntry {
            note_id: note_id.clone(),
            ephemeral_pubkey: encrypted_note.ephemeral_pubkey,
            nonce: encrypted_note.nonce,
            ciphertext: encrypted_note.ciphertext,
            commitment: encrypted_note.commitment,
            uploaded_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_hash: None,
            output_index: None,
            leaf_index: None,
        };
        
        // Store in database
        self.store_note_entry(&entry)?;
        
        // Cache entry
        self.note_cache.insert(note_id.clone(), entry);
        
        Ok(note_id)
    }
    
    /// Attach transaction metadata to note
    pub fn attach_tx(&mut self, note_id: &str, tx_hash: String, output_index: u32) -> Result<()> {
        // Update entry in database
        if let Some(entry) = self.note_cache.get_mut(note_id) {
            entry.tx_hash = Some(tx_hash.clone());
            entry.output_index = Some(output_index);
            
            // If we have a commitment, add it to Merkle tree
            if let Some(commitment) = entry.commitment {
                let leaf_index = self.merkle_tree.insert_commitment(commitment)
                    .map_err(|e| anyhow!("Failed to insert commitment: {}", e))?;
                entry.leaf_index = Some(leaf_index);
            }
            
            // Update database
            let entry_clone = entry.clone();
            self.store_note_entry(&entry_clone)?;
        } else {
            return Err(anyhow!("Note not found: {}", note_id));
        }
        
        Ok(())
    }
    
    /// Get ciphertexts since timestamp
    pub fn get_ciphertexts_since(&self, since: u64) -> Result<Vec<EncryptedNoteEntry>> {
        let mut results = Vec::new();
        
        // Get all entries from database
        let iter = self.db.iterator_cf(cf_names::ENCRYPTED_NOTES)?;
        
        for item in iter {
            let (key, value) = item?;
            
            // Deserialize entry
            let entry: EncryptedNoteEntry = bincode::deserialize(&value)
                .map_err(|e| anyhow!("Failed to deserialize entry: {}", e))?;
            
            // Filter by timestamp
            if entry.uploaded_at >= since {
                results.push(entry);
            }
        }
        
        Ok(results)
    }
    
    /// Get Merkle proof for commitment
    pub fn get_merkle_proof(&self, commitment: &[u8; 32]) -> Result<Option<crate::utxo::MerkleProof>> {
        Ok(self.merkle_tree.generate_proof(*commitment))
    }
    
    /// Find note by commitment
    pub fn find_note_by_commitment(&self, commitment: &[u8; 32]) -> Result<Option<EncryptedNoteEntry>> {
        // Search through cached entries
        for entry in self.note_cache.values() {
            if let Some(entry_commitment) = entry.commitment {
                if entry_commitment == *commitment {
                    return Ok(Some(entry.clone()));
                }
            }
        }
        
        // If not found in cache, search database
        let iter = self.db.iterator_cf(cf_names::ENCRYPTED_NOTES)?;
        
        for item in iter {
            let (key, value) = item?;
            
            let entry: EncryptedNoteEntry = bincode::deserialize(&value)
                .map_err(|e| anyhow!("Failed to deserialize entry: {}", e))?;
            
            if let Some(entry_commitment) = entry.commitment {
                if entry_commitment == *commitment {
                    return Ok(Some(entry));
                }
            }
        }
        
        Ok(None)
    }
    
    /// Get current Merkle root
    pub fn get_merkle_root(&self) -> [u8; 32] {
        self.merkle_tree.get_root()
    }
    
    /// Generate unique note ID
    fn generate_note_id(&self, encrypted_note: &EncryptedNote) -> String {
        let mut data = Vec::new();
        data.extend_from_slice(&encrypted_note.ephemeral_pubkey);
        data.extend_from_slice(&encrypted_note.nonce);
        data.extend_from_slice(&encrypted_note.ciphertext);
        
        let hash = crate::crypto::CryptoUtils::sha256(&data);
        format!("note_{}", hex::encode(hash))
    }
    
    /// Store note entry in database
    fn store_note_entry(&self, entry: &EncryptedNoteEntry) -> Result<()> {
        let key = entry.note_id.as_bytes();
        let value = bincode::serialize(entry)
            .map_err(|e| anyhow!("Failed to serialize entry: {}", e))?;
        
        self.db.put_cf(cf_names::ENCRYPTED_NOTES, key, &value)?;
        
        Ok(())
    }
}

/// API endpoints for encrypted notes
pub mod endpoints {
    use axum::{
        extract::{Path, Query},
        http::StatusCode,
        response::Json,
        routing::{get, post},
        Router,
    };
    use serde::{Deserialize, Serialize};
    
    use crate::utxo::note::EncryptedNote;
    
    /// Upload note request
    #[derive(Debug, Deserialize)]
    pub struct UploadNoteRequest {
        pub encrypted_note: EncryptedNote,
    }
    
    /// Upload note response
    #[derive(Debug, Serialize)]
    pub struct UploadNoteResponse {
        pub success: bool,
        pub note_id: String,
        pub message: String,
    }
    
    /// Attach transaction request
    #[derive(Debug, Deserialize)]
    pub struct AttachTxRequest {
        pub note_id: String,
        pub tx_hash: String,
        pub output_index: u32,
    }
    
    /// Attach transaction response
    #[derive(Debug, Serialize)]
    pub struct AttachTxResponse {
        pub success: bool,
        pub message: String,
    }
    
    /// Get ciphertexts query
    #[derive(Debug, Deserialize)]
    pub struct GetCiphertextsQuery {
        pub since: Option<u64>,
    }
    
    /// Merkle proof response
    #[derive(Debug, Serialize)]
    pub struct MerkleProofResponse {
        pub root: String,
        pub leaf_index: u64,
        pub siblings: Vec<String>,
        pub path: Vec<u32>,
    }
    
    /// Create router for encrypted notes endpoints
    pub fn create_router() -> Router<()> {
        Router::new()
            .route("/upload_note", post(upload_note))
            .route("/attach_tx", post(attach_tx))
            .route("/ciphertexts", get(get_ciphertexts))
            .route("/merkle_proof/:commitment", get(get_merkle_proof))
    }
    
    /// Upload encrypted note endpoint
    async fn upload_note(
        Json(request): Json<UploadNoteRequest>,
    ) -> Result<Json<UploadNoteResponse>, (StatusCode, Json<serde_json::Value>)> {
        // This would be implemented with proper state management
        // For now, return a mock response
        Ok(Json(UploadNoteResponse {
            success: true,
            note_id: "mock_note_id".to_string(),
            message: "Note uploaded successfully".to_string(),
        }))
    }
    
    /// Attach transaction endpoint
    async fn attach_tx(
        Json(request): Json<AttachTxRequest>,
    ) -> Result<Json<AttachTxResponse>, (StatusCode, Json<serde_json::Value>)> {
        // This would be implemented with proper state management
        Ok(Json(AttachTxResponse {
            success: true,
            message: "Transaction attached successfully".to_string(),
        }))
    }
    
    /// Get ciphertexts endpoint
    async fn get_ciphertexts(
        Query(query): Query<GetCiphertextsQuery>,
    ) -> Result<Json<Vec<serde_json::Value>>, (StatusCode, Json<serde_json::Value>)> {
        // This would be implemented with proper state management
        Ok(Json(vec![]))
    }
    
    /// Get Merkle proof endpoint
    async fn get_merkle_proof(
        Path(commitment): Path<String>,
    ) -> Result<Json<MerkleProofResponse>, (StatusCode, Json<serde_json::Value>)> {
        // This would be implemented with proper state management
        Ok(Json(MerkleProofResponse {
            root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            leaf_index: 0,
            siblings: vec![],
            path: vec![],
        }))
    }
}

// Serialization helpers for fixed-size arrays
fn serialize_array33<S>(arr: &[u8; 33], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(33))?;
    for &byte in arr.iter() {
        seq.serialize_element(&byte)?;
    }
    seq.end()
}

fn deserialize_array33<'de, D>(deserializer: D) -> Result<[u8; 33], D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{Deserializer, SeqAccess, Visitor};
    use std::fmt;

    struct ArrayVisitor;

    impl<'de> Visitor<'de> for ArrayVisitor {
        type Value = [u8; 33];

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a sequence of 33 bytes")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut arr = [0u8; 33];
            for i in 0..33 {
                arr[i] = seq.next_element()?.ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
            }
            Ok(arr)
        }
    }

    deserializer.deserialize_seq(ArrayVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::database::DBConfig;

    #[test]
    fn test_encrypted_notes_relayer() {
        let temp_dir = TempDir::new().unwrap();
        let db_config = DBConfig {
            db_path: temp_dir.path().to_str().unwrap().to_string(),
            ..Default::default()
        };
        
        let db = DatabaseManager::open(db_config).unwrap();
        let mut relayer = EncryptedNotesRelayer::new(db).unwrap();
        
        // Create test encrypted note
        let encrypted_note = EncryptedNote {
            ephemeral_pubkey: [0x42u8; 33],
            nonce: [0x24u8; 24],
            ciphertext: b"encrypted_data".to_vec(),
            commitment: Some([0x12u8; 32]),
        };
        
        // Upload note
        let note_id = relayer.upload_note(encrypted_note).unwrap();
        assert!(!note_id.is_empty());
        
        // Attach transaction
        relayer.attach_tx(&note_id, "0xabcdef".to_string(), 0).unwrap();
        
        // Get Merkle root
        let root = relayer.get_merkle_root();
        assert_ne!(root, [0u8; 32]);
    }
}
