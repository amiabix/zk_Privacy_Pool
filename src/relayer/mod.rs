//! Relayer Module - DataService and TreeService integration
//! Handles the complete flow from deposit events to Merkle tree management

pub mod data_service;
pub mod tree_service;

use data_service::{DataService, DepositEvent, DataServiceError};
use tree_service::{TreeService, MerkleProof, TreeServiceError};
use serde::{Serialize, Deserialize};

/// Relayer service that integrates DataService and TreeService
#[derive(Debug, Clone)]
pub struct RelayerService {
    data_service: DataService,
    pub tree_service: TreeService,
}

impl RelayerService {
    pub fn new(contract_address: String, rpc_endpoint: String) -> Self {
        Self {
            data_service: DataService::new(contract_address, rpc_endpoint),
            tree_service: TreeService::new(),
        }
    }

    /// Process deposit events and update Merkle tree
    pub fn process_deposits(&mut self, from_block: u64, to_block: u64) -> Result<Vec<DepositResult>, RelayerError> {
        println!("🔄 Processing deposits from block {} to {}", from_block, to_block);
        
        // Fetch deposit events
        let events = self.data_service.fetch_deposit_events(from_block, to_block)
            .map_err(RelayerError::DataServiceError)?;
        
        let mut results = Vec::new();
        
        for event in events {
            match self.tree_service.add_deposit(&event) {
                Ok(root_hash) => {
                    println!("✅ Successfully added deposit: {} ETH from {}", 
                        event.value as f64 / 1e18, event.depositor);
                    
                    results.push(DepositResult {
                        success: true,
                        event,
                        root_hash,
                        error: None,
                    });
                }
                Err(e) => {
                    println!("❌ Failed to add deposit: {}", e);
                    
                    results.push(DepositResult {
                        success: false,
                        event,
                        root_hash: String::new(),
                        error: Some(e.to_string()),
                    });
                }
            }
        }
        
        Ok(results)
    }

    /// Get Merkle proof for commitment
    pub fn get_merkle_proof(&self, commitment: &str) -> Result<MerkleProof, RelayerError> {
        self.tree_service.get_proof(commitment)
            .map_err(RelayerError::TreeServiceError)
    }

    /// Get current Merkle root
    pub fn get_merkle_root(&self) -> String {
        self.tree_service.get_root_hash()
    }

    /// Get tree statistics
    pub fn get_tree_stats(&self) -> TreeStats {
        TreeStats {
            depth: self.tree_service.get_depth(),
            leaf_count: self.tree_service.get_leaf_count(),
            root_hash: self.tree_service.get_root_hash(),
        }
    }

    /// Start monitoring for new deposits
    pub fn start_monitoring(&mut self) -> Result<(), RelayerError> {
        println!("🚀 Starting relayer monitoring...");
        
        // Start data service monitoring
        self.data_service.start_monitoring()
            .map_err(RelayerError::DataServiceError)?;
        
        Ok(())
    }

    /// Get all commitments in tree
    pub fn get_all_commitments(&self) -> Vec<String> {
        self.tree_service.get_all_commitments()
    }

    /// Check if commitment exists
    pub fn has_commitment(&self, commitment: &str) -> bool {
        self.tree_service.has_commitment(commitment)
    }
}

/// Result of processing a deposit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositResult {
    pub success: bool,
    pub event: DepositEvent,
    pub root_hash: String,
    pub error: Option<String>,
}

/// Tree statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeStats {
    pub depth: u32,
    pub leaf_count: u64,
    pub root_hash: String,
}

/// Relayer errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelayerError {
    DataServiceError(DataServiceError),
    TreeServiceError(TreeServiceError),
    IntegrationError(String),
}

impl std::fmt::Display for RelayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayerError::DataServiceError(e) => write!(f, "DataService Error: {}", e),
            RelayerError::TreeServiceError(e) => write!(f, "TreeService Error: {}", e),
            RelayerError::IntegrationError(msg) => write!(f, "Integration Error: {}", msg),
        }
    }
}

impl std::error::Error for RelayerError {}

impl From<DataServiceError> for RelayerError {
    fn from(error: DataServiceError) -> Self {
        RelayerError::DataServiceError(error)
    }
}

impl From<TreeServiceError> for RelayerError {
    fn from(error: TreeServiceError) -> Self {
        RelayerError::TreeServiceError(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relayer_service() {
        let mut relayer = RelayerService::new(
            "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".to_string(),
            "http://127.0.0.1:8545".to_string(),
        );
        
        // Process deposits
        let results = relayer.process_deposits(1, 10).unwrap();
        assert!(!results.is_empty());
        
        // Check tree stats
        let stats = relayer.get_tree_stats();
        assert!(stats.leaf_count > 0);
        
        // Get commitments
        let commitments = relayer.get_all_commitments();
        assert!(!commitments.is_empty());
        
        // Get proof for first commitment
        if let Some(first_commitment) = commitments.first() {
            let proof = relayer.get_merkle_proof(first_commitment);
            assert!(proof.is_ok());
        }
        
        println!("✅ RelayerService test passed");
    }
}
