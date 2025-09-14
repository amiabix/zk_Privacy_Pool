//! Merkle Tree Module
pub mod enhanced_merkle_tree;
pub mod canonical_smt;
pub mod tornado_merkle_tree;
pub mod tree_inspector;

// Re-export main types
pub use enhanced_merkle_tree::{EnhancedMerkleTree, RelayerService, RelayerConfig};
pub use canonical_smt::{CanonicalSMT, SMTNode, TreeStats};
pub use tornado_merkle_tree::{TornadoMerkleTree, TornadoMerkleProof, TornadoMerkleTreeStats, TornadoCommitmentHasher, TornadoWithdrawalCircuit, TornadoWithdrawalData};
pub use tree_inspector::{TreeInspector, demo_comprehensive_inspection, InspectionReport};
