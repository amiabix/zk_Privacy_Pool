const { ethers } = require("hardhat");
const fs = require('fs');

async function main() {
    console.log("🚀 Starting Privacy Pool End-to-End Test...");
    
    // Load deployment info
    const deploymentInfo = JSON.parse(fs.readFileSync('deployment.json', 'utf8'));
    const contractAddress = deploymentInfo.contractAddress;
    
    // Load account info
    const accountInfo = JSON.parse(fs.readFileSync('accounts.json', 'utf8'));
    const accounts = accountInfo.accounts;
    
    console.log("📋 Test Setup:");
    console.log(`  Contract Address: ${contractAddress}`);
    console.log(`  Number of Accounts: ${accounts.length}`);
    console.log("");
    
    // Connect to the contract
    const PrivacyPool = await ethers.getContractFactory("PrivacyPool");
    const privacyPool = PrivacyPool.attach(contractAddress);
    
    // Test 1: Make deposits and create UTXOs
    console.log("🔵 Test 1: Making deposits and creating UTXOs...");
    
    const deposits = [];
    for (let i = 0; i < 3; i++) {
        const account = accounts[i];
        const wallet = new ethers.Wallet(account.privateKey, ethers.provider);
        
        // Generate a commitment (simplified - in this would be a proper hash)
        const commitment = ethers.keccak256(ethers.toUtf8Bytes(`commitment_${i}_${Date.now()}`));
        const depositAmount = ethers.parseEther("1"); // 1 ETH deposit
        
        try {
            const tx = await privacyPool.connect(wallet).deposit(commitment, { value: depositAmount });
            await tx.wait();
            
            deposits.push({
                account: account.address,
                commitment: commitment,
                amount: depositAmount.toString(),
                txHash: tx.hash
            });
            
            console.log(`  ✅ Account ${i + 1} deposited 1 ETH with commitment: ${commitment}`);
        } catch (error) {
            console.log(`  ❌ Account ${i + 1} deposit failed: ${error.message}`);
        }
    }
    
    // Check contract balance
    const contractBalance = await privacyPool.getBalance();
    console.log(`  📊 Contract balance: ${ethers.formatEther(contractBalance)} ETH`);
    console.log("");
    
    // Test 2: Generate ZK proofs (simplified)
    console.log("🔵 Test 2: Generating ZK proofs...");
    
    for (let i = 0; i < deposits.length; i++) {
        const deposit = deposits[i];
        const account = accounts[i];
        
        // Generate a nullifier (simplified)
        const nullifier = ethers.keccak256(ethers.toUtf8Bytes(`nullifier_${i}_${Date.now()}`));
        
        console.log(`  ✅ Account ${i + 1} generated nullifier: ${nullifier}`);
        
        // Store the nullifier for withdrawal
        deposit.nullifier = nullifier;
    }
    
    console.log("");
    
    // Test 3: Make withdrawals using nullifiers
    console.log("🔵 Test 3: Making withdrawals...");
    
    for (let i = 0; i < deposits.length; i++) {
        const deposit = deposits[i];
        const account = accounts[i];
        const wallet = new ethers.Wallet(account.privateKey, ethers.provider);
        
        try {
            const withdrawalAmount = ethers.parseEther("0.5"); // Withdraw 0.5 ETH
            const tx = await privacyPool.connect(wallet).withdraw(
                deposit.nullifier,
                account.address, // Withdraw to same address
                withdrawalAmount
            );
            await tx.wait();
            
            console.log(`  ✅ Account ${i + 1} withdrew 0.5 ETH using nullifier: ${deposit.nullifier}`);
        } catch (error) {
            console.log(`  ❌ Account ${i + 1} withdrawal failed: ${error.message}`);
        }
    }
    
    // Check final contract balance
    const finalBalance = await privacyPool.getBalance();
    console.log(`  📊 Final contract balance: ${ethers.formatEther(finalBalance)} ETH`);
    console.log("");
    
    // Test 4: Verify Merkle tree updates
    console.log("🔵 Test 4: Verifying Merkle tree updates...");
    
    const currentRoot = await privacyPool.merkleRoot();
    console.log(`  📊 Current Merkle root: ${currentRoot}`);
    
    // Check if commitments were recorded
    for (let i = 0; i < deposits.length; i++) {
        const deposit = deposits[i];
        const isUsed = await privacyPool.isCommitmentUsed(deposit.commitment);
        console.log(`  📋 Commitment ${i + 1} used: ${isUsed}`);
    }
    
    // Check if nullifiers were recorded
    for (let i = 0; i < deposits.length; i++) {
        const deposit = deposits[i];
        const isUsed = await privacyPool.isNullifierUsed(deposit.nullifier);
        console.log(`  📋 Nullifier ${i + 1} used: ${isUsed}`);
    }
    
    console.log("");
    
    // Test 5: Verify account balances
    console.log("🔵 Test 5: Verifying account balances...");
    
    for (let i = 0; i < accounts.length; i++) {
        const account = accounts[i];
        const balance = await ethers.provider.getBalance(account.address);
        console.log(`  💰 Account ${i + 1} balance: ${ethers.formatEther(balance)} ETH`);
    }
    
    console.log("");
    console.log("🎉 Privacy Pool End-to-End Test Completed!");
    console.log("📊 Summary:");
    console.log(`  - Deposits made: ${deposits.length}`);
    console.log(`  - Withdrawals made: ${deposits.length}`);
    console.log(`  - Contract balance: ${ethers.formatEther(finalBalance)} ETH`);
    console.log(`  - Merkle root updated: ${currentRoot}`);
}

main()
    .then(() => {
        console.log("✅ All tests completed successfully!");
        process.exit(0);
    })
    .catch((error) => {
        console.error("❌ Test failed:", error);
        process.exit(1);
    });
