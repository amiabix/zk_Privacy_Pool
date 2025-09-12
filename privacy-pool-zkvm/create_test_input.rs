use std::fs;
use privacy_pool_zkvm::transaction::{PrivacyTransaction, TransactionType, PrivateInputs, MerkleProof, ZkProof};

fn main() {
    // Create a test deposit transaction
    let test_tx = PrivacyTransaction {
        tx_type: TransactionType::Deposit { 
            amount: 1000, 
            recipient: [1u8; 32] 
        },
        nullifier: [2u8; 32],
        out_commit: [3u8; 32],
        merkle_proof: MerkleProof {
            siblings: vec![[4u8; 32]],
            path: vec![true],
            root: [5u8; 32],
        },
        zk_proof: ZkProof {
            proof_data: vec![6u8; 100],
            public_inputs: vec![1000, 1],
        },
        private_inputs: PrivateInputs {
            value: 1000,
            secret: [7u8; 32],
            nullifier: [2u8; 32],
            new_secret: None,
            new_nullifier: None,
        },
    };

    // Serialize and save to input.bin
    let serialized = bincode::serialize(&test_tx).unwrap();
    fs::write("build/input.bin", serialized).unwrap();
    
    println!("Test input created: build/input.bin");
    println!("Transaction type: Deposit");
    println!("Amount: 1000");
}
