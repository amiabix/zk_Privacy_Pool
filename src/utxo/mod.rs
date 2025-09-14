//! UTXO System Module
//! Contains all UTXO-related functionality

pub mod utxo;
pub mod canonical_utxo;
pub mod utxo_manager;
pub mod indexing;
pub mod converter;
pub mod eth_deposit_handler;
pub mod transaction;
pub mod note;

// Re-export main types
pub use utxo::{UTXO, UTXOTransaction, User, UTXOInput, UTXOOutput, TransactionType};
pub use canonical_utxo::{CanonicalUTXO, lock_flags, UTXOError};
pub use utxo_manager::{UTXOManager, UTXOOperationResult, DepositResult};
pub use transaction::{TransactionResult, Error, MerkleProof};
pub use indexing::{UTXOIndex, IndexedUTXO, UTXOId, UTXOQueryBuilder};
pub use converter::{ETHToUTXOConverter, SecureCommitment, Nullifier, CryptoUtils};
pub use eth_deposit_handler::{ETHDepositHandler, ETHDepositEvent, DepositProof, DepositError};
pub use crate::relayer::DepositEvent;