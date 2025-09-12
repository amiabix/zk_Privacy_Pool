//! Simple Merkle Tree Test
//! Test basic Merkle tree functionality

use privacy_pool_zkvm::enhanced_merkle_tree::{EnhancedMerkleTree, UTXO};
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌳 Simple Merkle Tree Test");
    println!("{}", "=".repeat(40));

    let mut tree = EnhancedMerkleTree::new();
    
    // Test with 2 UTXOs first
    println!("\n1️⃣  Testing with 2 UTXOs...");
    
    let owner1 = [0x11; 32];
    let owner2 = [0x22; 32];
    
    // Insert first UTXO
    let index1 = tree.insert_commitment(1000, [0x11; 32], [0x11; 32], owner1, 1000)?;
    println!("   Inserted UTXO 0: index {}", index1);
    println!("   Tree root: {:?}", tree.get_root());
    
    // Insert second UTXO
    let index2 = tree.insert_commitment(2000, [0x22; 32], [0x22; 32], owner2, 1001)?;
    println!("   Inserted UTXO 1: index {}", index2);
    println!("   Tree root: {:?}", tree.get_root());
    
    // Debug: manually calculate what the root should be
    let leaf1 = tree.hash_utxo(&tree.utxos[0]);
    let leaf2 = tree.hash_utxo(&tree.utxos[1]);
    let manual_root = tree.hash_pair(leaf1, leaf2);
    println!("   Manual root calculation: {:?}", manual_root);
    println!("   Leaf 1: {:?}", leaf1);
    println!("   Leaf 2: {:?}", leaf2);
    
    // Test proof for first UTXO
    println!("\n2️⃣  Testing Merkle proof for UTXO 0...");
    let proof1 = tree.generate_proof(0)?;
    println!("   Proof siblings: {}", proof1.siblings.len());
    println!("   Proof path: {:?}", proof1.path_indices);
    
    let utxo1 = &tree.utxos[0];
    let leaf_hash1 = tree.hash_utxo(utxo1);
    println!("   Leaf hash: {:?}", leaf_hash1);
    
    // Manual verification
    let mut current = leaf_hash1;
    for (i, (sibling, is_right)) in proof1.siblings.iter().zip(proof1.path_indices.iter()).enumerate() {
        println!("   Step {}: sibling={:?}, is_right={}", i, sibling, is_right);
        current = if *is_right {
            tree.hash_pair(*sibling, current)
        } else {
            tree.hash_pair(current, *sibling)
        };
        println!("   Step {} result: {:?}", i, current);
    }
    
    let is_valid = current == proof1.root;
    println!("   Final result: {:?}", current);
    println!("   Proof root: {:?}", proof1.root);
    println!("   Verification: {}", if is_valid { "✅" } else { "❌" });
    
    // Test proof for second UTXO
    println!("\n3️⃣  Testing Merkle proof for UTXO 1...");
    let proof2 = tree.generate_proof(1)?;
    println!("   Proof siblings: {}", proof2.siblings.len());
    println!("   Proof path: {:?}", proof2.path_indices);
    
    let utxo2 = &tree.utxos[1];
    let leaf_hash2 = tree.hash_utxo(utxo2);
    println!("   Leaf hash: {:?}", leaf_hash2);
    
    // Manual verification
    let mut current = leaf_hash2;
    for (i, (sibling, is_right)) in proof2.siblings.iter().zip(proof2.path_indices.iter()).enumerate() {
        println!("   Step {}: sibling={:?}, is_right={}", i, sibling, is_right);
        current = if *is_right {
            tree.hash_pair(*sibling, current)
        } else {
            tree.hash_pair(current, *sibling)
        };
        println!("   Step {} result: {:?}", i, current);
    }
    
    let is_valid = current == proof2.root;
    println!("   Final result: {:?}", current);
    println!("   Proof root: {:?}", proof2.root);
    println!("   Verification: {}", if is_valid { "✅" } else { "❌" });
    
    // Test tree statistics
    println!("\n4️⃣  Tree Statistics...");
    let stats = tree.get_stats();
    println!("   Total UTXOs: {}", stats.total_utxos);
    println!("   Tree depth: {}", stats.tree_depth);
    println!("   Root hash: {:?}", stats.root_hash);
    
    Ok(())
}
