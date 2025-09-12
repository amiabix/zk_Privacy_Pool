#![no_main]
ziskos::entrypoint!(main);

use privacy_pool_zkvm::privacy_pool::PrivacyPool;
use privacy_pool_zkvm::utxo::UTXOTransaction;
use ziskos::{read_input, set_output};
use std::convert::TryInto;

fn main() {
    // Read transaction data from input.bin
    let input: Vec<u8> = read_input();
    
    // Deserialize transactions (now a vector of transactions)
    let transactions: Vec<UTXOTransaction> = bincode::deserialize(&input)
        .expect("Failed to deserialize transactions");
    
    // Process all transactions in ZisK zkVM
    let mut pool = PrivacyPool::new();
    let mut results = Vec::new();
    
    for transaction in transactions.into_iter() {
        let result = pool.process_transaction(transaction).unwrap_or_else(|e| {
            privacy_pool_zkvm::transaction::TransactionResult::Failure(format!("{:?}", e))
        });
        results.push(result);
    }
    
    // Get final pool stats
    let stats = pool.get_pool_stats();
    
    // Output only essential data (optimized for ZisK)
    // Output merkle root (8 x u32 = 32 bytes)
    for i in 0..8 {
        let chunk = u32::from_le_bytes(stats.merkle_root[i*4..(i+1)*4].try_into().unwrap());
        set_output(i, chunk);
    }
    
    // Output pool balance (2 x u32 = 8 bytes)
    set_output(8, stats.pool_balance as u32);
    set_output(9, (stats.pool_balance >> 32) as u32);
    
    // Output transaction count
    set_output(10, results.len() as u32);
    set_output(11, results.iter().filter(|r| r.is_success()).count() as u32);
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MultiUserTestResult {
    transaction_results: Vec<privacy_pool_zkvm::transaction::TransactionResult>,
    final_stats: privacy_pool_zkvm::privacy_pool::PoolStats,
}