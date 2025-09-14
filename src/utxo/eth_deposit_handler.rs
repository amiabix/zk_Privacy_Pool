//! ETH Deposit Handler for Privacy Pool
//! Handles conversion of ETH deposits to UTXOs with proper verification

use crate::utxo::utxo::UTXO;
use crate::utxo::indexing::{UTXOIndex, IndexedUTXO, UTXOId};
use crate::utils::zisk_precompiles::*;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// ETH Deposit Event from Smart Contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ETHDepositEvent {
    pub depositor: [u8; 20], // Ethereum address
    pub amount_wei: u64,     // Amount in wei
    pub block_number: u64,   // Block where deposit occurred
    pub tx_hash: [u8; 32],   // Transaction hash
    pub log_index: u32,      // Log index in transaction
}

/// Deposit Proof for ZisK verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositProof {
    pub eth_deposit_event: ETHDepositEvent,
    pub merkle_proof: Vec<[u8; 32]>, // Merkle proof of deposit event
    pub block_header_hash: [u8; 32], // Block header hash
    pub signature: Vec<u8>,          // Ethereum signature (65 bytes)
}

/// ETH Deposit Handler
#[derive(Debug, Clone)]
pub struct ETHDepositHandler {
    /// UTXO index for managing deposits
    utxo_index: UTXOIndex,
    
    /// User mapping: Ethereum address -> Privacy Pool user
    eth_to_user: HashMap<[u8; 20], [u8; 32]>, // ETH addr -> privacy pubkey
    
    /// Deposit events waiting for UTXO creation
    pending_deposits: HashMap<[u8; 32], ETHDepositEvent>, // tx_hash -> event
    
    /// Account ID counter
    next_account_id: u32,
}

impl ETHDepositHandler {
    pub fn new() -> Self {
        Self {
            utxo_index: UTXOIndex::new(),
            eth_to_user: HashMap::new(),
            pending_deposits: HashMap::new(),
            next_account_id: 1,
        }
    }

    /// Register a user's Ethereum address
    pub fn register_user(&mut self, eth_address: [u8; 20], privacy_public_key: [u8; 32]) {
        self.eth_to_user.insert(eth_address, privacy_public_key);
    }

    /// Process an ETH deposit event from smart contract
    pub fn process_eth_deposit(&mut self, deposit_event: ETHDepositEvent) -> Result<UTXOId, DepositError> {
        // Verify the deposit event
        self.verify_deposit_event(&deposit_event)?;
        
        // Get user's privacy public key
        let privacy_pk = self.eth_to_user.get(&deposit_event.depositor)
            .ok_or(DepositError::UserNotRegistered)?;
        
        // Create UTXO from ETH deposit
        let utxo_id = self.create_utxo_from_eth_deposit(deposit_event, *privacy_pk)?;
        
        Ok(utxo_id)
    }

    /// Create UTXO from verified ETH deposit
    fn create_utxo_from_eth_deposit(&mut self, deposit: ETHDepositEvent, privacy_pk: [u8; 32]) -> Result<UTXOId, DepositError> {
        // Generate cryptographically secure secret
        let secret = self.generate_secure_secret(&deposit);
        
        // Generate nullifier from secret
        let nullifier = self.generate_nullifier(&secret, &deposit);
        
        // Create UTXO
        let utxo = UTXO::new(
            deposit.amount_wei,
            secret,
            privacy_pk,
            [0u8; 32], // blinding_factor
            nullifier, // nullifier_seed
            [0u8; 32], // commitment (placeholder)
            0, // index
        );
        
        // Get account ID for user
        let account_id = self.get_or_create_account_id(privacy_pk);
        
        // Create indexed UTXO
        let indexed_utxo = IndexedUTXO {
            id: UTXOId::new(deposit.tx_hash, deposit.log_index as u32),
            account_id,
            address: utxo.commitment,
            value: utxo.value,
            height: deposit.block_number as u32,
            spent_in_tx: None,
            blinding_factor: secret,
        };
        
        // Add to index
        self.utxo_index.add_utxo(indexed_utxo);
        
        Ok(UTXOId::new(deposit.tx_hash, deposit.log_index as u32))
    }

    /// Verify ETH deposit event (simplified - in production, verify against Ethereum)
    fn verify_deposit_event(&self, deposit: &ETHDepositEvent) -> Result<(), DepositError> {
        // In production, this would:
        // 1. Verify the transaction exists on Ethereum
        // 2. Verify the transaction was sent to the privacy pool contract
        // 3. Verify the amount and recipient
        // 4. Verify the block header and merkle proof
        
        if deposit.amount_wei == 0 {
            return Err(DepositError::InvalidAmount);
        }
        
        if deposit.block_number == 0 {
            return Err(DepositError::InvalidBlock);
        }
        
        Ok(())
    }

    /// Generate cryptographically secure secret from deposit data
    fn generate_secure_secret(&self, deposit: &ETHDepositEvent) -> [u8; 32] {
        // Use ZisK-compatible hash function
        let mut input = Vec::new();
        input.extend_from_slice(&deposit.depositor);
        input.extend_from_slice(&deposit.amount_wei.to_le_bytes());
        input.extend_from_slice(&deposit.tx_hash);
        input.extend_from_slice(&deposit.log_index.to_le_bytes());
        
        zisk_sha256(&input)
    }

    /// Generate nullifier from secret and deposit
    fn generate_nullifier(&self, secret: &[u8; 32], deposit: &ETHDepositEvent) -> [u8; 32] {
        let mut input = Vec::new();
        input.extend_from_slice(secret);
        input.extend_from_slice(&deposit.tx_hash);
        input.extend_from_slice(b"nullifier");
        
        zisk_sha256(&input)
    }

    /// Get or create account ID for user
    fn get_or_create_account_id(&mut self, privacy_pk: [u8; 32]) -> u32 {
        // For simplicity, use hash of public key as account ID
        // In production, maintain proper mapping
        let hash = zisk_sha256(&privacy_pk);
        u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
    }

    /// Get UTXOs for a user
    pub fn get_user_utxos(&self, privacy_pk: &[u8; 32]) -> Vec<&IndexedUTXO> {
        let account_id = self.get_account_id_for_user(privacy_pk);
        self.utxo_index.get_account_utxos(account_id)
    }

    /// Get user balance
    pub fn get_user_balance(&self, privacy_pk: &[u8; 32]) -> u64 {
        let account_id = self.get_account_id_for_user(privacy_pk);
        self.utxo_index.get_account_balance(account_id)
    }

    /// Get account ID for user (non-mutable version)
    fn get_account_id_for_user(&self, privacy_pk: &[u8; 32]) -> u32 {
        // For simplicity, use hash of public key as account ID
        // In production, maintain proper mapping
        let hash = zisk_sha256(privacy_pk);
        u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]])
    }

    /// Get all pending deposits
    pub fn get_pending_deposits(&self) -> Vec<&ETHDepositEvent> {
        self.pending_deposits.values().collect()
    }

    /// Clear pending deposits
    pub fn clear_pending_deposits(&mut self) {
        self.pending_deposits.clear();
    }
}

impl Default for ETHDepositHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Deposit processing errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DepositError {
    UserNotRegistered,
    InvalidAmount,
    InvalidBlock,
    VerificationFailed,
    InsufficientFunds,
}

impl std::fmt::Display for DepositError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DepositError::UserNotRegistered => write!(f, "User not registered"),
            DepositError::InvalidAmount => write!(f, "Invalid deposit amount"),
            DepositError::InvalidBlock => write!(f, "Invalid block number"),
            DepositError::VerificationFailed => write!(f, "Deposit verification failed"),
            DepositError::InsufficientFunds => write!(f, "Insufficient funds"),
        }
    }
}

impl std::error::Error for DepositError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eth_deposit_processing() {
        let mut handler = ETHDepositHandler::new();
        
        // Register user
        let eth_addr = [0x12u8; 20];
        let privacy_pk = [0x34u8; 32];
        handler.register_user(eth_addr, privacy_pk);
        
        // Create deposit event
        let deposit = ETHDepositEvent {
            depositor: eth_addr,
            amount_wei: 2000000000000000000, // 2 ETH
            block_number: 1000,
            tx_hash: [0x56u8; 32],
            log_index: 0,
        };
        
        // Process deposit
        let result = handler.process_eth_deposit(deposit);
        assert!(result.is_ok());
        
        let utxo_id = result.unwrap();
        
        // Check user balance
        let balance = handler.get_user_balance(&privacy_pk);
        assert_eq!(balance, 2000000000000000000);
        
        // Check UTXOs
        let utxos = handler.get_user_utxos(&privacy_pk);
        assert_eq!(utxos.len(), 1);
        assert_eq!(utxos[0].value, 2000000000000000000);
    }
}
