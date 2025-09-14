//! Query Engine Implementation
//! 
//! High-performance query interface for the privacy pool database
//! with optimized patterns for wallet queries and proof generation.

use anyhow::{Result, anyhow};
use crate::database::schema::{DatabaseManager, cf_names};
use crate::canonical_spec::cf_prefixes;
use crate::utxo::CanonicalUTXO;

/// Query result types
#[derive(Debug, Clone)]
pub enum QueryResult {
    /// Single UTXO result
    UTXO(CanonicalUTXO),
    
    /// List of UTXOs for wallet queries
    UTXOList(Vec<CanonicalUTXO>),
    
    /// Balance information
    Balance {
        total_amount: u128,
        utxo_count: u32,
        last_updated_block: u64,
    },
    
    /// Raw bytes for custom queries
    Bytes(Vec<u8>),
    
    /// Not found
    NotFound,
}

/// Query errors
#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Database error: {0}")]
    Database(#[from] anyhow::Error),
    
    #[error("UTXO not found: {0:?}")]
    UTXONotFound([u8; 32]),
    
    #[error("Invalid query parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// High-performance query engine
pub struct QueryEngine {
    db: DatabaseManager,
}

impl QueryEngine {
    /// Create new query engine
    pub fn new(db: DatabaseManager) -> Self {
        Self { db }
    }

    /// Get UTXO by ID
    pub fn get_utxo(&self, utxo_id: &[u8; 32]) -> Result<QueryResult, QueryError> {
        let key = self.create_utxo_key(utxo_id);
        
        match self.db.get_cf(cf_names::UTXOS, &key)? {
            Some(data) => {
                let utxo = CanonicalUTXO::deserialize(&data)
                    .map_err(|e| QueryError::Serialization(e.to_string()))?;
                Ok(QueryResult::UTXO(utxo))
            },
            None => Ok(QueryResult::NotFound),
        }
    }

    /// Get UTXOs for owner with pagination
    pub fn get_owner_utxos(
        &self,
        owner_commitment: &[u8; 32],
        limit: usize,
        after_block: Option<u64>,
        asset_id: Option<&[u8; 20]>,
    ) -> Result<QueryResult, QueryError> {
        let mut utxos = Vec::new();
        let prefix = self.create_owner_index_prefix(owner_commitment);
        
        let mut iter = self.db.prefix_iterator_cf(cf_names::OWNER_INDEX, &prefix)?;
        
        // Skip to after_block if specified
        if let Some(block) = after_block {
            let start_key = self.create_owner_index_start_key(owner_commitment, block);
            iter = self.db.prefix_iterator_cf(cf_names::OWNER_INDEX, &start_key)?;
        }
        
        let mut count = 0;
        for item in iter {
            if count >= limit {
                break;
            }
            
            let (key, value) = item.map_err(|e| QueryError::Database(e.into()))?;
            
            // Parse owner index entry
            let utxo_id = self.parse_owner_index_utxo_id(&key)?;
            let (_amount, entry_asset_id, _flags) = self.parse_owner_index_value(&value)?;
            
            // Filter by asset_id if specified
            if let Some(filter_asset_id) = asset_id {
                if &entry_asset_id != filter_asset_id {
                    continue;
                }
            }
            
            // Get full UTXO data
            if let QueryResult::UTXO(utxo) = self.get_utxo(&utxo_id)? {
                utxos.push(utxo);
                count += 1;
            }
        }
        
        Ok(QueryResult::UTXOList(utxos))
    }

    /// Get aggregated balance for owner and asset
    pub fn get_balance(
        &self,
        owner_commitment: &[u8; 32],
        asset_id: &[u8; 20],
    ) -> Result<QueryResult, QueryError> {
        let key = self.create_asset_balance_key(owner_commitment, asset_id);
        
        match self.db.get_cf(cf_names::ASSET_BALANCES, &key)? {
            Some(data) => {
                let (total_amount, utxo_count, last_updated_block) = self.parse_asset_balance_value(&data)?;
                Ok(QueryResult::Balance {
                    total_amount,
                    utxo_count,
                    last_updated_block,
                })
            },
            None => Ok(QueryResult::Balance {
                total_amount: 0,
                utxo_count: 0,
                last_updated_block: 0,
            }),
        }
    }

    /// Check if UTXO is spent
    pub fn is_utxo_spent(&self, utxo_id: &[u8; 32]) -> Result<bool, QueryError> {
        let key = self.create_spent_tracker_key(utxo_id);
        let exists = self.db.get_cf(cf_names::SPENT_TRACKER, &key)?.is_some();
        Ok(exists)
    }

    /// Get SMT leaf data for UTXO
    pub fn get_smt_leaf(&self, utxo_id: &[u8; 32]) -> Result<Option<([u8; 32], u64)>, QueryError> {
        let key = self.create_smt_leaf_key(utxo_id);
        
        match self.db.get_cf(cf_names::SMT_LEAVES, &key)? {
            Some(data) => {
                let (leaf_hash, tree_position) = self.parse_smt_leaf_value(&data)?;
                Ok(Some((leaf_hash, tree_position)))
            },
            None => Ok(None),
        }
    }

    /// Get SMT node data
    pub fn get_smt_node(&self, node_hash: &[u8; 32]) -> Result<Option<([u8; 32], [u8; 32], u8, u32)>, QueryError> {
        let key = self.create_smt_node_key(node_hash);
        
        match self.db.get_cf(cf_names::SMT_NODES, &key)? {
            Some(data) => {
                let (left_hash, right_hash, height, ref_count) = self.parse_smt_node_value(&data)?;
                Ok(Some((left_hash, right_hash, height, ref_count)))
            },
            None => Ok(None),
        }
    }

    // Key creation helpers
    fn create_utxo_key(&self, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::UTXOS);
        key.extend_from_slice(utxo_id);
        key
    }

    fn create_owner_index_prefix(&self, owner_commitment: &[u8; 32]) -> Vec<u8> {
        let mut prefix = Vec::with_capacity(33);
        prefix.push(cf_prefixes::OWNER_INDEX);
        prefix.extend_from_slice(owner_commitment);
        prefix
    }

    fn create_owner_index_start_key(&self, owner_commitment: &[u8; 32], start_block: u64) -> Vec<u8> {
        let mut key = Vec::with_capacity(41);
        key.push(cf_prefixes::OWNER_INDEX);
        key.extend_from_slice(owner_commitment);
        key.extend_from_slice(&start_block.to_be_bytes());
        key
    }

    fn create_asset_balance_key(&self, owner_commitment: &[u8; 32], asset_id: &[u8; 20]) -> Vec<u8> {
        let mut key = Vec::with_capacity(53);
        key.push(cf_prefixes::ASSET_BALANCES);
        key.extend_from_slice(owner_commitment);
        key.extend_from_slice(asset_id);
        key
    }

    fn create_spent_tracker_key(&self, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::SPENT_TRACKER);
        key.extend_from_slice(utxo_id);
        key
    }

    fn create_smt_leaf_key(&self, utxo_id: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::SMT_LEAVES);
        key.extend_from_slice(utxo_id);
        key
    }

    fn create_smt_node_key(&self, node_hash: &[u8; 32]) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::SMT_NODES);
        key.extend_from_slice(node_hash);
        key
    }

    // Value parsing helpers
    fn parse_owner_index_utxo_id(&self, key: &[u8]) -> Result<[u8; 32], QueryError> {
        if key.len() < 73 {
            return Err(QueryError::InvalidParameters("Owner index key too short".to_string()));
        }
        
        let utxo_id_bytes: [u8; 32] = key[41..73].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid UTXO ID in owner index key".to_string()))?;
        
        Ok(utxo_id_bytes)
    }

    fn parse_owner_index_value(&self, value: &[u8]) -> Result<(u128, [u8; 20], u8), QueryError> {
        if value.len() < 37 {
            return Err(QueryError::InvalidParameters("Owner index value too short".to_string()));
        }
        
        let amount_bytes: [u8; 16] = value[0..16].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid amount in owner index value".to_string()))?;
        let asset_id_bytes: [u8; 20] = value[16..36].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid asset ID in owner index value".to_string()))?;
        let flags = value[36];
        
        Ok((u128::from_be_bytes(amount_bytes), asset_id_bytes, flags))
    }

    fn parse_asset_balance_value(&self, value: &[u8]) -> Result<(u128, u32, u64), QueryError> {
        if value.len() < 28 {
            return Err(QueryError::InvalidParameters("Asset balance value too short".to_string()));
        }
        
        let amount_bytes: [u8; 16] = value[0..16].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid amount in asset balance value".to_string()))?;
        let count_bytes: [u8; 4] = value[16..20].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid count in asset balance value".to_string()))?;
        let block_bytes: [u8; 8] = value[20..28].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid block in asset balance value".to_string()))?;
        
        Ok((
            u128::from_be_bytes(amount_bytes),
            u32::from_be_bytes(count_bytes),
            u64::from_be_bytes(block_bytes),
        ))
    }

    fn parse_smt_leaf_value(&self, value: &[u8]) -> Result<([u8; 32], u64), QueryError> {
        if value.len() < 40 {
            return Err(QueryError::InvalidParameters("SMT leaf value too short".to_string()));
        }
        
        let leaf_hash_bytes: [u8; 32] = value[0..32].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid leaf hash in SMT leaf value".to_string()))?;
        let position_bytes: [u8; 8] = value[32..40].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid position in SMT leaf value".to_string()))?;
        
        Ok((leaf_hash_bytes, u64::from_be_bytes(position_bytes)))
    }

    fn parse_smt_node_value(&self, value: &[u8]) -> Result<([u8; 32], [u8; 32], u8, u32), QueryError> {
        if value.len() < 69 {
            return Err(QueryError::InvalidParameters("SMT node value too short".to_string()));
        }
        
        let left_hash_bytes: [u8; 32] = value[0..32].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid left hash in SMT node value".to_string()))?;
        let right_hash_bytes: [u8; 32] = value[32..64].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid right hash in SMT node value".to_string()))?;
        let height = value[64];
        let ref_count_bytes: [u8; 4] = value[65..69].try_into()
            .map_err(|_| QueryError::InvalidParameters("Invalid ref count in SMT node value".to_string()))?;
        
        Ok((left_hash_bytes, right_hash_bytes, height, u32::from_be_bytes(ref_count_bytes)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use crate::database::schema::DBConfig;

    #[test]
    fn test_query_engine_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db").to_string_lossy().to_string();
        
        let config = DBConfig {
            db_path,
            ..Default::default()
        };
        
        let db_manager = DatabaseManager::open(config).unwrap();
        let _query_engine = QueryEngine::new(db_manager);
    }
}