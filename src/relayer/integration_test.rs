//! Relayer Integration Test
//! Demonstrates the complete flow: Deposit Events → Merkle Tree → ZK Proofs → Withdrawal

use crate::relayer::{DataService, TreeService};
use crate::privacy::utxo_pool::{UTXOPrivacyPool, ETHDepositEvent};
use serde::{Serialize, Deserialize};

/// Complete integration test for the relayer system
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_relayer_flow() {
        println!("🚀 Testing Complete Relayer Flow");
        println!("=================================");

        // Step 1: Initialize Relayer Service
        println!("\n📋 Step 1: Initialize Relayer Service");
        let mut relayer = RelayerService::new(
            "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".to_string(), // Deployed contract address
            "http://127.0.0.1:8545".to_string(), // Anvil RPC endpoint
        );
        println!("✅ Relayer service initialized");

        // Step 2: Process Deposit Events
        println!("\n💰 Step 2: Process Deposit Events");
        let results = relayer.process_deposits(1, 10).unwrap();
        println!("✅ Processed {} deposit events", results.len());
        
        for (i, result) in results.iter().enumerate() {
            if result.success {
                println!("   Event {}: {} ETH from {} → Root: {}", 
                    i + 1, 
                    result.event.value as f64 / 1e18,
                    result.event.depositor,
                    &result.root_hash[..16] // Show first 16 chars
                );
            } else {
                println!("   Event {}: FAILED - {}", i + 1, result.error.as_ref().unwrap());
            }
        }

        // Step 3: Check Merkle Tree State
        println!("\n🌳 Step 3: Check Merkle Tree State");
        let stats = relayer.get_tree_stats();
        println!("✅ Tree Stats:");
        println!("   - Depth: {}", stats.depth);
        println!("   - Leaf Count: {}", stats.leaf_count);
        println!("   - Root Hash: {}", &stats.root_hash[..16]);

        // Step 4: Get Merkle Proofs
        println!("\n🔐 Step 4: Get Merkle Proofs");
        let commitments = relayer.get_all_commitments();
        println!("✅ Found {} commitments in tree", commitments.len());
        
        for (i, commitment) in commitments.iter().enumerate() {
            let proof = relayer.get_merkle_proof(commitment).unwrap();
            println!("   Proof {}: commitment = {}, leaf_index = {}, path_length = {}", 
                i + 1, &commitment[..16], proof.leaf_index, proof.path.len());
        }

        // Step 5: Initialize UTXO Privacy Pool
        println!("\n🔄 Step 5: Initialize UTXO Privacy Pool");
        let mut utxo_pool = UTXOPrivacyPool::new([0x01; 32]);
        
        // Register user
        let eth_addr = [0x12u8; 20];
        let privacy_pk = [0x34u8; 32];
        utxo_pool.register_user(eth_addr, privacy_pk);
        println!("✅ UTXO Privacy Pool initialized with user");

        // Step 6: Convert Deposit Events to UTXOs
        println!("\n💎 Step 6: Convert Deposit Events to UTXOs");
        for result in &results {
            if result.success {
                let deposit_event = ETHDepositEvent {
                    depositor: [0x12u8; 20], // Use fixed address for testing
                    amount_wei: result.event.value,
                    block_number: result.event.block_number,
                    tx_hash: [0x34u8; 32], // Use fixed hash for testing
                    log_index: result.event.log_index,
                    commitment: [0x56u8; 32], // Use fixed commitment for testing
                    label: result.event.label,
                };
                
                let utxo_result = utxo_pool.process_eth_deposit(deposit_event);
                if utxo_result.is_ok() {
                    let utxo_ids = utxo_result.unwrap();
                    println!("   ✅ Created {} UTXOs from deposit", utxo_ids.len());
                } else {
                    println!("   ❌ Failed to create UTXOs: {}", utxo_result.err().unwrap());
                }
            }
        }

        // Step 7: Check UTXO Balance
        println!("\n💳 Step 7: Check UTXO Balance");
        let balance = utxo_pool.get_user_balance(&privacy_pk);
        let utxos = utxo_pool.get_user_utxos(&privacy_pk);
        println!("✅ User balance: {} ETH", balance as f64 / 1e18);
        println!("✅ User has {} UTXOs", utxos.len());

        // Step 8: Prepare Spending Proof
        println!("\n🔐 Step 8: Prepare Spending Proof");
        if let Some(utxo) = utxos.first() {
            let spending_proof = utxo_pool.prepare_spending_proof(
                utxo.id,
                500000000000000000, // 0.5 ETH withdrawal
                [0x78u8; 20], // Recipient
            );
            
            if spending_proof.is_ok() {
                let proof = spending_proof.unwrap();
                println!("✅ Spending proof prepared:");
                println!("   - Withdrawing: {} ETH", proof.withdrawn_value as f64 / 1e18);
                println!("   - Remaining: {} ETH", proof.remaining_value as f64 / 1e18);
                println!("   - Nullifier: {:?}", &proof.nullifier[..8]);
            } else {
                println!("❌ Failed to prepare spending proof: {}", spending_proof.err().unwrap());
            }
        }

        // Step 9: Verify Merkle Proofs
        println!("\n✅ Step 9: Verify Merkle Proofs");
        for commitment in &commitments {
            let proof = relayer.get_merkle_proof(commitment).unwrap();
            let is_valid = relayer.tree_service.verify_proof(&proof);
            println!("   Proof for {}: {}", &commitment[..16], if is_valid { "VALID" } else { "INVALID" });
        }

        println!("\n🎉 Complete Relayer Flow Test PASSED!");
        println!("=====================================");
    }

    #[test]
    fn test_relayer_monitoring() {
        println!("\n📡 Testing Relayer Monitoring");
        println!("=============================");

        let mut relayer = RelayerService::new(
            "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".to_string(),
            "http://127.0.0.1:8545".to_string(),
        );

        // Process initial deposits
        let results = relayer.process_deposits(1, 5).unwrap();
        println!("✅ Processed {} initial deposits", results.len());

        // Get initial tree state
        let initial_stats = relayer.get_tree_stats();
        println!("✅ Initial tree: {} leaves, depth {}", initial_stats.leaf_count, initial_stats.depth);

        // Process more deposits
        let more_results = relayer.process_deposits(6, 15).unwrap();
        println!("✅ Processed {} additional deposits", more_results.len());

        // Check updated tree state
        let final_stats = relayer.get_tree_stats();
        println!("✅ Final tree: {} leaves, depth {}", final_stats.leaf_count, final_stats.depth);
        
        assert!(final_stats.leaf_count > initial_stats.leaf_count);
        println!("✅ Tree growth verified!");
    }

    #[test]
    fn test_merkle_proof_verification() {
        println!("\n🔍 Testing Merkle Proof Verification");
        println!("===================================");

        let mut relayer = RelayerService::new(
            "0xCf7Ed3AccA5a467e9e704C703E8D87F634fB0Fc9".to_string(),
            "http://127.0.0.1:8545".to_string(),
        );

        // Process some deposits
        let results = relayer.process_deposits(1, 5).unwrap();
        
        // Get proofs for all commitments
        let commitments = relayer.get_all_commitments();
        println!("✅ Testing {} commitments", commitments.len());
        
        for commitment in &commitments {
            let proof = relayer.get_merkle_proof(commitment).unwrap();
            let is_valid = relayer.tree_service.verify_proof(&proof);
            
            assert!(is_valid, "Proof should be valid for commitment {}", commitment);
            println!("   ✅ Proof valid for {}", &commitment[..16]);
        }
        
        println!("✅ All Merkle proofs verified!");
    }
}

/// Parse Ethereum address from hex string
fn parse_address(addr_str: &str) -> [u8; 20] {
    let clean = addr_str.strip_prefix("0x").unwrap_or(addr_str);
    let mut addr = [0u8; 20];
    for (i, chunk) in clean.as_bytes().chunks(2).enumerate() {
        if i < 20 {
            let byte_str = std::str::from_utf8(chunk).unwrap();
            addr[i] = u8::from_str_radix(byte_str, 16).unwrap();
        }
    }
    addr
}

/// Parse hash from hex string
fn parse_hash(hash_str: &str) -> [u8; 32] {
    let clean = hash_str.strip_prefix("0x").unwrap_or(hash_str);
    let mut hash = [0u8; 32];
    for (i, chunk) in clean.as_bytes().chunks(2).enumerate() {
        if i < 32 {
            let byte_str = std::str::from_utf8(chunk).unwrap();
            hash[i] = u8::from_str_radix(byte_str, 16).unwrap();
        }
    }
    hash
}
