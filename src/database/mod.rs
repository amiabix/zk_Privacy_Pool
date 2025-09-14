//! Database Module
//! 
//! Production-grade RocksDB implementation with proper column families
//! and atomic write batch operations following the canonical specification.

pub mod schema;
pub mod batch_writer;
pub mod query_engine;
pub mod cache_manager;

// Re-export main types
pub use schema::{DatabaseManager, DBConfig};
pub use batch_writer::{AtomicBatchWriter, BatchOperation, WriteBatchError};
pub use query_engine::{QueryEngine, QueryResult, QueryError};
pub use cache_manager::{CacheManager, CacheConfig, CacheStats};