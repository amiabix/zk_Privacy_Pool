//! Shared types for privacy pool implementations

use serde::{Deserialize, Serialize};

/// Pool statistics structure shared across different pool implementations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Current Merkle root
    pub merkle_root: [u8; 32],
    /// Pool balance
    pub pool_balance: u64,
    /// Current size (number of UTXOs)
    pub size: u32,
    /// Pool capacity (maximum UTXOs)
    pub capacity: u32,
    /// Number of used nullifiers
    pub nullifier_count: u32,
    /// Number of users (optional, may be 0 in some implementations)
    pub user_count: u32,
    /// Number of approved addresses (optional, may be 0 in some implementations)
    pub approved_address_count: u32,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            merkle_root: [0u8; 32],
            pool_balance: 0,
            size: 0,
            capacity: 2u32.pow(32),
            nullifier_count: 0,
            user_count: 0,
            approved_address_count: 0,
        }
    }
}