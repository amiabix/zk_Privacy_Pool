use sha2::{Digest, Sha256};
use crate::MerkleProof;

pub struct ZkProofGenerator;

impl ZkProofGenerator {
    pub fn new() -> Self {
        Self
    }
    
    pub fn generate_commitment(&self, value: u64, secret: &[u8; 32], nullifier: &[u8; 32]) -> [u8; 32] {
        // Generate commitment using SHA-256 (you can replace with Poseidon later)
        let mut hasher = Sha256::new();
        hasher.update(&value.to_le_bytes());
        hasher.update(secret);
        hasher.update(nullifier);
        hasher.finalize().into()
    }
    
    pub fn generate_nullifier_hash(&self, nullifier: &[u8; 32]) -> [u8; 32] {
        // Generate nullifier hash using SHA-256
        let mut hasher = Sha256::new();
        hasher.update(nullifier);
        hasher.finalize().into()
    }
    
    pub fn verify_merkle_proof(&self, proof: &MerkleProof, leaf: &[u8; 32]) -> bool {
        // Verify Merkle proof
        let mut current = *leaf;
        
        for (sibling, is_right) in proof.siblings.iter().zip(proof.path.iter()) {
            if *is_right == 1 {
                current = self.hash_pair(current, *sibling);
            } else {
                current = self.hash_pair(*sibling, current);
            }
        }
        
        current == proof.root
    }
    
    fn hash_pair(&self, left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(&left);
        hasher.update(&right);
        hasher.finalize().into()
    }
}
