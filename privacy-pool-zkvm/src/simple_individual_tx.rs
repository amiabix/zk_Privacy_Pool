#![no_main]
ziskos::entrypoint!(main);

use privacy_pool_zkvm::utxo::UTXOTransaction;
use ziskos::{read_input, set_output};
use std::convert::TryInto;

fn main() {
    // Read single transaction from input
    let input: Vec<u8> = read_input();
    let transaction: UTXOTransaction = bincode::deserialize(&input)
        .expect("Failed to deserialize transaction");
    
    // Simple validation without full pool state
    let mut is_valid = true;
    let mut error_code = 0u32;
    
    // 1. Check transaction has inputs or outputs
    if transaction.inputs.is_empty() && transaction.outputs.is_empty() {
        is_valid = false;
        error_code = 1; // Empty transaction
    }
    
    // 2. Check fee is reasonable (not too high)
    if transaction.fee > 1000 {
        is_valid = false;
        error_code = 2; // Fee too high
    }
    
    // 3. Check output values are reasonable
    for output in &transaction.outputs {
        if output.value == 0 || output.value > 1000000 {
            is_valid = false;
            error_code = 3; // Invalid output value
            break;
        }
    }
    
    // 4. Check input values are reasonable
    for input in &transaction.inputs {
        if input.utxo.value == 0 || input.utxo.value > 1000000 {
            is_valid = false;
            error_code = 4; // Invalid input value
            break;
        }
    }
    
    // 5. Basic signature validation (simplified)
    for input in &transaction.inputs {
        if input.signature == [0u8; 64] {
            is_valid = false;
            error_code = 5; // Missing signature
            break;
        }
    }
    
    // Output validation results
    set_output(0, if is_valid { 1 } else { 0 }); // Success flag
    set_output(1, error_code); // Error code
    
    // Output transaction hash for reference
    let tx_hash = compute_transaction_hash(&transaction);
    for i in 0..8 {
        let chunk = u32::from_le_bytes(tx_hash[i*4..(i+1)*4].try_into().unwrap());
        set_output(2 + i, chunk);
    }
    
    // Output basic transaction stats
    set_output(10, transaction.inputs.len() as u32);
    set_output(11, transaction.outputs.len() as u32);
    set_output(12, transaction.fee as u32);
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
