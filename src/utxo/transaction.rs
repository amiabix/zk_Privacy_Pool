use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionResult {
    Success,
    Failure(String),
}

impl TransactionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TransactionResult::Success)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Error {
    DoubleSpend,
    InvalidMerkleProof,
    InsufficientBalance,
    InvalidTransaction,
    MerkleTreeError(String),
    InvalidUTXO,
    UserNotFound,
    InvalidSignature,
    Other(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Other(err.to_string())
    }
}

/// Merkle proof structure for transaction validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    /// Sibling hashes
    pub siblings: Vec<[u8; 32]>,
    /// Path indices
    pub path: Vec<u32>,
    /// Root hash
    pub root: [u8; 32],
    /// Leaf index
    pub leaf_index: u64,
}

impl MerkleProof {
    /// Create a new Merkle proof
    pub fn new(siblings: Vec<[u8; 32]>, path: Vec<u32>, root: [u8; 32], leaf_index: u64) -> Self {
        Self {
            siblings,
            path,
            root,
            leaf_index,
        }
    }

    /// Verify the Merkle proof
    pub fn verify(&self, leaf: [u8; 32]) -> bool {
        let mut current = leaf;
        let mut path_index = 0;

        for sibling in &self.siblings {
            if path_index < self.path.len() && self.path[path_index] == 1 {
                // Right child
                current = Self::hash_children(*sibling, current);
            } else {
                // Left child
                current = Self::hash_children(current, *sibling);
            }
            path_index += 1;
        }

        current == self.root
    }

    /// Hash two children nodes
    fn hash_children(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(left);
        hasher.update(right);
        hasher.finalize().into()
    }
}