//! API Request and Response Types
//! 
//! Defines all HTTP request/response structures for the privacy pool API.

use serde::{Deserialize, Serialize};
use web3::types::{Address, H256, U256};

/// Request to process an ETH deposit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositRequest {
    /// Depositor's Ethereum address
    pub depositor: Address,
    /// Privacy commitment hash
    pub commitment: H256,
    /// Deposit amount in wei
    pub amount: U256,
    /// Block number where deposit occurred
    pub block_number: u64,
    /// Transaction hash
    pub tx_hash: H256,
    /// Additional label/metadata
    pub label: Option<U256>,
    /// Precommitment hash (if any)
    pub precommitment_hash: Option<H256>,
}

/// Response after processing a deposit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositResponse {
    /// Success status
    pub success: bool,
    /// Created UTXO ID (hex encoded)
    pub utxo_id: String,
    /// New tree root (hex encoded)
    pub new_root: String,
    /// Tree position where UTXO was placed
    pub tree_position: u64,
    /// Leaf hash (hex encoded)
    pub leaf_hash: String,
    /// Root version
    pub root_version: u64,
    /// Processing timestamp
    pub processed_at: u64,
}

/// Request for owner's UTXOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOQuery {
    /// Maximum number of UTXOs to return
    pub limit: Option<usize>,
    /// Skip UTXOs created before this block
    pub after_block: Option<u64>,
    /// Filter by specific asset ID (hex encoded)
    pub asset_id: Option<String>,
}

/// UTXO information for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOInfo {
    /// UTXO ID (hex encoded)
    pub utxo_id: String,
    /// Amount in smallest unit
    pub amount: String,
    /// Asset ID (hex encoded)
    pub asset_id: String,
    /// Block when UTXO was created
    pub created_block: u64,
    /// Tree position
    pub tree_position: u64,
    /// Lock expiry (if any)
    pub lock_expiry: Option<u64>,
    /// Lock flags
    pub lock_flags: u8,
    /// Whether UTXO is spent
    pub is_spent: bool,
}

/// Response for UTXO queries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOListResponse {
    /// List of UTXOs
    pub utxos: Vec<UTXOInfo>,
    /// Total count (may be larger than returned list)
    pub total_count: usize,
    /// Cursor for pagination
    pub next_cursor: Option<String>,
}

/// Balance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    /// Total balance as string (to handle large numbers)
    pub balance: String,
    /// Number of UTXOs
    pub utxo_count: u32,
    /// Last updated block
    pub last_updated_block: u64,
    /// Asset ID (hex encoded)
    pub asset_id: String,
}

/// Tree statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeStatsResponse {
    /// Current tree root (hex encoded)
    pub current_root: String,
    /// Current root version
    pub root_version: u64,
    /// Tree depth
    pub depth: u8,
    /// Total number of UTXOs in tree
    pub total_utxos: u64,
    /// Total number of tree nodes
    pub total_nodes: u64,
    /// Tree salt for reproducibility
    pub tree_salt: u64,
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    /// System status
    pub status: String,
    /// Current timestamp
    pub timestamp: u64,
    /// Version information
    pub version: String,
    /// Database status
    pub database_status: String,
    /// Tree status
    pub tree_status: String,
}

/// Error response format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    /// Error code
    pub error: String,
    /// Human readable message
    pub message: String,
    /// Additional details (optional)
    pub details: Option<serde_json::Value>,
    /// Timestamp
    pub timestamp: u64,
}

/// ETH asset ID constant (20 zero bytes)
pub const ETH_ASSET_ID: &str = "0000000000000000000000000000000000000000";

/// Utility functions for hex encoding/decoding
pub mod utils {
    
    
    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        hex::encode(bytes)
    }
    
    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex_str: &str) -> Result<Vec<u8>, hex::FromHexError> {
        let clean_hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        hex::decode(clean_hex)
    }
    
    /// Convert 32-byte array to hex string
    pub fn hash_to_hex(hash: [u8; 32]) -> String {
        format!("0x{}", hex::encode(hash))
    }
}