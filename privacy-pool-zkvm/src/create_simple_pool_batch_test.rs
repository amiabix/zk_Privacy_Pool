use std::fs;

#[derive(serde::Serialize, serde::Deserialize)]
struct SimplePoolBatch {
    transactions: Vec<SimpleTransaction>,
    old_balance: u64,
    expected_new_balance: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct SimpleTransaction {
    amount: u64,
    fee: u64,
    recipient: Vec<u8>,
    signature: Vec<u8>,
    tx_type: String, // "deposit", "withdraw", "transfer"
}

fn main() {
    println!("Creating simple pool batch test...");
    
    // Create a batch of simple transactions
    let transactions = vec![
        SimpleTransaction {
            amount: 1000,
            fee: 10,
            recipient: vec![1u8; 32],
            signature: vec![2u8; 64],
            tx_type: "deposit".to_string(),
        },
        SimpleTransaction {
            amount: 500,
            fee: 5,
            recipient: vec![3u8; 32],
            signature: vec![4u8; 64],
            tx_type: "withdraw".to_string(),
        },
        SimpleTransaction {
            amount: 200,
            fee: 3,
            recipient: vec![5u8; 32],
            signature: vec![6u8; 64],
            tx_type: "transfer".to_string(),
        },
    ];
    
    // Calculate expected final balance
    let old_balance = 0u64;
    let mut expected_balance = old_balance;
    
    for tx in &transactions {
        match tx.tx_type.as_str() {
            "deposit" => expected_balance += tx.amount,
            "withdraw" => {
                if expected_balance >= tx.amount {
                    expected_balance -= tx.amount;
                }
            }
            "transfer" => {
                // Transfer doesn't change pool balance
            }
            _ => {}
        }
    }
    
    // Create batch data
    let batch = SimplePoolBatch {
        transactions,
        old_balance,
        expected_new_balance: expected_balance,
    };
    
    // Save batch to input.bin
    let serialized = bincode::serialize(&batch).unwrap();
    fs::write("build/simple_pool_batch_input.bin", serialized).unwrap();
    
    println!("=== SIMPLE POOL BATCH TEST CREATED ===");
    println!("Total transactions: {}", batch.transactions.len());
    println!("Old pool balance: {}", old_balance);
    println!("Expected new balance: {}", expected_balance);
    println!("Saved to: build/simple_pool_batch_input.bin");
    println!("Expected result: SUCCESS (valid batch accounting)");
}
