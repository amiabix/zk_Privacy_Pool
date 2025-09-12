//! Integration test for complete ETH → UTXO conversion flow
//! Demonstrates the nuanced 6-step process

use crate::utxo_privacy_pool::{UTXOPrivacyPool, ETHDepositEvent, SpendingProof};
use crate::utxo_indexing::UTXOId;

/// Test the complete ETH → UTXO conversion flow
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complete_eth_to_utxo_flow() {
        println!("🚀 Testing Complete ETH → UTXO Conversion Flow");
        println!("================================================");

        // Step 1: Initialize Privacy Pool
        println!("\n📋 Step 1: Initialize Privacy Pool");
        let mut pool = UTXOPrivacyPool::new([0x01; 32]);
        
        // Register user
        let eth_addr = [0x12u8; 20];
        let privacy_pk = [0x34u8; 32];
        pool.register_user(eth_addr, privacy_pk);
        println!("✅ User registered: ETH addr = {:?}, Privacy PK = {:?}", eth_addr, privacy_pk);

        // Step 2: Create ETH Deposit Event (from smart contract)
        println!("\n💰 Step 2: ETH Deposit Event");
        let deposit_event = ETHDepositEvent {
            depositor: eth_addr,
            amount_wei: 2000000000000000000, // 2 ETH
            block_number: 1000,
            tx_hash: [0x56u8; 32],
            log_index: 0,
            commitment: [0u8; 32],
            label: 0,
        };
        println!("✅ Deposit event created: {} ETH", deposit_event.amount_wei as f64 / 1e18);

        // Step 3: Process Deposit → Create UTXOs
        println!("\n🔄 Step 3: Process Deposit → Create UTXOs");
        let result = pool.process_eth_deposit(deposit_event);
        assert!(result.is_ok());
        
        let utxo_ids = result.unwrap();
        println!("✅ Created {} UTXOs from deposit", utxo_ids.len());
        
        for (i, utxo_id) in utxo_ids.iter().enumerate() {
            println!("   UTXO {}: tx_hash = {:?}, output_index = {}", 
                i + 1, utxo_id.tx_hash, utxo_id.output_index);
        }

        // Step 4: Verify UTXOs are in Merkle Tree
        println!("\n🌳 Step 4: UTXOs in Merkle Tree");
        let utxos = pool.get_user_utxos(&privacy_pk);
        println!("✅ {} UTXOs found in Merkle tree", utxos.len());
        
        for (i, utxo) in utxos.iter().enumerate() {
            println!("   UTXO {}: value = {} ETH, commitment = {:?}", 
                i + 1, utxo.value as f64 / 1e18, utxo.address);
        }

        // Step 5: Check User Balance
        println!("\n💳 Step 5: User Balance");
        let balance = pool.get_user_balance(&privacy_pk);
        println!("✅ User balance: {} ETH", balance as f64 / 1e18);
        assert_eq!(balance, 2000000000000000000);

        // Step 6: Prepare Spending Proof
        println!("\n🔐 Step 6: Prepare Spending Proof");
        if let Some(utxo_id) = utxo_ids.first() {
            let spending_proof = pool.prepare_spending_proof(
                *utxo_id,
                1000000000000000000, // 1 ETH withdrawal
                [0x78u8; 20], // Recipient address
            );
            
            assert!(spending_proof.is_ok());
            let proof = spending_proof.unwrap();
            println!("✅ Spending proof prepared:");
            println!("   - Existing value: {} ETH", proof.existing_value as f64 / 1e18);
            println!("   - Withdrawn value: {} ETH", proof.withdrawn_value as f64 / 1e18);
            println!("   - Remaining value: {} ETH", proof.remaining_value as f64 / 1e18);
            println!("   - Nullifier: {:?}", proof.nullifier);
            println!("   - New nullifier: {:?}", proof.new_nullifier);

            // Step 7: Submit Withdrawal
            println!("\n💸 Step 7: Submit Withdrawal");
            let withdrawal_result = pool.submit_withdrawal(proof);
            assert!(withdrawal_result.is_ok());
            
            let tx_hash = withdrawal_result.unwrap();
            println!("✅ Withdrawal submitted: tx_hash = {:?}", tx_hash);

            // Verify final balance
            let final_balance = pool.get_user_balance(&privacy_pk);
            println!("✅ Final balance: {} ETH", final_balance as f64 / 1e18);
        }

        println!("\n🎉 Complete ETH → UTXO Conversion Flow Test PASSED!");
        println!("================================================");
    }

    #[test]
    fn test_utxo_denominations() {
        println!("\n🪙 Testing UTXO Denominations");
        println!("=============================");

        let mut pool = UTXOPrivacyPool::new([0x02; 32]);
        
        // Register user
        let eth_addr = [0x12u8; 20];
        let privacy_pk = [0x34u8; 32];
        pool.register_user(eth_addr, privacy_pk);

        // Test different deposit amounts
        let test_amounts = [
            1000000000000000000,  // 1 ETH
            1500000000000000000,  // 1.5 ETH
            2500000000000000000,  // 2.5 ETH
            5000000000000000000,  // 5 ETH
        ];

        for amount in test_amounts {
            let deposit_event = ETHDepositEvent {
                depositor: eth_addr,
                amount_wei: amount,
                block_number: 1000,
                tx_hash: [0x56u8; 32],
                log_index: 0,
                commitment: [0u8; 32],
                label: 0,
            };

            let result = pool.process_eth_deposit(deposit_event);
            assert!(result.is_ok());
            
            let utxo_ids = result.unwrap();
            let utxos = pool.get_user_utxos(&privacy_pk);
            
            println!("✅ {} ETH → {} UTXOs", amount as f64 / 1e18, utxo_ids.len());
            
            for utxo in utxos {
                println!("   - {} ETH UTXO", utxo.value as f64 / 1e18);
            }
        }
    }

    #[test]
    fn test_multiple_users() {
        println!("\n👥 Testing Multiple Users");
        println!("========================");

        let mut pool = UTXOPrivacyPool::new([0x03; 32]);
        
        // Register multiple users
        let users = [
            ([0x11u8; 20], [0x21u8; 32], "Alice"),
            ([0x12u8; 20], [0x22u8; 32], "Bob"),
            ([0x13u8; 20], [0x23u8; 32], "Charlie"),
        ];

        for (eth_addr, privacy_pk, name) in users {
            pool.register_user(eth_addr, privacy_pk);
            println!("✅ Registered user: {}", name);
        }

        // Each user deposits different amounts
        let deposits = [
            (users[0].0, 1000000000000000000, "Alice deposits 1 ETH"),
            (users[1].0, 2000000000000000000, "Bob deposits 2 ETH"),
            (users[2].0, 3000000000000000000, "Charlie deposits 3 ETH"),
        ];

        for (eth_addr, amount, description) in deposits {
            let deposit_event = ETHDepositEvent {
                depositor: eth_addr,
                amount_wei: amount,
                block_number: 1000,
                tx_hash: [0x56u8; 32],
                log_index: 0,
                commitment: [0u8; 32],
                label: 0,
            };

            let result = pool.process_eth_deposit(deposit_event);
            assert!(result.is_ok());
            
            println!("✅ {}", description);
        }

        // Check each user's balance
        for (eth_addr, privacy_pk, name) in users {
            let balance = pool.get_user_balance(&privacy_pk);
            let utxos = pool.get_user_utxos(&privacy_pk);
            
            println!("✅ {}: {} ETH, {} UTXOs", 
                name, balance as f64 / 1e18, utxos.len());
        }

        println!("✅ Multiple users test completed!");
    }
}
