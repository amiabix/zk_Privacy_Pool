use std::fs;

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
    println!("Creating ultra simple pool batch test...");
    
    // Create a batch of ultra simple transactions
    let transactions = vec![
        UltraSimpleTransaction {
            amount: 1000,
            fee: 10,
            recipient: vec![1u8; 32],
            signature: vec![2u8; 64],
            tx_type: 0, // deposit
        },
        UltraSimpleTransaction {
            amount: 500,
            fee: 5,
            recipient: vec![3u8; 32],
            signature: vec![4u8; 64],
            tx_type: 1, // withdraw
        },
        UltraSimpleTransaction {
            amount: 200,
            fee: 3,
            recipient: vec![5u8; 32],
            signature: vec![6u8; 64],
            tx_type: 2, // transfer
        },
    ];
    
    // Calculate expected final balance
    let old_balance = 0u64;
    let mut expected_balance = old_balance;
    
    for tx in &transactions {
        match tx.tx_type {
            0 => expected_balance += tx.amount, // deposit
            1 => { // withdraw
                if expected_balance >= tx.amount {
                    expected_balance -= tx.amount;
                }
            }
            2 => { // transfer
                // Transfer doesn't change pool balance
            }
            _ => {}
        }
    }
    
    // Create batch data
    let batch = UltraSimplePoolBatch {
        transactions,
        old_balance,
        expected_new_balance: expected_balance,
    };
    
    // Save batch to input.bin
    let serialized = bincode::serialize(&batch).unwrap();
    fs::write("build/ultra_simple_pool_input.bin", serialized).unwrap();
    
    println!("=== ULTRA SIMPLE POOL BATCH TEST CREATED ===");
    println!("Total transactions: {}", batch.transactions.len());
    println!("Old pool balance: {}", old_balance);
    println!("Expected new balance: {}", expected_balance);
    println!("Saved to: build/ultra_simple_pool_input.bin");
    println!("Expected result: SUCCESS (valid batch accounting)");
}
