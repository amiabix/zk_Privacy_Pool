//! Full Integration Test for Privacy Pool
//! Tests the complete flow: deposit -> proof generation -> withdrawal

use crate::enhanced_merkle_tree::{EnhancedMerkleTree, UTXO, MerkleProof};
use anyhow::Result;

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test the complete privacy pool workflow
    #[test]
    fn test_complete_privacy_pool_workflow() {
        let mut tree = EnhancedMerkleTree::new();
        
        // === STEP 1: DEPOSIT ===
        println!("=== STEP 1: DEPOSIT ===");
        
        // Alice deposits 1 ETH
        let alice_owner = [0x01; 32];
        let alice_value = 1_000_000_000_000_000_000u64; // 1 ETH in wei
        let alice_secret = [0x11; 32];
        let alice_nullifier = [0x21; 32];
        let block_height = 1000;
        
        let alice_index = tree.insert_commitment(
            alice_value, 
            alice_secret, 
            alice_nullifier, 
            alice_owner, 
            block_height
        ).unwrap();
        
        println!("✅ Alice deposited {} ETH at index {}", alice_value as f64 / 1e18, alice_index);
        println!("   Alice's commitment: {:?}", tree.utxos[alice_index].commitment);
        
        // Bob deposits 2 ETH
        let bob_owner = [0x02; 32];
        let bob_value = 2_000_000_000_000_000_000u64; // 2 ETH in wei
        let bob_secret = [0x12; 32];
        let bob_nullifier = [0x22; 32];
        
        let bob_index = tree.insert_commitment(
            bob_value, 
            bob_secret, 
            bob_nullifier, 
            bob_owner, 
            block_height + 1
        ).unwrap();
        
        println!("✅ Bob deposited {} ETH at index {}", bob_value as f64 / 1e18, bob_index);
        println!("   Bob's commitment: {:?}", tree.utxos[bob_index].commitment);
        
        // Check tree state
        let stats = tree.get_stats();
        println!("📊 Tree stats: {} UTXOs, root: {:?}", stats.total_utxos, stats.root_hash);
        
        // === STEP 2: PROOF GENERATION ===
        println!("\n=== STEP 2: PROOF GENERATION ===");
        
        // Generate Merkle proof for Alice's UTXO
        let alice_proof = tree.generate_proof(alice_index as u64).unwrap();
        let alice_leaf_hash = tree.hash_utxo(&tree.utxos[alice_index]);
        
        println!("🔍 Generated Merkle proof for Alice:");
        println!("   Proof siblings: {}", alice_proof.siblings.len());
        println!("   Proof root: {:?}", alice_proof.root);
        println!("   Alice's leaf hash: {:?}", alice_leaf_hash);
        
        // Verify the proof
        let proof_valid = tree.verify_proof(&alice_proof, &alice_leaf_hash);
        println!("✅ Alice's proof verification: {}", proof_valid);
        assert!(proof_valid, "Alice's Merkle proof should be valid");
        
        // === STEP 3: SPENDING (WITHDRAWAL) ===
        println!("\n=== STEP 3: SPENDING (WITHDRAWAL) ===");
        
        // Alice wants to spend her UTXO
        let alice_leaf_index = tree.utxos[alice_index].leaf_index;
        let alice_nullifier_hash = tree.utxos[alice_index].compute_nullifier_hash();
        let alice_owns_utxo = tree.utxos[alice_index].verify_ownership(&alice_owner);
        
        println!("💸 Alice initiating withdrawal:");
        println!("   Value: {} ETH", alice_value as f64 / 1e18);
        println!("   Nullifier hash: {:?}", alice_nullifier_hash);
        
        // Check that Alice owns the UTXO
        assert!(alice_owns_utxo, "Alice should own her UTXO");
        
        // Check nullifier hasn't been used
        assert!(!tree.is_nullifier_used(&alice_nullifier_hash), "Alice's nullifier should not be used yet");
        
        // Perform the spend (this would be done by the smart contract after ZK proof verification)
        let spend_result = tree.spend_utxo(alice_nullifier_hash, alice_leaf_index);
        assert!(spend_result.is_ok(), "Alice's spend should succeed");
        
        println!("✅ Alice successfully withdrew {} ETH", alice_value as f64 / 1e18);
        
        // === STEP 4: DOUBLE-SPEND PREVENTION ===
        println!("\n=== STEP 4: DOUBLE-SPEND PREVENTION ===");
        
        // Try to spend the same UTXO again (should fail)
        let double_spend_result = tree.spend_utxo(alice_nullifier_hash, alice_leaf_index);
        assert!(double_spend_result.is_err(), "Double spend should be prevented");
        
        println!("🛡️  Double-spend attempt blocked: {}", double_spend_result.unwrap_err());
        
        // === STEP 5: VERIFY FINAL STATE ===
        println!("\n=== STEP 5: VERIFY FINAL STATE ===");
        
        // Check Alice's UTXOs are spent
        let alice_unspent = tree.get_unspent_utxos_by_owner(&alice_owner);
        assert_eq!(alice_unspent.len(), 0, "Alice should have no unspent UTXOs");
        
        // Check Bob's UTXOs are still unspent
        let bob_unspent = tree.get_unspent_utxos_by_owner(&bob_owner);
        assert_eq!(bob_unspent.len(), 1, "Bob should have 1 unspent UTXO");
        assert_eq!(bob_unspent[0].value, bob_value, "Bob's UTXO value should be correct");
        
        println!("✅ Alice has {} unspent UTXOs", alice_unspent.len());
        println!("✅ Bob has {} unspent UTXOs worth {} ETH", 
                 bob_unspent.len(), 
                 bob_unspent[0].value as f64 / 1e18);
        
        // Final tree statistics
        let final_stats = tree.get_stats();
        println!("📊 Final tree stats: {} UTXOs, {} nullifiers used", 
                 final_stats.total_utxos, 
                 tree.used_nullifiers.len());
        
        println!("\n🎉 COMPLETE PRIVACY POOL WORKFLOW SUCCESSFUL! 🎉");
    }
    
    /// Test multiple deposits and selective spending
    #[test]
    fn test_multiple_deposits_and_selective_spending() {
        let mut tree = EnhancedMerkleTree::new();
        
        // Create multiple users
        let users = [
            ([0x01; 32], 1_000_000_000_000_000_000u64), // Alice: 1 ETH
            ([0x02; 32], 2_000_000_000_000_000_000u64), // Bob: 2 ETH  
            ([0x03; 32], 3_000_000_000_000_000_000u64), // Carol: 3 ETH
        ];
        
        let mut utxo_indices = Vec::new();
        
        // All users deposit
        for (i, (owner, value)) in users.iter().enumerate() {
            let secret = [i as u8 + 0x10; 32];
            let nullifier = [i as u8 + 0x20; 32];
            
            let index = tree.insert_commitment(
                *value,
                secret,
                nullifier,
                *owner,
                1000 + i as u32
            ).unwrap();
            
            utxo_indices.push(index);
            println!("User {} deposited {} ETH", i + 1, *value as f64 / 1e18);
        }
        
        // Verify all UTXOs are unspent initially
        let total_unspent: usize = users.iter()
            .map(|(owner, _)| tree.get_unspent_utxos_by_owner(owner).len())
            .sum();
        assert_eq!(total_unspent, 3, "All UTXOs should be unspent initially");
        
        // Bob (middle user) spends his UTXO
        let bob_utxo = &tree.utxos[utxo_indices[1]];
        let bob_nullifier_hash = bob_utxo.compute_nullifier_hash();
        
        assert!(tree.spend_utxo(bob_nullifier_hash, bob_utxo.leaf_index).is_ok());
        println!("Bob withdrew {} ETH", users[1].1 as f64 / 1e18);
        
        // Verify spending state
        assert_eq!(tree.get_unspent_utxos_by_owner(&users[0].0).len(), 1); // Alice: 1 unspent
        assert_eq!(tree.get_unspent_utxos_by_owner(&users[1].0).len(), 0); // Bob: 0 unspent  
        assert_eq!(tree.get_unspent_utxos_by_owner(&users[2].0).len(), 1); // Carol: 1 unspent
        
        println!("✅ Selective spending test passed");
    }
    
    /// Test Merkle proof generation for different tree sizes
    #[test]
    fn test_merkle_proof_scaling() {
        let mut tree = EnhancedMerkleTree::new();
        
        let test_sizes = [1, 2, 4, 8, 16];
        
        for &size in &test_sizes {
            // Clear tree
            tree = EnhancedMerkleTree::new();
            
            // Insert UTXOs
            for i in 0..size {
                let owner = [i as u8; 32];
                let value = (i + 1) as u64 * 1_000_000_000_000_000_000;
                let secret = [i as u8 + 0x10; 32];
                let nullifier = [i as u8 + 0x20; 32];
                
                tree.insert_commitment(value, secret, nullifier, owner, 1000 + i as u32).unwrap();
            }
            
            // Generate and verify proofs for all UTXOs
            for i in 0..size {
                let proof = tree.generate_proof(i as u64).unwrap();
                let leaf_hash = tree.hash_utxo(&tree.utxos[i]);
                let is_valid = tree.verify_proof(&proof, &leaf_hash);
                
                assert!(is_valid, "Proof should be valid for UTXO {} in tree of size {}", i, size);
            }
            
            println!("✅ All proofs valid for tree size: {}", size);
        }
    }
}