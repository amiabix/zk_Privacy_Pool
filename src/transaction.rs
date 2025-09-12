use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionResult {
    Success,
    Failure(String),
}

impl TransactionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TransactionResult::Success)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    DoubleSpend,
    InvalidMerkleProof,
    InsufficientBalance,
    InvalidTransaction,
    MerkleTreeError(String),
    InvalidUTXO,
    UserNotFound,
    InvalidSignature,
}