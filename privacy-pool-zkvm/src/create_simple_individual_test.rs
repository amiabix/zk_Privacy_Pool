use std::fs;
use privacy_pool_zkvm::utxo::{UTXOTransaction, UTXOInput, UTXOOutput, TransactionType, UTXO};

fn main() {
    println!("Creating simple individual transaction test...");
    
    // Create a simple transaction without complex Merkle proofs
    let transaction = UTXOTransaction {
        inputs: vec![UTXOInput {
            utxo: UTXO {
                value: 1000,
                commitment: [1u8; 32],
                nullifier: [2u8; 32],
                owner: [3u8; 32],
                index: 0,
                secret: [4u8; 32],
            },
            merkle_proof: privacy_pool_zkvm::utxo::MerkleProof {
                siblings: vec![[0u8; 32]],
                path: vec![true],
                root: [0u8; 32],
                leaf_index: 0,
            },
            secret: [4u8; 32],
            signature: [5u8; 64], // Non-zero signature
        }],
        outputs: vec![UTXOOutput {
            value: 500,
            secret: [6u8; 32],
            nullifier: [7u8; 32],
            recipient: [8u8; 32],
        }],
        fee: 10,
        tx_type: TransactionType::Withdraw { amount: 500, recipient: [8u8; 32] },
    };
    
    // Save transaction to input.bin
    let serialized = bincode::serialize(&transaction).unwrap();
    fs::write("build/simple_individual_input.bin", serialized).unwrap();
    
    println!("=== SIMPLE INDIVIDUAL TRANSACTION TEST CREATED ===");
    println!("Transaction type: Withdrawal");
    println!("Amount: 500 tokens");
    println!("Fee: 10 tokens");
    println!("Saved to: build/simple_individual_input.bin");
    println!("Expected result: SUCCESS (valid transaction)");
}
