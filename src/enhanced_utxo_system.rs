//! Enhanced UTXO System with Zcash-style indexing
//! Integrates the basic UTXO types with efficient indexing

use crate::utxo::{UTXO, User};
use crate::utxo_indexing::{UTXOIndex, IndexedUTXO, UTXOId};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// Enhanced UTXO System with proper indexing
#[derive(Debug, Clone)]
pub struct EnhancedUTXOSystem {
    /// Indexed UTXO storage for O(1) lookups
    utxo_index: UTXOIndex,
    
    /// User accounts mapping
    users: HashMap<[u8; 32], User>,
    
    /// Account ID counter
    next_account_id: u32,
    
    /// Transaction counter for UTXO IDs
    tx_counter: u32,
}

impl EnhancedUTXOSystem {
    pub fn new() -> Self {
        Self {
            utxo_index: UTXOIndex::new(),
            users: HashMap::new(),
            next_account_id: 1,
            tx_counter: 0,
        }
    }

    /// Create a new user account
    pub fn create_user(&mut self, public_key: [u8; 32], private_key: [u8; 32]) -> u32 {
        let account_id = self.next_account_id;
        self.next_account_id += 1;
        
        let user = User::new(public_key, private_key);
        self.users.insert(public_key, user);
        
        account_id
    }

    /// Get user by public key
    pub fn get_user(&self, public_key: &[u8; 32]) -> Option<&User> {
        self.users.get(public_key)
    }

    /// Get user by public key (mutable)
    pub fn get_user_mut(&mut self, public_key: &[u8; 32]) -> Option<&mut User> {
        self.users.get_mut(public_key)
    }

    /// Add a UTXO to the system with proper indexing
    pub fn add_utxo(&mut self, utxo: UTXO, account_id: u32, height: u32) -> UTXOId {
        let tx_hash = self.generate_tx_hash();
        let utxo_id = UTXOId::new(tx_hash, 0); // Single output per transaction for simplicity
        
        let indexed_utxo = IndexedUTXO {
            id: utxo_id,
            account_id,
            address: utxo.commitment,
            value: utxo.value,
            height,
            spent_in_tx: None,
            blinding_factor: utxo.secret, // Using secret as blinding factor
        };
        
        self.utxo_index.add_utxo(indexed_utxo);
        
        // Also add to user's UTXO list
        if let Some(user) = self.users.values_mut().find(|u| u.public_key == utxo.owner) {
            user.add_utxo(utxo);
        }
        
        utxo_id
    }

    /// Remove a UTXO from the system
    pub fn remove_utxo(&mut self, utxo_id: UTXOId) -> Option<IndexedUTXO> {
        if let Some(indexed_utxo) = self.utxo_index.remove_utxo(utxo_id) {
            // Also remove from user's UTXO list
            if let Some(user) = self.users.values_mut().find(|u| u.public_key == indexed_utxo.address) {
                user.remove_utxo(&indexed_utxo.address);
            }
            Some(indexed_utxo)
        } else {
            None
        }
    }

    /// Get UTXO by ID
    pub fn get_utxo(&self, utxo_id: &UTXOId) -> Option<&IndexedUTXO> {
        self.utxo_index.get_utxo(utxo_id)
    }

    /// Get all UTXOs for an account
    pub fn get_account_utxos(&self, account_id: u32) -> Vec<&IndexedUTXO> {
        self.utxo_index.get_account_utxos(account_id)
    }

    /// Get unspent UTXOs for an account
    pub fn get_unspent_account_utxos(&self, account_id: u32) -> Vec<&IndexedUTXO> {
        self.utxo_index.get_unspent_account_utxos(account_id)
    }

    /// Get account balance
    pub fn get_account_balance(&self, account_id: u32) -> u64 {
        self.utxo_index.get_account_balance(account_id)
    }

    /// Mark UTXO as spent
    pub fn mark_utxo_spent(&mut self, utxo_id: UTXOId, spending_tx: [u8; 32]) -> bool {
        self.utxo_index.mark_spent(utxo_id, spending_tx)
    }

    /// Find UTXOs for spending (coin selection algorithm)
    pub fn select_utxos_for_spending(&self, account_id: u32, amount: u64) -> Vec<&IndexedUTXO> {
        let unspent_utxos = self.get_unspent_account_utxos(account_id);
        
        // Simple greedy algorithm: select smallest UTXOs first
        let mut sorted_utxos = unspent_utxos;
        sorted_utxos.sort_by_key(|utxo| utxo.value);
        
        let mut selected = Vec::new();
        let mut total = 0u64;
        
        for utxo in sorted_utxos {
            selected.push(utxo);
            total += utxo.value;
            if total >= amount {
                break;
            }
        }
        
        if total >= amount {
            selected
        } else {
            Vec::new() // Not enough funds
        }
    }

    /// Create a deposit transaction
    pub fn create_deposit(&mut self, user_public_key: [u8; 32], amount: u64, height: u32) -> Option<UTXOId> {
        if self.get_user(&user_public_key).is_some() {
            // Create new UTXO for deposit
            let secret = self.generate_secret();
            let nullifier = self.generate_nullifier(&secret);
            
            let utxo = UTXO::new(amount, secret, nullifier, user_public_key);
            let account_id = self.get_account_id_for_user(&user_public_key)?;
            
            Some(self.add_utxo(utxo, account_id, height))
        } else {
            None
        }
    }

    /// Create a withdrawal transaction
    pub fn create_withdrawal(&mut self, user_public_key: [u8; 32], amount: u64, recipient: [u8; 32]) -> Option<Vec<UTXOId>> {
        let account_id = self.get_account_id_for_user(&user_public_key)?;
        
        // Select UTXOs for spending - collect IDs first to avoid borrowing issues
        let selected_utxo_ids: Vec<UTXOId> = self.select_utxos_for_spending(account_id, amount)
            .into_iter()
            .map(|utxo| utxo.id)
            .collect();
            
        if selected_utxo_ids.is_empty() {
            return None; // Insufficient funds
        }
        
        // Mark selected UTXOs as spent
        let mut spent_utxo_ids = Vec::new();
        let spending_tx = self.generate_tx_hash();
        
        for utxo_id in selected_utxo_ids {
            self.mark_utxo_spent(utxo_id, spending_tx);
            spent_utxo_ids.push(utxo_id);
        }
        
        // Create new UTXO for recipient (if not external)
        if self.users.contains_key(&recipient) {
            let secret = self.generate_secret();
            let nullifier = self.generate_nullifier(&secret);
            let utxo = UTXO::new(amount, secret, nullifier, recipient);
            let recipient_account_id = self.get_account_id_for_user(&recipient)?;
            let new_utxo_id = self.add_utxo(utxo, recipient_account_id, 0); // Height will be set when mined
            spent_utxo_ids.push(new_utxo_id);
        }
        
        Some(spent_utxo_ids)
    }

    /// Get account ID for a user (simplified - in real implementation, store this mapping)
    fn get_account_id_for_user(&self, user_public_key: &[u8; 32]) -> Option<u32> {
        // For simplicity, use a hash of the public key as account ID
        // In a real implementation, maintain a proper mapping
        Some(u32::from_le_bytes([
            user_public_key[0], user_public_key[1], user_public_key[2], user_public_key[3]
        ]))
    }

    /// Generate a transaction hash
    fn generate_tx_hash(&mut self) -> [u8; 32] {
        self.tx_counter += 1;
        let mut hash = [0u8; 32];
        hash[0..4].copy_from_slice(&self.tx_counter.to_le_bytes());
        hash
    }

    /// Generate a random secret
    fn generate_secret(&self) -> [u8; 32] {
        // In production, use proper random generation
        let mut secret = [0u8; 32];
        for i in 0..32 {
            secret[i] = (self.tx_counter as u8).wrapping_add(i as u8);
        }
        secret
    }

    /// Generate a nullifier from secret
    fn generate_nullifier(&self, secret: &[u8; 32]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(secret);
        hasher.update(b"nullifier");
        hasher.finalize().into()
    }

    /// Get system statistics
    pub fn get_stats(&self) -> UTXOSystemStats {
        UTXOSystemStats {
            total_utxos: self.utxo_index.count(),
            total_users: self.users.len(),
            total_accounts: self.next_account_id - 1,
        }
    }

    /// Get all users
    pub fn get_all_users(&self) -> Vec<&User> {
        self.users.values().collect()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.utxo_index.clear();
        self.users.clear();
        self.next_account_id = 1;
        self.tx_counter = 0;
    }
}

impl Default for EnhancedUTXOSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// System statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOSystemStats {
    pub total_utxos: usize,
    pub total_users: usize,
    pub total_accounts: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enhanced_utxo_system() {
        let mut system = EnhancedUTXOSystem::new();
        
        // Create users
        let user1_pk = [1u8; 32];
        let user1_sk = [0x11u8; 32];
        let account1 = system.create_user(user1_pk, user1_sk);
        
        let user2_pk = [2u8; 32];
        let user2_sk = [0x22u8; 32];
        let account2 = system.create_user(user2_pk, user2_sk);
        
        // Create deposits
        let deposit1 = system.create_deposit(user1_pk, 100, 1000);
        assert!(deposit1.is_some());
        
        let deposit2 = system.create_deposit(user2_pk, 200, 1001);
        assert!(deposit2.is_some());
        
        // Check balances
        let balance1 = system.get_account_balance(account1);
        assert_eq!(balance1, 100);
        
        let balance2 = system.get_account_balance(account2);
        assert_eq!(balance2, 200);
        
        // Create withdrawal
        let withdrawal = system.create_withdrawal(user1_pk, 50, user2_pk);
        assert!(withdrawal.is_some());
        
        // Check updated balances
        let new_balance1 = system.get_account_balance(account1);
        let new_balance2 = system.get_account_balance(account2);
        
        // User1 should have less, user2 should have more
        assert!(new_balance1 < balance1);
        assert!(new_balance2 > balance2);
        
        // Check stats
        let stats = system.get_stats();
        assert!(stats.total_utxos > 0);
        assert_eq!(stats.total_users, 2);
        assert_eq!(stats.total_accounts, 2);
    }
}
