const { ethers } = require("hardhat");

async function main() {
    console.log("🚀 Starting Privacy Pool deployment...");
    
    // Get the contract factory
    const PrivacyPool = await ethers.getContractFactory("PrivacyPool");
    
    // Deploy the contract
    console.log("📦 Deploying PrivacyPool contract...");
    const privacyPool = await PrivacyPool.deploy();
    
    // Wait for deployment to complete
    await privacyPool.waitForDeployment();
    
    const contractAddress = await privacyPool.getAddress();
    console.log("✅ PrivacyPool deployed to:", contractAddress);
    
    // Get initial state
    const balance = await privacyPool.getBalance();
    const merkleRoot = await privacyPool.merkleRoot();
    
    console.log("📊 Initial state:");
    console.log("  - Contract balance:", ethers.formatEther(balance), "ETH");
    console.log("  - Merkle root:", merkleRoot);
    
    // Save deployment info
    const deploymentInfo = {
        contractAddress: contractAddress,
        merkleRoot: merkleRoot,
        balance: balance.toString(),
        timestamp: new Date().toISOString()
    };
    
    const fs = require('fs');
    fs.writeFileSync('deployment.json', JSON.stringify(deploymentInfo, null, 2));
    console.log("💾 Deployment info saved to deployment.json");
    
    return contractAddress;
}

main()
    .then((address) => {
        console.log("🎉 Deployment completed successfully!");
        console.log("Contract address:", address);
        process.exit(0);
    })
    .catch((error) => {
        console.error("❌ Deployment failed:", error);
        process.exit(1);
    });