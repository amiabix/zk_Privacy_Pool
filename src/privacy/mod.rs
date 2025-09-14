//! Privacy Module
pub mod privacy_pool;
pub mod utxo_pool;
pub mod enhanced_privacy_pool;
pub mod complete_example;

// Re-export main types
pub use privacy_pool::{PrivacyPool, PoolStats};
pub use utxo_pool::{UTXOPrivacyPool, ETHDepositEvent};
pub use enhanced_privacy_pool::{EnhancedPrivacyPool, EnhancedUTXO, EnhancedTransaction, TransactionType as EnhancedTransactionType, MerkleProof as EnhancedMerkleProof, PoolStats as EnhancedPoolStats};
pub use complete_example::{CompletePrivacyPoolExample, CompleteSystemStats, PrivacyPoolTransaction, TransactionType as ExampleTransactionType};
