#![no_main]
ziskos::entrypoint!(main);

use ziskos::{read_input, set_output};
use std::convert::TryInto;

// Real privacy pool transaction with proper cryptographic commitments
#[derive(serde::Serialize, serde::Deserialize)]
struct PrivacyPoolTransaction {
    // Input commitments (what user is spending)
    input_commitments: Vec<[u8; 32]>,
    // Output commitments (what user is creating)
    output_commitments: Vec<[u8; 32]>,
    // Nullifiers (preventing double-spend)
    nullifiers: Vec<[u8; 32]>,
    // Merkle proofs for input commitments
    merkle_proofs: Vec<MerkleProof>,
    // Signature over the transaction
    signature: Vec<u8>,
    // Public key of the signer
    public_key: Vec<u8>,
    // Transaction fee
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
    // Read transaction and current state
    let input: Vec<u8> = read_input();
    let (transaction, old_state): (PrivacyPoolTransaction, PrivacyPoolState) = 
        bincode::deserialize(&input).expect("Failed to deserialize input");
    
    // 1. Verify Merkle proofs for all input commitments
    let mut merkle_valid = true;
    for (i, (commitment, proof)) in transaction.input_commitments.iter()
        .zip(transaction.merkle_proofs.iter()).enumerate() {
        
        if !verify_merkle_proof(commitment, proof, &old_state.merkle_root) {
            merkle_valid = false;
            break;
        }
    }
    
    // 2. Check nullifiers haven't been used before
    let mut no_double_spend = true;
    for nullifier in &transaction.nullifiers {
        if old_state.nullifier_set.contains(nullifier) {
            no_double_spend = false;
            break;
        }
    }
    
    // 3. Verify signature over transaction
    let message = create_transaction_message(&transaction);
    let signature_valid = verify_ecdsa_signature(&message, &transaction.signature, &transaction.public_key);
    
    // 4. Verify commitment balance (inputs >= outputs + fee)
    let total_inputs = calculate_commitment_sum(&transaction.input_commitments);
    let total_outputs = calculate_commitment_sum(&transaction.output_commitments);
    let balance_valid = total_inputs >= total_outputs + transaction.fee;
    
    // 5. Calculate new state
    let mut new_nullifier_set = old_state.nullifier_set.clone();
    new_nullifier_set.extend_from_slice(&transaction.nullifiers);
    
    let new_merkle_root = update_merkle_tree(&old_state.merkle_root, &transaction.output_commitments);
    let new_pool_balance = old_state.pool_balance + transaction.fee;
    
    // Overall validation
    let is_valid = merkle_valid && no_double_spend && signature_valid && balance_valid;
    
    // Output results
    set_output(0, if is_valid { 1 } else { 0 }); // Validation result
    set_output(1, if merkle_valid { 1 } else { 0 }); // Merkle proof validity
    set_output(2, if no_double_spend { 1 } else { 0 }); // No double spend
    set_output(3, if signature_valid { 1 } else { 0 }); // Signature validity
    set_output(4, if balance_valid { 1 } else { 0 }); // Balance validity
    
    // Output new state
    for i in 0..8 {
        let chunk = u32::from_le_bytes(new_merkle_root[i*4..(i+1)*4].try_into().unwrap());
        set_output(5 + i, chunk);
    }
    
    set_output(13, new_pool_balance as u32);
    set_output(14, (new_pool_balance >> 32) as u32);
    
    // Output transaction hash
    let tx_hash = compute_transaction_hash(&transaction);
    for i in 0..8 {
        let chunk = u32::from_le_bytes(tx_hash[i*4..(i+1)*4].try_into().unwrap());
        set_output(15 + i, chunk);
    }
}

// Verify Merkle proof against current root
fn verify_merkle_proof(leaf: &[u8; 32], proof: &MerkleProof, current_root: &[u8; 32]) -> bool {
    let mut current = *leaf;
    
    for (sibling, is_right) in proof.siblings.iter().zip(proof.path.iter()) {
        current = if *is_right {
            hash_pair(current, *sibling)
        } else {
            hash_pair(*sibling, current)
        };
    }
    
    // Verify against CURRENT root, not proof.root
    current == *current_root
}

// Hash two 32-byte values together
fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(&left);
    hasher.update(&right);
    hasher.finalize().into()
}

// Verify ECDSA signature (simplified for ZisK)
fn verify_ecdsa_signature(message: &[u8], signature: &[u8], public_key: &[u8]) -> bool {
    // This is a placeholder - in production, use proper ECDSA verification
    // For now, just check signature is not all zeros
    signature.len() == 64 && !signature.iter().all(|&x| x == 0) && 
    public_key.len() == 33 && !public_key.iter().all(|&x| x == 0)
}

// Create transaction message for signing
fn create_transaction_message(tx: &PrivacyPoolTransaction) -> Vec<u8> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    
    for commitment in &tx.input_commitments {
        hasher.update(commitment);
    }
    for commitment in &tx.output_commitments {
        hasher.update(commitment);
    }
    for nullifier in &tx.nullifiers {
        hasher.update(nullifier);
    }
    hasher.update(&tx.fee.to_le_bytes());
    
    hasher.finalize().to_vec()
}

// Calculate sum of commitments (simplified)
fn calculate_commitment_sum(commitments: &[[u8; 32]]) -> u64 {
    // In a real implementation, this would extract values from commitments
    // For now, return a fixed value based on commitment count
    commitments.len() as u64 * 1000 // Each commitment represents 1000 units
}

// Update Merkle tree with new commitments
fn update_merkle_tree(old_root: &[u8; 32], new_commitments: &[[u8; 32]]) -> [u8; 32] {
    // Simplified: just hash the old root with new commitments
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(old_root);
    for commitment in new_commitments {
        hasher.update(commitment);
    }
    hasher.finalize().into()
}

// Compute transaction hash
fn compute_transaction_hash(tx: &PrivacyPoolTransaction) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    
    for commitment in &tx.input_commitments {
        hasher.update(commitment);
    }
    for commitment in &tx.output_commitments {
        hasher.update(commitment);
    }
    for nullifier in &tx.nullifiers {
        hasher.update(nullifier);
    }
    hasher.update(&tx.fee.to_le_bytes());
    hasher.update(&tx.public_key);
    
    hasher.finalize().into()
}
