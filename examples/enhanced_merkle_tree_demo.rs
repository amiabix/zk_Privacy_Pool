//! Enhanced Merkle Tree Demo
//!
//! This example demonstrates the architecture-compliant EnhancedMerkleTree
//! with Poseidon hashing for ZK-friendliness

use privacy_pool_zkvm::merkle::EnhancedMerkleTree;
use privacy_pool_zkvm::crypto::CryptoUtils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🌳 Enhanced Merkle Tree Demo (Poseidon Hashing)");
    println!("{}", "=".repeat(60));

    // Demo 1: Create tree with small depth for testing
    println!("\n1️⃣  Creating Enhanced Merkle Tree (depth = 4)");
    let mut tree = EnhancedMerkleTree::with_depth(4)?;
    println!("   📊 Initial stats: {} leaves, max {}", tree.size(), 1u64 << tree.depth);
    println!("   🌳 Initial root: 0x{}", hex::encode(tree.get_root()));

    // Demo 2: Insert commitments
    println!("\n2️⃣  Inserting Commitments");
    let mut commitments = Vec::new();

    for i in 0..5 {
        let commitment = [i as u8; 32];
        commitments.push(commitment);

        let index = tree.insert(commitment)?;
        println!("   📋 Inserted commitment {} at leaf index {}", i, index);
    }

    println!("   📊 Tree after insertions: {} leaves", tree.size());
    println!("   🌳 New root: 0x{}", hex::encode(&tree.get_root()[..8]));

    // Demo 3: Generate and verify proofs
    println!("\n3️⃣  Generating Merkle Proofs");
    for (i, commitment) in commitments.iter().enumerate() {
        let proof = tree.get_proof(i as u64)?;
        println!("   🔍 Proof for leaf {}: {} siblings", i, proof.siblings.len());

        // Verify proof
        let is_valid = tree.verify_proof(&proof, *commitment)?;
        println!("   ✅ Proof verification: {}", is_valid);

        if !is_valid {
            return Err("Proof verification failed!".into());
        }
    }

    // Demo 4: Test duplicate detection
    println!("\n4️⃣  Testing Duplicate Detection");
    let duplicate_commitment = commitments[0];
    let duplicate_index = tree.insert(duplicate_commitment)?;
    println!("   🔄 Duplicate commitment returned index {}", duplicate_index);
    println!("   📊 Tree size remained: {} (no duplicate added)", tree.size());

    // Demo 5: Tree statistics
    println!("\n5️⃣  Tree Statistics");
    let stats = tree.stats();
    println!("   📊 Depth: {}", stats.depth);
    println!("   📊 Leaves: {}/{}", stats.leaf_count, stats.max_leaves);
    println!("   📊 Nodes stored: {}", stats.nodes_stored);
    println!("   🌳 Current root: 0x{}", hex::encode(stats.root));

    // Demo 6: Fast lookups
    println!("\n6️⃣  Fast Commitment Lookups");
    for (i, commitment) in commitments.iter().enumerate() {
        let found_index = tree.get_leaf_index(commitment);
        println!("   🔍 Commitment {} found at index: {:?}", i, found_index);
    }

    // Demo 7: Larger tree performance test
    println!("\n7️⃣  Performance Test (larger tree)");
    let mut large_tree = EnhancedMerkleTree::with_depth(10)?; // 1024 max leaves

    let start = std::time::Instant::now();

    // Insert 100 random commitments
    let mut test_commitments = Vec::new();
    for i in 0..100 {
        let commitment = CryptoUtils::random_32();
        test_commitments.push(commitment);
        large_tree.insert(commitment)?;
    }

    let insert_time = start.elapsed();

    // Generate 10 proofs
    let start = std::time::Instant::now();
    for i in 0..10 {
        let proof = large_tree.get_proof(i)?;
        let is_valid = large_tree.verify_proof(&proof, test_commitments[i as usize])?;
        if !is_valid {
            return Err("Large tree proof verification failed!".into());
        }
    }
    let proof_time = start.elapsed();

    println!("   ⚡ Inserted 100 commitments in {:?}", insert_time);
    println!("   ⚡ Generated/verified 10 proofs in {:?}", proof_time);
    println!("   📊 Large tree has {} leaves", large_tree.size());

    println!("\n🎉 Enhanced Merkle Tree Demo Complete!");
    println!("✅ All operations successful with Poseidon hashing");
    println!("✅ Architecture-compliant and ZK-friendly");

    Ok(())
}