use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::crypto::ecies::Ecies;
use crate::utxo::note::{Note, EncryptedNote};
use crate::database::DatabaseManager;

/// Encrypted note upload request
#[derive(Debug, Deserialize)]
pub struct NoteUploadRequest {
    pub encrypted_note: EncryptedNoteData,
    pub uploader_sig: Option<String>, // optional EOA signature proving uploader control
}

/// Encrypted note data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedNoteData {
    pub ephemeral_pubkey: String, // 33 bytes hex
    pub nonce: String,            // 24 bytes hex
    pub ciphertext: String,       // hex encoded
    pub commitment: String,       // 32 bytes hex
    pub owner_enc_pk: String,     // 33 bytes hex
}

/// Note upload response
#[derive(Debug, Serialize)]
pub struct NoteUploadResponse {
    pub note_id: String,
    pub attached: bool,
    pub tx_hash: Option<String>,
    pub leaf_index: Option<u64>,
}

/// Note query request
#[derive(Debug, Deserialize)]
pub struct NoteQueryRequest {
    pub owner_pk: String, // owner public key (33 bytes hex)
}

/// Note query response
#[derive(Debug, Serialize)]
pub struct NoteQueryResponse {
    pub notes: Vec<EncryptedNoteData>,
    pub total_count: usize,
}

/// Encrypted note storage structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEncryptedNote {
    pub note_id: String,
    pub encrypted_note: EncryptedNoteData,
    pub uploaded_at: u64,
    pub attached_tx_hash: Option<String>,
    pub attached_leaf_index: Option<u64>,
}

/// Application state for encrypted notes
pub struct EncryptedNotesState {
    pub db: Arc<Mutex<DatabaseManager>>,
}

impl EncryptedNotesState {
    pub fn new(db: Arc<Mutex<DatabaseManager>>) -> Self {
        Self { db }
    }
}

/// Create router for encrypted notes API
pub fn create_router() -> Router<EncryptedNotesState> {
    Router::new()
        .route("/notes/upload", post(upload_note))
        .route("/notes/query", get(query_notes))
        .route("/notes/attach/:commitment", post(attach_note))
        .route("/notes/:note_id", get(get_note))
}

/// Upload encrypted note
async fn upload_note(
    State(state): State<EncryptedNotesState>,
    Json(request): Json<NoteUploadRequest>,
) -> Result<Json<NoteUploadResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Generate note ID
    let note_id = format!("note_{}", hex::encode(&CryptoUtils::random_32()));
    
    // Parse commitment
    let commitment = hex::decode(&request.encrypted_note.commitment)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid commitment format"
        }))))?;
    
    if commitment.len() != 32 {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid commitment length"
        }))));
    }
    
    // Store encrypted note
    let stored_note = StoredEncryptedNote {
        note_id: note_id.clone(),
        encrypted_note: request.encrypted_note,
        uploaded_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        attached_tx_hash: None,
        attached_leaf_index: None,
    };
    
    // Store in database
    {
        let mut db = state.db.lock().await;
        if let Err(e) = db.store_encrypted_note(&stored_note).await {
            log::error!("Failed to store encrypted note: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to store note"
            }))));
        }
    }
    
    // Check if commitment is already attached to a deposit
    let (attached, tx_hash, leaf_index) = {
        let mut db = state.db.lock().await;
        if let Ok(Some(attachment)) = db.get_commitment_attachment(&commitment).await {
            (true, Some(attachment.tx_hash), Some(attachment.leaf_index))
        } else {
            (false, None, None)
        }
    };
    
    Ok(Json(NoteUploadResponse {
        note_id,
        attached,
        tx_hash,
        leaf_index,
    }))
}

/// Query encrypted notes for a specific owner
async fn query_notes(
    State(state): State<EncryptedNotesState>,
    Query(params): Query<NoteQueryRequest>,
) -> Result<Json<NoteQueryResponse>, (StatusCode, Json<serde_json::Value>)> {
    // Parse owner public key
    let owner_pk = hex::decode(&params.owner_pk)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid owner public key format"
        }))))?;
    
    if owner_pk.len() != 33 {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid owner public key length"
        }))));
    }
    
    // Query notes from database
    let notes = {
        let mut db = state.db.lock().await;
        db.query_encrypted_notes_by_owner(&owner_pk).await
            .map_err(|e| {
                log::error!("Failed to query notes: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "Failed to query notes"
                })))
            })?
    };
    
    Ok(Json(NoteQueryResponse {
        total_count: notes.len(),
        notes: notes.into_iter().map(|note| note.encrypted_note).collect(),
    }))
}

/// Attach note to a deposit (called by relayer when deposit is processed)
async fn attach_note(
    State(state): State<EncryptedNotesState>,
    Path(commitment): Path<String>,
    Json(attachment): Json<NoteAttachment>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let commitment_bytes = hex::decode(&commitment)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid commitment format"
        }))))?;
    
    if commitment_bytes.len() != 32 {
        return Err((StatusCode::BAD_REQUEST, Json(serde_json::json!({
            "error": "Invalid commitment length"
        }))));
    }
    
    // Update note attachment in database
    {
        let mut db = state.db.lock().await;
        if let Err(e) = db.attach_note_to_deposit(&commitment_bytes, &attachment.tx_hash, attachment.leaf_index).await {
            log::error!("Failed to attach note: {:?}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                "error": "Failed to attach note"
            }))));
        }
    }
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Note attached successfully"
    })))
}

/// Get specific note by ID
async fn get_note(
    State(state): State<EncryptedNotesState>,
    Path(note_id): Path<String>,
) -> Result<Json<StoredEncryptedNote>, (StatusCode, Json<serde_json::Value>)> {
    let note = {
        let mut db = state.db.lock().await;
        db.get_encrypted_note_by_id(&note_id).await
            .map_err(|e| {
                log::error!("Failed to get note: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({
                    "error": "Failed to get note"
                })))
            })?
    };
    
    match note {
        Some(note) => Ok(Json(note)),
        None => Err((StatusCode::NOT_FOUND, Json(serde_json::json!({
            "error": "Note not found"
        })))),
    }
}

/// Note attachment data
#[derive(Debug, Deserialize)]
pub struct NoteAttachment {
    pub tx_hash: String,
    pub leaf_index: u64,
}

/// Commitment attachment data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitmentAttachment {
    pub tx_hash: String,
    pub leaf_index: u64,
    pub block_number: u64,
}

// Extension traits for DatabaseManager to support encrypted notes
pub trait EncryptedNotesDB {
    async fn store_encrypted_note(&mut self, note: &StoredEncryptedNote) -> Result<()>;
    async fn get_encrypted_note_by_id(&mut self, note_id: &str) -> Result<Option<StoredEncryptedNote>>;
    async fn get_encrypted_note_by_commitment(&mut self, commitment: &[u8]) -> Result<Option<StoredEncryptedNote>>;
    async fn query_encrypted_notes_by_owner(&mut self, owner_pk: &[u8]) -> Result<Vec<StoredEncryptedNote>>;
    async fn get_commitment_attachment(&mut self, commitment: &[u8]) -> Result<Option<CommitmentAttachment>>;
    async fn attach_note_to_deposit(&mut self, commitment: &[u8], tx_hash: &str, leaf_index: u64) -> Result<()>;
}

// Note: These methods would need to be implemented in your DatabaseManager
// For now, we'll provide placeholder implementations
impl EncryptedNotesDB for DatabaseManager {
    async fn store_encrypted_note(&mut self, _note: &StoredEncryptedNote) -> Result<()> {
        // TODO: Implement actual database storage
        Ok(())
    }
    
    async fn get_encrypted_note_by_id(&mut self, _note_id: &str) -> Result<Option<StoredEncryptedNote>> {
        // TODO: Implement actual database retrieval
        Ok(None)
    }
    
    async fn get_encrypted_note_by_commitment(&mut self, _commitment: &[u8]) -> Result<Option<StoredEncryptedNote>> {
        // TODO: Implement actual database retrieval
        Ok(None)
    }
    
    async fn query_encrypted_notes_by_owner(&mut self, _owner_pk: &[u8]) -> Result<Vec<StoredEncryptedNote>> {
        // TODO: Implement actual database query
        Ok(vec![])
    }
    
    async fn get_commitment_attachment(&mut self, _commitment: &[u8]) -> Result<Option<CommitmentAttachment>> {
        // TODO: Implement actual database retrieval
        Ok(None)
    }
    
    async fn attach_note_to_deposit(&mut self, _commitment: &[u8], _tx_hash: &str, _leaf_index: u64) -> Result<()> {
        // TODO: Implement actual database update
        Ok(())
    }
}
