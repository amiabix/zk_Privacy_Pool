//! Cache Manager Implementation
//! 
//! Multi-level caching system for optimal performance
//! following the canonical specification requirements.

use std::sync::Arc;
use parking_lot::RwLock;
use lru::LruCache;
use anyhow::Result;

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// L1 cache size for UTXO data (entries)
    pub utxo_cache_size: usize,
    
    /// L1 cache size for tree nodes (entries)
    pub node_cache_size: usize,
    
    /// L1 cache size for sibling paths (entries)
    pub sibling_cache_size: usize,
    
    /// Enable cache statistics collection
    pub enable_stats: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            utxo_cache_size: 10_000_000,  // 10M entries
            node_cache_size: 5_000_000,   // 5M entries
            sibling_cache_size: 1_000_000, // 1M entries
            enable_stats: true,
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    pub utxo_hits: u64,
    pub utxo_misses: u64,
    pub node_hits: u64,
    pub node_misses: u64,
    pub sibling_hits: u64,
    pub sibling_misses: u64,
}

impl CacheStats {
    /// Calculate hit rate for UTXO cache
    pub fn utxo_hit_rate(&self) -> f64 {
        if self.utxo_hits + self.utxo_misses == 0 {
            0.0
        } else {
            self.utxo_hits as f64 / (self.utxo_hits + self.utxo_misses) as f64
        }
    }

    /// Calculate hit rate for node cache
    pub fn node_hit_rate(&self) -> f64 {
        if self.node_hits + self.node_misses == 0 {
            0.0
        } else {
            self.node_hits as f64 / (self.node_hits + self.node_misses) as f64
        }
    }

    /// Calculate hit rate for sibling cache
    pub fn sibling_hit_rate(&self) -> f64 {
        if self.sibling_hits + self.sibling_misses == 0 {
            0.0
        } else {
            self.sibling_hits as f64 / (self.sibling_hits + self.sibling_misses) as f64
        }
    }
}

/// Multi-level cache manager
pub struct CacheManager {
    /// UTXO data cache: utxo_id -> serialized_utxo
    utxo_cache: Arc<RwLock<LruCache<[u8; 32], Vec<u8>>>>,
    
    /// Tree node cache: (index, level) -> node_hash
    node_cache: Arc<RwLock<LruCache<(u64, u8), [u8; 32]>>>,
    
    /// Sibling path cache: index -> sibling_path
    sibling_cache: Arc<RwLock<LruCache<u64, Vec<[u8; 32]>>>>,
    
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    
    /// Configuration
    config: CacheConfig,
}

impl CacheManager {
    /// Create new cache manager
    pub fn new(config: CacheConfig) -> Self {
        Self {
            utxo_cache: Arc::new(RwLock::new(
                LruCache::new(
                    std::num::NonZeroUsize::new(config.utxo_cache_size).unwrap()
                )
            )),
            node_cache: Arc::new(RwLock::new(
                LruCache::new(
                    std::num::NonZeroUsize::new(config.node_cache_size).unwrap()
                )
            )),
            sibling_cache: Arc::new(RwLock::new(
                LruCache::new(
                    std::num::NonZeroUsize::new(config.sibling_cache_size).unwrap()
                )
            )),
            stats: Arc::new(RwLock::new(CacheStats::default())),
            config,
        }
    }

    /// Get UTXO from cache
    pub fn get_utxo(&self, utxo_id: &[u8; 32]) -> Option<Vec<u8>> {
        let mut cache = self.utxo_cache.write();
        let result = cache.get(utxo_id).cloned();
        
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            if result.is_some() {
                stats.utxo_hits += 1;
            } else {
                stats.utxo_misses += 1;
            }
        }
        
        result
    }

    /// Put UTXO into cache
    pub fn put_utxo(&self, utxo_id: [u8; 32], data: Vec<u8>) {
        let mut cache = self.utxo_cache.write();
        cache.put(utxo_id, data);
    }

    /// Get tree node from cache
    pub fn get_node(&self, index: u64, level: u8) -> Option<[u8; 32]> {
        let mut cache = self.node_cache.write();
        let result = cache.get(&(index, level)).copied();
        
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            if result.is_some() {
                stats.node_hits += 1;
            } else {
                stats.node_misses += 1;
            }
        }
        
        result
    }

    /// Put tree node into cache
    pub fn put_node(&self, index: u64, level: u8, node_hash: [u8; 32]) {
        let mut cache = self.node_cache.write();
        cache.put((index, level), node_hash);
    }

    /// Get sibling path from cache
    pub fn get_sibling_path(&self, index: u64) -> Option<Vec<[u8; 32]>> {
        let mut cache = self.sibling_cache.write();
        let result = cache.get(&index).cloned();
        
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            if result.is_some() {
                stats.sibling_hits += 1;
            } else {
                stats.sibling_misses += 1;
            }
        }
        
        result
    }

    /// Put sibling path into cache
    pub fn put_sibling_path(&self, index: u64, siblings: Vec<[u8; 32]>) {
        let mut cache = self.sibling_cache.write();
        cache.put(index, siblings);
    }

    /// Warm caches with batch data
    pub fn warm_caches(&self, _utxos: &[(u64, u8)]) -> Result<()> {
        // TODO: Implement batch cache warming
        // This would batch read from database and populate caches
        Ok(())
    }

    /// Clear all caches
    pub fn clear_all(&self) {
        {
            let mut cache = self.utxo_cache.write();
            cache.clear();
        }
        {
            let mut cache = self.node_cache.write();
            cache.clear();
        }
        {
            let mut cache = self.sibling_cache.write();
            cache.clear();
        }
        
        if self.config.enable_stats {
            let mut stats = self.stats.write();
            *stats = CacheStats::default();
        }
    }

    /// Get cache statistics
    pub fn get_stats(&self) -> CacheStats {
        self.stats.read().clone()
    }

    /// Get cache sizes
    pub fn get_cache_sizes(&self) -> (usize, usize, usize) {
        let utxo_len = self.utxo_cache.read().len();
        let node_len = self.node_cache.read().len();
        let sibling_len = self.sibling_cache.read().len();
        
        (utxo_len, node_len, sibling_len)
    }

    /// Resize caches (useful for dynamic optimization)
    pub fn resize_caches(&self, utxo_size: Option<usize>, node_size: Option<usize>, sibling_size: Option<usize>) {
        if let Some(size) = utxo_size {
            let mut cache = self.utxo_cache.write();
            cache.resize(std::num::NonZeroUsize::new(size).unwrap());
        }
        
        if let Some(size) = node_size {
            let mut cache = self.node_cache.write();
            cache.resize(std::num::NonZeroUsize::new(size).unwrap());
        }
        
        if let Some(size) = sibling_size {
            let mut cache = self.sibling_cache.write();
            cache.resize(std::num::NonZeroUsize::new(size).unwrap());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_manager_creation() {
        let config = CacheConfig::default();
        let _cache_manager = CacheManager::new(config);
    }

    #[test]
    fn test_utxo_cache() {
        let config = CacheConfig {
            utxo_cache_size: 100,
            ..Default::default()
        };
        let cache_manager = CacheManager::new(config);
        
        let utxo_id = [1u8; 32];
        let data = vec![1, 2, 3, 4];
        
        // Should be miss initially
        assert!(cache_manager.get_utxo(&utxo_id).is_none());
        
        // Put data
        cache_manager.put_utxo(utxo_id, data.clone());
        
        // Should be hit now
        assert_eq!(cache_manager.get_utxo(&utxo_id), Some(data));
        
        // Check stats
        let stats = cache_manager.get_stats();
        assert_eq!(stats.utxo_hits, 1);
        assert_eq!(stats.utxo_misses, 1);
    }

    #[test]
    fn test_node_cache() {
        let config = CacheConfig {
            node_cache_size: 100,
            ..Default::default()
        };
        let cache_manager = CacheManager::new(config);
        
        let index = 42u64;
        let level = 5u8;
        let node_hash = [2u8; 32];
        
        // Should be miss initially
        assert!(cache_manager.get_node(index, level).is_none());
        
        // Put data
        cache_manager.put_node(index, level, node_hash);
        
        // Should be hit now
        assert_eq!(cache_manager.get_node(index, level), Some(node_hash));
        
        // Check stats
        let stats = cache_manager.get_stats();
        assert_eq!(stats.node_hits, 1);
        assert_eq!(stats.node_misses, 1);
    }

    #[test]
    fn test_cache_stats() {
        let config = CacheConfig {
            enable_stats: true,
            ..Default::default()
        };
        let cache_manager = CacheManager::new(config);
        
        // Generate some cache activity
        for i in 0..10 {
            let utxo_id = [i; 32];
            cache_manager.get_utxo(&utxo_id); // miss
            cache_manager.put_utxo(utxo_id, vec![i]);
            cache_manager.get_utxo(&utxo_id); // hit
        }
        
        let stats = cache_manager.get_stats();
        assert_eq!(stats.utxo_hits, 10);
        assert_eq!(stats.utxo_misses, 10);
        assert_eq!(stats.utxo_hit_rate(), 0.5);
    }
}