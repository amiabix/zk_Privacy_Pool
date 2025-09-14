const { ethers } = require("hardhat");

async function main() {
    console.log("Deploying Privacy Pool contracts...");

    // Get the deployer account
    const [deployer, operator] = await ethers.getSigners();
    console.log("Deploying contracts with account:", deployer.address);
    console.log("Account balance:", (await deployer.getBalance()).toString());

    // Deploy MockVerifier first
    console.log("\n1. Deploying MockVerifier...");
    const MockVerifier = await ethers.getContractFactory("MockVerifier");
    const verifier = await MockVerifier.deploy();
    await verifier.deployed();
    console.log("MockVerifier deployed to:", verifier.address);

    // Deploy PrivacyPool contract
    console.log("\n2. Deploying PrivacyPool...");
    const PrivacyPool = await ethers.getContractFactory("PrivacyPool");
    const privacyPool = await PrivacyPool.deploy(
        deployer.address,  // admin
        operator.address,  // initial operator
        verifier.address   // verifier
    );
    await privacyPool.deployed();
    console.log("PrivacyPool deployed to:", privacyPool.address);

    // Verify deployment
    console.log("\n3. Verifying deployment...");
    const merkleRoot = await privacyPool.merkleRoot();
    const admin = await privacyPool.admin();
    const verifierAddress = await privacyPool.verifier();
    
    console.log("Merkle Root:", merkleRoot);
    console.log("Admin:", admin);
    console.log("Verifier:", verifierAddress);

    // Test basic functionality
    console.log("\n4. Testing basic functionality...");
    
    // Test deposit (requires ETH)
    const depositAmount = ethers.utils.parseEther("0.1");
    const commitment = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("test-commitment"));
    
    console.log("Testing deposit with commitment:", commitment);
    const depositTx = await privacyPool.depositETH(commitment, { value: depositAmount });
    const depositReceipt = await depositTx.wait();
    console.log("Deposit successful! Gas used:", depositReceipt.gasUsed.toString());

    // Test operator functionality
    console.log("\n5. Testing operator functionality...");
    const newRoot = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("new-merkle-root"));
    
    // Switch to operator account
    const operatorSigner = await ethers.getSigner(operator.address);
    const privacyPoolOperator = privacyPool.connect(operatorSigner);
    
    const publishTx = await privacyPoolOperator.publishMerkleRoot(newRoot);
    const publishReceipt = await publishTx.wait();
    console.log("Merkle root published! Gas used:", publishReceipt.gasUsed.toString());

    // Verify new root
    const currentRoot = await privacyPool.merkleRoot();
    console.log("Current Merkle Root:", currentRoot);

    // Test withdrawal (mock proof)
    console.log("\n6. Testing withdrawal...");
    const nullifier = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("test-nullifier"));
    const recipient = deployer.address;
    const amount = ethers.utils.parseEther("0.05");
    const asset = ethers.constants.AddressZero; // ETH
    
    // Mock proof and public signals
    const proof = "0x" + "00".repeat(128); // Mock proof
    const publicSignals = [
        nullifier,                    // nullifier
        ethers.BigNumber.from(recipient), // recipient (as uint256)
        amount,                       // amount
        ethers.BigNumber.from(asset), // asset (as uint256)
        currentRoot                   // merkle root
    ];

    const withdrawTx = await privacyPool.withdraw(
        nullifier,
        recipient,
        amount,
        asset,
        proof,
        publicSignals
    );
    const withdrawReceipt = await withdrawTx.wait();
    console.log("Withdrawal successful! Gas used:", withdrawReceipt.gasUsed.toString());

    // Verify nullifier is marked as used
    const isNullifierUsed = await privacyPool.nullifiers(nullifier);
    console.log("Nullifier used:", isNullifierUsed);

    console.log("\n7. Deployment Summary:");
    console.log("===================");
    console.log("MockVerifier:", verifier.address);
    console.log("PrivacyPool:", privacyPool.address);
    console.log("Admin:", deployer.address);
    console.log("Operator:", operator.address);
    console.log("Initial Merkle Root:", merkleRoot);
    console.log("Current Merkle Root:", currentRoot);

    // Save deployment info
    const deploymentInfo = {
        network: await ethers.provider.getNetwork(),
        contracts: {
            MockVerifier: verifier.address,
            PrivacyPool: privacyPool.address
        },
        accounts: {
            admin: deployer.address,
            operator: operator.address
        },
        merkleRoot: currentRoot,
        timestamp: new Date().toISOString()
    };

    console.log("\n8. Deployment info saved to deployment.json");
    require('fs').writeFileSync(
        'deployment.json', 
        JSON.stringify(deploymentInfo, null, 2)
    );

    console.log("\n✅ Deployment completed successfully!");
}

main()
    .then(() => process.exit(0))
    .catch((error) => {
        console.error("❌ Deployment failed:", error);
        process.exit(1);
    });
