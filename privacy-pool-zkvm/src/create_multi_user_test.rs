use std::fs;
use privacy_pool_zkvm::privacy_pool::PrivacyPool;
use privacy_pool_zkvm::utxo::{UTXOTransaction, User, UTXOInput, UTXOOutput, TransactionType, MerkleProof, UTXO};

fn main() {
    println!("Creating multi-user privacy pool test with 5 users...");
    
    // Create 5 users with different balances
    let mut users = Vec::new();
    
    for i in 0..5 {
        let public_key = [i as u8; 32];
        let private_key = [i as u8 + 100; 32];
        let mut user = User::new(public_key, private_key);
        
        // Give each user different initial balances
        let initial_balance = 1000 + (i as u64 * 500);
        user.balance = initial_balance;
        
        // Create initial UTXOs for each user (these will be external UTXOs)
        for j in 0..3 {
            let value = initial_balance / 3;
            let secret = [i as u8 + j as u8; 32];
            let nullifier = [i as u8 + j as u8 + 50; 32];
            let utxo = UTXO::new(value, secret, nullifier, public_key);
            user.add_utxo(utxo);
        }
        
        users.push(user);
        println!("User {}: Balance = {}, UTXOs = {}", i, initial_balance, users[i].utxos.len());
    }
    
    // Create privacy pool and add users
    let mut pool = PrivacyPool::new();
    for user in users {
        pool.add_user(user);
    }
    
    println!("\n=== PRIVACY POOL INITIALIZED ===");
    let stats = pool.get_pool_stats();
    println!("Total users: {}", stats.total_users);
    println!("Pool balance: {}", stats.pool_balance);
    println!("Merkle root: {:02x?}", stats.merkle_root);
    
    // Create test transactions - simplified approach
    let mut transactions = Vec::new();
    
    // Transaction 1: User 0 deposits 500 (external UTXO -> pool UTXO)
    println!("\n=== TRANSACTION 1: User 0 deposits 500 ===");
    let user0 = pool.get_user(&[0u8; 32]).unwrap();
    let external_utxo = user0.utxos[0].clone();
    
    // Create a mock Merkle proof for external UTXO (simplified)
    let mock_proof = MerkleProof {
        siblings: vec![[0u8; 32]],
        path: vec![true],
        root: [0u8; 32],
        leaf_index: 0,
    };
    
    let mut deposit_tx = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: external_utxo,
            merkle_proof: mock_proof,
            secret: [0u8; 32],
            signature: [0u8; 64], // Will be filled below
        }],
        outputs: vec![UTXOOutput {
            value: 500,
            secret: [1u8; 32],
            nullifier: [2u8; 32],
            recipient: [0u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Deposit { amount: 500, recipient: [0u8; 32] },
    };
    
    // Sign the transaction
    let message = pool.create_transaction_message(&deposit_tx);
    let signature = {
        let user0 = pool.get_user(&[0u8; 32]).unwrap();
        user0.sign_transaction(&message)
    };
    deposit_tx.inputs[0].signature = signature;
    
    transactions.push(deposit_tx);
    
    // Transaction 2: User 1 deposits 300
    println!("=== TRANSACTION 2: User 1 deposits 300 ===");
    let user1 = pool.get_user(&[1u8; 32]).unwrap();
    let external_utxo = user1.utxos[0].clone();
    
    let mock_proof = MerkleProof {
        siblings: vec![[0u8; 32]],
        path: vec![true],
        root: [0u8; 32],
        leaf_index: 0,
    };
    
    let mut deposit_tx2 = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: external_utxo,
            merkle_proof: mock_proof,
            secret: [1u8; 32],
            signature: [0u8; 64], // Will be filled below
        }],
        outputs: vec![UTXOOutput {
            value: 300,
            secret: [3u8; 32],
            nullifier: [4u8; 32],
            recipient: [1u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Deposit { amount: 300, recipient: [1u8; 32] },
    };
    
    // Sign transaction 2
    let message2 = pool.create_transaction_message(&deposit_tx2);
    let signature2 = {
        let user1 = pool.get_user(&[1u8; 32]).unwrap();
        user1.sign_transaction(&message2)
    };
    deposit_tx2.inputs[0].signature = signature2;
    transactions.push(deposit_tx2);
    
    // Transaction 3: User 2 deposits 800
    println!("=== TRANSACTION 3: User 2 deposits 800 ===");
    let user2 = pool.get_user(&[2u8; 32]).unwrap();
    let external_utxo = user2.utxos[0].clone();
    
    let mock_proof = MerkleProof {
        siblings: vec![[0u8; 32]],
        path: vec![true],
        root: [0u8; 32],
        leaf_index: 0,
    };
    
    let mut deposit_tx3 = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: external_utxo,
            merkle_proof: mock_proof,
            secret: [2u8; 32],
            signature: [0u8; 64], // Will be filled below
        }],
        outputs: vec![UTXOOutput {
            value: 800,
            secret: [5u8; 32],
            nullifier: [6u8; 32],
            recipient: [2u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Deposit { amount: 800, recipient: [2u8; 32] },
    };
    
    // Sign transaction 3
    let message3 = pool.create_transaction_message(&deposit_tx3);
    let signature3 = {
        let user2 = pool.get_user(&[2u8; 32]).unwrap();
        user2.sign_transaction(&message3)
    };
    deposit_tx3.inputs[0].signature = signature3;
    transactions.push(deposit_tx3);
    
    // Transaction 4: User 0 withdraws 200 (pool UTXO -> external)
    println!("=== TRANSACTION 4: User 0 withdraws 200 ===");
    // For withdrawal, we need a UTXO that's already in the pool
    // This is a simplified test - in reality, this would be a UTXO from a previous deposit
    let pool_utxo = UTXO::new(500, [1u8; 32], [2u8; 32], [0u8; 32]);
    let utxo_index = pool.merkle_tree.insert_utxo(pool_utxo.clone()).unwrap();
    let merkle_proof = pool.merkle_tree.generate_proof(utxo_index as usize).unwrap();
    
    let mut withdraw_tx = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: pool_utxo,
            merkle_proof,
            secret: [0u8; 32],
            signature: [0u8; 64], // Will be filled below
        }],
        outputs: vec![UTXOOutput {
            value: 200,
            secret: [7u8; 32],
            nullifier: [8u8; 32],
            recipient: [0u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Withdraw { amount: 200, recipient: [0u8; 32] },
    };
    
    // Sign transaction 4
    let message4 = pool.create_transaction_message(&withdraw_tx);
    let signature4 = {
        let user0 = pool.get_user(&[0u8; 32]).unwrap();
        user0.sign_transaction(&message4)
    };
    withdraw_tx.inputs[0].signature = signature4;
    transactions.push(withdraw_tx);
    
    // Transaction 5: User 3 transfers 400 to User 4 (pool UTXO -> pool UTXO)
    println!("=== TRANSACTION 5: User 3 transfers 400 to User 4 ===");
    let pool_utxo2 = UTXO::new(400, [9u8; 32], [10u8; 32], [3u8; 32]);
    let utxo_index2 = pool.merkle_tree.insert_utxo(pool_utxo2.clone()).unwrap();
    let merkle_proof2 = pool.merkle_tree.generate_proof(utxo_index2 as usize).unwrap();
    
    let mut transfer_tx = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: pool_utxo2,
            merkle_proof: merkle_proof2,
            secret: [3u8; 32],
            signature: [0u8; 64], // Will be filled below
        }],
        outputs: vec![UTXOOutput {
            value: 400,
            secret: [11u8; 32],
            nullifier: [12u8; 32],
            recipient: [4u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Transfer { recipient: [4u8; 32] },
    };
    
    // Sign transaction 5
    let message5 = pool.create_transaction_message(&transfer_tx);
    let signature5 = {
        let user3 = pool.get_user(&[3u8; 32]).unwrap();
        user3.sign_transaction(&message5)
    };
    transfer_tx.inputs[0].signature = signature5;
    transactions.push(transfer_tx);
    
    // Save all transactions to input.bin
    let serialized = bincode::serialize(&transactions).unwrap();
    fs::write("build/input.bin", serialized).unwrap();
    
    println!("\n=== TEST DATA CREATED ===");
    println!("Total transactions: {}", transactions.len());
    println!("Saved to: build/input.bin");
    println!("\nTransaction summary:");
    println!("1. User 0 deposits 500");
    println!("2. User 1 deposits 300");
    println!("3. User 2 deposits 800");
    println!("4. User 0 withdraws 200");
    println!("5. User 3 transfers 400 to User 4");
    
    println!("\nExpected final state:");
    println!("- Pool should have 3 deposits (500 + 300 + 800 = 1600)");
    println!("- Pool should have 1 withdrawal (200)");
    println!("- Net pool balance: 1400");
    println!("- All users should have updated balances");
    println!("- All nullifiers should be marked as used");
}