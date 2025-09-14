//! UTXO Manager with SMT Integration
//! 
//! This module provides the complete UTXO lifecycle management
//! integrated with the canonical SMT tree operations.

use anyhow::{Result, anyhow, Context};
use crate::database::schema::DatabaseManager;
use crate::database::batch_writer::{AtomicBatchWriter, BatchOperation};
use crate::utxo::CanonicalUTXO;
use crate::merkle::CanonicalSMT;
use crate::relayer::DepositEvent;
use web3::types::{Address, U256};

/// Comprehensive UTXO manager with SMT integration
pub struct UTXOManager {
    /// Database manager
    db: DatabaseManager,
    
    /// Canonical SMT tree
    smt: CanonicalSMT,
    
    /// Current operator entropy for UTXO ID generation
    operator_entropy_counter: u64,
}

/// Result of UTXO operations
#[derive(Debug, Clone)]
pub struct UTXOOperationResult {
    /// The UTXO that was created/updated
    pub utxo: CanonicalUTXO,
    
    /// New tree root after operation
    pub new_root: [u8; 32],
    
    /// Tree root version
    pub root_version: u64,
    
    /// Tree position where UTXO was placed
    pub tree_position: u64,
    
    /// Leaf hash of the UTXO
    pub leaf_hash: [u8; 32],
}

/// Deposit processing result
#[derive(Debug, Clone)]
pub struct DepositResult {
    /// Operation result
    pub operation: UTXOOperationResult,
    
    /// Original deposit event
    pub deposit_event: DepositEvent,
    
    /// Processing timestamp
    pub processed_at: u64,
}

impl UTXOManager {
    /// Create new UTXO manager
    pub fn new(db: DatabaseManager) -> Result<Self> {
        let smt = CanonicalSMT::with_default_config(db.clone())?;
        
        Ok(Self {
            db,
            smt,
            operator_entropy_counter: rand::random::<u64>(),
        })
    }

    /// Create UTXO manager with specific tree configuration
    pub fn with_tree_config(db: DatabaseManager, tree_depth: u8, tree_salt: u64) -> Result<Self> {
        let smt = CanonicalSMT::new(db.clone(), tree_depth, tree_salt)?;
        
        Ok(Self {
            db,
            smt,
            operator_entropy_counter: rand::random::<u64>(),
        })
    }

    /// Process ETH deposit into UTXO with full SMT integration
    pub fn process_eth_deposit(&mut self, deposit_event: DepositEvent) -> Result<DepositResult> {
        // Generate next entropy value
        self.operator_entropy_counter = self.operator_entropy_counter.wrapping_add(1);
        
        // Derive privacy-preserving owner commitment from deposit
        let owner_commitment = self.derive_owner_commitment(&deposit_event)?;
        
        // Create canonical UTXO
        let utxo = CanonicalUTXO::new_eth(
            deposit_event.transaction_hash.0,  // txid
            0,                                 // vout (always 0 for deposits)
            deposit_event.block_number,        // created_block
            self.operator_entropy_counter,     // entropy
            deposit_event.value.as_u128(),     // amount in wei
            owner_commitment,                  // privacy commitment
        );

        // Insert UTXO into tree and database atomically
        let operation_result = self.insert_utxo_with_tree_update(utxo)?;
        
        Ok(DepositResult {
            operation: operation_result,
            deposit_event,
            processed_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }

    /// Insert UTXO with complete tree and database updates
    pub fn insert_utxo_with_tree_update(&mut self, utxo: CanonicalUTXO) -> Result<UTXOOperationResult> {
        // Validate UTXO
        utxo.validate()
            .context("UTXO validation failed")?;

        // Get tree position
        let tree_position = crate::canonical_spec::generate_tree_index(
            utxo.utxo_id, 
            self.smt.get_tree_salt()
        );
        let leaf_hash = utxo.leaf_hash()?;

        // Create atomic batch for all operations
        let mut batch_writer = AtomicBatchWriter::new(self.db.clone());

        // Phase 1: cf_spent_tracker - SKIP (new UTXO)

        // Phase 2: cf_utxos - Insert UTXO
        batch_writer.add_operation(BatchOperation::InsertUTXO { 
            utxo: utxo.clone() 
        });

        // Phase 3: SMT tree update (this will be handled by SMT)
        // We'll update the tree first, then record the nodes

        // Phase 4: cf_smt_leaves - Store leaf mapping
        batch_writer.add_operation(BatchOperation::UpdateSMTLeaf {
            utxo_id: utxo.utxo_id,
            leaf_hash,
            tree_position,
        });

        // Phase 5: cf_asset_balances - Update aggregated balance
        batch_writer.add_operation(BatchOperation::UpdateAssetBalance {
            owner_commitment: utxo.owner_commitment,
            asset_id: utxo.asset_id,
            amount_delta: utxo.amount as i128,
            utxo_count_delta: 1,
            last_updated_block: utxo.created_block,
        });

        // Phase 6: cf_owner_index - Insert ownership record
        batch_writer.add_operation(BatchOperation::InsertOwnerIndex {
            owner_commitment: utxo.owner_commitment,
            created_block: utxo.created_block,
            utxo_id: utxo.utxo_id,
            amount: utxo.amount,
            asset_id: utxo.asset_id,
            flags: utxo.lock_flags,
        });

        // Insert into SMT and get new root
        let new_root = self.smt.insert_utxo(&utxo)
            .context("Failed to insert UTXO into SMT")?;

        // Phase 7: cf_root_history - Commit new root
        batch_writer.add_operation(BatchOperation::CommitRoot {
            root_version: self.smt.get_root_version(),
            root_hash: new_root,
            batch_id: self.smt.get_root_version(), // Use root version as batch ID
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_count: 1,
            operator_signature: self.sign_root(new_root)?,
        });

        // Phases 8-10: cf_input_locks, cf_mempool, cf_block_index - SKIP for deposits

        // Execute all operations atomically
        batch_writer.commit()
            .context("Failed to commit UTXO insertion batch")?;

        Ok(UTXOOperationResult {
            utxo,
            new_root,
            root_version: self.smt.get_root_version(),
            tree_position,
            leaf_hash,
        })
    }

    /// Remove UTXO (mark as spent) with tree update
    pub fn remove_utxo(&mut self, utxo_id: &[u8; 32], spent_txid: [u8; 32]) -> Result<UTXOOperationResult> {
        // Get the UTXO first
        let utxo_data = self.db.get_cf("cf_utxos", &self.create_utxo_key(utxo_id))?
            .ok_or_else(|| anyhow!("UTXO not found: {:?}", utxo_id))?;
        let utxo = CanonicalUTXO::deserialize(&utxo_data)?;

        let tree_position = crate::canonical_spec::generate_tree_index(
            utxo.utxo_id, 
            self.smt.get_tree_salt()
        );

        // Create atomic batch
        let mut batch_writer = AtomicBatchWriter::new(self.db.clone());

        // Phase 1: cf_spent_tracker - Mark as spent
        batch_writer.add_operation(BatchOperation::MarkSpent {
            utxo_id: *utxo_id,
            spent_txid,
            spent_block: 0, // Would be filled with actual block
            spent_timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // Phase 2: cf_utxos - Delete UTXO
        batch_writer.add_operation(BatchOperation::DeleteUTXO {
            utxo_id: *utxo_id,
        });

        // Remove from SMT and get new root
        let new_root = self.smt.remove_utxo(utxo_id)
            .context("Failed to remove UTXO from SMT")?;

        // Phase 4: cf_smt_leaves - Remove leaf mapping
        batch_writer.add_operation(BatchOperation::DeleteSMTLeaf {
            utxo_id: *utxo_id,
        });

        // Phase 5: cf_asset_balances - Update balance
        batch_writer.add_operation(BatchOperation::UpdateAssetBalance {
            owner_commitment: utxo.owner_commitment,
            asset_id: utxo.asset_id,
            amount_delta: -(utxo.amount as i128),
            utxo_count_delta: -1,
            last_updated_block: 0, // Would be current block
        });

        // Phase 6: cf_owner_index - Remove ownership record
        batch_writer.add_operation(BatchOperation::DeleteOwnerIndex {
            owner_commitment: utxo.owner_commitment,
            created_block: utxo.created_block,
            utxo_id: *utxo_id,
        });

        // Phase 7: cf_root_history - Commit new root
        batch_writer.add_operation(BatchOperation::CommitRoot {
            root_version: self.smt.get_root_version(),
            root_hash: new_root,
            batch_id: self.smt.get_root_version(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_count: 1,
            operator_signature: self.sign_root(new_root)?,
        });

        // Execute atomically
        batch_writer.commit()
            .context("Failed to commit UTXO removal batch")?;

        Ok(UTXOOperationResult {
            utxo,
            new_root,
            root_version: self.smt.get_root_version(),
            tree_position,
            leaf_hash: crate::canonical_spec::generate_empty_leaf_hash(),
        })
    }

    /// Batch process multiple deposits efficiently
    pub fn batch_process_deposits(&mut self, deposit_events: &[DepositEvent]) -> Result<Vec<DepositResult>> {
        let mut results = Vec::new();
        let mut utxos = Vec::new();

        // Create all UTXOs first
        for deposit_event in deposit_events {
            self.operator_entropy_counter = self.operator_entropy_counter.wrapping_add(1);
            
            let owner_commitment = self.derive_owner_commitment(deposit_event)?;
            
            let utxo = CanonicalUTXO::new_eth(
                deposit_event.transaction_hash.0,
                0,
                deposit_event.block_number,
                self.operator_entropy_counter,
                deposit_event.value.as_u128(),
                owner_commitment,
            );

            utxos.push(utxo);
        }

        // Batch insert into tree
        let new_root = self.smt.batch_insert_utxos(&utxos)?;

        // Create batch writer for database operations
        let mut batch_writer = AtomicBatchWriter::new(self.db.clone());

        // Add all database operations
        for (i, utxo) in utxos.iter().enumerate() {
            let tree_position = crate::canonical_spec::generate_tree_index(
                utxo.utxo_id, 
                self.smt.get_tree_salt()
            );

            // Insert UTXO
            batch_writer.add_operation(BatchOperation::InsertUTXO { 
                utxo: utxo.clone() 
            });

            // Store leaf mapping
            batch_writer.add_operation(BatchOperation::UpdateSMTLeaf {
                utxo_id: utxo.utxo_id,
                leaf_hash: utxo.leaf_hash()?,
                tree_position,
            });

            // Update balances
            batch_writer.add_operation(BatchOperation::UpdateAssetBalance {
                owner_commitment: utxo.owner_commitment,
                asset_id: utxo.asset_id,
                amount_delta: utxo.amount as i128,
                utxo_count_delta: 1,
                last_updated_block: utxo.created_block,
            });

            // Insert owner index
            batch_writer.add_operation(BatchOperation::InsertOwnerIndex {
                owner_commitment: utxo.owner_commitment,
                created_block: utxo.created_block,
                utxo_id: utxo.utxo_id,
                amount: utxo.amount,
                asset_id: utxo.asset_id,
                flags: utxo.lock_flags,
            });

            // Create result
            results.push(DepositResult {
                operation: UTXOOperationResult {
                    utxo: utxo.clone(),
                    new_root,
                    root_version: self.smt.get_root_version(),
                    tree_position,
                    leaf_hash: utxo.leaf_hash()?,
                },
                deposit_event: deposit_events[i].clone(),
                processed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            });
        }

        // Commit new root
        batch_writer.add_operation(BatchOperation::CommitRoot {
            root_version: self.smt.get_root_version(),
            root_hash: new_root,
            batch_id: self.smt.get_root_version(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tx_count: utxos.len() as u32,
            operator_signature: self.sign_root(new_root)?,
        });

        // Execute all operations atomically
        batch_writer.commit()
            .context("Failed to commit batch deposit processing")?;

        Ok(results)
    }

    /// Get current tree statistics
    pub fn get_tree_stats(&self) -> Result<crate::merkle::TreeStats> {
        self.smt.get_tree_stats()
    }

    /// Get current tree root
    pub fn get_current_root(&self) -> [u8; 32] {
        self.smt.get_root()
    }

    /// Get current root version
    pub fn get_root_version(&self) -> u64 {
        self.smt.get_root_version()
    }

    // Helper methods

    /// Derive privacy-preserving owner commitment from deposit
    fn derive_owner_commitment(&self, deposit: &DepositEvent) -> Result<[u8; 32]> {
        // For now, use a simple hash of depositor + commitment
        // In this would use more sophisticated privacy-preserving derivation
        use sha3::{Keccak256, Digest};
        
        let mut hasher = Keccak256::new();
        hasher.update(b"OWNER_COMMITMENT"); // Domain separator
        hasher.update(deposit.depositor.as_bytes());
        hasher.update(deposit.commitment.as_bytes());
        hasher.update(&deposit.block_number.to_be_bytes());
        
        Ok(hasher.finalize().into())
    }

    /// Create UTXO database key
    fn create_utxo_key(&self, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(0x01); // cf_prefix::UTXOS
        key.extend_from_slice(utxo_id);
        key
    }

    /// Sign tree root (placeholder implementation)
    fn sign_root(&self, root: [u8; 32]) -> Result<Vec<u8>> {
        // Placeholder: In this would use the operator's private key
        // to sign the root hash for accountability
        use sha3::{Keccak256, Digest};
        
        let mut hasher = Keccak256::new();
        hasher.update(b"OPERATOR_SIGNATURE");
        hasher.update(&root);
        hasher.update(&std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .to_be_bytes());
        
        Ok(hasher.finalize().to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::database::schema::DBConfig;
    use web3::types::H256;

    #[test]
    fn test_utxo_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db").to_string_lossy().to_string();
        
        let config = DBConfig {
            db_path,
            ..Default::default()
        };
        
        let db_manager = DatabaseManager::open(config).unwrap();
        let _utxo_manager = UTXOManager::new(db_manager).unwrap();
    }

    #[test]
    fn test_deposit_processing() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db").to_string_lossy().to_string();
        
        let config = DBConfig {
            db_path,
            ..Default::default()
        };
        
        let db_manager = DatabaseManager::open(config).unwrap();
        let mut utxo_manager = UTXOManager::new(db_manager).unwrap();
        
        let initial_root = utxo_manager.get_current_root();
        
        // Create test deposit event
        let deposit_event = DepositEvent {
            depositor: Address::from_low_u64_be(12345),
            commitment: H256::from_low_u64_be(67890),
            label: U256::from(0),
            value: U256::from(1_000_000_000_000_000_000u64), // 1 ETH
            precommitment_hash: H256::from_low_u64_be(11111),
            block_number: 12345,
            transaction_hash: H256::from_low_u64_be(54321),
            log_index: 0,
        };
        
        // Process deposit
        let result = utxo_manager.process_eth_deposit(deposit_event).unwrap();
        
        // Verify results
        assert_ne!(initial_root, result.operation.new_root);
        assert_eq!(result.operation.utxo.amount, 1_000_000_000_000_000_000u128);
        assert_eq!(utxo_manager.get_current_root(), result.operation.new_root);
        assert_eq!(utxo_manager.get_root_version(), 1);
    }
}