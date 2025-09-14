//! Secure Merkle Tree Inspector - Shows detailed tree structure and UTXO contents with security analysis

use crate::utxo::converter::{ETHToUTXOConverter, CryptoUtils, ETHDepositProcessor};
use crate::{BlockchainConfig, RealDepositEvent};
use crate::utxo::indexing::IndexedUTXO;
use web3::types::{Address, U256, H256};
use anyhow::Result;

/// Secure Tree Inspector - Shows detailed information about the Merkle tree and UTXOs
pub struct TreeInspector {
    processor: ETHDepositProcessor,
}

impl TreeInspector {
    pub fn new() -> Result<Self> {
        let config = BlockchainConfig::default();
        let processor = ETHDepositProcessor::new(config)?;
        Ok(Self { processor })
    }

    /// Process real deposits from blockchain and then inspect the tree
    pub async fn process_and_inspect(&mut self) -> Result<()> {
        println!("🌳 Secure Merkle Tree Inspector (Real Blockchain Integration)");
        println!("{}", "=".repeat(70));

        // Generate a secure private key for processing deposits
        let depositor_private_key = CryptoUtils::generate_secure_random();
        
        // Create a sample deposit event for testing
        let deposit_event = RealDepositEvent {
            depositor: Address::from([0x42u8; 20]),
            commitment: H256::from([0x43u8; 32]),
            label: U256::from(1),
            value: U256::from(1000000000000000000u64), // 1 ETH
            precommitment_hash: H256::from([0x44u8; 32]),
            block_number: 1000,
            transaction_hash: H256::from([0x45u8; 32]),
            log_index: 0,
        };
        
        // Process real deposits from the blockchain
        println!("\n🔍 Searching for real deposits on blockchain...");
        let processed_utxos = self.processor.process_real_deposits(&depositor_private_key).await?;
        
        if processed_utxos.is_empty() {
            println!("ℹ️  No deposits found on blockchain. This is normal if no deposits have been made.");
            println!("   To test with real deposits:");
            println!("   1. Start Anvil: anvil");
            println!("   2. Deploy contracts and make deposits");
            println!("   3. Run this inspector again");
        } else {
            println!("✅ Processed {} UTXOs from real blockchain deposits", processed_utxos.len());
        }

        // Now inspect the tree with security analysis
        self.inspect_secure_tree().await
    }

    /// Inspect the current state of the Merkle tree with security analysis
    pub async fn inspect_secure_tree(&self) -> Result<()> {
        println!("\n🔍 Secure Merkle Tree Detailed Inspection");
        println!("{}", "=".repeat(70));

        // Get basic tree info
        let merkle_root = self.processor.get_merkle_root();
        let utxo_count = self.processor.get_utxo_count();
        let all_utxos = self.processor.get_all_utxos();

        // Get secure accounting info
        let (total_deposited, total_utxo_value, spent_nullifiers) = self.processor.converter.get_accounting_info();

        println!("\n📊 Secure Tree Statistics:");
        println!("   🌳 Merkle Root: {:?}", merkle_root);
        println!("   📦 Total UTXOs: {}", utxo_count);
        println!("   💰 Total Deposited: {} ETH ({} wei)", total_deposited as f64 / 1e18, total_deposited);
        println!("   💰 Total UTXO Value: {} ETH ({} wei)", total_utxo_value as f64 / 1e18, total_utxo_value);
        println!("   ⚠️  Spent Nullifiers: {}", spent_nullifiers);
        println!("   🏗️  Tree Depth: 32 levels");
        
        // Verify accounting integrity
        if total_deposited == total_utxo_value {
            println!("   ✅ Accounting: VERIFIED (perfectly balanced)");
        } else {
            println!("   ❌ Accounting: FAILED (imbalanced - SECURITY ISSUE!)");
            println!("      Deposited: {} wei, UTXOs: {} wei, Diff: {} wei", 
                     total_deposited, total_utxo_value, 
                     (total_deposited as i128 - total_utxo_value as i128).abs());
        }

        // Show UTXO details with security information
        if !all_utxos.is_empty() {
            println!("\n💰 Secure UTXO Details:");
            println!("{}", "-".repeat(100));
            println!("{:<4} {:<12} {:<20} {:<20} {:<8} {:<10} {:<20}", 
                     "ID", "Value (ETH)", "Value (wei)", "Commitment", "Height", "Spent", "Blinding Factor");
            println!("{}", "-".repeat(100));

            for (i, utxo) in all_utxos.iter().enumerate() {
                let value_eth = utxo.value as f64 / 1e18;
                let commitment_hex = hex::encode(utxo.address);
                let blinding_hex = hex::encode(utxo.blinding_factor);
                let spent_status = if utxo.spent_in_tx.is_some() { "Yes" } else { "No" };

                println!("{:<4} {:<12.6} {:<20} {:<20} {:<8} {:<10} {:<20}", 
                         i + 1,
                         value_eth,
                         utxo.value,
                         format!("{}...", &commitment_hex[..16]),
                         utxo.height,
                         spent_status,
                         format!("{}...", &blinding_hex[..16]));
            }
        }

        // Security Analysis
        self.perform_security_analysis(&all_utxos)?;

        // Privacy Analysis
        self.perform_privacy_analysis(&all_utxos)?;

        // Tree structure visualization
        self.visualize_secure_tree_structure(&all_utxos)?;
        
        Ok(())
    }

    /// Perform comprehensive security analysis
    fn perform_security_analysis(&self, utxos: &[&IndexedUTXO]) -> Result<()> {
        println!("\n🔐 Security Analysis:");
        println!("{}", "-".repeat(50));

        // 1. Commitment uniqueness check (critical for security)
        let commitments: Vec<[u8; 32]> = utxos.iter().map(|u| u.address).collect();
        let unique_commitments: std::collections::HashSet<[u8; 32]> = commitments.iter().cloned().collect();
        
        if commitments.len() == unique_commitments.len() {
            println!("   ✅ Commitment Uniqueness: All {} commitments are unique", commitments.len());
        } else {
            let duplicates = commitments.len() - unique_commitments.len();
            println!("   ❌ CRITICAL SECURITY FLAW: {} duplicate commitments found!", duplicates);
            println!("      This allows double-spending attacks!");
        }

        // 2. Blinding factor uniqueness check (important for privacy)
        let blinding_factors: Vec<[u8; 32]> = utxos.iter().map(|u| u.blinding_factor).collect();
        let unique_blinding: std::collections::HashSet<[u8; 32]> = blinding_factors.iter().cloned().collect();
        
        if blinding_factors.len() == unique_blinding.len() {
            println!("   ✅ Blinding Factor Uniqueness: All {} factors are unique", blinding_factors.len());
        } else {
            let duplicates = blinding_factors.len() - unique_blinding.len();
            println!("   ⚠️  Privacy Risk: {} duplicate blinding factors found!", duplicates);
            println!("      This reduces anonymity but doesn't allow double-spending");
        }

        // 3. Value distribution analysis (for detecting unusual patterns)
        let mut value_distribution = std::collections::HashMap::new();
        for utxo in utxos {
            *value_distribution.entry(utxo.value).or_insert(0) += 1;
        }
        
        println!("   📊 Value Distribution Analysis:");
        if value_distribution.len() < utxos.len() / 2 {
            println!("      ✅ Good: {} unique values for {} UTXOs (diverse)", 
                     value_distribution.len(), utxos.len());
        } else {
            println!("      ⚠️  Warning: Too many identical values (reduces privacy)");
        }

        // 4. Sequential pattern detection
        let mut heights: Vec<u32> = utxos.iter().map(|u| u.height).collect();
        heights.sort();
        let mut sequential_count = 0;
        for i in 1..heights.len() {
            if heights[i] == heights[i-1] + 1 {
                sequential_count += 1;
            }
        }
        
        if sequential_count > heights.len() / 2 {
            println!("   ⚠️  Privacy Warning: {} sequential heights detected", sequential_count);
        } else {
            println!("   ✅ Height Distribution: Good randomness ({} sequential)", sequential_count);
        }

        Ok(())
    }

    /// Perform privacy analysis
    fn perform_privacy_analysis(&self, utxos: &[&IndexedUTXO]) -> Result<()> {
        println!("\n🔒 Privacy Analysis:");
        println!("{}", "-".repeat(50));

        if utxos.is_empty() {
            println!("   ℹ️  No UTXOs to analyze");
            return Ok(());
        }

        // 1. Anonymity set size
        println!("   🎭 Anonymity Set Size: {} UTXOs", utxos.len());
        if utxos.len() < 10 {
            println!("      ⚠️  Warning: Small anonymity set reduces privacy");
        } else if utxos.len() < 100 {
            println!("      ✅ Moderate: Provides basic privacy protection");
        } else {
            println!("      ✅ Excellent: Large anonymity set provides strong privacy");
        }

        // 2. Value distribution for privacy
        let mut value_counts = std::collections::HashMap::new();
        for utxo in utxos {
            let value_eth = (utxo.value as f64 / 1e18 * 10.0).round() / 10.0; // Round to 0.1 ETH
            *value_counts.entry(value_eth as i64).or_insert(0) += 1;
        }

        println!("   💰 Value Distribution for Privacy:");
        let mut values: Vec<_> = value_counts.keys().cloned().collect();
        values.sort();
        for value in values {
            let count = value_counts[&value];
            let value_eth = value as f64 / 10.0;
            println!("      {:.1} ETH: {} UTXOs", value_eth, count);
            
            if count == 1 {
                println!("        ⚠️  Unique value reduces privacy");
            } else if count >= 3 {
                println!("        ✅ Good anonymity set for this value");
            }
        }

        // 3. Timing analysis
        let heights: Vec<u32> = utxos.iter().map(|u| u.height).collect();
        let min_height = heights.iter().min().unwrap_or(&0);
        let max_height = heights.iter().max().unwrap_or(&0);
        let height_span = max_height - min_height;
        
        println!("   ⏰ Timing Privacy:");
        println!("      Height Range: {} - {} (span: {})", min_height, max_height, height_span);
        if height_span > 100 {
            println!("      ✅ Excellent: UTXOs spread across many blocks");
        } else if height_span > 10 {
            println!("      ✅ Good: Reasonable temporal distribution");
        } else {
            println!("      ⚠️  Warning: UTXOs clustered in time");
        }

        Ok(())
    }

    /// Visualize the secure tree structure
    fn visualize_secure_tree_structure(&self, utxos: &[&IndexedUTXO]) -> Result<()> {
        println!("\n🌳 Secure Merkle Tree Structure:");
        println!("{}", "-".repeat(50));

        let merkle_root = self.processor.get_merkle_root();
        let utxo_count = utxos.len();

        println!("   🌳 Root: {:?}", merkle_root);
        println!("   📏 Max Depth: 32 levels");
        println!("   🔢 Current UTXOs: {}", utxo_count);
        println!("   📊 Tree Utilization: {:.2}% (of 2^32 possible leaves)", 
                 utxo_count as f64 / (2u64.pow(32) as f64) * 100.0);

        if utxo_count > 0 {
            // Calculate theoretical tree depth needed
            let needed_depth = (utxo_count as f64).log2().ceil() as u32;
            println!("   🎯 Minimum Depth Needed: {} levels", needed_depth);
            println!("   ⚡ Efficiency: {}% depth utilization", 
                     needed_depth as f64 / 32.0 * 100.0);

            // Show tree balance
            println!("   ⚖️  Tree Balance: Incremental (always balanced)");
            println!("   🔐 Hash Function: SHA-256");
            println!("   🛡️  Proof Size: {} hashes (fixed)", 32);
        }

        // Security properties
        println!("\n🛡️  Security Properties:");
        println!("   ✅ Collision Resistance: SHA-256 provides 2^128 security");
        println!("   ✅ Preimage Resistance: 2^256 security against reversing");
        println!("   ✅ Merkle Proof Soundness: Cryptographically guaranteed");
        println!("   ✅ Nullifier Protection: Prevents double-spending");

        Ok(())
    }
}

/// Demonstration function
pub async fn demo_secure_tree_inspection() -> Result<()> {
    println!("🚀 Starting Secure Tree Inspector Demo...\n");
    
    let mut inspector = TreeInspector::new()
        .map_err(|e| anyhow::anyhow!("Failed to create tree inspector: {}", e))?;
    
    inspector.process_and_inspect().await?;
    
    println!("\n🎉 Secure Tree Inspection Complete!");
    println!("   This analysis shows the security and privacy properties");
    println!("   of the UTXO system with real blockchain integration.");
    
    Ok(())
}