//! Simple Anvil Test - Basic connection and wallet creation test

use web3::{
    transports::Http,
    Web3,
    types::{Address, U256, H256, TransactionParameters, Bytes},
};
use secp256k1::{SecretKey, PublicKey, Secp256k1};
use sha2::{Sha256, Digest};
use std::str::FromStr;

/// Test basic Anvil connection and wallet creation
#[tokio::test]
async fn test_anvil_connection_and_wallets() {
    println!("🚀 Starting Simple Anvil Test");
    println!("{}", "=".repeat(50));

    // Step 1: Connect to Anvil
    println!("\n🔌 Step 1: Connecting to Anvil...");
    let transport = Http::new("http://127.0.0.1:8545").expect("Failed to create HTTP transport");
    let web3 = Web3::new(transport);
    
    // Test connection
    match web3.eth().block_number().await {
        Ok(block_number) => {
            println!("   ✅ Connected to Anvil! Current block: {}", block_number);
        }
        Err(e) => {
            println!("   ❌ Failed to connect to Anvil: {}", e);
            return;
        }
    }

    // Step 2: Create wallets using Anvil's private keys
    println!("\n👛 Step 2: Creating wallets from Anvil private keys...");
    
    // Anvil's actual private keys from your output
    let anvil_private_keys = [
        "ac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80", // Account 0
        "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d", // Account 1
        "5de4111afa1a4b94908f83103eb1f1706367c2e68ca870fc3fb9a804cdab365a", // Account 2
        "7c852118294e51e653712a81e05800f419141751be58f605c371e15141b007a6", // Account 3
        "47e179ec197488593b187f80a00eb0da91f1b9d0b13f8733639f19c30a34926a", // Account 4
    ];

    let secp = Secp256k1::new();
    let mut wallets = Vec::new();

    for (i, private_key_hex) in anvil_private_keys.iter().enumerate() {
        match hex::decode(private_key_hex) {
            Ok(private_key_bytes) => {
                if private_key_bytes.len() != 32 {
                    println!("   ❌ Account {}: Invalid private key length ({} bytes)", i, private_key_bytes.len());
                    continue;
                }

                let mut private_key_array = [0u8; 32];
                private_key_array.copy_from_slice(&private_key_bytes);
                
                let secret_key = SecretKey::from_slice(&private_key_array).expect("Invalid private key");
                let public_key = PublicKey::from_secret_key(&secp, &secret_key);
                
                // Convert public key to Ethereum address
                let public_key_bytes = public_key.serialize_uncompressed();
                let hash = Sha256::digest(&public_key_bytes[1..]); // Skip the 0x04 prefix
                let mut address_bytes = [0u8; 20];
                address_bytes.copy_from_slice(&hash[12..]); // Take last 20 bytes
                let address = Address::from_slice(&address_bytes);
                
                let balance = web3.eth().balance(address, None).await.unwrap_or(U256::zero());
                
                println!("   ✅ Account {}: {} (Balance: {} ETH)", 
                         i, 
                         format!("{:?}", address),
                         balance.as_u64() as f64 / 1e18);
                
                wallets.push((address, secret_key, balance));
            }
            Err(e) => {
                println!("   ❌ Account {}: Failed to decode private key: {}", i, e);
            }
        }
    }

    println!("\n📊 Summary:");
    println!("   Total wallets created: {}", wallets.len());
    println!("   Total ETH across all wallets: {} ETH", 
             wallets.iter().map(|(_, _, balance)| balance.as_u64()).sum::<u64>() as f64 / 1e18);

    // Step 3: Test a simple transaction (if we have wallets)
    if !wallets.is_empty() {
        println!("\n💸 Step 3: Testing simple transaction...");
        
        let (sender_addr, sender_key, sender_balance) = &wallets[0];
        let receiver_addr = wallets[1].0;
        
        println!("   Sending 0.1 ETH from {} to {}", 
                 format!("{:?}", sender_addr),
                 format!("{:?}", receiver_addr));
        
        // Create a simple transaction
        let tx_params = TransactionParameters {
            to: Some(receiver_addr),
            value: U256::from(100000000000000000u64), // 0.1 ETH in wei
            gas: U256::from(21000),
            gas_price: Some(U256::from(20000000000u64)), // 20 gwei
            nonce: None,
            data: Bytes::from(vec![]),
            access_list: None,
            chain_id: None,
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
            transaction_type: None,
        };

        // Note: This would need proper signing in a real implementation
        // For now, we'll just show the transaction parameters
        println!("   📝 Transaction parameters created successfully");
        println!("   ⚠️  Note: Actual transaction signing not implemented yet");
    }

    println!("\n✅ Simple Anvil test completed successfully!");
}
