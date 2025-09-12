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
    
    for (i, transaction) in transactions.into_iter().enumerate() {
        println!("Processing transaction {}", i + 1);
        let result = pool.process_transaction(transaction).unwrap_or_else(|e| {
            println!("Transaction {} failed: {:?}", i + 1, e);
            privacy_pool_zkvm::transaction::TransactionResult::Failure(format!("{:?}", e))
        });
        results.push(result);
    }
    
    // Get final pool stats
    let stats = pool.get_pool_stats();
    
    // Create final result
    let final_result = MultiUserTestResult {
        transaction_results: results,
        final_stats: stats,
    };
    
    // Output result
    let output = bincode::serialize(&final_result).expect("Failed to serialize result");
    for (i, chunk) in output.chunks(4).enumerate() {
        let val = u32::from_le_bytes(chunk.try_into().unwrap_or([0; 4]));
        set_output(i, val);
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MultiUserTestResult {
    transaction_results: Vec<privacy_pool_zkvm::transaction::TransactionResult>,
    final_stats: privacy_pool_zkvm::privacy_pool::PoolStats,
}