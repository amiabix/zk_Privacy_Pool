//! Atomic Batch Writer Implementation
//! 
//! Critical: All state transitions MUST use the exact WriteBatch ordering
//! specified in the canonical specification to prevent deadlocks and ensure consistency.

use anyhow::{Result, anyhow, Context};
use crate::database::schema::{DatabaseManager, cf_names};
use crate::canonical_spec::cf_prefixes;
use crate::utxo::CanonicalUTXO;

/// Batch operation types for atomic state transitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BatchOperation {
    /// Mark UTXO as spent (cf_spent_tracker)
    MarkSpent {
        utxo_id: [u8; 32],
        spent_txid: [u8; 32],
        spent_block: u64,
        spent_timestamp: u64,
    },
    
    /// Delete spent UTXO (cf_utxos)
    DeleteUTXO {
        utxo_id: [u8; 32],
    },
    
    /// Insert new UTXO (cf_utxos)
    InsertUTXO {
        utxo: CanonicalUTXO,
    },
    
    /// Update SMT node with reference counting (cf_smt_nodes)
    UpdateSMTNode {
        node_hash: [u8; 32],
        left_hash: [u8; 32],
        right_hash: [u8; 32],
        height: u8,
        ref_count_delta: i32, // can be negative for decrements
    },
    
    /// Update SMT leaf mapping (cf_smt_leaves)
    UpdateSMTLeaf {
        utxo_id: [u8; 32],
        leaf_hash: [u8; 32],
        tree_position: u64,
    },
    
    /// Delete SMT leaf (cf_smt_leaves)
    DeleteSMTLeaf {
        utxo_id: [u8; 32],
    },
    
    /// Update asset balance (cf_asset_balances)
    UpdateAssetBalance {
        owner_commitment: [u8; 32],
        asset_id: [u8; 20],
        amount_delta: i128, // can be negative
        utxo_count_delta: i32, // can be negative
        last_updated_block: u64,
    },
    
    /// Insert owner index entry (cf_owner_index)
    InsertOwnerIndex {
        owner_commitment: [u8; 32],
        created_block: u64,
        utxo_id: [u8; 32],
        amount: u128,
        asset_id: [u8; 20],
        flags: u8,
    },
    
    /// Delete owner index entry (cf_owner_index)
    DeleteOwnerIndex {
        owner_commitment: [u8; 32],
        created_block: u64,
        utxo_id: [u8; 32],
    },
    
    /// Commit new root (cf_root_history)
    CommitRoot {
        root_version: u64,
        root_hash: [u8; 32],
        batch_id: u64,
        timestamp: u64,
        tx_count: u32,
        operator_signature: Vec<u8>,
    },
    
    /// Release input lock (cf_input_locks)
    ReleaseInputLock {
        utxo_id: [u8; 32],
    },
    
    /// Remove transaction from mempool (cf_mempool)
    RemoveFromMempool {
        priority: u8,
        fee_rate: u64,
        txid: [u8; 32],
    },
    
    /// Record block operation (cf_block_index)
    RecordBlockOperation {
        block_number: u64,
        tx_index: u32,
        operation_id: [u8; 16],
        operation_type: u8,
        utxo_id: [u8; 32],
        prev_state_hash: [u8; 32],
    },
}

/// Atomic batch writer with mandatory ordering
pub struct AtomicBatchWriter {
    db: DatabaseManager,
    operations: Vec<BatchOperation>,
}

impl AtomicBatchWriter {
    /// Create new batch writer
    pub fn new(db: DatabaseManager) -> Self {
        Self {
            db,
            operations: Vec::new(),
        }
    }

    /// Add operation to batch (will be sorted by execution order)
    pub fn add_operation(&mut self, operation: BatchOperation) {
        self.operations.push(operation);
    }

    /// Execute all operations atomically with mandatory ordering
    /// 
    /// CRITICAL: This order must NEVER be changed as it prevents deadlocks:
    /// 1. cf_spent_tracker (mark consumed UTXOs first)
    /// 2. cf_utxos (delete spent, insert new)  
    /// 3. cf_smt_nodes (decrement ref counts, insert new nodes)
    /// 4. cf_smt_leaves (update tree leaf mappings)
    /// 5. cf_asset_balances (update aggregated balances)
    /// 6. cf_owner_index (update ownership indices)
    /// 7. cf_root_history (commit new root)
    /// 8. cf_input_locks (release consumed locks)
    /// 9. cf_mempool (remove processed transactions)
    /// 10. cf_block_index (record operations)
    pub fn commit(self) -> Result<()> {
        if self.operations.is_empty() {
            return Ok(());
        }

        let mut batch = self.db.create_write_batch();

        // Phase 1: cf_spent_tracker (mark consumed UTXOs first)
        for operation in &self.operations {
            if let BatchOperation::MarkSpent { 
                utxo_id, spent_txid, spent_block, spent_timestamp 
            } = operation {
                let key = self.create_spent_tracker_key(utxo_id);
                let value = self.create_spent_tracker_value(*spent_txid, *spent_block, *spent_timestamp);
                let cf = self.db.cf_handle(cf_names::SPENT_TRACKER)?;
                batch.put_cf(cf, &key, &value);
            }
        }

        // Phase 2: cf_utxos (delete spent, insert new)
        for operation in &self.operations {
            match operation {
                BatchOperation::DeleteUTXO { utxo_id } => {
                    let key = self.create_utxo_key(utxo_id);
                    let cf = self.db.cf_handle(cf_names::UTXOS)?;
                    batch.delete_cf(cf, &key);
                },
                BatchOperation::InsertUTXO { utxo } => {
                    let key = utxo.db_key();
                    let value = utxo.serialize()
                        .context("Failed to serialize UTXO")?;
                    let cf = self.db.cf_handle(cf_names::UTXOS)?;
                    batch.put_cf(cf, &key, &value);
                },
                _ => {}
            }
        }

        // Phase 3: cf_smt_nodes (decrement ref counts, insert new nodes)
        for operation in &self.operations {
            if let BatchOperation::UpdateSMTNode { 
                node_hash, left_hash, right_hash, height, ref_count_delta 
            } = operation {
                let key = self.create_smt_node_key(node_hash);
                
                if *ref_count_delta < 0 {
                    // Handle reference count decrement (possibly delete)
                    let cf = self.db.cf_handle(cf_names::SMT_NODES)?;
                    
                    if let Some(existing_value) = self.db.get_cf(cf_names::SMT_NODES, &key)? {
                        let current_ref_count = self.parse_smt_node_ref_count(&existing_value)?;
                        let new_ref_count = (current_ref_count as i32) + ref_count_delta;
                        
                        if new_ref_count <= 0 {
                            // Delete node when ref count reaches zero
                            batch.delete_cf(cf, &key);
                        } else {
                            // Update with new ref count
                            let value = self.create_smt_node_value(
                                *left_hash, *right_hash, *height, new_ref_count as u32
                            );
                            batch.put_cf(cf, &key, &value);
                        }
                    }
                } else {
                    // Handle reference count increment or new node
                    let cf = self.db.cf_handle(cf_names::SMT_NODES)?;
                    
                    let new_ref_count = if let Some(existing_value) = self.db.get_cf(cf_names::SMT_NODES, &key)? {
                        let current_ref_count = self.parse_smt_node_ref_count(&existing_value)?;
                        (current_ref_count as i32) + ref_count_delta
                    } else {
                        *ref_count_delta
                    };
                    
                    if new_ref_count > 0 {
                        let value = self.create_smt_node_value(
                            *left_hash, *right_hash, *height, new_ref_count as u32
                        );
                        batch.put_cf(cf, &key, &value);
                    }
                }
            }
        }

        // Phase 4: cf_smt_leaves (update tree leaf mappings)
        for operation in &self.operations {
            match operation {
                BatchOperation::UpdateSMTLeaf { utxo_id, leaf_hash, tree_position } => {
                    let key = self.create_smt_leaf_key(utxo_id);
                    let value = self.create_smt_leaf_value(*leaf_hash, *tree_position);
                    let cf = self.db.cf_handle(cf_names::SMT_LEAVES)?;
                    batch.put_cf(cf, &key, &value);
                },
                BatchOperation::DeleteSMTLeaf { utxo_id } => {
                    let key = self.create_smt_leaf_key(utxo_id);
                    let cf = self.db.cf_handle(cf_names::SMT_LEAVES)?;
                    batch.delete_cf(cf, &key);
                },
                _ => {}
            }
        }

        // Phase 5: cf_asset_balances (update aggregated balances)
        for operation in &self.operations {
            if let BatchOperation::UpdateAssetBalance { 
                owner_commitment, asset_id, amount_delta, utxo_count_delta, last_updated_block 
            } = operation {
                let key = self.create_asset_balance_key(owner_commitment, asset_id);
                let cf = self.db.cf_handle(cf_names::ASSET_BALANCES)?;
                
                // Get current balance or create new
                let (current_amount, current_count, _) = if let Some(existing_value) = self.db.get_cf(cf_names::ASSET_BALANCES, &key)? {
                    self.parse_asset_balance_value(&existing_value)?
                } else {
                    (0u128, 0u32, 0u64)
                };
                
                let new_amount = if *amount_delta < 0 {
                    current_amount.saturating_sub((-amount_delta) as u128)
                } else {
                    current_amount.saturating_add(*amount_delta as u128)
                };
                
                let new_count = if *utxo_count_delta < 0 {
                    current_count.saturating_sub((-utxo_count_delta) as u32)
                } else {
                    current_count.saturating_add(*utxo_count_delta as u32)
                };
                
                let value = self.create_asset_balance_value(new_amount, new_count, *last_updated_block);
                batch.put_cf(cf, &key, &value);
            }
        }

        // Phase 6: cf_owner_index (update ownership indices)
        for operation in &self.operations {
            match operation {
                BatchOperation::InsertOwnerIndex { 
                    owner_commitment, created_block, utxo_id, amount, asset_id, flags 
                } => {
                    let key = self.create_owner_index_key(owner_commitment, *created_block, utxo_id);
                    let value = self.create_owner_index_value(*amount, asset_id, *flags);
                    let cf = self.db.cf_handle(cf_names::OWNER_INDEX)?;
                    batch.put_cf(cf, &key, &value);
                },
                BatchOperation::DeleteOwnerIndex { 
                    owner_commitment, created_block, utxo_id 
                } => {
                    let key = self.create_owner_index_key(owner_commitment, *created_block, utxo_id);
                    let cf = self.db.cf_handle(cf_names::OWNER_INDEX)?;
                    batch.delete_cf(cf, &key);
                },
                _ => {}
            }
        }

        // Phase 7: cf_root_history (commit new root)
        for operation in &self.operations {
            if let BatchOperation::CommitRoot { 
                root_version, root_hash, batch_id, timestamp, tx_count, operator_signature 
            } = operation {
                let key = self.create_root_history_key(*root_version);
                let value = self.create_root_history_value(
                    *root_hash, *batch_id, *timestamp, *tx_count, operator_signature
                );
                let cf = self.db.cf_handle(cf_names::ROOT_HISTORY)?;
                batch.put_cf(cf, &key, &value);
            }
        }

        // Phase 8: cf_input_locks (release consumed locks)
        for operation in &self.operations {
            if let BatchOperation::ReleaseInputLock { utxo_id } = operation {
                let key = self.create_input_lock_key(utxo_id);
                let cf = self.db.cf_handle(cf_names::INPUT_LOCKS)?;
                batch.delete_cf(cf, &key);
            }
        }

        // Phase 9: cf_mempool (remove processed transactions)
        for operation in &self.operations {
            if let BatchOperation::RemoveFromMempool { priority, fee_rate, txid } = operation {
                let key = self.create_mempool_key(*priority, *fee_rate, txid);
                let cf = self.db.cf_handle(cf_names::MEMPOOL)?;
                batch.delete_cf(cf, &key);
            }
        }

        // Phase 10: cf_block_index (record operations)
        for operation in &self.operations {
            if let BatchOperation::RecordBlockOperation { 
                block_number, tx_index, operation_id, operation_type, utxo_id, prev_state_hash 
            } = operation {
                let key = self.create_block_index_key(*block_number, *tx_index, operation_id);
                let value = self.create_block_index_value(*operation_type, utxo_id, prev_state_hash);
                let cf = self.db.cf_handle(cf_names::BLOCK_INDEX)?;
                batch.put_cf(cf, &key, &value);
            }
        }

        // Execute atomic write batch
        self.db.write_batch(batch)
            .context("Failed to execute atomic write batch")?;

        Ok(())
    }

    // Key creation methods
    fn create_spent_tracker_key(&self, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::SPENT_TRACKER);
        key.extend_from_slice(utxo_id);
        key
    }

    fn create_utxo_key(&self, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::UTXOS);
        key.extend_from_slice(utxo_id);
        key
    }

    fn create_smt_node_key(&self, node_hash: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::SMT_NODES);
        key.extend_from_slice(node_hash);
        key
    }

    fn create_smt_leaf_key(&self, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::SMT_LEAVES);
        key.extend_from_slice(utxo_id);
        key
    }

    fn create_asset_balance_key(&self, owner_commitment: &[u8; 32], asset_id: &[u8; 20]) -> Vec<u8> {
        let mut key = Vec::with_capacity(53);
        key.push(cf_prefixes::ASSET_BALANCES);
        key.extend_from_slice(owner_commitment);
        key.extend_from_slice(asset_id);
        key
    }

    fn create_owner_index_key(&self, owner_commitment: &[u8; 32], created_block: u64, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(73);
        key.push(cf_prefixes::OWNER_INDEX);
        key.extend_from_slice(owner_commitment);
        key.extend_from_slice(&created_block.to_be_bytes());
        key.extend_from_slice(utxo_id);
        key
    }

    fn create_root_history_key(&self, root_version: u64) -> Vec<u8> {
        let mut key = Vec::with_capacity(9);
        key.push(cf_prefixes::ROOT_HISTORY);
        key.extend_from_slice(&root_version.to_be_bytes());
        key
    }

    fn create_input_lock_key(&self, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::INPUT_LOCKS);
        key.extend_from_slice(utxo_id);
        key
    }

    fn create_mempool_key(&self, priority: u8, fee_rate: u64, txid: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(42);
        key.push(cf_prefixes::MEMPOOL);
        key.push(priority);
        key.extend_from_slice(&fee_rate.to_be_bytes());
        key.extend_from_slice(txid);
        key
    }

    fn create_block_index_key(&self, block_number: u64, tx_index: u32, operation_id: &[u8; 16]) -> Vec<u8> {
        let mut key = Vec::with_capacity(29);
        key.push(cf_prefixes::BLOCK_INDEX);
        key.extend_from_slice(&block_number.to_be_bytes());
        key.extend_from_slice(&tx_index.to_be_bytes());
        key.extend_from_slice(operation_id);
        key
    }

    // Value creation methods
    fn create_spent_tracker_value(&self, spent_txid: [u8; 32], spent_block: u64, spent_timestamp: u64) -> Vec<u8> {
        let mut value = Vec::with_capacity(48);
        value.extend_from_slice(&spent_txid);
        value.extend_from_slice(&spent_block.to_be_bytes());
        value.extend_from_slice(&spent_timestamp.to_be_bytes());
        value
    }

    fn create_smt_node_value(&self, left_hash: [u8; 32], right_hash: [u8; 32], height: u8, ref_count: u32) -> Vec<u8> {
        let mut value = Vec::with_capacity(69);
        value.extend_from_slice(&left_hash);
        value.extend_from_slice(&right_hash);
        value.push(height);
        value.extend_from_slice(&ref_count.to_be_bytes());
        value
    }

    fn create_smt_leaf_value(&self, leaf_hash: [u8; 32], tree_position: u64) -> Vec<u8> {
        let mut value = Vec::with_capacity(40);
        value.extend_from_slice(&leaf_hash);
        value.extend_from_slice(&tree_position.to_be_bytes());
        value
    }

    fn create_asset_balance_value(&self, total_amount: u128, utxo_count: u32, last_updated_block: u64) -> Vec<u8> {
        let mut value = Vec::with_capacity(28);
        value.extend_from_slice(&total_amount.to_be_bytes());
        value.extend_from_slice(&utxo_count.to_be_bytes());
        value.extend_from_slice(&last_updated_block.to_be_bytes());
        value
    }

    fn create_owner_index_value(&self, amount: u128, asset_id: &[u8; 20], flags: u8) -> Vec<u8> {
        let mut value = Vec::with_capacity(37);
        value.extend_from_slice(&amount.to_be_bytes());
        value.extend_from_slice(asset_id);
        value.push(flags);
        value
    }

    fn create_root_history_value(&self, root_hash: [u8; 32], batch_id: u64, timestamp: u64, tx_count: u32, operator_signature: &[u8]) -> Vec<u8> {
        let mut value = Vec::with_capacity(54 + operator_signature.len());
        value.extend_from_slice(&root_hash);
        value.extend_from_slice(&batch_id.to_be_bytes());
        value.extend_from_slice(&timestamp.to_be_bytes());
        value.extend_from_slice(&tx_count.to_be_bytes());
        value.extend_from_slice(&(operator_signature.len() as u16).to_be_bytes());
        value.extend_from_slice(operator_signature);
        value
    }

    fn create_block_index_value(&self, operation_type: u8, utxo_id: &[u8; 32], prev_state_hash: &[u8; 32]) -> Vec<u8> {
        let mut value = Vec::with_capacity(65);
        value.push(operation_type);
        value.extend_from_slice(utxo_id);
        value.extend_from_slice(prev_state_hash);
        value
    }

    // Value parsing methods
    fn parse_smt_node_ref_count(&self, value: &[u8]) -> Result<u32> {
        if value.len() < 69 {
            return Err(anyhow!("SMT node value too short"));
        }
        let ref_count_bytes: [u8; 4] = value[65..69].try_into()
            .map_err(|_| anyhow!("Invalid ref count bytes"))?;
        Ok(u32::from_be_bytes(ref_count_bytes))
    }

    fn parse_asset_balance_value(&self, value: &[u8]) -> Result<(u128, u32, u64)> {
        if value.len() < 28 {
            return Err(anyhow!("Asset balance value too short"));
        }
        
        let amount_bytes: [u8; 16] = value[0..16].try_into()
            .map_err(|_| anyhow!("Invalid amount bytes"))?;
        let count_bytes: [u8; 4] = value[16..20].try_into()
            .map_err(|_| anyhow!("Invalid count bytes"))?;
        let block_bytes: [u8; 8] = value[20..28].try_into()
            .map_err(|_| anyhow!("Invalid block bytes"))?;
        
        Ok((
            u128::from_be_bytes(amount_bytes),
            u32::from_be_bytes(count_bytes),
            u64::from_be_bytes(block_bytes),
        ))
    }
}

/// Errors that can occur during batch writing
#[derive(Debug, thiserror::Error)]
pub enum WriteBatchError {
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Invalid operation order")]
    InvalidOrder,
    
    #[error("Reference count underflow for node {0:?}")]
    RefCountUnderflow([u8; 32]),
    
    #[error("Missing required operation: {0}")]
    MissingOperation(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::database::schema::DBConfig;

    #[test]
    fn test_batch_writer_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db").to_string_lossy().to_string();
        
        let config = DBConfig {
            db_path,
            ..Default::default()
        };
        
        let db_manager = DatabaseManager::open(config).unwrap();
        let batch_writer = AtomicBatchWriter::new(db_manager);
        
        assert_eq!(batch_writer.operations.len(), 0);
    }

    #[test]
    fn test_key_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db").to_string_lossy().to_string();
        
        let config = DBConfig {
            db_path,
            ..Default::default()
        };
        
        let db_manager = DatabaseManager::open(config).unwrap();
        let batch_writer = AtomicBatchWriter::new(db_manager);
        
        let utxo_id = [1u8; 32];
        let key = batch_writer.create_utxo_key(&utxo_id);
        
        assert_eq!(key.len(), 33);
        assert_eq!(key[0], cf_prefixes::UTXOS);
        assert_eq!(&key[1..], &utxo_id[..]);
    }
}