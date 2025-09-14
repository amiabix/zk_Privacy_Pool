//! Relayer Module
pub mod data_service;
pub mod tree_service;
pub mod integration_test;
pub mod real_blockchain_integration;
pub mod wallet_deposit_test;

// Re-export main types
pub use data_service::{DataService, DepositEvent};
pub use tree_service::{TreeService, MerkleProof};
pub use real_blockchain_integration::{BlockchainConfig, RealDepositEvent, RealBlockchainClient, RealWallet, AccountManager, RealDepositManager};
pub use wallet_deposit_test::{TestWallet, DepositTransaction};
