#![no_main]
ziskos::entrypoint!(main);

use ziskos::{read_input, set_output};

#[derive(serde::Serialize, serde::Deserialize)]
struct SimpleTransaction {
    amount: u64,
    fee: u64,
    recipient: Vec<u8>,
    signature: Vec<u8>,
}

fn main() {
    // Read simple transaction from input
    let input: Vec<u8> = read_input();
    let transaction: SimpleTransaction = bincode::deserialize(&input)
        .expect("Failed to deserialize transaction");
    
    // Simple validation
    let mut is_valid = true;
    let mut error_code = 0u32;
    
    // 1. Check amount is reasonable
    if transaction.amount == 0 || transaction.amount > 1000000 {
        is_valid = false;
        error_code = 1; // Invalid amount
    }
    
    // 2. Check fee is reasonable
    if transaction.fee > 1000 {
        is_valid = false;
        error_code = 2; // Fee too high
    }
    
    // 3. Check signature is not all zeros
    if transaction.signature.len() != 64 || transaction.signature.iter().all(|&x| x == 0) {
        is_valid = false;
        error_code = 3; // Missing signature
    }
    
    // 4. Check recipient is not all zeros
    if transaction.recipient.len() != 32 || transaction.recipient.iter().all(|&x| x == 0) {
        is_valid = false;
        error_code = 4; // Invalid recipient
    }
    
    // Output validation results
    set_output(0, if is_valid { 1 } else { 0 }); // Success flag
    set_output(1, error_code); // Error code
    set_output(2, transaction.amount as u32); // Amount (low 32 bits)
    set_output(3, (transaction.amount >> 32) as u32); // Amount (high 32 bits)
    set_output(4, transaction.fee as u32); // Fee
    
    // Output transaction hash
    let tx_hash = compute_transaction_hash(&transaction);
    for i in 0..8 {
        let chunk = u32::from_le_bytes(tx_hash[i*4..(i+1)*4].try_into().unwrap());
        set_output(5 + i, chunk);
    }
}

fn compute_transaction_hash(tx: &SimpleTransaction) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    
    hasher.update(&tx.amount.to_le_bytes());
    hasher.update(&tx.fee.to_le_bytes());
    hasher.update(&tx.recipient);
    hasher.update(&tx.signature);
    
    hasher.finalize().into()
}
