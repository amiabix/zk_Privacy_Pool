// Deploy Fixed Privacy Pool Contract
// This script deploys the PrivacyPoolFixed contract with unambiguous function names

const { ethers } = require("hardhat");

async function main() {
    console.log(" Deploying Fixed Privacy Pool Contract...");
    
    // Get the ContractFactory and Signers
    const [deployer] = await ethers.getSigners();
    
    console.log(" Deploying contracts with the account:", deployer.address);
    console.log(" Account balance:", (await ethers.provider.getBalance(deployer.address)).toString());
    
    // Deploy the fixed contract
    const PrivacyPoolFixed = await ethers.getContractFactory("PrivacyPoolFixed");
    
    console.log(" Deploying PrivacyPoolFixed...");
    const privacyPool = await PrivacyPoolFixed.deploy();
    
    await privacyPool.waitForDeployment();
    
    console.log(" PrivacyPoolFixed deployed to:", await privacyPool.getAddress());
    
    // Verify deployment by calling some view functions
    const merkleRoot = await privacyPool.getCurrentMerkleRoot();
    const contractBalance = await privacyPool.getContractBalance();
    const totalDeposits = await privacyPool.getTotalDeposits();
    
    console.log("\n Contract Status:");
    console.log("  Merkle Root:", merkleRoot);
    console.log("  Contract Balance:", ethers.formatEther(contractBalance), "ETH");
    console.log("  Total Deposits:", ethers.formatEther(totalDeposits), "ETH");
    
    // Save deployment info
    const contractAddress = await privacyPool.getAddress();
    const deploymentInfo = {
        contractAddress: contractAddress,
        deployerAddress: deployer.address,
        network: (await ethers.provider.getNetwork()).name,
        chainId: (await ethers.provider.getNetwork()).chainId,
        blockNumber: await ethers.provider.getBlockNumber(),
        timestamp: new Date().toISOString(),
        contractName: "PrivacyPoolFixed",
        functions: {
            deposit: "depositAuto() - for simple deposits",
            depositWithCommitment: "depositWithCommitment(bytes32) - for custom commitments",
            withdraw: "withdraw(bytes32,address,uint256) - for withdrawals",
            getBalance: "getContractBalance() - for total balance",
            getUserBalance: "getUserBalance(address) - for user balance"
        }
    };
    
    // Write deployment info to file
    const fs = require('fs');
    fs.writeFileSync(
        'deployment-fixed.json', 
        JSON.stringify(deploymentInfo, null, 2)
    );
    
    console.log("\n Deployment info saved to deployment-fixed.json");
    
    // Generate ABI for frontend
    const abi = JSON.stringify(privacyPool.interface.format('json'), null, 2);
    fs.writeFileSync('PrivacyPoolFixed-ABI.json', abi);
    
    console.log(" ABI saved to PrivacyPoolFixed-ABI.json");
    
    console.log("\n Deployment completed successfully!");
    console.log("\n For frontend integration, use:");
    console.log(`  Contract Address: ${contractAddress}`);
    console.log("  Functions:");
    console.log("    - depositAuto() for simple deposits");
    console.log("    - depositWithCommitment(bytes32) for custom commitments");
    console.log("    - getContractBalance() for total balance");
    console.log("    - getUserBalance(address) for user balance");
    
    return privacyPool;
}

// We recommend this pattern to be able to use async/await everywhere
// and properly handle errors.
main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error(" Deployment failed:", error);
        process.exit(1);
    });