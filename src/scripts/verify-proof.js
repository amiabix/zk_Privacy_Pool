const { ethers } = require("hardhat");
const fs = require("fs");

/**
 * Script to verify a ZisK privacy pool proof on-chain
 * This script demonstrates how to interact with the deployed verifier contract
 */

async function main() {
  console.log(" Privacy Pool Proof Verification Script");
  
  // Get the contract address from deployment info
  const deploymentFiles = fs.readdirSync("./deployments").filter(f => f.endsWith(".json"));
  if (deploymentFiles.length === 0) {
    throw new Error("No deployment files found. Please deploy the contract first.");
  }
  
  const latestDeployment = deploymentFiles[deploymentFiles.length - 1];
  const deploymentInfo = JSON.parse(fs.readFileSync(`./deployments/${latestDeployment}`, "utf8"));
  
  console.log("Using deployment:", latestDeployment);
  console.log("Contract address:", deploymentInfo.verifierAddress);
  
  // Connect to the deployed contract
  const PrivacyPoolVerifier = await ethers.getContractFactory("PrivacyPoolVerifier");
  const verifier = PrivacyPoolVerifier.attach(deploymentInfo.verifierAddress);
  
  // Get current pool state
  const [merkleRoot, poolBalance, nullifierCount] = await verifier.getPoolState();
  console.log("\n Current Pool State:");
  console.log("Merkle Root:", merkleRoot);
  console.log("Pool Balance:", ethers.formatEther(poolBalance), "ETH");
  console.log("Nullifier Count:", nullifierCount.toString());
  
  // Example proof data (this would come from your ZisK proof generation)
  const exampleProofData = {
    publicInputs: [
      1,    // validation result (1 = valid)
      1,    // merkle valid
      1,    // no double spend
      1,    // signature valid
      1,    // balance valid
      // Merkle root (8 u32 values) - example
      0x12345678, 0x9abcdef0, 0x11111111, 0x22222222,
      0x33333333, 0x44444444, 0x55555555, 0x66666666,
      // Pool balance (2 u32 values) - example
      1000000, 0, // 1000000 wei
      // Transaction hash (8 u32 values) - example
      0x77777777, 0x88888888, 0x99999999, 0xaaaaaaaa,
      0xbbbbbbbb, 0xcccccccc, 0xdddddddd, 0xeeeeeeee
    ],
    proof: "0x" + "00".repeat(1000) // Placeholder proof data
  };
  
  // Example transaction data
  const inputCommitments = [
    ethers.keccak256(ethers.toUtf8Bytes("input1")),
    ethers.keccak256(ethers.toUtf8Bytes("input2"))
  ];
  
  const outputCommitments = [
    ethers.keccak256(ethers.toUtf8Bytes("output1"))
  ];
  
  const nullifiers = [
    ethers.keccak256(ethers.toUtf8Bytes("nullifier1")),
    ethers.keccak256(ethers.toUtf8Bytes("nullifier2"))
  ];
  
  const merkleProofs = [
    [ethers.keccak256(ethers.toUtf8Bytes("sibling1"))],
    [ethers.keccak256(ethers.toUtf8Bytes("sibling2"))]
  ];
  
  const signature = "0x" + "00".repeat(64); // Placeholder signature
  const publicKey = "0x" + "00".repeat(33); // Placeholder public key
  const fee = ethers.parseEther("0.001"); // 0.001 ETH fee
  
  console.log("\n Verifying proof...");
  console.log("Input commitments:", inputCommitments.length);
  console.log("Output commitments:", outputCommitments.length);
  console.log("Nullifiers:", nullifiers.length);
  console.log("Fee:", ethers.formatEther(fee), "ETH");
  
  try {
    // Call the verifyAndUpdateProof function
    const tx = await verifier.verifyAndUpdateProof(
      exampleProofData,
      inputCommitments,
      outputCommitments,
      nullifiers,
      merkleProofs,
      signature,
      publicKey,
      fee
    );
    
    console.log("⏳ Waiting for transaction confirmation...");
    const receipt = await tx.wait();
    console.log(" Proof verification transaction confirmed!");
    console.log("Transaction hash:", receipt.hash);
    console.log("Gas used:", receipt.gasUsed.toString());
    
    // Get updated pool state
    const [newMerkleRoot, newPoolBalance, newNullifierCount] = await verifier.getPoolState();
    console.log("\n Updated Pool State:");
    console.log("New Merkle Root:", newMerkleRoot);
    console.log("New Pool Balance:", ethers.formatEther(newPoolBalance), "ETH");
    console.log("New Nullifier Count:", newNullifierCount.toString());
    
  } catch (error) {
    console.error(" Proof verification failed:", error.message);
    
    // Check if it's a known error
    if (error.message.includes("Pool not initialized")) {
      console.log(" Try initializing the pool first");
    } else if (error.message.includes("ZisK verifier not set")) {
      console.log(" Set the ZisK verifier address first");
    } else if (error.message.includes("Proof already verified")) {
      console.log(" This proof has already been verified");
    } else if (error.message.includes("Nullifier already used")) {
      console.log(" One or more nullifiers have already been used");
    }
  }
}

main()
  .then(() => process.exit(0))
  .catch((error) => {
    console.error(error);
    process.exit(1);
  });
