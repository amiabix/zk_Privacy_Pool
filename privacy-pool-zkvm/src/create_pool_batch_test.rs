use std::fs;
use privacy_pool_zkvm::privacy_pool::PrivacyPool;
use privacy_pool_zkvm::utxo::{UTXOTransaction, User, UTXOInput, UTXOOutput, TransactionType, MerkleProof, UTXO};

#[derive(serde::Serialize, serde::Deserialize)]
struct PoolBatch {
    transactions: Vec<UTXOTransaction>,
    old_merkle_root: [u8; 32],
    old_pool_balance: u64,
    expected_new_merkle_root: [u8; 32],
    expected_new_pool_balance: u64,
}

fn main() {
    println!("Creating pool accounting batch test...");
    
    // Create multiple users
    let mut users = Vec::new();
    for i in 0..3 {
        let public_key = [i as u8; 32];
        let private_key = [i as u8 + 100; 32];
        users.push(User::new(public_key, private_key));
    }
    
    // Create privacy pool and add users
    let mut pool = PrivacyPool::new();
    for user in users {
        pool.add_user(user);
    }
    
    // Create some initial UTXOs in the pool
    let utxo1 = UTXO::new(1000, [1u8; 32], [2u8; 32], [0u8; 32]);
    let utxo2 = UTXO::new(2000, [3u8; 32], [4u8; 32], [1u8; 32]);
    let utxo3 = UTXO::new(1500, [5u8; 32], [6u8; 32], [2u8; 32]);
    
    let index1 = pool.merkle_tree.insert_utxo(utxo1.clone()).unwrap();
    let index2 = pool.merkle_tree.insert_utxo(utxo2.clone()).unwrap();
    let index3 = pool.merkle_tree.insert_utxo(utxo3.clone()).unwrap();
    
    let proof1 = pool.merkle_tree.generate_proof(index1).unwrap();
    let proof2 = pool.merkle_tree.generate_proof(index2).unwrap();
    let proof3 = pool.merkle_tree.generate_proof(index3).unwrap();
    
    // Record initial state
    let old_merkle_root = pool.merkle_tree.root;
    let old_pool_balance = pool.pool_balance;
    
    // Create batch of transactions
    let mut transactions = Vec::new();
    
    // Transaction 1: User 0 withdraws 300
    let mut tx1 = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: utxo1,
            merkle_proof: proof1,
            secret: [0u8; 32],
            signature: [0u8; 64],
        }],
        outputs: vec![UTXOOutput {
            value: 300,
            secret: [7u8; 32],
            nullifier: [8u8; 32],
            recipient: [0u8; 32],
        }],
        fee: 5,
        tx_type: TransactionType::Withdraw { amount: 300, recipient: [0u8; 32] },
    };
    
    // Sign transaction 1
    let message1 = pool.create_transaction_message(&tx1);
    let signature1 = {
        let user = pool.get_user(&[0u8; 32]).unwrap();
        user.sign_transaction(&message1)
    };
    tx1.inputs[0].signature = signature1;
    transactions.push(tx1);
    
    // Transaction 2: User 1 deposits 500
    let mut tx2 = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: utxo2,
            merkle_proof: proof2,
            secret: [1u8; 32],
            signature: [0u8; 64],
        }],
        outputs: vec![UTXOOutput {
            value: 500,
            secret: [9u8; 32],
            nullifier: [10u8; 32],
            recipient: [1u8; 32],
        }],
        fee: 8,
        tx_type: TransactionType::Deposit { amount: 500, recipient: [1u8; 32] },
    };
    
    // Sign transaction 2
    let message2 = pool.create_transaction_message(&tx2);
    let signature2 = {
        let user = pool.get_user(&[1u8; 32]).unwrap();
        user.sign_transaction(&message2)
    };
    tx2.inputs[0].signature = signature2;
    transactions.push(tx2);
    
    // Transaction 3: User 2 transfers 200 to User 0
    let mut tx3 = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: utxo3,
            merkle_proof: proof3,
            secret: [2u8; 32],
            signature: [0u8; 64],
        }],
        outputs: vec![UTXOOutput {
            value: 200,
            secret: [11u8; 32],
            nullifier: [12u8; 32],
            recipient: [0u8; 32],
        }],
        fee: 3,
        tx_type: TransactionType::Transfer { recipient: [0u8; 32] },
    };
    
    // Sign transaction 3
    let message3 = pool.create_transaction_message(&tx3);
    let signature3 = {
        let user = pool.get_user(&[2u8; 32]).unwrap();
        user.sign_transaction(&message3)
    };
    tx3.inputs[0].signature = signature3;
    transactions.push(tx3);
    
    // Process transactions to get expected final state
    for tx in &transactions {
        let _ = pool.process_transaction(tx.clone());
    }
    
    let final_stats = pool.get_pool_stats();
    
    // Create batch data
    let batch = PoolBatch {
        transactions,
        old_merkle_root,
        old_pool_balance,
        expected_new_merkle_root: final_stats.merkle_root,
        expected_new_pool_balance: final_stats.pool_balance,
    };
    
    // Save batch to input.bin
    let serialized = bincode::serialize(&batch).unwrap();
    fs::write("build/pool_batch_input.bin", serialized).unwrap();
    
    println!("=== POOL ACCOUNTING BATCH TEST CREATED ===");
    println!("Total transactions: {}", batch.transactions.len());
    println!("Old pool balance: {}", old_pool_balance);
    println!("Expected new balance: {}", final_stats.pool_balance);
    println!("Saved to: build/pool_batch_input.bin");
    println!("Expected result: SUCCESS (valid batch accounting)");
}
