#![no_main]
ziskos::entrypoint!(main);

use ziskos::{read_input, set_output};

#[derive(serde::Serialize, serde::Deserialize)]
struct UltraSimplePoolBatch {
    transactions: Vec<UltraSimpleTransaction>,
    old_balance: u64,
    expected_new_balance: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct UltraSimpleTransaction {
    amount: u64,
    fee: u64,
    recipient: Vec<u8>, // Vec but we'll ensure it's always 32 bytes
    signature: Vec<u8>, // Vec but we'll ensure it's always 64 bytes
    tx_type: u8, // 0=deposit, 1=withdraw, 2=transfer (instead of String)
}

fn main() {
    // Read batch of transactions and expected state
    let input: Vec<u8> = read_input();
    let batch: UltraSimplePoolBatch = bincode::deserialize(&input)
        .expect("Failed to deserialize batch");
    
    // Process all transactions in batch
    let mut successful_txs = 0;
    let mut total_fees = 0u64;
    let mut current_balance = batch.old_balance;
    let total_txs = batch.transactions.len();
    
    for transaction in &batch.transactions {
        let fee = transaction.fee;
        let amount = transaction.amount;
        
        // Simple validation
        let mut is_valid = true;
        
        // Check amount is reasonable
        if amount == 0 || amount > 1000000 {
            is_valid = false;
        }
        
        // Check fee is reasonable
        if fee > 1000 {
            is_valid = false;
        }
        
        // Check signature is not all zeros
        if transaction.signature.len() != 64 || transaction.signature.iter().all(|&x| x == 0) {
            is_valid = false;
        }
        
        // Check recipient is not all zeros
        if transaction.recipient.len() != 32 || transaction.recipient.iter().all(|&x| x == 0) {
            is_valid = false;
        }
        
        if is_valid {
            successful_txs += 1;
            total_fees += fee;
            
            // Update balance based on transaction type
            match transaction.tx_type {
                0 => { // deposit
                    current_balance += amount;
                }
                1 => { // withdraw
                    if current_balance >= amount {
                        current_balance -= amount;
                    } else {
                        // Insufficient balance - transaction fails
                        successful_txs -= 1;
                        total_fees -= fee;
                    }
                }
                2 => { // transfer
                    // Transfer doesn't change pool balance
                }
                _ => { // Unknown transaction type - transaction fails
                    successful_txs -= 1;
                    total_fees -= fee;
                }
            }
        }
    }
    
    // Verify final state matches expected
    let state_correct = current_balance == batch.expected_new_balance;
    
    // Output validation results
    set_output(0, if state_correct { 1 } else { 0 }); // State validation
    set_output(1, successful_txs as u32); // Successful transactions
    set_output(2, total_txs as u32); // Total transactions
    set_output(3, total_fees as u32); // Total fees collected
    set_output(4, (total_fees >> 32) as u32); // Total fees (high 32 bits)
    
    // Output final balance (2 x u32 = 8 bytes)
    set_output(5, current_balance as u32);
    set_output(6, (current_balance >> 32) as u32);
    
    // Output batch hash for reference
    let batch_hash = compute_batch_hash(&batch);
    for i in 0..8 {
        let chunk = u32::from_le_bytes(batch_hash[i*4..(i+1)*4].try_into().unwrap());
        set_output(7 + i, chunk);
    }
}

fn compute_batch_hash(batch: &UltraSimplePoolBatch) -> [u8; 32] {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    
    hasher.update(&batch.old_balance.to_le_bytes());
    hasher.update(&batch.expected_new_balance.to_le_bytes());
    
    for transaction in &batch.transactions {
        hasher.update(&transaction.amount.to_le_bytes());
        hasher.update(&transaction.fee.to_le_bytes());
        hasher.update(&transaction.recipient);
        hasher.update(&transaction.signature);
        hasher.update(&[transaction.tx_type]); // Single byte instead of string
    }
    
    hasher.finalize().into()
}
