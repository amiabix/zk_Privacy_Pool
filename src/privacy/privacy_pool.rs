//! Privacy Pool Implementation
//! Core privacy pool functionality for the ZisK zkVM system

use crate::utxo::{UTXO, User, MerkleProof, TransactionType};
use crate::merkle::EnhancedMerkleTree;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

/// Privacy Pool State
/// Manages the core privacy pool functionality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPool {
    /// Merkle tree for commitments
    pub merkle_tree: EnhancedMerkleTree,
    /// Pool users
    pub users: HashMap<[u8; 32], User>, // address -> user
    /// Nullifier set (prevents double-spending)
    pub nullifier_set: HashSet<[u8; 32]>,
    /// Pool balance (total committed value)
    pub pool_balance: u64,
    /// Pool capacity
    pub capacity: u32,
    /// Current size (number of commitments)
    pub size: u32,
    /// Pool scope
    pub scope: [u8; 32],
}

impl PrivacyPool {
    /// Create a new privacy pool
    pub fn new(scope: [u8; 32]) -> Self {
        Self {
            merkle_tree: EnhancedMerkleTree::new(),
            users: HashMap::new(),
            nullifier_set: HashSet::new(),
            pool_balance: 0,
            capacity: 2u32.pow(32), // 32-level tree
            size: 0,
            scope,
        }
    }

    /// Add a user to the pool
    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.public_key, user);
    }

    /// Get user by address
    pub fn get_user(&mut self, address: [u8; 32]) -> Option<&mut User> {
        self.users.get_mut(&address)
    }

    /// Deposit UTXO into the pool
    pub fn deposit_utxo(&mut self, utxo: UTXO, user_address: [u8; 32]) -> Result<u64, String> {
        // Check if user exists
        if !self.users.contains_key(&user_address) {
            return Err("User not found".to_string());
        }

        // Check capacity
        if self.size >= self.capacity {
            return Err("Pool is full".to_string());
        }

        // Insert UTXO into Merkle tree
        let leaf_index = self.merkle_tree.insert_utxo(&utxo)?;

        // Add UTXO to user
        if let Some(user) = self.users.get_mut(&user_address) {
            user.add_utxo(utxo.clone());
        }

        // Update pool state
        self.pool_balance += utxo.value;
        self.size += 1;

        Ok(leaf_index)
    }

    /// Withdraw UTXO from the pool
    pub fn withdraw_utxo(&mut self, utxo: &UTXO, user_address: [u8; 32]) -> Result<(), String> {
        // Check if user exists
        if !self.users.contains_key(&user_address) {
            return Err("User not found".to_string());
        }

        // Generate nullifier
        let nullifier = utxo.generate_nullifier();

        // Check if nullifier already used
        if self.nullifier_set.contains(&nullifier) {
            return Err("UTXO already spent".to_string());
        }

        // Add nullifier to set
        self.nullifier_set.insert(nullifier);

        // Remove UTXO from user
        if let Some(user) = self.users.get_mut(&user_address) {
            user.utxos.retain(|u| u.commitment != utxo.commitment);
        }

        // Update pool state
        self.pool_balance -= utxo.value;
        self.size -= 1;

        Ok(())
    }

    /// Generate Merkle proof for UTXO
    pub fn generate_proof(&self, utxo: &UTXO) -> Option<MerkleProof> {
        self.merkle_tree.generate_proof(utxo.commitment)
    }

    /// Verify Merkle proof
    pub fn verify_proof(&self, proof: &MerkleProof, leaf: [u8; 32]) -> bool {
        self.merkle_tree.verify_proof(proof, leaf)
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            merkle_root: self.merkle_tree.get_root(),
            pool_balance: self.pool_balance,
            size: self.size,
            capacity: self.capacity,
            nullifier_count: self.nullifier_set.len() as u32,
            user_count: self.users.len() as u32,
        }
    }

    /// Get current Merkle root
    pub fn get_merkle_root(&self) -> [u8; 32] {
        self.merkle_tree.get_root()
    }

    /// Check if nullifier is used
    pub fn is_nullifier_used(&self, nullifier: [u8; 32]) -> bool {
        self.nullifier_set.contains(&nullifier)
    }

    /// Get user balance
    pub fn get_user_balance(&self, user_address: [u8; 32]) -> u64 {
        self.users.get(&user_address)
            .map(|user| user.get_balance())
            .unwrap_or(0)
    }

    /// Get all users
    pub fn get_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    /// Get user UTXOs
    pub fn get_user_utxos(&self, user_address: [u8; 32]) -> Vec<&UTXO> {
        self.users.get(&user_address)
            .map(|user| user.utxos.iter().collect())
            .unwrap_or_default()
    }
}

/// Pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Current Merkle root
    pub merkle_root: [u8; 32],
    /// Pool balance
    pub pool_balance: u64,
    /// Current size
    pub size: u32,
    /// Pool capacity
    pub capacity: u32,
    /// Number of used nullifiers
    pub nullifier_count: u32,
    /// Number of users
    pub user_count: u32,
}

impl Default for PrivacyPool {
    fn default() -> Self {
        Self::new([0u8; 32])
    }
}
