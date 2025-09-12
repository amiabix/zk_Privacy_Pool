//! UTXO Indexing System based on Zcash librustzcash patterns
//! Provides O(1) lookups instead of O(n) linear searches

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};

/// UTXO identifier combining transaction hash and output index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UTXOId {
    pub tx_hash: [u8; 32],
    pub output_index: u32,
}

impl UTXOId {
    pub fn new(tx_hash: [u8; 32], output_index: u32) -> Self {
        Self { tx_hash, output_index }
    }
}

/// UTXO record with proper indexing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedUTXO {
    pub id: UTXOId,
    pub account_id: u32,
    pub address: [u8; 32], // Commitment hash
    pub value: u64,
    pub height: u32,
    pub spent_in_tx: Option<[u8; 32]>,
    pub blinding_factor: [u8; 32],
}

/// UTXO Indexing System for O(1) lookups
#[derive(Debug, Clone)]
pub struct UTXOIndex {
    // Primary storage: UTXO ID -> UTXO record
    utxos: HashMap<UTXOId, IndexedUTXO>,
    
    // Account-based index: Account ID -> Set of UTXO IDs
    account_utxos: HashMap<u32, HashSet<UTXOId>>,
    
    // Address-based index: Address -> Set of UTXO IDs
    address_utxos: HashMap<[u8; 32], HashSet<UTXOId>>,
    
    // Value-based index: Value -> Set of UTXO IDs (for range queries)
    value_utxos: HashMap<u64, HashSet<UTXOId>>,
    
    // Height-based index: Height -> Set of UTXO IDs
    height_utxos: HashMap<u32, HashSet<UTXOId>>,
}

impl UTXOIndex {
    pub fn new() -> Self {
        Self {
            utxos: HashMap::new(),
            account_utxos: HashMap::new(),
            address_utxos: HashMap::new(),
            value_utxos: HashMap::new(),
            height_utxos: HashMap::new(),
        }
    }

    /// Add a UTXO to the index
    pub fn add_utxo(&mut self, utxo: IndexedUTXO) {
        let utxo_id = utxo.id;
        
        // Add to primary storage
        self.utxos.insert(utxo_id, utxo.clone());
        
        // Add to account index
        self.account_utxos
            .entry(utxo.account_id)
            .or_insert_with(HashSet::new)
            .insert(utxo_id);
        
        // Add to address index
        self.address_utxos
            .entry(utxo.address)
            .or_insert_with(HashSet::new)
            .insert(utxo_id);
        
        // Add to value index
        self.value_utxos
            .entry(utxo.value)
            .or_insert_with(HashSet::new)
            .insert(utxo_id);
        
        // Add to height index
        self.height_utxos
            .entry(utxo.height)
            .or_insert_with(HashSet::new)
            .insert(utxo_id);
    }

    /// Remove a UTXO from the index
    pub fn remove_utxo(&mut self, utxo_id: UTXOId) -> Option<IndexedUTXO> {
        if let Some(utxo) = self.utxos.remove(&utxo_id) {
            // Remove from all indexes
            self.account_utxos
                .get_mut(&utxo.account_id)
                .and_then(|set| { set.remove(&utxo_id); Some(()) });
            
            self.address_utxos
                .get_mut(&utxo.address)
                .and_then(|set| { set.remove(&utxo_id); Some(()) });
            
            self.value_utxos
                .get_mut(&utxo.value)
                .and_then(|set| { set.remove(&utxo_id); Some(()) });
            
            self.height_utxos
                .get_mut(&utxo.height)
                .and_then(|set| { set.remove(&utxo_id); Some(()) });
            
            Some(utxo)
        } else {
            None
        }
    }

    /// Get UTXO by ID (O(1))
    pub fn get_utxo(&self, utxo_id: &UTXOId) -> Option<&IndexedUTXO> {
        self.utxos.get(utxo_id)
    }

    /// Get all UTXOs for an account (O(1) + O(k) where k is number of UTXOs)
    pub fn get_account_utxos(&self, account_id: u32) -> Vec<&IndexedUTXO> {
        self.account_utxos
            .get(&account_id)
            .map(|utxo_ids| {
                utxo_ids.iter()
                    .filter_map(|id| self.utxos.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get all UTXOs for an address (O(1) + O(k))
    pub fn get_address_utxos(&self, address: &[u8; 32]) -> Vec<&IndexedUTXO> {
        self.address_utxos
            .get(address)
            .map(|utxo_ids| {
                utxo_ids.iter()
                    .filter_map(|id| self.utxos.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get UTXOs by value range (O(1) + O(k) for each value)
    pub fn get_utxos_by_value_range(&self, min_value: u64, max_value: u64) -> Vec<&IndexedUTXO> {
        let mut result = Vec::new();
        for value in min_value..=max_value {
            if let Some(utxo_ids) = self.value_utxos.get(&value) {
                for utxo_id in utxo_ids {
                    if let Some(utxo) = self.utxos.get(utxo_id) {
                        result.push(utxo);
                    }
                }
            }
        }
        result
    }

    /// Get UTXOs by height range (O(1) + O(k) for each height)
    pub fn get_utxos_by_height_range(&self, min_height: u32, max_height: u32) -> Vec<&IndexedUTXO> {
        let mut result = Vec::new();
        for height in min_height..=max_height {
            if let Some(utxo_ids) = self.height_utxos.get(&height) {
                for utxo_id in utxo_ids {
                    if let Some(utxo) = self.utxos.get(utxo_id) {
                        result.push(utxo);
                    }
                }
            }
        }
        result
    }

    /// Get unspent UTXOs for an account (O(1) + O(k))
    pub fn get_unspent_account_utxos(&self, account_id: u32) -> Vec<&IndexedUTXO> {
        self.get_account_utxos(account_id)
            .into_iter()
            .filter(|utxo| utxo.spent_in_tx.is_none())
            .collect()
    }

    /// Get total balance for an account (O(1) + O(k))
    pub fn get_account_balance(&self, account_id: u32) -> u64 {
        self.get_unspent_account_utxos(account_id)
            .iter()
            .map(|utxo| utxo.value)
            .sum()
    }

    /// Mark UTXO as spent
    pub fn mark_spent(&mut self, utxo_id: UTXOId, spending_tx: [u8; 32]) -> bool {
        if let Some(utxo) = self.utxos.get_mut(&utxo_id) {
            utxo.spent_in_tx = Some(spending_tx);
            true
        } else {
            false
        }
    }

    /// Get all UTXOs (for iteration)
    pub fn get_all_utxos(&self) -> Vec<&IndexedUTXO> {
        self.utxos.values().collect()
    }

    /// Get UTXO count
    pub fn count(&self) -> usize {
        self.utxos.len()
    }

    /// Check if UTXO exists
    pub fn contains(&self, utxo_id: &UTXOId) -> bool {
        self.utxos.contains_key(utxo_id)
    }

    /// Clear all UTXOs
    pub fn clear(&mut self) {
        self.utxos.clear();
        self.account_utxos.clear();
        self.address_utxos.clear();
        self.value_utxos.clear();
        self.height_utxos.clear();
    }
}

impl Default for UTXOIndex {
    fn default() -> Self {
        Self::new()
    }
}

/// UTXO Query Builder for complex queries
pub struct UTXOQueryBuilder<'a> {
    index: &'a UTXOIndex,
    account_filter: Option<u32>,
    address_filter: Option<[u8; 32]>,
    value_range: Option<(u64, u64)>,
    height_range: Option<(u32, u32)>,
    unspent_only: bool,
}

impl<'a> UTXOQueryBuilder<'a> {
    pub fn new(index: &'a UTXOIndex) -> Self {
        Self {
            index,
            account_filter: None,
            address_filter: None,
            value_range: None,
            height_range: None,
            unspent_only: false,
        }
    }

    pub fn account(mut self, account_id: u32) -> Self {
        self.account_filter = Some(account_id);
        self
    }

    pub fn address(mut self, address: [u8; 32]) -> Self {
        self.address_filter = Some(address);
        self
    }

    pub fn value_range(mut self, min: u64, max: u64) -> Self {
        self.value_range = Some((min, max));
        self
    }

    pub fn height_range(mut self, min: u32, max: u32) -> Self {
        self.height_range = Some((min, max));
        self
    }

    pub fn unspent_only(mut self) -> Self {
        self.unspent_only = true;
        self
    }

    pub fn execute(self) -> Vec<&'a IndexedUTXO> {
        let mut result = Vec::new();
        
        // Start with appropriate base set
        let base_utxos = if let Some(account_id) = self.account_filter {
            self.index.get_account_utxos(account_id)
        } else if let Some(address) = self.address_filter {
            self.index.get_address_utxos(&address)
        } else {
            self.index.get_all_utxos()
        };

        // Apply filters
        for utxo in base_utxos {
            // Check unspent filter
            if self.unspent_only && utxo.spent_in_tx.is_some() {
                continue;
            }

            // Check value range filter
            if let Some((min_val, max_val)) = self.value_range {
                if utxo.value < min_val || utxo.value > max_val {
                    continue;
                }
            }

            // Check height range filter
            if let Some((min_height, max_height)) = self.height_range {
                if utxo.height < min_height || utxo.height > max_height {
                    continue;
                }
            }

            result.push(utxo);
        }

        result
    }
}

impl<'a> UTXOIndex {
    pub fn query(&'a self) -> UTXOQueryBuilder<'a> {
        UTXOQueryBuilder::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_indexing() {
        let mut index = UTXOIndex::new();
        
        // Create test UTXOs
        let utxo1 = IndexedUTXO {
            id: UTXOId::new([1u8; 32], 0),
            account_id: 1,
            address: [0x01; 32],
            value: 100,
            height: 1000,
            spent_in_tx: None,
            blinding_factor: [0x11; 32],
        };
        
        let utxo2 = IndexedUTXO {
            id: UTXOId::new([2u8; 32], 0),
            account_id: 1,
            address: [0x02; 32],
            value: 200,
            height: 1001,
            spent_in_tx: None,
            blinding_factor: [0x22; 32],
        };
        
        let utxo3 = IndexedUTXO {
            id: UTXOId::new([3u8; 32], 0),
            account_id: 2,
            address: [0x01; 32],
            value: 150,
            height: 1002,
            spent_in_tx: None,
            blinding_factor: [0x33; 32],
        };

        // Add UTXOs
        index.add_utxo(utxo1);
        index.add_utxo(utxo2);
        index.add_utxo(utxo3);

        // Test account queries
        let account1_utxos = index.get_account_utxos(1);
        assert_eq!(account1_utxos.len(), 2);
        
        let account2_utxos = index.get_account_utxos(2);
        assert_eq!(account2_utxos.len(), 1);

        // Test address queries
        let address_utxos = index.get_address_utxos(&[0x01; 32]);
        assert_eq!(address_utxos.len(), 2);

        // Test value range queries
        let value_utxos = index.get_utxos_by_value_range(100, 150);
        assert_eq!(value_utxos.len(), 2);

        // Test height range queries
        let height_utxos = index.get_utxos_by_height_range(1000, 1001);
        assert_eq!(height_utxos.len(), 2);

        // Test unspent queries
        let unspent_utxos = index.get_unspent_account_utxos(1);
        assert_eq!(unspent_utxos.len(), 2);

        // Test balance calculation
        let balance = index.get_account_balance(1);
        assert_eq!(balance, 300);

        // Test spending
        index.mark_spent(UTXOId::new([1u8; 32], 0), [0xff; 32]);
        let unspent_after_spend = index.get_unspent_account_utxos(1);
        assert_eq!(unspent_after_spend.len(), 1);

        // Test complex query
        let complex_query = index.query()
            .account(1)
            .value_range(100, 200)
            .unspent_only()
            .execute();
        assert_eq!(complex_query.len(), 1);
    }
}
