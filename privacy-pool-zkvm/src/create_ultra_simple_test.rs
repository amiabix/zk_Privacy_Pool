use std::fs;

#[derive(serde::Serialize, serde::Deserialize)]
struct SimpleTransaction {
    amount: u64,
    fee: u64,
    recipient: Vec<u8>,
    signature: Vec<u8>,
}

fn main() {
    println!("Creating ultra simple transaction test...");
    
    // Create a very simple transaction
    let transaction = SimpleTransaction {
        amount: 500,
        fee: 10,
        recipient: vec![1u8; 32],
        signature: vec![2u8; 64], // Non-zero signature
    };
    
    // Save transaction to input.bin
    let serialized = bincode::serialize(&transaction).unwrap();
    fs::write("build/ultra_simple_input.bin", serialized).unwrap();
    
    println!("=== ULTRA SIMPLE TRANSACTION TEST CREATED ===");
    println!("Amount: 500 tokens");
    println!("Fee: 10 tokens");
    println!("Saved to: build/ultra_simple_input.bin");
    println!("Expected result: SUCCESS (valid transaction)");
}
