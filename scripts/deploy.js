const { ethers } = require("hardhat");

async function main() {
  console.log("Deploying Privacy Pool contracts...");

  // Get the deployer account
  const [deployer] = await ethers.getSigners();
  console.log("Deploying contracts with account:", deployer.address);
  console.log("Account balance:", (await deployer.provider.getBalance(deployer.address)).toString());

  // Deploy Entrypoint
  console.log("\nDeploying Entrypoint...");
  const Entrypoint = await ethers.getContractFactory("Entrypoint");
  const entrypoint = await Entrypoint.deploy();
  await entrypoint.waitForDeployment();
  console.log("Entrypoint deployed to:", await entrypoint.getAddress());

  // Deploy Verifiers
  console.log("\nDeploying Verifiers...");
  const Verifier = await ethers.getContractFactory("Verifier");
  const withdrawalVerifier = await Verifier.deploy();
  await withdrawalVerifier.waitForDeployment();
  console.log("Withdrawal Verifier deployed to:", await withdrawalVerifier.getAddress());

  const ragequitVerifier = await Verifier.deploy();
  await ragequitVerifier.waitForDeployment();
  console.log("Ragequit Verifier deployed to:", await ragequitVerifier.getAddress());

  // Deploy ETH Privacy Pool
  console.log("\nDeploying ETH Privacy Pool...");
  const ETHPrivacyPool = await ethers.getContractFactory("ETHPrivacyPool");
  const ethPrivacyPool = await ETHPrivacyPool.deploy(
    await entrypoint.getAddress(),
    await withdrawalVerifier.getAddress(),
    await ragequitVerifier.getAddress()
  );
  await ethPrivacyPool.waitForDeployment();
  console.log("ETH Privacy Pool deployed to:", await ethPrivacyPool.getAddress());

  // Submit initial root to entrypoint
  console.log("\nSubmitting initial root...");
  const initialRoot = ethers.keccak256(ethers.toUtf8Bytes("initial_root"));
  await entrypoint.submitRoot(initialRoot);
  console.log("Initial root submitted:", initialRoot);

  console.log("\n=== Deployment Summary ===");
  console.log("Entrypoint:", await entrypoint.getAddress());
  console.log("Withdrawal Verifier:", await withdrawalVerifier.getAddress());
  console.log("Ragequit Verifier:", await ragequitVerifier.getAddress());
  console.log("ETH Privacy Pool:", await ethPrivacyPool.getAddress());
  console.log("Initial Root:", initialRoot);

  // Save deployment info
  const deploymentInfo = {
    network: "anvil",
    entrypoint: await entrypoint.getAddress(),
    withdrawalVerifier: await withdrawalVerifier.getAddress(),
    ragequitVerifier: await ragequitVerifier.getAddress(),
    ethPrivacyPool: await ethPrivacyPool.getAddress(),
    initialRoot: initialRoot,
    deployer: deployer.address
  };

  const fs = require('fs');
  fs.writeFileSync('deployment.json', JSON.stringify(deploymentInfo, null, 2));
  console.log("\nDeployment info saved to deployment.json");
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });