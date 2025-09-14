const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying Privacy Pool contract...");

  // Get the contract factory
  const PrivacyPool = await ethers.getContractFactory("PrivacyPool");

  // Deploy the contract
  const privacyPool = await PrivacyPool.deploy();

  // Wait for deployment to complete
  await privacyPool.waitForDeployment();

  const contractAddress = await privacyPool.getAddress();
  
  console.log("Privacy Pool deployed to:", contractAddress);
  console.log("Contract owner:", await privacyPool.owner());
  console.log("Initial merkle root:", await privacyPool.merkleRoot());
  console.log("Contract balance:", ethers.formatEther(await privacyPool.getBalance()), "ETH");

  // Save deployment info
  const deploymentInfo = {
    contractAddress: contractAddress,
    network: "anvil",
    timestamp: new Date().toISOString(),
    owner: await privacyPool.owner(),
    merkleRoot: await privacyPool.merkleRoot()
  };

  console.log("\n=== Deployment Summary ===");
  console.log("Contract Address:", contractAddress);
  console.log("Network: Anvil (Local)");
  console.log("Owner:", await privacyPool.owner());
  console.log("Merkle Root:", await privacyPool.merkleRoot());
  console.log("\n=== Frontend Configuration ===");
  console.log("Update your frontend App.jsx with this contract address:");
  console.log(`const CONTRACT_ADDRESS = "${contractAddress}"`);
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
