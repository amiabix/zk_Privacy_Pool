#![no_main]
ziskos::entrypoint!(main);

use privacy_pool_zkvm::utxo::UTXOTransaction;
use privacy_pool_zkvm::privacy_pool::PrivacyPool;
use ziskos::{read_input, set_output};
use std::convert::TryInto;

fn main() {
    // Read single transaction from input
    let input: Vec<u8> = read_input();
    let transaction: UTXOTransaction = bincode::deserialize(&input)
        .expect("Failed to deserialize transaction");
    
    // Create a minimal pool state for validation
    let mut pool = PrivacyPool::new();
    
    // Compute transaction hash before processing
    let tx_hash = compute_transaction_hash(&transaction);
    
    // Process the single transaction
    let result = pool.process_transaction(transaction).unwrap_or_else(|e| {
        privacy_pool_zkvm::transaction::TransactionResult::Failure(format!("{:?}", e))
    });
    
    // Output only essential validation results
    match result {
        privacy_pool_zkvm::transaction::TransactionResult::Success => {
            // Transaction is valid - output success flag
            set_output(0, 1); // Success flag
            set_output(1, 0); // Error code (none)
        }
        privacy_pool_zkvm::transaction::TransactionResult::Failure(_) => {
            // Transaction is invalid - output failure flag
            set_output(0, 0); // Failure flag
            set_output(1, 1); // Error code (validation failed)
        }
    }
    
    // Output transaction hash for reference
    for i in 0..8 {
        let chunk = u32::from_le_bytes(tx_hash[i*4..(i+1)*4].try_into().unwrap());
        set_output(2 + i, chunk);
    }
}

fn compute_transaction_hash(tx: &UTXOTransaction) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    
    // Hash all input commitments
    for input in &tx.inputs {
        hasher.update(&input.utxo.commitment);
    }
    
    // Hash all output values and recipients
    for output in &tx.outputs {
        hasher.update(&output.value.to_le_bytes());
        hasher.update(&output.recipient);
    }
    
    // Hash fee
    hasher.update(&tx.fee.to_le_bytes());
    
    hasher.finalize().into()
}
