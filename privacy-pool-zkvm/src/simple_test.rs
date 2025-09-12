use std::fs;
use privacy_pool_zkvm::privacy_pool::PrivacyPool;
use privacy_pool_zkvm::utxo::{UTXOTransaction, UTXOInput, UTXOOutput, TransactionType, MerkleProof, UTXO};

fn main() {
    println!("Creating simple privacy pool test...");
    
    // Create a simple test with just deposits (no Merkle proof validation needed)
    let mut transactions = Vec::new();
    
    // Transaction 1: Simple deposit
    println!("=== TRANSACTION 1: Deposit 500 ===");
    let deposit_tx = UTXOTransaction {
        inputs: vec![], // No inputs for deposits
        outputs: vec![UTXOOutput {
            value: 500,
            secret: [1u8; 32],
            nullifier: [2u8; 32],
            recipient: [0u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Deposit { amount: 500, recipient: [0u8; 32] },
    };
    transactions.push(deposit_tx);
    
    // Transaction 2: Another deposit
    println!("=== TRANSACTION 2: Deposit 300 ===");
    let deposit_tx2 = UTXOTransaction {
        inputs: vec![],
        outputs: vec![UTXOOutput {
            value: 300,
            secret: [3u8; 32],
            nullifier: [4u8; 32],
            recipient: [1u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Deposit { amount: 300, recipient: [1u8; 32] },
    };
    transactions.push(deposit_tx2);
    
    // Transaction 3: Another deposit
    println!("=== TRANSACTION 3: Deposit 800 ===");
    let deposit_tx3 = UTXOTransaction {
        inputs: vec![],
        outputs: vec![UTXOOutput {
            value: 800,
            secret: [5u8; 32],
            nullifier: [6u8; 32],
            recipient: [2u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Deposit { amount: 800, recipient: [2u8; 32] },
    };
    transactions.push(deposit_tx3);
    
    // Save all transactions to input.bin
    let serialized = bincode::serialize(&transactions).unwrap();
    fs::write("build/input.bin", serialized).unwrap();
    
    println!("\n=== SIMPLE TEST DATA CREATED ===");
    println!("Total transactions: {}", transactions.len());
    println!("Saved to: build/input.bin");
    println!("\nTransaction summary:");
    println!("1. Deposit 500");
    println!("2. Deposit 300");
    println!("3. Deposit 800");
    println!("\nExpected final state:");
    println!("- Pool balance: 1600");
    println!("- 3 UTXOs in Merkle tree");
    println!("- All transactions should succeed");
}
