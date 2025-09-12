use serde::{Deserialize, Serialize};
use crate::utxo::UTXO;
use crate::transaction::Error;
use crate::zisk_precompiles;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleTree {
    pub utxos: Vec<UTXO>,
    pub root: [u8; 32],
    pub depth: usize,
}

impl MerkleTree {
    pub fn new() -> Self {
        Self {
            utxos: Vec::new(),
            root: [0u8; 32],
            depth: 0,
        }
    }
    
    pub fn insert_utxo(&mut self, utxo: UTXO) -> Result<usize, Error> {
        let index = self.utxos.len();
        self.utxos.push(utxo);
        self.update_root()?;
        Ok(index)
    }
    
    pub fn generate_proof(&self, index: usize) -> Result<crate::utxo::MerkleProof, Error> {
        if index >= self.utxos.len() {
            return Err(Error::MerkleTreeError("Index out of bounds".to_string()));
        }
        
        let mut siblings = Vec::new();
        let mut path = Vec::new();
        let mut current_index = index;
        
        // Generate proof by walking up the tree
        let mut level_size = self.utxos.len();
        while level_size > 1 {
            let is_right = current_index % 2 == 1;
            let sibling_index = if is_right { current_index - 1 } else { current_index + 1 };
            
            if sibling_index < level_size {
                let sibling_hash = hash_utxo(&self.utxos[sibling_index]);
                siblings.push(sibling_hash);
                path.push(is_right);
            }
            
            current_index /= 2;
            level_size = (level_size + 1) / 2;
        }
        
        Ok(crate::utxo::MerkleProof {
            siblings,
            path,
            root: self.root,
            leaf_index: index as u64,
        })
    }
    
    pub fn verify_merkle_proof(&self, proof: &crate::utxo::MerkleProof, leaf: &[u8; 32]) -> bool {
        let mut current = *leaf;
        for (sibling, is_right) in proof.siblings.iter().zip(proof.path.iter()) {
            current = if *is_right {
                hash_pair(current, *sibling)
            } else {
                hash_pair(*sibling, current)
            };
        }
        // CRITICAL FIX: Verify against current tree root, not proof.root
        current == self.root
    }

    fn update_root(&mut self) -> Result<(), Error> {
        if self.utxos.is_empty() {
            self.root = [0u8; 32];
            return Ok(());
        }
        
        if self.utxos.len() == 1 {
            self.root = hash_utxo(&self.utxos[0]);
            return Ok(());
        }
        
        // Build tree bottom-up
        let mut current_level = self.utxos.iter().map(|utxo| hash_utxo(utxo)).collect::<Vec<_>>();
        
        while current_level.len() > 1 {
            let mut next_level = Vec::new();
            for i in (0..current_level.len()).step_by(2) {
                let left = current_level[i];
                let right = if i + 1 < current_level.len() {
                    current_level[i + 1]
                } else {
                    current_level[i] // Duplicate last element if odd number
                };
                
                let parent = hash_pair(left, right);
                next_level.push(parent);
            }
            current_level = next_level;
        }
        
        self.root = current_level[0];
        self.depth = (self.utxos.len() as f64).log2().ceil() as usize;
        Ok(())
    }
}

fn hash_utxo(utxo: &UTXO) -> [u8; 32] {
    // Use ZisK-compatible hash function
    // TODO: Replace with ZisK SHA-256 precompile (cost: 9,000 constraint units)
    let mut data = Vec::new();
    data.extend_from_slice(&utxo.commitment);
    data.extend_from_slice(&utxo.value.to_le_bytes());
    data.extend_from_slice(&utxo.owner);
    
    zisk_precompiles::zisk_sha256(&data)
}

fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    // Use ZisK-compatible hash function
    // TODO: Replace with ZisK SHA-256 precompile (cost: 9,000 constraint units)
    zisk_precompiles::zisk_hash_pair(left, right)
}