//! Relayer Module
pub mod data_service;
pub mod tree_service;
pub mod integration_test;
pub mod blockchain_integration;
pub mod wallet_deposit_test;
pub mod encrypted_notes;
pub mod encrypted_notes_integration_test;
pub mod deposit_watcher;

// Re-export main types
pub use data_service::{DataService, DepositEvent};
pub use tree_service::{TreeService, MerkleProof};
pub use blockchain_integration::{BlockchainConfig, DepositEvent as BlockchainDepositEvent, BlockchainClient, Wallet, AccountManager, DepositManager};
pub use wallet_deposit_test::{TestWallet, DepositTransaction};
pub use encrypted_notes::{EncryptedNotesRelayer, EncryptedNoteEntry, endpoints};
