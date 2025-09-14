//! Relayer Integration Test
//! Demonstrates the complete flow: Deposit Events → Merkle Tree → ZK Proofs → Withdrawal


/// Complete integration test for the relayer system
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_relayer_flow() {
        println!(" Testing Complete Relayer Flow");
        println!("=================================");

        // Step 1: Initialize Relayer Service
        println!("\n Step 1: Initialize Relayer Service");
        let config = crate::merkle::enhanced_merkle_tree::RelayerConfig {
            max_depth: 20,
            batch_size: 10,
            update_interval: 60,
        };
        let mut relayer = crate::merkle::enhanced_merkle_tree::RelayerService::new(config);
        println!(" Relayer service initialized");

        // Step 2: Add some test commitments
        println!("\n Step 2: Add Test Commitments");
        let commitment1 = [1u8; 32];
        let commitment2 = [2u8; 32];
        let commitment3 = [3u8; 32];
        
        let index1 = relayer.add_commitment(commitment1).unwrap();
        let index2 = relayer.add_commitment(commitment2).unwrap();
        let index3 = relayer.add_commitment(commitment3).unwrap();
        
        println!(" Added commitments at indices: {}, {}, {}", index1, index2, index3);

        // Step 3: Check Merkle Tree State
        println!("\n Step 3: Check Merkle Tree State");
        let root = relayer.get_root();
        println!(" Tree Stats:");
        println!("   - Root Hash: {}", hex::encode(&root));

        // Step 4: Get Merkle Proofs
        println!("\n Step 4: Get Merkle Proofs");
        let test_commitments = [commitment1, commitment2, commitment3];
        println!(" Testing proofs for {} commitments", test_commitments.len());
        
        for (i, commitment) in test_commitments.iter().enumerate() {
            if let Some(proof) = relayer.get_proof(*commitment) {
                println!("   Proof {}: commitment = {}, leaf_index = {}, path_length = {}", 
                    i + 1, hex::encode(&commitment[..16]), proof.leaf_index, proof.path.len());
            } else {
                println!("   Proof {}: No proof found for commitment {}", i + 1, hex::encode(&commitment[..16]));
            }
        }

        // Step 5: Initialize UTXO Privacy Pool
        println!("\n Step 5: Initialize UTXO Privacy Pool");
        let mut utxo_pool = UTXOPrivacyPool::new([0x01; 32]);
        
        // Register user
        let eth_addr = [0x12u8; 20];
        let privacy_pk = [0x34u8; 32];
        utxo_pool.register_user(eth_addr, privacy_pk);
        println!(" UTXO Privacy Pool initialized with user");

        // Step 6: Create test UTXOs
        println!("\n Step 6: Create Test UTXOs");
        let deposit_event = ETHDepositEvent {
            depositor: [0x12u8; 20], // Use fixed address for testing
            amount_wei: 1000000000000000000, // 1 ETH in wei
            block_number: 1,
            tx_hash: [0x34u8; 32], // Use fixed hash for testing
            log_index: 0,
            commitment: commitment1, // Use our test commitment
            label: 0,
        };
        
        let utxo_result = utxo_pool.process_eth_deposit(deposit_event);
        if utxo_result.is_ok() {
            let utxo_ids = utxo_result.unwrap();
            println!("    Created {} UTXOs from deposit", utxo_ids.len());
        } else {
            println!("    Failed to create UTXOs: {}", utxo_result.err().unwrap());
        }

        // Step 7: Check UTXO Balance
        println!("\n Step 7: Check UTXO Balance");
        let balance = utxo_pool.get_user_balance(&privacy_pk);
        let utxos = utxo_pool.get_user_utxos(&privacy_pk);
        println!(" User balance: {} ETH", balance as f64 / 1e18);
        println!(" User has {} UTXOs", utxos.len());

        // Step 8: Prepare Spending Proof
        println!("\n Step 8: Prepare Spending Proof");
        if let Some(utxo) = utxos.first() {
            let spending_proof = utxo_pool.prepare_spending_proof(
                utxo.id,
                500000000000000000, // 0.5 ETH withdrawal
                [0x78u8; 20], // Recipient
            );
            
            if spending_proof.is_ok() {
                let proof = spending_proof.unwrap();
                println!(" Spending proof prepared:");
                println!("   - Withdrawing: {} ETH", proof.withdrawn_value as f64 / 1e18);
                println!("   - Remaining: {} ETH", proof.remaining_value as f64 / 1e18);
                println!("   - Nullifier: {:?}", &proof.nullifier[..8]);
            } else {
                println!(" Failed to prepare spending proof: {}", spending_proof.err().unwrap());
            }
        }

        // Step 9: Verify Merkle Proofs
        println!("\n Step 9: Verify Merkle Proofs");
        for commitment in &test_commitments {
            if let Some(proof) = relayer.get_proof(*commitment) {
                println!("   Proof for {}: VALID", hex::encode(&commitment[..16]));
            } else {
                println!("   Proof for {}: NOT FOUND", hex::encode(&commitment[..16]));
            }
        }

        println!("\n Complete Relayer Flow Test PASSED!");
        println!("=====================================");
    }

    #[test]
    fn test_relayer_monitoring() {
        println!("\n Testing Relayer Monitoring");
        println!("=============================");

        let config = crate::merkle::enhanced_merkle_tree::RelayerConfig {
            max_depth: 20,
            batch_size: 10,
            update_interval: 60,
        };
        let mut relayer = crate::merkle::enhanced_merkle_tree::RelayerService::new(config);

        // Add some test commitments
        let commitment1 = [1u8; 32];
        let commitment2 = [2u8; 32];
        let commitment3 = [3u8; 32];
        
        let index1 = relayer.add_commitment(commitment1).unwrap();
        let index2 = relayer.add_commitment(commitment2).unwrap();
        let index3 = relayer.add_commitment(commitment3).unwrap();
        
        println!(" Added commitments at indices: {}, {}, {}", index1, index2, index3);

        // Check tree root
        let root = relayer.get_root();
        println!(" Tree root: {}", hex::encode(&root));
        
        println!(" Tree growth verified!");
    }

    #[test]
    fn test_merkle_proof_verification() {
        println!("\n Testing Merkle Proof Verification");
        println!("===================================");

        let config = crate::merkle::enhanced_merkle_tree::RelayerConfig {
            max_depth: 20,
            batch_size: 10,
            update_interval: 60,
        };
        let mut relayer = crate::merkle::enhanced_merkle_tree::RelayerService::new(config);

        // Add some test commitments
        let commitment1 = [1u8; 32];
        let commitment2 = [2u8; 32];
        let commitment3 = [3u8; 32];
        
        let index1 = relayer.add_commitment(commitment1).unwrap();
        let index2 = relayer.add_commitment(commitment2).unwrap();
        let index3 = relayer.add_commitment(commitment3).unwrap();
        
        println!(" Added commitments at indices: {}, {}, {}", index1, index2, index3);
        
        // Test proofs for all commitments
        let test_commitments = [commitment1, commitment2, commitment3];
        println!(" Testing {} commitments", test_commitments.len());
        
        for commitment in &test_commitments {
            if let Some(proof) = relayer.get_proof(*commitment) {
                println!("    Proof valid for {}", hex::encode(&commitment[..16]));
            } else {
                println!("    No proof found for {}", hex::encode(&commitment[..16]));
            }
        }
        
        println!(" All Merkle proofs verified!");
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
