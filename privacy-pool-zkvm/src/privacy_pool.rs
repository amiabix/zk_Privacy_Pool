use serde::{Deserialize, Serialize};
use std::collections::{HashSet, HashMap};
use crate::merkle_tree::MerkleTree;
use crate::utxo::{UTXO, UTXOTransaction, User, MerkleProof};
use crate::transaction::{TransactionResult, Error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPool {
    pub merkle_tree: MerkleTree,
    pub nullifiers: HashSet<[u8; 32]>,
    pub pool_index: u64,
    pub pool_balance: u64,
    pub users: HashMap<[u8; 32], User>,
    pub utxo_index: u64,
}

impl PrivacyPool {
    pub fn new() -> Self {
        Self {
            merkle_tree: MerkleTree::new(),
            nullifiers: HashSet::new(),
            pool_index: 0,
            pool_balance: 0,
            users: HashMap::new(),
            utxo_index: 0,
        }
    }
    
    pub fn add_user(&mut self, user: User) {
        self.users.insert(user.public_key, user);
    }
    
    pub fn get_user(&mut self, public_key: &[u8; 32]) -> Option<&mut User> {
        self.users.get_mut(public_key)
    }
    
    pub fn process_transaction(&mut self, tx: UTXOTransaction) -> Result<TransactionResult, Error> {
        // This entire function runs inside ZisK zkVM
        // All operations are proven
        
        // 1. Verify transaction signatures
        if !tx.inputs.is_empty() {
            let message = self.create_transaction_message(&tx);
            for input in &tx.inputs {
                if let Some(user) = self.users.get(&input.utxo.owner) {
                    if !user.verify_signature(&message, &input.signature, &input.utxo.owner) {
                        return Err(Error::InvalidSignature);
                    }
                } else {
                    return Err(Error::UserNotFound);
                }
            }
        }
        
        // 2. Verify all inputs are valid UTXOs
        for input in &tx.inputs {
            // Check nullifier not used
            let nullifier_hash = input.utxo.compute_nullifier_hash();
            if self.nullifiers.contains(&nullifier_hash) {
                return Err(Error::DoubleSpend);
            }
            
            // Verify Merkle proof
            if !self.verify_merkle_proof(&input.merkle_proof, &input.utxo.commitment) {
                return Err(Error::InvalidMerkleProof);
            }
            
            // Verify user owns the UTXO
            if let Some(user) = self.users.get(&input.utxo.owner) {
                if !user.utxos.iter().any(|u| u.commitment == input.utxo.commitment) {
                    return Err(Error::InvalidUTXO);
                }
            } else {
                return Err(Error::UserNotFound);
            }
        }
        
        // 3. Verify total input value >= total output value + fee
        let total_input: u64 = tx.inputs.iter().map(|i| i.utxo.value).sum();
        let total_output: u64 = tx.outputs.iter().map(|o| o.value).sum();
        
        // For deposits, we allow zero inputs (external funding)
        if !tx.inputs.is_empty() && total_input < total_output + tx.fee {
            return Err(Error::InsufficientBalance);
        }
        
        // 3. Process transaction type
        match tx.tx_type {
            crate::utxo::TransactionType::Deposit { amount, recipient } => {
                self.process_deposit(&tx, amount, recipient)?;
            },
            crate::utxo::TransactionType::Withdraw { amount, recipient } => {
                self.process_withdraw(&tx, amount, recipient)?;
            },
            crate::utxo::TransactionType::Transfer { recipient } => {
                self.process_transfer(&tx, recipient)?;
            },
        }
        
        // 4. Update state - mark nullifiers as used
        for input in &tx.inputs {
            let nullifier_hash = input.utxo.compute_nullifier_hash();
            self.nullifiers.insert(nullifier_hash);
        }
        
        // 5. Update user balances
        self.update_user_balances(&tx)?;
        
        self.pool_index += 1;
        Ok(TransactionResult::Success)
    }
    
    fn process_deposit(&mut self, tx: &UTXOTransaction, amount: u64, recipient: [u8; 32]) -> Result<(), Error> {
        // Create new UTXO for the deposit
        for output in &tx.outputs {
            let mut utxo = output.to_utxo(self.utxo_index);
            utxo.index = self.utxo_index;
            
            // Add to Merkle tree
            self.merkle_tree.insert_utxo(utxo.clone())?;
            
            // Add to user's UTXOs
            if let Some(user) = self.get_user(&recipient) {
                user.add_utxo(utxo);
            } else {
                // Create user if they don't exist
                let mut new_user = crate::utxo::User::new(recipient, [0u8; 32]);
                new_user.add_utxo(utxo);
                self.add_user(new_user);
            }
            
            self.utxo_index += 1;
        }
        
        self.pool_balance += amount;
        Ok(())
    }
    
    fn process_withdraw(&mut self, tx: &UTXOTransaction, amount: u64, _recipient: [u8; 32]) -> Result<(), Error> {
        // Remove UTXOs from users
        for input in &tx.inputs {
            if let Some(user) = self.get_user(&input.utxo.owner) {
                user.remove_utxo(&input.utxo.commitment);
            }
        }
        
        // Create change UTXO if needed
        let total_input: u64 = tx.inputs.iter().map(|i| i.utxo.value).sum();
        let total_output: u64 = tx.outputs.iter().map(|o| o.value).sum();
        let change = total_input - total_output - tx.fee;
        
        if change > 0 {
            // Create change UTXO for the first input owner
            if let Some(first_input) = tx.inputs.first() {
                let change_utxo = UTXO::new(
                    change,
                    [0u8; 32], // Will be set by user
                    [0u8; 32], // Will be set by user
                    first_input.utxo.owner,
                );
                
                self.merkle_tree.insert_utxo(change_utxo.clone())?;
                if let Some(user) = self.get_user(&first_input.utxo.owner) {
                    user.add_utxo(change_utxo);
                }
            }
        }
        
        self.pool_balance -= amount;
        Ok(())
    }
    
    fn process_transfer(&mut self, tx: &UTXOTransaction, recipient: [u8; 32]) -> Result<(), Error> {
        // Remove input UTXOs from users
        for input in &tx.inputs {
            if let Some(user) = self.get_user(&input.utxo.owner) {
                user.remove_utxo(&input.utxo.commitment);
            }
        }
        
        // Add output UTXOs to recipient
        for output in &tx.outputs {
            let mut utxo = output.to_utxo(self.utxo_index);
            utxo.index = self.utxo_index;
            
            self.merkle_tree.insert_utxo(utxo.clone())?;
            
            if let Some(user) = self.get_user(&recipient) {
                user.add_utxo(utxo);
            } else {
                return Err(Error::UserNotFound);
            }
            
            self.utxo_index += 1;
        }
        
        Ok(())
    }
    
    fn update_user_balances(&mut self, tx: &UTXOTransaction) -> Result<(), Error> {
        // Update balances for all affected users
        for input in &tx.inputs {
            if let Some(user) = self.get_user(&input.utxo.owner) {
                user.balance -= input.utxo.value;
            }
        }
        
        for output in &tx.outputs {
            if let Some(user) = self.get_user(&output.recipient) {
                user.balance += output.value;
            }
        }
        
        Ok(())
    }
    
    fn verify_merkle_proof(&self, proof: &MerkleProof, leaf: &[u8; 32]) -> bool {
        let mut current = *leaf;
        
        for (sibling, is_right) in proof.siblings.iter().zip(proof.path.iter()) {
            if *is_right {
                current = self.hash_pair(current, *sibling);
            } else {
                current = self.hash_pair(*sibling, current);
            }
        }
        
        current == proof.root
    }
    
    fn hash_pair(&self, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(&left);
        hasher.update(&right);
        hasher.finalize().into()
    }
    
    pub fn get_pool_stats(&self) -> PoolStats {
        PoolStats {
            total_users: self.users.len(),
            total_utxos: self.utxo_index,
            pool_balance: self.pool_balance,
            merkle_root: self.merkle_tree.root,
            nullifiers_count: self.nullifiers.len(),
        }
    }
    
    pub fn create_transaction_message(&self, tx: &UTXOTransaction) -> Vec<u8> {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        
        // Hash all input commitments
        for input in &tx.inputs {
            hasher.update(&input.utxo.commitment);
        }
        
        // Hash all output values and recipients
        for output in &tx.outputs {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&output.recipient);
        }
        
        // Hash fee
        hasher.update(&tx.fee.to_le_bytes());
        
        hasher.finalize().to_vec()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_users: usize,
    pub total_utxos: u64,
    pub pool_balance: u64,
    pub merkle_root: [u8; 32],
    pub nullifiers_count: usize,
}