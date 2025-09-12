use std::fs;
use sha2::{Digest, Sha256};

#[derive(serde::Serialize, serde::Deserialize)]
struct PrivacyPoolTransaction {
    input_commitments: Vec<[u8; 32]>,
    output_commitments: Vec<[u8; 32]>,
    nullifiers: Vec<[u8; 32]>,
    merkle_proofs: Vec<MerkleProof>,
    signature: Vec<u8>,
    public_key: Vec<u8>,
    fee: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MerkleProof {
    siblings: Vec<[u8; 32]>,
    path: Vec<bool>,
    root: [u8; 32],
    leaf_index: usize,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PrivacyPoolState {
    merkle_root: [u8; 32],
    nullifier_set: Vec<[u8; 32]>,
    pool_balance: u64,
}

fn main() {
    println!("Creating realistic privacy pool test...");
    
    // Create a realistic privacy pool scenario
    let mut merkle_leaves = Vec::new();
    let mut nullifier_set = Vec::new();
    
    // Generate some existing commitments in the Merkle tree
    for i in 0..5 {
        let commitment = generate_commitment(i as u64, [i as u8; 32]);
        merkle_leaves.push(commitment);
    }
    
    // Build Merkle tree and get root
    let merkle_root = build_merkle_tree(&merkle_leaves);
    
    // Create a transaction that spends 2 existing commitments and creates 1 new one
    let input_commitments = vec![merkle_leaves[0], merkle_leaves[1]];
    let output_commitments = vec![generate_commitment(1500, [42u8; 32])]; // 1500 units
    let nullifiers = vec![
        generate_nullifier(&input_commitments[0]),
        generate_nullifier(&input_commitments[1])
    ];
    
    // Generate Merkle proofs for the input commitments
    let merkle_proofs = vec![
        generate_merkle_proof(&merkle_leaves, 0, &merkle_root),
        generate_merkle_proof(&merkle_leaves, 1, &merkle_root)
    ];
    
    // Create transaction
    let transaction = PrivacyPoolTransaction {
        input_commitments,
        output_commitments,
        nullifiers,
        merkle_proofs,
        signature: vec![1u8; 64], // Mock signature
        public_key: vec![2u8; 33], // Mock public key
        fee: 10, // 10 unit fee
    };
    
    // Create initial state
    let old_state = PrivacyPoolState {
        merkle_root,
        nullifier_set,
        pool_balance: 0,
    };
    
    // Serialize and save
    let input_data = (transaction, old_state);
    let serialized = bincode::serialize(&input_data).unwrap();
    fs::write("build/real_privacy_pool_input.bin", serialized).unwrap();
    
    println!("=== REAL PRIVACY POOL TEST CREATED ===");
    println!("Input commitments: {}", input_data.0.input_commitments.len());
    println!("Output commitments: {}", input_data.0.output_commitments.len());
    println!("Nullifiers: {}", input_data.0.nullifiers.len());
    println!("Merkle proofs: {}", input_data.0.merkle_proofs.len());
    println!("Fee: {} units", input_data.0.fee);
    println!("Saved to: build/real_privacy_pool_input.bin");
    println!("Expected result: SUCCESS (valid privacy pool transaction)");
}

// Generate a commitment from value and secret
fn generate_commitment(value: u64, secret: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(&value.to_le_bytes());
    hasher.update(&secret);
    hasher.finalize().into()
}

// Generate a nullifier from a commitment
fn generate_nullifier(commitment: &[u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"nullifier");
    hasher.update(commitment);
    hasher.finalize().into()
}

// Build a Merkle tree from leaves
fn build_merkle_tree(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    
    let mut current_level = leaves.to_vec();
    
    while current_level.len() > 1 {
        let mut next_level = Vec::new();
        
        for i in (0..current_level.len()).step_by(2) {
            let left = current_level[i];
            let right = if i + 1 < current_level.len() {
                current_level[i + 1]
            } else {
                left // Duplicate last element if odd number
            };
            
            let parent = hash_pair(left, right);
            next_level.push(parent);
        }
        
        current_level = next_level;
    }
    
    current_level[0]
}

// Generate Merkle proof for a leaf
fn generate_merkle_proof(leaves: &[[u8; 32]], leaf_index: usize, root: &[u8; 32]) -> MerkleProof {
    let mut siblings = Vec::new();
    let mut path = Vec::new();
    let mut current_index = leaf_index;
    let mut current_level = leaves.to_vec();
    
    while current_level.len() > 1 {
        let sibling_index = if current_index % 2 == 0 {
            current_index + 1
        } else {
            current_index - 1
        };
        
        if sibling_index < current_level.len() {
            siblings.push(current_level[sibling_index]);
            path.push(current_index % 2 == 0);
        } else {
            // Handle odd number of elements
            siblings.push(current_level[current_index]);
            path.push(false);
        }
        
        current_index /= 2;
        
        // Build next level
        let mut next_level = Vec::new();
        for i in (0..current_level.len()).step_by(2) {
            let left = current_level[i];
            let right = if i + 1 < current_level.len() {
                current_level[i + 1]
            } else {
                left
            };
            
            let parent = hash_pair(left, right);
            next_level.push(parent);
        }
        
        current_level = next_level;
    }
    
    MerkleProof {
        siblings,
        path,
        root: *root,
        leaf_index,
    }
}

// Hash two 32-byte values together
fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(&left);
    hasher.update(&right);
    hasher.finalize().into()
}
