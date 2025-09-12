use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
pub struct UTXO {
    pub commitment: [u8; 32],
    pub value: u64,
    pub secret: [u8; 32],
    pub nullifier: [u8; 32],
    pub owner: [u8; 32], // User's public key
    pub index: u64,      // Position in Merkle tree
}

impl UTXO {
    pub fn new(value: u64, secret: [u8; 32], nullifier: [u8; 32], owner: [u8; 32]) -> Self {
        let commitment = Self::compute_commitment(value, &secret, &nullifier);
        Self {
            commitment,
            value,
            secret,
            nullifier,
            owner,
            index: 0, // Will be set when added to tree
        }
    }
    
    pub fn compute_commitment(value: u64, secret: &[u8; 32], nullifier: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&value.to_le_bytes());
        hasher.update(secret);
        hasher.update(nullifier);
        hasher.finalize().into()
    }
    
    pub fn compute_nullifier_hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&self.nullifier);
        hasher.finalize().into()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOInput {
    pub utxo: UTXO,
    pub merkle_proof: MerkleProof,
    pub secret: [u8; 32], // Private key to spend
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOOutput {
    pub value: u64,
    pub secret: [u8; 32],
    pub nullifier: [u8; 32],
    pub recipient: [u8; 32],
}

impl UTXOOutput {
    pub fn to_utxo(&self, _index: u64) -> UTXO {
        UTXO::new(self.value, self.secret, self.nullifier, self.recipient)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UTXOTransaction {
    pub inputs: Vec<UTXOInput>,
    pub outputs: Vec<UTXOOutput>,
    pub fee: u64,
    pub tx_type: TransactionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit { amount: u64, recipient: [u8; 32] },
    Withdraw { amount: u64, recipient: [u8; 32] },
    Transfer { recipient: [u8; 32] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub siblings: Vec<[u8; 32]>,
    pub path: Vec<bool>,
    pub root: [u8; 32],
    pub leaf_index: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub public_key: [u8; 32],
    pub private_key: [u8; 32],
    pub utxos: Vec<UTXO>,
    pub balance: u64,
}

impl User {
    pub fn new(public_key: [u8; 32], private_key: [u8; 32]) -> Self {
        Self {
            public_key,
            private_key,
            utxos: Vec::new(),
            balance: 0,
        }
    }
    
    pub fn add_utxo(&mut self, utxo: UTXO) {
        let value = utxo.value;
        self.utxos.push(utxo);
        self.balance += value;
    }
    
    pub fn remove_utxo(&mut self, commitment: &[u8; 32]) -> Option<UTXO> {
        if let Some(pos) = self.utxos.iter().position(|u| u.commitment == *commitment) {
            let utxo = self.utxos.remove(pos);
            self.balance -= utxo.value;
            Some(utxo)
        } else {
            None
        }
    }
    
    pub fn can_spend(&self, amount: u64) -> bool {
        self.balance >= amount
    }
    
    pub fn select_utxos_for_spending(&self, amount: u64) -> Vec<&UTXO> {
        let mut selected = Vec::new();
        let mut total = 0u64;
        
        for utxo in &self.utxos {
            selected.push(utxo);
            total += utxo.value;
            if total >= amount {
                break;
            }
        }
        
        selected
    }
}
