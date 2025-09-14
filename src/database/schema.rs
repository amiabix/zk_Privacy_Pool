//! Database Schema Implementation
//! 
//! RocksDB column families with grade configuration
//! following the canonical specification exactly.

use rocksdb::{DB, ColumnFamilyDescriptor, Options, WriteBatch, ReadOptions, WriteOptions, Cache, BlockBasedOptions, DBCompactionStyle};
use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::{Result, anyhow, Context};
use crate::canonical_spec::cf_prefixes;

/// Column family names matching the specification
pub mod cf_names {
    pub const UTXOS: &str = "cf_utxos";
    pub const SMT_LEAVES: &str = "cf_smt_leaves";
    pub const SMT_NODES: &str = "cf_smt_nodes";
    pub const OWNER_INDEX: &str = "cf_owner_index";
    pub const ASSET_BALANCES: &str = "cf_asset_balances";
    pub const SPENT_TRACKER: &str = "cf_spent_tracker";
    pub const INPUT_LOCKS: &str = "cf_input_locks";
    pub const MEMPOOL: &str = "cf_mempool";
    pub const ROOT_HISTORY: &str = "cf_root_history";
    pub const BLOCK_INDEX: &str = "cf_block_index";
    pub const TREE_METADATA: &str = "cf_tree_metadata";
}

/// Database configuration for deployment
#[derive(Debug, Clone)]
pub struct DBConfig {
    /// Database path
    pub db_path: String,
    
    /// Write buffer size per column family (default: 256MB)
    pub write_buffer_size: usize,
    
    /// Total write buffer limit across all CFs (default: 2GB)
    pub total_write_buffer_size: u64,
    
    /// Block cache size (default: 8GB)
    pub block_cache_size: usize,
    
    /// Maximum open files (default: 10000)
    pub max_open_files: i32,
    
    /// Enable bloom filters for faster lookups
    pub enable_bloom_filters: bool,
    
    /// Compression algorithm (LZ4, Snappy, ZSTD)
    pub compression_type: rocksdb::DBCompressionType,
    
    /// Background thread count for compaction
    pub max_background_jobs: i32,
    
    /// WAL size limit (default: 1GB)
    pub wal_size_limit: u64,
}

impl Default for DBConfig {
    fn default() -> Self {
        Self {
            db_path: "./privacy_pool_db".to_string(),
            write_buffer_size: 256 * 1024 * 1024, // 256MB
            total_write_buffer_size: 2 * 1024 * 1024 * 1024, // 2GB
            block_cache_size: 8 * 1024 * 1024 * 1024, // 8GB
            max_open_files: 10000,
            enable_bloom_filters: true,
            compression_type: rocksdb::DBCompressionType::Lz4,
            max_background_jobs: 16,
            wal_size_limit: 1024 * 1024 * 1024, // 1GB
        }
    }
}

/// Column family configuration with specific optimizations
#[derive(Debug, Clone)]
pub struct CFConfig {
    pub name: String,
    pub write_buffer_size: usize,
    pub enable_bloom_filter: bool,
    pub compaction_style: DBCompactionStyle,
    pub target_file_size_base: u64,
    pub compression_type: rocksdb::DBCompressionType,
    pub optimize_for_point_lookup: bool,
}

impl CFConfig {
    /// Configuration for cf_utxos (primary UTXO storage)
    pub fn utxos() -> Self {
        Self {
            name: cf_names::UTXOS.to_string(),
            write_buffer_size: 256 * 1024 * 1024, // 256MB
            enable_bloom_filter: true, // Fast existence checks
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 256 * 1024 * 1024, // 256MB
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: true,
        }
    }

    /// Configuration for cf_smt_leaves (tree leaf mapping)
    pub fn smt_leaves() -> Self {
        Self {
            name: cf_names::SMT_LEAVES.to_string(),
            write_buffer_size: 128 * 1024 * 1024, // 128MB
            enable_bloom_filter: true,
            compaction_style: DBCompactionStyle::Level, // Frequent updates
            target_file_size_base: 128 * 1024 * 1024,
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: true,
        }
    }

    /// Configuration for cf_smt_nodes (internal tree nodes with GC)
    pub fn smt_nodes() -> Self {
        Self {
            name: cf_names::SMT_NODES.to_string(),
            write_buffer_size: 512 * 1024 * 1024, // 512MB - lots of nodes
            enable_bloom_filter: true,
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 512 * 1024 * 1024,
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: true,
        }
    }

    /// Configuration for cf_owner_index (chronologically ordered queries)
    pub fn owner_index() -> Self {
        Self {
            name: cf_names::OWNER_INDEX.to_string(),
            write_buffer_size: 128 * 1024 * 1024,
            enable_bloom_filter: false, // Range scans, not point lookups
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 128 * 1024 * 1024,
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: false,
        }
    }

    /// Configuration for cf_asset_balances (fast aggregated queries)
    pub fn asset_balances() -> Self {
        Self {
            name: cf_names::ASSET_BALANCES.to_string(),
            write_buffer_size: 64 * 1024 * 1024,
            enable_bloom_filter: true, // O(1) balance lookups
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 64 * 1024 * 1024,
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: true,
        }
    }

    /// Configuration for cf_spent_tracker (audit trail)
    pub fn spent_tracker() -> Self {
        Self {
            name: cf_names::SPENT_TRACKER.to_string(),
            write_buffer_size: 128 * 1024 * 1024,
            enable_bloom_filter: true,
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 256 * 1024 * 1024, // Larger files for archival
            compression_type: rocksdb::DBCompressionType::Zstd, // Better compression
            optimize_for_point_lookup: true,
        }
    }

    /// Configuration for cf_input_locks (concurrency control)
    pub fn input_locks() -> Self {
        Self {
            name: cf_names::INPUT_LOCKS.to_string(),
            write_buffer_size: 32 * 1024 * 1024,
            enable_bloom_filter: true,
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 32 * 1024 * 1024,
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: true,
        }
    }

    /// Configuration for cf_mempool (priority-ordered transactions)
    pub fn mempool() -> Self {
        Self {
            name: cf_names::MEMPOOL.to_string(),
            write_buffer_size: 64 * 1024 * 1024,
            enable_bloom_filter: false, // Range scans by priority
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 64 * 1024 * 1024,
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: false,
        }
    }

    /// Configuration for cf_root_history (immutable audit trail)
    pub fn root_history() -> Self {
        Self {
            name: cf_names::ROOT_HISTORY.to_string(),
            write_buffer_size: 64 * 1024 * 1024,
            enable_bloom_filter: false, // Sequential access
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 512 * 1024 * 1024, // Large files
            compression_type: rocksdb::DBCompressionType::Zstd, // High compression
            optimize_for_point_lookup: false,
        }
    }

    /// Configuration for cf_block_index (reorganization handling)
    pub fn block_index() -> Self {
        Self {
            name: cf_names::BLOCK_INDEX.to_string(),
            write_buffer_size: 64 * 1024 * 1024,
            enable_bloom_filter: false, // Range queries by block
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 128 * 1024 * 1024,
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: false,
        }
    }

    /// Configuration for cf_tree_metadata (system state)
    pub fn tree_metadata() -> Self {
        Self {
            name: cf_names::TREE_METADATA.to_string(),
            write_buffer_size: 16 * 1024 * 1024,
            enable_bloom_filter: true,
            compaction_style: DBCompactionStyle::Level,
            target_file_size_base: 16 * 1024 * 1024,
            compression_type: rocksdb::DBCompressionType::Lz4,
            optimize_for_point_lookup: true,
        }
    }

    /// Create RocksDB Options from configuration
    pub fn to_options(&self, shared_cache: &Cache) -> Options {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        opts.set_write_buffer_size(self.write_buffer_size);
        opts.set_compression_type(self.compression_type);
        opts.set_compaction_style(self.compaction_style);
        opts.set_target_file_size_base(self.target_file_size_base);

        // Block-based table options
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_cache(shared_cache);
        
        if self.enable_bloom_filter {
            block_opts.set_bloom_filter(10.0, false); // 10 bits per key
        }
        
        if self.optimize_for_point_lookup {
            block_opts.set_cache_index_and_filter_blocks(true);
            block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        }

        opts.set_block_based_table_factory(&block_opts);
        opts
    }
}

/// Production-grade database manager
#[derive(Clone)]
pub struct DatabaseManager {
    db: Arc<DB>,
    config: DBConfig,
    column_families: HashMap<String, Arc<rocksdb::ColumnFamily>>,
    block_cache: Cache,
}

impl DatabaseManager {
    /// Open database with all column families
    pub fn open(config: DBConfig) -> Result<Self> {
        let db_path = Path::new(&config.db_path);
        
        // Create shared block cache
        let block_cache = Cache::new_lru_cache(config.block_cache_size)?;

        // Define all column families with their specific configurations
        let cf_configs = vec![
            CFConfig::utxos(),
            CFConfig::smt_leaves(),
            CFConfig::smt_nodes(),
            CFConfig::owner_index(),
            CFConfig::asset_balances(),
            CFConfig::spent_tracker(),
            CFConfig::input_locks(),
            CFConfig::mempool(),
            CFConfig::root_history(),
            CFConfig::block_index(),
            CFConfig::tree_metadata(),
        ];

        // Create column family descriptors
        let cf_descriptors: Vec<ColumnFamilyDescriptor> = cf_configs
            .iter()
            .map(|cf_config| {
                let opts = cf_config.to_options(&block_cache);
                ColumnFamilyDescriptor::new(&cf_config.name, opts)
            })
            .collect();

        // Database-level options
        let mut db_opts = Options::default();
        db_opts.create_if_missing(true);
        db_opts.create_missing_column_families(true);
        db_opts.set_max_open_files(config.max_open_files);
        db_opts.set_db_write_buffer_size(config.total_write_buffer_size);
        db_opts.set_max_background_jobs(config.max_background_jobs);
        db_opts.set_wal_size_limit_mb(config.wal_size_limit / 1024 / 1024);
        
        // Enable statistics for monitoring
        db_opts.enable_statistics();
        
        // Open database
        let db = DB::open_cf_descriptors(&db_opts, db_path, cf_descriptors)
            .with_context(|| format!("Failed to open database at {}", config.db_path))?;

        // Cache column family handles
        let mut column_families = HashMap::new();
        for cf_config in &cf_configs {
            let cf_handle = db.cf_handle(&cf_config.name)
                .ok_or_else(|| anyhow!("Column family {} not found", cf_config.name))?;
            column_families.insert(cf_config.name.clone(), Arc::new(cf_handle.clone()));
        }

        Ok(Self {
            db: Arc::new(db),
            config,
            column_families,
            block_cache,
        })
    }

    /// Get column family handle
    pub fn cf_handle(&self, name: &str) -> Result<&rocksdb::ColumnFamily> {
        self.column_families
            .get(name)
            .map(|cf| cf.as_ref().as_ref())
            .ok_or_else(|| anyhow!("Column family '{}' not found", name))
    }

    /// Get database reference
    pub fn db(&self) -> &DB {
        &self.db
    }

    /// Get database configuration
    pub fn config(&self) -> &DBConfig {
        &self.config
    }

    /// Get database statistics
    pub fn get_statistics(&self) -> Result<String> {
        self.db.property_value("rocksdb.stats")
            .ok_or_else(|| anyhow!("Statistics not enabled"))
    }

    /// Perform manual compaction for a column family
    pub fn compact_cf(&self, cf_name: &str) -> Result<()> {
        let cf = self.cf_handle(cf_name)?;
        self.db.compact_range_cf(cf, None::<&[u8]>, None::<&[u8]>);
        Ok(())
    }

    /// Get approximate sizes for column families
    pub fn get_cf_sizes(&self) -> Result<HashMap<String, u64>> {
        let mut sizes = HashMap::new();
        
        for (cf_name, cf_handle) in &self.column_families {
            let size_str = self.db.property_value_cf(cf_handle, "rocksdb.estimate-live-data-size")
                .unwrap_or_default();
            let size_bytes = size_str.parse::<u64>().unwrap_or(0);
            sizes.insert(cf_name.clone(), size_bytes);
        }
        
        Ok(sizes)
    }

    /// Create atomic write batch
    pub fn create_write_batch(&self) -> WriteBatch {
        WriteBatch::default()
    }

    /// Execute atomic write batch
    pub fn write_batch(&self, batch: WriteBatch) -> Result<()> {
        let mut write_opts = WriteOptions::default();
        write_opts.set_sync(true); // Ensure durability
        
        self.db.write_opt(batch, &write_opts)
            .context("Failed to execute write batch")
    }

    /// Get value from column family
    pub fn get_cf(&self, cf_name: &str, key: &[u8]) -> Result<Option<Vec<u8>>> {
        let cf = self.cf_handle(cf_name)?;
        let read_opts = ReadOptions::default();
        
        self.db.get_cf_opt(cf, key, &read_opts)
            .with_context(|| format!("Failed to get key from {}", cf_name))
    }

    /// Put value to column family
    pub fn put_cf(&self, cf_name: &str, key: &[u8], value: &[u8]) -> Result<()> {
        let cf = self.cf_handle(cf_name)?;
        let write_opts = WriteOptions::default();
        
        self.db.put_cf_opt(cf, key, value, &write_opts)
            .with_context(|| format!("Failed to put key to {}", cf_name))
    }

    /// Delete key from column family
    pub fn delete_cf(&self, cf_name: &str, key: &[u8]) -> Result<()> {
        let cf = self.cf_handle(cf_name)?;
        let write_opts = WriteOptions::default();
        
        self.db.delete_cf_opt(cf, key, &write_opts)
            .with_context(|| format!("Failed to delete key from {}", cf_name))
    }

    /// Create iterator for column family
    pub fn iterator_cf(&self, cf_name: &str) -> Result<rocksdb::DBIteratorWithThreadMode<rocksdb::DB>> {
        let cf = self.cf_handle(cf_name)?;
        let read_opts = ReadOptions::default();
        
        Ok(self.db.iterator_cf_opt(cf, read_opts, rocksdb::IteratorMode::Start))
    }

    /// Create prefix iterator for column family
    pub fn prefix_iterator_cf(&self, cf_name: &str, prefix: &[u8]) -> Result<rocksdb::DBIteratorWithThreadMode<rocksdb::DB>> {
        let cf = self.cf_handle(cf_name)?;
        let mut read_opts = ReadOptions::default();
        read_opts.set_prefix_same_as_start(true);
        
        Ok(self.db.iterator_cf_opt(cf, read_opts, rocksdb::IteratorMode::From(prefix, rocksdb::Direction::Forward)))
    }

    /// Shutdown database gracefully
    pub fn shutdown(&self) -> Result<()> {
        // RocksDB handles shutdown automatically when DB is dropped
        // This is here for explicit shutdown if needed
        Ok(())
    }
}

/// Database utility functions
pub mod utils {
    use super::*;

    /// Create database key with prefix
    pub fn create_key_with_prefix(prefix: u8, key_parts: &[&[u8]]) -> Vec<u8> {
        let total_len = 1 + key_parts.iter().map(|part| part.len()).sum::<usize>();
        let mut key = Vec::with_capacity(total_len);
        key.push(prefix);
        for part in key_parts {
            key.extend_from_slice(part);
        }
        key
    }

    /// Parse key with expected prefix
    pub fn parse_key_with_prefix(key: &[u8], expected_prefix: u8) -> Result<&[u8]> {
        if key.is_empty() {
            return Err(anyhow!("Key is empty"));
        }
        if key[0] != expected_prefix {
            return Err(anyhow!("Key prefix mismatch: expected {}, got {}", expected_prefix, key[0]));
        }
        Ok(&key[1..])
    }

    /// Create UTXO database key
    pub fn utxo_key(utxo_id: &[u8; 32]) -> Vec<u8> {
        create_key_with_prefix(cf_prefixes::UTXOS, &[utxo_id])
    }

    /// Create owner index key
    pub fn owner_index_key(owner_commitment: &[u8; 32], created_block: u64, utxo_id: &[u8; 32]) -> Vec<u8> {
        create_key_with_prefix(
            cf_prefixes::OWNER_INDEX,
            &[owner_commitment, &created_block.to_be_bytes(), utxo_id]
        )
    }

    /// Create asset balance key
    pub fn asset_balance_key(owner_commitment: &[u8; 32], asset_id: &[u8; 20]) -> Vec<u8> {
        create_key_with_prefix(
            cf_prefixes::ASSET_BALANCES,
            &[owner_commitment, asset_id]
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db").to_string_lossy().to_string();
        
        let config = DBConfig {
            db_path,
            ..Default::default()
        };

        let db_manager = DatabaseManager::open(config).unwrap();
        
        // Verify all column families exist
        assert!(db_manager.cf_handle(cf_names::UTXOS).is_ok());
        assert!(db_manager.cf_handle(cf_names::SMT_LEAVES).is_ok());
        assert!(db_manager.cf_handle(cf_names::SMT_NODES).is_ok());
        assert!(db_manager.cf_handle(cf_names::OWNER_INDEX).is_ok());
        assert!(db_manager.cf_handle(cf_names::ASSET_BALANCES).is_ok());
        assert!(db_manager.cf_handle(cf_names::SPENT_TRACKER).is_ok());
        assert!(db_manager.cf_handle(cf_names::INPUT_LOCKS).is_ok());
        assert!(db_manager.cf_handle(cf_names::MEMPOOL).is_ok());
        assert!(db_manager.cf_handle(cf_names::ROOT_HISTORY).is_ok());
        assert!(db_manager.cf_handle(cf_names::BLOCK_INDEX).is_ok());
        assert!(db_manager.cf_handle(cf_names::TREE_METADATA).is_ok());
    }

    #[test]
    fn test_basic_operations() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_db").to_string_lossy().to_string();
        
        let config = DBConfig {
            db_path,
            ..Default::default()
        };

        let db_manager = DatabaseManager::open(config).unwrap();
        
        let key = b"test_key";
        let value = b"test_value";
        
        // Test put and get
        db_manager.put_cf(cf_names::UTXOS, key, value).unwrap();
        let retrieved = db_manager.get_cf(cf_names::UTXOS, key).unwrap();
        assert_eq!(retrieved.as_ref(), Some(value.as_ref()));
        
        // Test delete
        db_manager.delete_cf(cf_names::UTXOS, key).unwrap();
        let retrieved = db_manager.get_cf(cf_names::UTXOS, key).unwrap();
        assert_eq!(retrieved, None);
    }

    #[test]
    fn test_key_utils() {
        let prefix = cf_prefixes::UTXOS;
        let utxo_id = [1u8; 32];
        
        let key = utils::utxo_key(&utxo_id);
        assert_eq!(key.len(), 33);
        assert_eq!(key[0], prefix);
        assert_eq!(&key[1..], &utxo_id[..]);
        
        let parsed = utils::parse_key_with_prefix(&key, prefix).unwrap();
        assert_eq!(parsed, &utxo_id[..]);
    }
}