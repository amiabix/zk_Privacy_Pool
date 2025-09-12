use std::fs;
use privacy_pool_zkvm::privacy_pool::PrivacyPool;
use privacy_pool_zkvm::utxo::{UTXOTransaction, User, UTXOInput, UTXOOutput, TransactionType, MerkleProof, UTXO};

fn main() {
    println!("Creating individual transaction test...");
    
    // Create a user with some UTXOs
    let mut user = User::new([1u8; 32], [101u8; 32]);
    
    // Create a privacy pool and add the user
    let mut pool = PrivacyPool::new();
    pool.add_user(user);
    
    // Create a UTXO for the user to spend
    let utxo = UTXO::new(1000, [1u8; 32], [2u8; 32], [1u8; 32]);
    let utxo_index = pool.merkle_tree.insert_utxo(utxo.clone()).unwrap();
    let merkle_proof = pool.merkle_tree.generate_proof(utxo_index).unwrap();
    
    // Create a withdrawal transaction
    let mut transaction = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: utxo,
            merkle_proof: merkle_proof,
            secret: [1u8; 32],
            signature: [0u8; 64], // Will be filled below
        }],
        outputs: vec![UTXOOutput {
            value: 500,
            secret: [3u8; 32],
            nullifier: [4u8; 32],
            recipient: [2u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Withdraw { amount: 500, recipient: [2u8; 32] },
    };
    
    // Sign the transaction
    let message = pool.create_transaction_message(&transaction);
    let signature = {
        let user = pool.get_user(&[1u8; 32]).unwrap();
        user.sign_transaction(&message)
    };
    transaction.inputs[0].signature = signature;
    
    // Save transaction to input.bin
    let serialized = bincode::serialize(&transaction).unwrap();
    fs::write("build/individual_input.bin", serialized).unwrap();
    
    println!("=== INDIVIDUAL TRANSACTION TEST CREATED ===");
    println!("Transaction type: Withdrawal");
    println!("Amount: 500 tokens");
    println!("Fee: 10 tokens");
    println!("Saved to: build/individual_input.bin");
    println!("Expected result: SUCCESS (valid transaction)");
}
