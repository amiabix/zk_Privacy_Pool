#![no_main]
ziskos::entrypoint!(main);

use privacy_pool_zkvm::utxo::UTXOTransaction;
use privacy_pool_zkvm::privacy_pool::PrivacyPool;
use ziskos::{read_input, set_output};
use std::convert::TryInto;

#[derive(serde::Serialize, serde::Deserialize)]
struct PoolBatch {
    transactions: Vec<UTXOTransaction>,
    old_merkle_root: [u8; 32],
    old_pool_balance: u64,
    expected_new_merkle_root: [u8; 32],
    expected_new_pool_balance: u64,
}

fn main() {
    // Read batch of transactions and expected state
    let input: Vec<u8> = read_input();
    let batch: PoolBatch = bincode::deserialize(&input)
        .expect("Failed to deserialize batch");
    
    // Initialize pool with old state
    let mut pool = PrivacyPool::new();
    pool.merkle_tree.root = batch.old_merkle_root;
    pool.pool_balance = batch.old_pool_balance;
    
    // Process all transactions in batch
    let mut successful_txs = 0;
    let mut total_fees = 0u64;
    let total_txs = batch.transactions.len();
    
    for transaction in batch.transactions {
        let fee = transaction.fee;
        match pool.process_transaction(transaction) {
            Ok(privacy_pool_zkvm::transaction::TransactionResult::Success) => {
                successful_txs += 1;
                total_fees += fee;
            }
            _ => {
                // Transaction failed - this should not happen if individual proofs were valid
                // But we continue processing the batch
            }
        }
    }
    
    // Verify final state matches expected
    let final_stats = pool.get_pool_stats();
    let state_correct = final_stats.merkle_root == batch.expected_new_merkle_root &&
                       final_stats.pool_balance == batch.expected_new_pool_balance;
    
    // Output validation results
    set_output(0, if state_correct { 1 } else { 0 }); // State validation
    set_output(1, successful_txs as u32); // Successful transactions
    set_output(2, total_txs as u32); // Total transactions
    set_output(3, total_fees as u32); // Total fees collected
    set_output(4, (total_fees >> 32) as u32); // Total fees (high 32 bits)
    
    // Output final Merkle root (8 x u32 = 32 bytes)
    for i in 0..8 {
        let chunk = u32::from_le_bytes(final_stats.merkle_root[i*4..(i+1)*4].try_into().unwrap());
        set_output(5 + i, chunk);
    }
    
    // Output final pool balance (2 x u32 = 8 bytes)
    set_output(13, final_stats.pool_balance as u32);
    set_output(14, (final_stats.pool_balance >> 32) as u32);
    
    // Output nullifier count
    set_output(15, final_stats.nullifiers_count as u32);
}
