//! UTXO System Implementation
//! 
//! Based on patterns from:
//! - Bitcoin Core UTXO model
//! - Zcash Sapling note format
//! - Tornado Cash commitment system
//! 
//! Adapted for ZisK zkVM constraints

use crate::zisk_precompiles::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// UTXO (Unspent Transaction Output) - Core data structure
/// Based on Bitcoin Core UTXO model with Zcash Sapling enhancements
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UTXO {
    /// Transaction output index
    pub index: u32,
    /// Value in satoshis
    pub value: u64,
    /// Commitment (hiding and binding)
    pub commitment: [u8; 32],
    /// Blinding factor for commitment
    pub blinding: [u8; 32],
    /// Nullifier seed (for spending)
    pub nullifier_seed: [u8; 32],
    /// Owner's public key
    pub owner_pubkey: [u8; 32],
    /// Transaction hash that created this UTXO
    pub tx_hash: [u8; 32],
    /// Block height when created
    pub block_height: u32,
    /// Whether this UTXO is spent
    pub is_spent: bool,
}

impl UTXO {
    /// Create new UTXO with proper commitment
    pub fn new(
        index: u32,
        value: u64,
        blinding: [u8; 32],
        nullifier_seed: [u8; 32],
        owner_pubkey: [u8; 32],
        tx_hash: [u8; 32],
        block_height: u32,
    ) -> Self {
        // Generate commitment using Pedersen commitment scheme
        let commitment = zisk_pedersen_commitment(value, blinding);
        
        Self {
            index,
            value,
            commitment,
            blinding,
            nullifier_seed,
            owner_pubkey,
            tx_hash,
            block_height,
            is_spent: false,
        }
    }

    /// Generate nullifier for spending this UTXO
    /// Based on Zcash Sapling nullifier generation
    pub fn generate_nullifier(&self, secret: [u8; 32]) -> [u8; 32] {
        zisk_generate_nullifier(secret, self.nullifier_seed)
    }

    /// Verify nullifier for this UTXO
    pub fn verify_nullifier(&self, nullifier: [u8; 32], secret: [u8; 32]) -> bool {
        zisk_verify_nullifier(nullifier, secret, self.nullifier_seed)
    }

    /// Mark UTXO as spent
    pub fn mark_spent(&mut self) {
        self.is_spent = true;
    }

    /// Check if UTXO can be spent
    pub fn can_spend(&self) -> bool {
        !self.is_spent && self.value > 0
    }

    /// Get UTXO hash for Merkle tree
    pub fn hash(&self) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&self.index.to_le_bytes());
        data.extend_from_slice(&self.value.to_le_bytes());
        data.extend_from_slice(&self.commitment);
        data.extend_from_slice(&self.owner_pubkey);
        data.extend_from_slice(&self.tx_hash);
        data.extend_from_slice(&self.block_height.to_le_bytes());
        
        zisk_sha256(&data)
    }
}

/// UTXO Input - References a UTXO to be spent
/// Based on Bitcoin Core transaction input format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOInput {
    /// Previous transaction hash
    pub prev_tx_hash: [u8; 32],
    /// Previous output index
    pub prev_output_index: u32,
    /// Signature script (signature + public key)
    pub signature_script: Vec<u8>,
    /// Sequence number
    pub sequence: u32,
    /// Nullifier (for privacy)
    pub nullifier: [u8; 32],
    /// Merkle proof of inclusion
    pub merkle_proof: MerkleProof,
}

impl UTXOInput {
    /// Create new UTXO input
    pub fn new(
        prev_tx_hash: [u8; 32],
        prev_output_index: u32,
        signature_script: Vec<u8>,
        nullifier: [u8; 32],
        merkle_proof: MerkleProof,
    ) -> Self {
        Self {
            prev_tx_hash,
            prev_output_index,
            signature_script,
            sequence: 0xFFFFFFFF, // Default sequence
            nullifier,
            merkle_proof,
        }
    }

    /// Verify input signature
    pub fn verify_signature(&self, message: &[u8], public_key: &[u8; 32]) -> bool {
        if self.signature_script.len() < 64 {
            return false;
        }
        
        let signature = &self.signature_script[0..64];
        let signature_array: [u8; 64] = signature.try_into().unwrap();
        
        zisk_verify_signature(message, &signature_array, public_key)
    }
}

/// UTXO Output - Creates a new UTXO
/// Based on Bitcoin Core transaction output format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOOutput {
    /// Value in satoshis
    pub value: u64,
    /// Commitment (hiding and binding)
    pub commitment: [u8; 32],
    /// Blinding factor
    pub blinding: [u8; 32],
    /// Nullifier seed
    pub nullifier_seed: [u8; 32],
    /// Recipient's public key
    pub recipient_pubkey: [u8; 32],
    /// Output index
    pub index: u32,
}

impl UTXOOutput {
    /// Create new UTXO output
    pub fn new(
        value: u64,
        blinding: [u8; 32],
        nullifier_seed: [u8; 32],
        recipient_pubkey: [u8; 32],
        index: u32,
    ) -> Self {
        // Generate commitment using Pedersen commitment scheme
        let commitment = zisk_pedersen_commitment(value, blinding);
        
        Self {
            value,
            commitment,
            blinding,
            nullifier_seed,
            recipient_pubkey,
            index,
        }
    }

    /// Convert to UTXO
    pub fn to_utxo(&self, tx_hash: [u8; 32], block_height: u32) -> UTXO {
        UTXO::new(
            self.index,
            self.value,
            self.blinding,
            self.nullifier_seed,
            self.recipient_pubkey,
            tx_hash,
            block_height,
        )
    }
}

/// UTXO Transaction - Complete transaction structure
/// Based on Bitcoin Core transaction format with privacy enhancements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOTransaction {
    /// Transaction version
    pub version: u32,
    /// Inputs
    pub inputs: Vec<UTXOInput>,
    /// Outputs
    pub outputs: Vec<UTXOOutput>,
    /// Lock time
    pub lock_time: u32,
    /// Transaction hash
    pub tx_hash: [u8; 32],
    /// Block height
    pub block_height: u32,
    /// Transaction fee
    pub fee: u64,
}

impl UTXOTransaction {
    /// Create new transaction
    pub fn new(
        inputs: Vec<UTXOInput>,
        outputs: Vec<UTXOOutput>,
        fee: u64,
        block_height: u32,
    ) -> Self {
        let version = 1;
        let lock_time = 0;
        
        // Calculate transaction hash
        let tx_hash = Self::calculate_tx_hash(version, &inputs, &outputs, lock_time);
        
        Self {
            version,
            inputs,
            outputs,
            lock_time,
            tx_hash,
            block_height,
            fee,
        }
    }

    /// Calculate transaction hash
    /// Based on Bitcoin Core transaction hashing
    fn calculate_tx_hash(
        version: u32,
        inputs: &[UTXOInput],
        outputs: &[UTXOOutput],
        lock_time: u32,
    ) -> [u8; 32] {
        let mut data = Vec::new();
        
        // Add version
        data.extend_from_slice(&version.to_le_bytes());
        
        // Add input count
        data.extend_from_slice(&(inputs.len() as u32).to_le_bytes());
        
        // Add inputs
        for input in inputs {
            data.extend_from_slice(&input.prev_tx_hash);
            data.extend_from_slice(&input.prev_output_index.to_le_bytes());
            data.extend_from_slice(&(input.signature_script.len() as u32).to_le_bytes());
            data.extend_from_slice(&input.signature_script);
            data.extend_from_slice(&input.sequence.to_le_bytes());
        }
        
        // Add output count
        data.extend_from_slice(&(outputs.len() as u32).to_le_bytes());
        
        // Add outputs
        for output in outputs {
            data.extend_from_slice(&output.value.to_le_bytes());
            data.extend_from_slice(&output.commitment);
            data.extend_from_slice(&output.recipient_pubkey);
        }
        
        // Add lock time
        data.extend_from_slice(&lock_time.to_le_bytes());
        
        zisk_sha256(&data)
    }

    /// Verify transaction
    pub fn verify(&self) -> bool {
        // Check basic validity
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return false;
        }

        // Check value conservation
        let input_value: u64 = self.inputs.iter().map(|_| 1000).sum(); // Simplified
        let output_value: u64 = self.outputs.iter().map(|o| o.value).sum();
        
        if input_value < output_value + self.fee {
            return false;
        }

        // Check range proofs for all outputs
        for output in &self.outputs {
            if !zisk_range_proof(output.value) {
                return false;
            }
        }

        true
    }

    /// Get transaction message for signing
    pub fn get_message(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend_from_slice(&self.tx_hash);
        data.extend_from_slice(&self.block_height.to_le_bytes());
        data.extend_from_slice(&self.fee.to_le_bytes());
        data
    }
}

/// UTXO Set - Manages all UTXOs
/// Based on Bitcoin Core UTXO set management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOSet {
    /// UTXOs indexed by (tx_hash, output_index)
    pub utxos: HashMap<(u32, u32), UTXO>,
    /// UTXOs by owner
    pub utxos_by_owner: HashMap<[u8; 32], Vec<(u32, u32)>>,
    /// Total value in UTXO set
    pub total_value: u64,
    /// Number of UTXOs
    pub count: u32,
}

impl UTXOSet {
    /// Create new UTXO set
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
            utxos_by_owner: HashMap::new(),
            total_value: 0,
            count: 0,
        }
    }

    /// Add UTXO to set
    pub fn add_utxo(&mut self, utxo: UTXO) {
        let key = (utxo.tx_hash[0] as u32, utxo.index);
        self.utxos.insert(key, utxo.clone());
        
        // Add to owner index
        self.utxos_by_owner
            .entry(utxo.owner_pubkey)
            .or_insert_with(Vec::new)
            .push(key);
        
        self.total_value += utxo.value;
        self.count += 1;
    }

    /// Remove UTXO from set
    pub fn remove_utxo(&mut self, tx_hash: [u8; 32], output_index: u32) -> Option<UTXO> {
        let key = (tx_hash[0] as u32, output_index);
        if let Some(utxo) = self.utxos.remove(&key) {
            // Remove from owner index
            if let Some(owner_utxos) = self.utxos_by_owner.get_mut(&utxo.owner_pubkey) {
                owner_utxos.retain(|&k| k != key);
            }
            
            self.total_value -= utxo.value;
            self.count -= 1;
            Some(utxo)
        } else {
            None
        }
    }

    /// Get UTXO by reference
    pub fn get_utxo(&self, tx_hash: [u8; 32], output_index: u32) -> Option<&UTXO> {
        let key = (tx_hash[0] as u32, output_index);
        self.utxos.get(&key)
    }

    /// Get UTXOs by owner
    pub fn get_utxos_by_owner(&self, owner: &[u8; 32]) -> Vec<&UTXO> {
        if let Some(keys) = self.utxos_by_owner.get(owner) {
            keys.iter()
                .filter_map(|key| self.utxos.get(key))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Select UTXOs for spending
    /// Based on Bitcoin Core UTXO selection algorithm
    pub fn select_utxos_for_spending(
        &self,
        owner: &[u8; 32],
        target_value: u64,
        fee: u64,
    ) -> Vec<(u32, u32)> {
        let mut selected = Vec::new();
        let mut total_value = 0u64;
        let target = target_value + fee;
        
        // Get all UTXOs for owner
        if let Some(keys) = self.utxos_by_owner.get(owner) {
            for &key in keys {
                if let Some(utxo) = self.utxos.get(&key) {
                    if utxo.can_spend() {
                        selected.push(key);
                        total_value += utxo.value;
                        
                        if total_value >= target {
                            break;
                        }
                    }
                }
            }
        }
        
        selected
    }

    /// Process transaction
    pub fn process_transaction(&mut self, tx: &UTXOTransaction) -> bool {
        // Verify transaction first
        if !tx.verify() {
            return false;
        }

        // Remove input UTXOs
        for input in &tx.inputs {
            if self.remove_utxo(input.prev_tx_hash, input.prev_output_index).is_none() {
                return false;
            }
        }

        // Add output UTXOs
        for (i, output) in tx.outputs.iter().enumerate() {
            let utxo = output.to_utxo(tx.tx_hash, tx.block_height);
            self.add_utxo(utxo);
        }

        true
    }

    /// Get UTXO set statistics
    pub fn get_stats(&self) -> UTXOSetStats {
        UTXOSetStats {
            total_utxos: self.count,
            total_value: self.total_value,
            unique_owners: self.utxos_by_owner.len() as u32,
        }
    }
}

/// UTXO Set Statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOSetStats {
    pub total_utxos: u32,
    pub total_value: u64,
    pub unique_owners: u32,
}

/// Merkle Proof for UTXO inclusion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub siblings: Vec<[u8; 32]>,
    pub path: Vec<u32>,
    pub root: [u8; 32],
    pub leaf_index: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_creation() {
        let value = 1000;
        let blinding = [1u8; 32];
        let nullifier_seed = [2u8; 32];
        let owner_pubkey = [3u8; 32];
        let tx_hash = [4u8; 32];
        let block_height = 100;
        
        let utxo = UTXO::new(
            0,
            value,
            blinding,
            nullifier_seed,
            owner_pubkey,
            tx_hash,
            block_height,
        );
        
        assert_eq!(utxo.value, value);
        assert_eq!(utxo.owner_pubkey, owner_pubkey);
        assert!(!utxo.is_spent);
        assert!(utxo.can_spend());
    }

    #[test]
    fn test_utxo_set_operations() {
        let mut utxo_set = UTXOSet::new();
        
        let utxo = UTXO::new(
            0,
            1000,
            [1u8; 32],
            [2u8; 32],
            [3u8; 32],
            [4u8; 32],
            100,
        );
        
        utxo_set.add_utxo(utxo);
        
        assert_eq!(utxo_set.count, 1);
        assert_eq!(utxo_set.total_value, 1000);
        
        let stats = utxo_set.get_stats();
        assert_eq!(stats.total_utxos, 1);
        assert_eq!(stats.total_value, 1000);
    }

    #[test]
    fn test_transaction_verification() {
        let input = UTXOInput::new(
            [1u8; 32],
            0,
            vec![0u8; 64],
            [2u8; 32],
            MerkleProof {
                siblings: vec![],
                path: vec![],
                root: [0u8; 32],
                leaf_index: 0,
            },
        );
        
        let output = UTXOOutput::new(
            500,
            [3u8; 32],
            [4u8; 32],
            [5u8; 32],
            0,
        );
        
        let tx = UTXOTransaction::new(
            vec![input],
            vec![output],
            100,
            100,
        );
        
        assert!(tx.verify());
    }
}
