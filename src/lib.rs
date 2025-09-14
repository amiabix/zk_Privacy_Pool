// Core modules
// pub mod integration_test; // Removed - file not found
// pub mod integration_test_full; // Removed - file not found

// New canonical implementation modules
pub mod canonical_spec;
pub mod database;

// Organized modules
pub mod utxo;
pub mod merkle;
pub mod privacy;
pub mod relayer;
pub mod utils;
pub mod api;

// Re-export main types for easy access
pub use privacy::{PrivacyPool, PoolStats, UTXOPrivacyPool, ETHDepositEvent};
pub use utxo::{UTXO, UTXOTransaction, User, UTXOInput, UTXOOutput, TransactionType, UTXOIndex, IndexedUTXO, UTXOId, UTXOQueryBuilder, ETHToUTXOConverter, SecureCommitment, Nullifier, CryptoUtils, TransactionResult, Error, MerkleProof};
pub use merkle::{EnhancedMerkleTree, RelayerService, RelayerConfig};
pub use utils::{sha256, keccak256, hash_pair, hash_multiple, zisk_sha256, zisk_keccak256, zisk_hash_pair, zisk_bn254_add, zisk_bn254_double, zisk_pedersen_commitment, zisk_generate_nullifier, zisk_verify_nullifier};
pub use relayer::{BlockchainConfig, RealDepositEvent, RealBlockchainClient, RealWallet, AccountManager, RealDepositManager};