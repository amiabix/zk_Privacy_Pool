const { ethers } = require("hardhat");
const fs = require("fs");

/**
 * Full deployment script for Privacy Pool Verifier
 * This script handles deployment, initialization, and verification setup
 */

async function main() {
  console.log("🚀 Starting Full Privacy Pool Verifier Deployment");
  console.log("=" .repeat(50));
  
  // Get network info
  const network = hre.network.name;
  const chainId = hre.network.config.chainId;
  console.log(`Network: ${network} (Chain ID: ${chainId})`);
  
  // Get deployer account
  const [deployer] = await ethers.getSigners();
  console.log(`Deployer: ${deployer.address}`);
  console.log(`Balance: ${ethers.formatEther(await deployer.getBalance())} ETH`);
  
  // Check if we have enough balance
  const balance = await deployer.getBalance();
  if (balance < ethers.parseEther("0.1")) {
    console.log("⚠️  Warning: Low balance. You may need more ETH for deployment.");
  }
  
  console.log("\n📦 Step 1: Deploying PrivacyPoolVerifier Contract");
  console.log("-".repeat(40));
  
  // Deploy the contract
  const PrivacyPoolVerifier = await ethers.getContractFactory("PrivacyPoolVerifier");
  const verifier = await PrivacyPoolVerifier.deploy();
  
  console.log("⏳ Waiting for deployment transaction...");
  await verifier.waitForDeployment();
  
  const verifierAddress = await verifier.getAddress();
  console.log(`✅ Contract deployed to: ${verifierAddress}`);
  
  // Get deployment transaction details
  const deploymentTx = verifier.deploymentTransaction();
  console.log(`📄 Deployment TX: ${deploymentTx.hash}`);
  console.log(`⛽ Gas Used: ${deploymentTx.gasLimit.toString()}`);
  
  console.log("\n🔧 Step 2: Initializing Pool");
  console.log("-".repeat(40));
  
  // Initialize with empty state
  const initTx = await verifier.initializePool(
    ethers.ZeroHash, // Empty Merkle root initially
    0 // Zero balance initially
  );
  
  console.log("⏳ Waiting for initialization transaction...");
  await initTx.wait();
  
  console.log("✅ Pool initialized successfully");
  
  // Verify initialization
  const [merkleRoot, poolBalance, nullifierCount] = await verifier.getPoolState();
  console.log(`📊 Initial State:`);
  console.log(`   Merkle Root: ${merkleRoot}`);
  console.log(`   Pool Balance: ${ethers.formatEther(poolBalance)} ETH`);
  console.log(`   Nullifier Count: ${nullifierCount}`);
  
  console.log("\n🔍 Step 3: Contract Verification");
  console.log("-".repeat(40));
  
  // Verify contract on block explorer (if not local network)
  if (network !== "localhost" && network !== "hardhat") {
    console.log("🔍 Verifying contract on block explorer...");
    
    try {
      await hre.run("verify:verify", {
        address: verifierAddress,
        constructorArguments: []
      });
      console.log("✅ Contract verified successfully");
    } catch (error) {
      console.log("❌ Contract verification failed:");
      console.log(`   Error: ${error.message}`);
      console.log("💡 You can verify manually later with:");
      console.log(`   npx hardhat verify --network ${network} ${verifierAddress}`);
    }
  } else {
    console.log("⏭️  Skipping verification for local network");
  }
  
  console.log("\n📋 Step 4: Saving Deployment Information");
  console.log("-".repeat(40));
  
  // Create deployments directory if it doesn't exist
  if (!fs.existsSync("./deployments")) {
    fs.mkdirSync("./deployments");
  }
  
  // Save deployment information
  const deploymentInfo = {
    network: network,
    chainId: chainId,
    verifierAddress: verifierAddress,
    deployerAddress: deployer.address,
    deploymentTime: new Date().toISOString(),
    deploymentTxHash: deploymentTx.hash,
    initializationTxHash: initTx.hash,
    gasUsed: {
      deployment: deploymentTx.gasLimit.toString(),
      initialization: (await initTx.wait()).gasUsed.toString()
    },
    contractABI: PrivacyPoolVerifier.interface.format("json"),
    // Next steps
    nextSteps: [
      "Set ZisK verifier address: verifier.setZiskVerifier(address)",
      "Deploy ZisK verifier contract (if not already deployed)",
      "Test proof verification with sample data",
      "Monitor contract events and state changes"
    ]
  };
  
  const filename = `deployments/${network}-${Date.now()}.json`;
  fs.writeFileSync(filename, JSON.stringify(deploymentInfo, null, 2));
  console.log(`📄 Deployment info saved to: ${filename}`);
  
  console.log("\n🎯 Step 5: Next Steps");
  console.log("-".repeat(40));
  
  console.log("1. Set ZisK Verifier Address:");
  console.log(`   await verifier.setZiskVerifier("ZISK_VERIFIER_ADDRESS");`);
  
  console.log("\n2. Test Contract Functions:");
  console.log(`   const [root, balance, count] = await verifier.getPoolState();`);
  
  console.log("\n3. Monitor Events:");
  console.log(`   verifier.on("ProofVerified", (proofHash, txHash, ...) => { ... });`);
  
  console.log("\n4. Deploy ZisK Verifier (if needed):");
  console.log("   Deploy the actual ZisK verifier contract and set its address");
  
  console.log("\n5. Test Proof Verification:");
  console.log("   Use the verify-proof.js script to test with sample data");
  
  console.log("\n🎉 Deployment Completed Successfully!");
  console.log("=" .repeat(50));
  console.log(`Contract Address: ${verifierAddress}`);
  console.log(`Network: ${network}`);
  console.log(`Chain ID: ${chainId}`);
  console.log(`Deployment File: ${filename}`);
  
  return {
    verifierAddress,
    verifier,
    deploymentInfo
  };
}

// Handle errors gracefully
main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error("\n❌ Deployment failed:");
    console.error(error);
    
    // Provide helpful error messages
    if (error.message.includes("insufficient funds")) {
      console.log("\n💡 Solution: Add more ETH to your account");
    } else if (error.message.includes("network")) {
      console.log("\n💡 Solution: Check your network configuration");
    } else if (error.message.includes("gas")) {
      console.log("\n💡 Solution: Increase gas limit or gas price");
    }
    
    process.exit(1);
  });
