const { ethers } = require("hardhat");
const fs = require("fs");

/**
 * Script to generate test proof data compatible with the PrivacyPoolVerifier contract
 * This simulates the proof generation process that would happen with ZisK zkVM
 */

async function generateTestProofData() {
  console.log(" Generating Test Proof Data for Privacy Pool Verifier");
  console.log("=" .repeat(60));
  
  // Simulate ZisK proof generation
  console.log(" Step 1: Simulating ZisK Proof Generation");
  console.log("-".repeat(40));
  
  // Create sample transaction data
  const transactionData = {
    inputCommitments: [
      ethers.keccak256(ethers.toUtf8Bytes("input_commitment_1")),
      ethers.keccak256(ethers.toUtf8Bytes("input_commitment_2"))
    ],
    outputCommitments: [
      ethers.keccak256(ethers.toUtf8Bytes("output_commitment_1")),
      ethers.keccak256(ethers.toUtf8Bytes("output_commitment_2"))
    ],
    nullifiers: [
      ethers.keccak256(ethers.toUtf8Bytes("nullifier_1")),
      ethers.keccak256(ethers.toUtf8Bytes("nullifier_2"))
    ],
    merkleProofs: [
      [ethers.keccak256(ethers.toUtf8Bytes("merkle_sibling_1"))],
      [ethers.keccak256(ethers.toUtf8Bytes("merkle_sibling_2"))]
    ],
    signature: "0x" + "1234567890abcdef".repeat(8), // 64 bytes
    publicKey: "0x" + "abcdef1234567890".repeat(2) + "01", // 33 bytes
    fee: ethers.parseEther("0.001") // 0.001 ETH
  };
  
  console.log(" Transaction data generated");
  console.log(`   Input commitments: ${transactionData.inputCommitments.length}`);
  console.log(`   Output commitments: ${transactionData.outputCommitments.length}`);
  console.log(`   Nullifiers: ${transactionData.nullifiers.length}`);
  console.log(`   Fee: ${ethers.formatEther(transactionData.fee)} ETH`);
  
  console.log("\n Step 2: Generating ZisK Public Inputs");
  console.log("-".repeat(40));
  
  // Simulate ZisK output (23 u32 values)
  const ziskOutputs = generateZiskOutputs(transactionData);
  console.log(" ZisK outputs generated");
  console.log(`   Validation result: ${ziskOutputs[0]}`);
  console.log(`   Merkle valid: ${ziskOutputs[1]}`);
  console.log(`   No double spend: ${ziskOutputs[2]}`);
  console.log(`   Signature valid: ${ziskOutputs[3]}`);
  console.log(`   Balance valid: ${ziskOutputs[4]}`);
  
  console.log("\n Step 3: Creating Proof Data Structure");
  console.log("-".repeat(40));
  
  const proofData = {
    publicInputs: ziskOutputs,
    proof: generateMockProof() // Mock ZK proof
  };
  
  console.log(" Proof data structure created");
  console.log(`   Public inputs: ${proofData.publicInputs.length} u32 values`);
  console.log(`   Proof length: ${proofData.proof.length} bytes`);
  
  console.log("\n Step 4: Saving Test Data");
  console.log("-".repeat(40));
  
  const testData = {
    transactionData,
    proofData,
    metadata: {
      generatedAt: new Date().toISOString(),
      network: "test",
      version: "1.0.0"
    }
  };
  
  const filename = `test-data-${Date.now()}.json`;
  fs.writeFileSync(filename, JSON.stringify(testData, null, 2));
  console.log(` Test data saved to: ${filename}`);
  
  console.log("\n Step 5: Testing Contract Integration");
  console.log("-".repeat(40));
  
  // Test with deployed contract (if available)
  try {
    const deploymentFiles = fs.readdirSync("./deployments").filter(f => f.endsWith(".json"));
    if (deploymentFiles.length > 0) {
      const latestDeployment = deploymentFiles[deploymentFiles.length - 1];
      const deploymentInfo = JSON.parse(fs.readFileSync(`./deployments/${latestDeployment}`, "utf8"));
      
      console.log(` Found deployment: ${latestDeployment}`);
      console.log(`   Contract: ${deploymentInfo.verifierAddress}`);
      
      // Test contract interaction
      await testContractInteraction(deploymentInfo.verifierAddress, proofData, transactionData);
    } else {
      console.log("⏭  No deployment found. Deploy contract first to test integration.");
    }
  } catch (error) {
    console.log("  Could not test contract integration:", error.message);
  }
  
  console.log("\n Test Proof Data Generation Complete!");
  console.log("=" .repeat(60));
  console.log(`Test data file: ${filename}`);
  console.log("\nNext steps:");
  console.log("1. Deploy the contract: npm run deploy:plasma");
  console.log("2. Test proof verification: node scripts/verify-proof.js");
  console.log("3. Monitor contract events and state changes");
  
  return testData;
}

function generateZiskOutputs(transactionData) {
  // Simulate the 23 u32 outputs from ZisK zkVM
  const outputs = new Array(23).fill(0);
  
  // Validation results (indices 0-4)
  outputs[0] = 1; // Overall validation result
  outputs[1] = 1; // Merkle proof validity
  outputs[2] = 1; // No double spend
  outputs[3] = 1; // Signature validity
  outputs[4] = 1; // Balance validity
  
  // New Merkle root (indices 5-12, 8 u32 values)
  const newMerkleRoot = ethers.keccak256(ethers.toUtf8Bytes("new_merkle_root"));
  const merkleRootBytes = ethers.getBytes(newMerkleRoot);
  for (let i = 0; i < 8; i++) {
    outputs[5 + i] = ethers.getUint(merkleRootBytes, i * 4, 4);
  }
  
  // New pool balance (indices 13-14, 2 u32 values)
  const newPoolBalance = ethers.parseEther("1000.001"); // 1000.001 ETH
  outputs[13] = Number(newPoolBalance & 0xFFFFFFFFn);
  outputs[14] = Number((newPoolBalance >> 32n) & 0xFFFFFFFFn);
  
  // Transaction hash (indices 15-22, 8 u32 values)
  const txHash = ethers.keccak256(ethers.toUtf8Bytes("test_transaction_hash"));
  const txHashBytes = ethers.getBytes(txHash);
  for (let i = 0; i < 8; i++) {
    outputs[15 + i] = ethers.getUint(txHashBytes, i * 4, 4);
  }
  
  return outputs;
}

function generateMockProof() {
  // Generate a mock ZK proof (in this would come from ZisK)
  const proofLength = 1000; // Mock proof length
  const proof = new Uint8Array(proofLength);
  
  // Fill with pseudo-random data
  for (let i = 0; i < proofLength; i++) {
    proof[i] = Math.floor(Math.random() * 256);
  }
  
  return "0x" + Array.from(proof).map(b => b.toString(16).padStart(2, '0')).join('');
}

async function testContractInteraction(contractAddress, proofData, transactionData) {
  try {
    console.log(" Testing contract interaction...");
    
    // Connect to deployed contract
    const PrivacyPoolVerifier = await ethers.getContractFactory("PrivacyPoolVerifier");
    const verifier = PrivacyPoolVerifier.attach(contractAddress);
    
    // Check current state
    const [merkleRoot, poolBalance, nullifierCount] = await verifier.getPoolState();
    console.log(`   Current Merkle Root: ${merkleRoot}`);
    console.log(`   Current Pool Balance: ${ethers.formatEther(poolBalance)} ETH`);
    console.log(`   Current Nullifier Count: ${nullifierCount}`);
    
    // Check if ZisK verifier is set
    const ziskVerifier = await verifier.ziskVerifier();
    if (ziskVerifier === ethers.ZeroAddress) {
      console.log("  ZisK verifier not set. Set it first before testing proof verification.");
    } else {
      console.log(`   ZisK Verifier: ${ziskVerifier}`);
    }
    
    console.log(" Contract interaction test completed");
    
  } catch (error) {
    console.log(" Contract interaction test failed:", error.message);
  }
}

// Run the script
if (require.main === module) {
  generateTestProofData()
    .then(() => process.exit(0))
    .catch((error) => {
      console.error(" Script failed:", error);
      process.exit(1);
    });
}

module.exports = { generateTestProofData };
