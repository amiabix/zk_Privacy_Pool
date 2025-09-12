use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionResult {
    Success,
    Failure(String),
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
}