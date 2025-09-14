//! Merkle Tree Module
//! Architecture-compliant Merkle trees with Poseidon hashing
pub mod enhanced_merkle_tree;
pub mod canonical_smt;
pub mod tornado_merkle_tree;
pub mod tree_inspector;

// Re-export main types
pub use enhanced_merkle_tree::{EnhancedMerkleTree, TreeStats};
pub use canonical_smt::{CanonicalSMT, SMTNode};
pub use tornado_merkle_tree::{TornadoMerkleTree, TornadoMerkleProof, TornadoMerkleTreeStats, TornadoCommitmentHasher, TornadoWithdrawalCircuit, TornadoWithdrawalData};
pub use tree_inspector::{TreeInspector, demo_comprehensive_inspection, InspectionReport};
