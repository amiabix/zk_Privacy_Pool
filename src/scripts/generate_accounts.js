const { ethers } = require("hardhat");

async function main() {
    console.log("🔑 Generating test accounts...");
    
    const accounts = [];
    const privateKeys = [];
    
    // Generate 6 test accounts
    for (let i = 0; i < 6; i++) {
        const wallet = ethers.Wallet.createRandom();
        accounts.push({
            address: wallet.address,
            privateKey: wallet.privateKey,
            index: i
        });
        privateKeys.push(wallet.privateKey);
    }
    
    console.log("📋 Generated accounts:");
    accounts.forEach((account, index) => {
        console.log(`  Account ${index + 1}:`);
        console.log(`    Address: ${account.address}`);
        console.log(`    Private Key: ${account.privateKey}`);
        console.log("");
    });
    
    // Fund accounts from Anvil's first account (which has 1000 ETH)
    console.log("💰 Funding accounts...");
    
    const [deployer] = await ethers.getSigners();
    const fundingAmount = ethers.parseEther("10"); // 10 ETH per account
    
    for (let i = 0; i < accounts.length; i++) {
        const account = accounts[i];
        const wallet = new ethers.Wallet(account.privateKey, ethers.provider);
        
        try {
            // Send ETH from deployer to the account
            const tx = await deployer.sendTransaction({
                to: account.address,
                value: fundingAmount
            });
            await tx.wait();
            
            const balance = await ethers.provider.getBalance(account.address);
            console.log(`  ✅ Account ${i + 1} funded: ${ethers.formatEther(balance)} ETH`);
        } catch (error) {
            console.log(`  ❌ Failed to fund account ${i + 1}: ${error.message}`);
        }
    }
    
    // Save account info
    const accountInfo = {
        accounts: accounts,
        privateKeys: privateKeys,
        fundingAmount: fundingAmount.toString(),
        timestamp: new Date().toISOString()
    };
    
    const fs = require('fs');
    fs.writeFileSync('accounts.json', JSON.stringify(accountInfo, null, 2));
    console.log("💾 Account info saved to accounts.json");
    
    return accounts;
}

main()
    .then((accounts) => {
        console.log("🎉 Account generation completed successfully!");
        console.log(`Generated ${accounts.length} accounts`);
        process.exit(0);
    })
    .catch((error) => {
        console.error("❌ Account generation failed:", error);
        process.exit(1);
    });
