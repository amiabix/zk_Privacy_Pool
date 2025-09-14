//! Core UTXO Data Structures
//! Defines the fundamental UTXO types for the privacy pool system

use serde::{Serialize, Deserialize};
use crate::utxo::transaction::MerkleProof;

/// Core UTXO structure for the privacy pool
/// Based on Zcash Sapling note format with privacy enhancements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXO {
    /// Value in wei
    pub value: u64,
    /// Secret key for nullifier generation
    pub secret: [u8; 32],
    /// Owner's public key (32 bytes for Serde compatibility)
    pub owner: [u8; 32],
    /// Blinding factor for commitment
    pub blinding_factor: [u8; 32],
    /// Nullifier seed for spending
    pub nullifier_seed: [u8; 32],
    /// Commitment hash
    pub commitment: [u8; 32],
    /// Index in Merkle tree
    pub index: u64,
}

impl UTXO {
    /// Create a new UTXO with proper cryptographic setup
    pub fn new(
        value: u64,
        secret: [u8; 32],
        owner: [u8; 32],
        blinding_factor: [u8; 32],
        nullifier_seed: [u8; 32],
        commitment: [u8; 32],
        index: u64,
    ) -> Self {
        Self {
            value,
            secret,
            owner,
            blinding_factor,
            nullifier_seed,
            commitment,
            index,
        }
    }

    /// Generate nullifier for this UTXO
    pub fn generate_nullifier(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.secret);
        hasher.update(&self.nullifier_seed);
        hasher.update(&self.commitment);
        hasher.finalize().into()
    }

    /// Verify nullifier
    pub fn verify_nullifier(&self, nullifier: [u8; 32]) -> bool {
        self.generate_nullifier() == nullifier
    }

    /// Compute commitment hash
    pub fn compute_commitment(&self) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(&self.value.to_le_bytes());
        hasher.update(&self.owner);
        hasher.update(&self.blinding_factor);
        hasher.update(&self.nullifier_seed);
        hasher.finalize().into()
    }

    /// Verify ownership
    pub fn verify_ownership(&self, owner: &[u8; 32]) -> bool {
        self.owner == *owner
    }
}

/// User structure for managing UTXOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// User's public key
    pub public_key: [u8; 32],
    /// User's private key (for signing)
    pub private_key: [u8; 32],
    /// User's UTXOs
    pub utxos: Vec<UTXO>,
}

impl User {
    /// Create a new user
    pub fn new(public_key: [u8; 32], private_key: [u8; 32]) -> Self {
        Self {
            public_key,
            private_key,
            utxos: Vec::new(),
        }
    }

    /// Add a UTXO to the user
    pub fn add_utxo(&mut self, utxo: UTXO) {
        self.utxos.push(utxo);
    }

    /// Remove a UTXO by commitment
    pub fn remove_utxo(&mut self, commitment: [u8; 32]) -> Option<UTXO> {
        let index = self.utxos.iter().position(|u| u.commitment == commitment)?;
        Some(self.utxos.remove(index))
    }

    /// Get UTXOs by value range
    pub fn get_utxos_by_value_range(&self, min_value: u64, max_value: u64) -> Vec<&UTXO> {
        self.utxos
            .iter()
            .filter(|utxo| utxo.value >= min_value && utxo.value <= max_value)
            .collect()
    }

    /// Get total balance
    pub fn get_balance(&self) -> u64 {
        self.utxos.iter().map(|utxo| utxo.value).sum()
    }
}

/// UTXO Input for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOInput {
    /// UTXO being spent
    pub utxo: UTXO,
    /// Merkle proof of inclusion
    pub merkle_proof: MerkleProof,
    /// Nullifier for double-spend prevention
    pub nullifier: [u8; 32],
}

/// UTXO Output for transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOOutput {
    /// Value of the output
    pub value: u64,
    /// Recipient address
    pub recipient: [u8; 32],
    /// Commitment hash
    pub commitment: [u8; 32],
    /// Blinding factor
    pub blinding_factor: [u8; 32],
}

/// Transaction types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Transfer,
}

/// UTXO Transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOTransaction {
    /// Transaction type
    pub tx_type: TransactionType,
    /// Input UTXOs
    pub inputs: Vec<UTXOInput>,
    /// Output UTXOs
    pub outputs: Vec<UTXOOutput>,
    /// Transaction fee
    pub fee: u64,
    /// Signature
    pub signature: Vec<u8>,
    /// Public key of signer
    pub public_key: [u8; 32],
    /// Transaction hash
    pub tx_hash: [u8; 32],
}

impl UTXOTransaction {
    /// Create a new transaction
    pub fn new(
        tx_type: TransactionType,
        inputs: Vec<UTXOInput>,
        outputs: Vec<UTXOOutput>,
        fee: u64,
        signature: Vec<u8>,
        public_key: [u8; 32],
    ) -> Self {
        let tx_hash = Self::compute_hash(&tx_type, &inputs, &outputs, fee);
        Self {
            tx_type,
            inputs,
            outputs,
            fee,
            signature,
            public_key,
            tx_hash,
        }
    }

    /// Compute transaction hash
    fn compute_hash(
        tx_type: &TransactionType,
        inputs: &[UTXOInput],
        outputs: &[UTXOOutput],
        fee: u64,
    ) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        
        // Add transaction type
        hasher.update(&(tx_type.clone() as u8).to_le_bytes());
        
        // Add input commitments
        for input in inputs {
            hasher.update(&input.utxo.commitment);
        }
        
        // Add output values and recipients
        for output in outputs {
            hasher.update(&output.value.to_le_bytes());
            hasher.update(&output.recipient);
        }
        
        // Add fee
        hasher.update(&fee.to_le_bytes());
        
        hasher.finalize().into()
    }

    /// Verify transaction signature
    pub fn verify_signature(&self) -> bool {
        // Simplified signature verification
        // In this would use proper cryptographic verification
        !self.signature.is_empty() && self.signature.len() == 64
    }

    /// Get total input value
    pub fn get_total_input_value(&self) -> u64 {
        self.inputs.iter().map(|input| input.utxo.value).sum()
    }

    /// Get total output value
    pub fn get_total_output_value(&self) -> u64 {
        self.outputs.iter().map(|output| output.value).sum()
    }

    /// Verify transaction balance
    pub fn verify_balance(&self) -> bool {
        let total_input = self.get_total_input_value();
        let total_output = self.get_total_output_value();
        total_input >= total_output + self.fee
    }
}

/// Merkle proof structure for UTXO inclusion
// MerkleProof is now defined in transaction.rs to avoid duplication

// MerkleProof implementation is now in transaction.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_creation() {
        let utxo = UTXO::new(
            1000000000000000000u64, // 1 ETH
            [0x42u8; 32],
            [0x43u8; 32],
            [0x44u8; 32],
            [0x45u8; 32],
            [0x46u8; 32],
            0,
        );

        assert_eq!(utxo.value, 1000000000000000000u64);
        assert_eq!(utxo.secret, [0x42u8; 32]);
        assert_eq!(utxo.owner, [0x43u8; 32]);
    }

    #[test]
    fn test_utxo_nullifier_generation() {
        let utxo = UTXO::new(
            1000000000000000000u64,
            [0x42u8; 32],
            [0x43u8; 32],
            [0x44u8; 32],
            [0x45u8; 32],
            [0x46u8; 32],
            0,
        );

        let nullifier = utxo.generate_nullifier();
        assert!(utxo.verify_nullifier(nullifier));
    }

    #[test]
    fn test_user_management() {
        let mut user = User::new([0x43u8; 32], [0x44u8; 32]);
        
        let utxo = UTXO::new(
            1000000000000000000u64,
            [0x42u8; 32],
            [0x43u8; 32],
            [0x44u8; 32],
            [0x45u8; 32],
            [0x46u8; 32],
            0,
        );

        user.add_utxo(utxo);
        assert_eq!(user.utxos.len(), 1);
        assert_eq!(user.get_balance(), 1000000000000000000u64);
    }

    #[test]
    fn test_transaction_creation() {
        let utxo = UTXO::new(
            1000000000000000000u64,
            [0x42u8; 32],
            [0x43u8; 32],
            [0x44u8; 32],
            [0x45u8; 32],
            [0x46u8; 32],
            0,
        );

        let input = UTXOInput {
            utxo: utxo.clone(),
            merkle_proof: MerkleProof::new(vec![[0u8; 32]], vec![0], [0u8; 32], 0),
            nullifier: [0u8; 32],
        };

        let output = UTXOOutput {
            value: 500000000000000000u64,
            recipient: [0x50u8; 32],
            commitment: [0x51u8; 32],
            blinding_factor: [0x52u8; 32],
        };

        let tx = UTXOTransaction::new(
            TransactionType::Transfer,
            vec![input],
            vec![output],
            10000000000000000u64, // 0.01 ETH fee
            vec![0u8; 64],
            [0x43u8; 32],
        );

        assert_eq!(tx.tx_type, TransactionType::Transfer);
        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 1);
        assert!(tx.verify_balance());
    }
}