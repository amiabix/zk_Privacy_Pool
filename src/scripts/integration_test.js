const { ethers } = require("hardhat");
const axios = require('axios');

// Configuration
const RELAYER_URL = process.env.RELAYER_URL || 'http://localhost:3000';
const RPC_URL = process.env.RPC_URL || 'http://localhost:8545';

class IntegrationTest {
    constructor() {
        this.contracts = {};
        this.accounts = {};
        this.testResults = [];
    }

    async setup() {
        console.log("🔧 Setting up integration test...");
        
        // Get accounts
        const [deployer, operator, user1, user2] = await ethers.getSigners();
        this.accounts = { deployer, operator, user1, user2 };
        
        // Deploy contracts
        await this.deployContracts();
        
        // Setup relayer (if running)
        await this.setupRelayer();
        
        console.log("✅ Setup completed");
    }

    async deployContracts() {
        console.log("📦 Deploying contracts...");
        
        // Deploy MockVerifier
        const MockVerifier = await ethers.getContractFactory("MockVerifier");
        this.contracts.verifier = await MockVerifier.deploy();
        await this.contracts.verifier.deployed();
        
        // Deploy PrivacyPool
        const PrivacyPool = await ethers.getContractFactory("PrivacyPool");
        this.contracts.privacyPool = await PrivacyPool.deploy(
            this.accounts.deployer.address,
            this.accounts.operator.address,
            this.contracts.verifier.address
        );
        await this.contracts.privacyPool.deployed();
        
        console.log("✅ Contracts deployed");
        console.log("  Verifier:", this.contracts.verifier.address);
        console.log("  PrivacyPool:", this.contracts.privacyPool.address);
    }

    async setupRelayer() {
        console.log("🔗 Setting up relayer connection...");
        
        try {
            const response = await axios.get(`${RELAYER_URL}/health`);
            console.log("✅ Relayer is running");
            this.relayerAvailable = true;
        } catch (error) {
            console.log("⚠️  Relayer not available, running without it");
            this.relayerAvailable = false;
        }
    }

    async runTests() {
        console.log("\n🧪 Running integration tests...");
        
        await this.testDepositFlow();
        await this.testMerkleRootUpdate();
        await this.testWithdrawalFlow();
        await this.testDoubleSpendPrevention();
        await this.testAccessControl();
        
        if (this.relayerAvailable) {
            await this.testRelayerIntegration();
        }
        
        this.printResults();
    }

    async testDepositFlow() {
        console.log("\n📥 Testing deposit flow...");
        
        try {
            const commitment = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("test-deposit"));
            const amount = ethers.utils.parseEther("0.1");
            
            // Deposit ETH
            const tx = await this.contracts.privacyPool
                .connect(this.accounts.user1)
                .depositETH(commitment, { value: amount });
            
            const receipt = await tx.wait();
            
            // Verify event was emitted
            const depositEvent = receipt.events.find(e => e.event === 'DepositIndexed');
            assert(depositEvent, "DepositIndexed event should be emitted");
            assert(depositEvent.args.commitment === commitment, "Commitment should match");
            assert(depositEvent.args.value.eq(amount), "Amount should match");
            
            this.testResults.push({ test: "Deposit Flow", status: "PASS" });
            console.log("✅ Deposit flow test passed");
            
        } catch (error) {
            this.testResults.push({ test: "Deposit Flow", status: "FAIL", error: error.message });
            console.log("❌ Deposit flow test failed:", error.message);
        }
    }

    async testMerkleRootUpdate() {
        console.log("\n🌳 Testing Merkle root update...");
        
        try {
            const newRoot = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("new-merkle-root"));
            
            // Only operator should be able to update root
            const tx = await this.contracts.privacyPool
                .connect(this.accounts.operator)
                .publishMerkleRoot(newRoot);
            
            const receipt = await tx.wait();
            
            // Verify event was emitted
            const rootEvent = receipt.events.find(e => e.event === 'MerkleRootPublished');
            assert(rootEvent, "MerkleRootPublished event should be emitted");
            assert(rootEvent.args.newRoot === newRoot, "New root should match");
            
            // Verify root was updated
            const currentRoot = await this.contracts.privacyPool.merkleRoot();
            assert(currentRoot === newRoot, "Merkle root should be updated");
            
            this.testResults.push({ test: "Merkle Root Update", status: "PASS" });
            console.log("✅ Merkle root update test passed");
            
        } catch (error) {
            this.testResults.push({ test: "Merkle Root Update", status: "FAIL", error: error.message });
            console.log("❌ Merkle root update test failed:", error.message);
        }
    }

    async testWithdrawalFlow() {
        console.log("\n📤 Testing withdrawal flow...");
        
        try {
            const nullifier = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("test-nullifier"));
            const recipient = this.accounts.user2.address;
            const amount = ethers.utils.parseEther("0.05");
            const asset = ethers.constants.AddressZero; // ETH
            
            // Mock proof and public signals
            const proof = "0x" + "00".repeat(128);
            const publicSignals = [
                nullifier,
                ethers.BigNumber.from(recipient),
                amount,
                ethers.BigNumber.from(asset),
                await this.contracts.privacyPool.merkleRoot()
            ];
            
            // Get initial balance
            const initialBalance = await this.accounts.user2.getBalance();
            
            // Withdraw
            const tx = await this.contracts.privacyPool
                .connect(this.accounts.user1)
                .withdraw(nullifier, recipient, amount, asset, proof, publicSignals);
            
            const receipt = await tx.wait();
            
            // Verify event was emitted
            const withdrawEvent = receipt.events.find(e => e.event === 'Withdrawn');
            assert(withdrawEvent, "Withdrawn event should be emitted");
            assert(withdrawEvent.args.recipient === recipient, "Recipient should match");
            assert(withdrawEvent.args.amount.eq(amount), "Amount should match");
            assert(withdrawEvent.args.nullifier === nullifier, "Nullifier should match");
            
            // Verify nullifier is marked as used
            const isNullifierUsed = await this.contracts.privacyPool.nullifiers(nullifier);
            assert(isNullifierUsed, "Nullifier should be marked as used");
            
            // Verify balance increased
            const finalBalance = await this.accounts.user2.getBalance();
            const balanceIncrease = finalBalance.sub(initialBalance);
            assert(balanceIncrease.gte(amount), "Balance should increase by at least the withdrawal amount");
            
            this.testResults.push({ test: "Withdrawal Flow", status: "PASS" });
            console.log("✅ Withdrawal flow test passed");
            
        } catch (error) {
            this.testResults.push({ test: "Withdrawal Flow", status: "FAIL", error: error.message });
            console.log("❌ Withdrawal flow test failed:", error.message);
        }
    }

    async testDoubleSpendPrevention() {
        console.log("\n🚫 Testing double-spend prevention...");
        
        try {
            const nullifier = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("double-spend-nullifier"));
            const recipient = this.accounts.user2.address;
            const amount = ethers.utils.parseEther("0.01");
            const asset = ethers.constants.AddressZero;
            
            const proof = "0x" + "00".repeat(128);
            const publicSignals = [
                nullifier,
                ethers.BigNumber.from(recipient),
                amount,
                ethers.BigNumber.from(asset),
                await this.contracts.privacyPool.merkleRoot()
            ];
            
            // First withdrawal should succeed
            const tx1 = await this.contracts.privacyPool
                .connect(this.accounts.user1)
                .withdraw(nullifier, recipient, amount, asset, proof, publicSignals);
            await tx1.wait();
            
            // Second withdrawal should fail
            try {
                const tx2 = await this.contracts.privacyPool
                    .connect(this.accounts.user1)
                    .withdraw(nullifier, recipient, amount, asset, proof, publicSignals);
                await tx2.wait();
                
                // If we get here, the test failed
                throw new Error("Second withdrawal should have failed");
                
            } catch (error) {
                if (error.message.includes("nullifier used")) {
                    // This is expected
                    this.testResults.push({ test: "Double-Spend Prevention", status: "PASS" });
                    console.log("✅ Double-spend prevention test passed");
                } else {
                    throw error;
                }
            }
            
        } catch (error) {
            this.testResults.push({ test: "Double-Spend Prevention", status: "FAIL", error: error.message });
            console.log("❌ Double-spend prevention test failed:", error.message);
        }
    }

    async testAccessControl() {
        console.log("\n🔐 Testing access control...");
        
        try {
            const newRoot = ethers.utils.keccak256(ethers.utils.toUtf8Bytes("unauthorized-root"));
            
            // Non-operator should not be able to update root
            try {
                await this.contracts.privacyPool
                    .connect(this.accounts.user1)
                    .publishMerkleRoot(newRoot);
                
                throw new Error("Non-operator should not be able to update root");
                
            } catch (error) {
                if (error.message.includes("AccessControl") || error.message.includes("missing role")) {
                    // This is expected
                    this.testResults.push({ test: "Access Control", status: "PASS" });
                    console.log("✅ Access control test passed");
                } else {
                    throw error;
                }
            }
            
        } catch (error) {
            this.testResults.push({ test: "Access Control", status: "FAIL", error: error.message });
            console.log("❌ Access control test failed:", error.message);
        }
    }

    async testRelayerIntegration() {
        console.log("\n🔗 Testing relayer integration...");
        
        try {
            // Test note upload
            const encryptedNote = {
                ephemeral_pubkey: "0x" + "02".repeat(33),
                nonce: "0x" + "00".repeat(24),
                ciphertext: "0x" + "00".repeat(64),
                commitment: ethers.utils.keccak256(ethers.utils.toUtf8Bytes("relayer-test")),
                owner_enc_pk: "0x" + "02".repeat(33)
            };
            
            const response = await axios.post(`${RELAYER_URL}/notes/upload`, {
                encrypted_note: encryptedNote
            });
            
            assert(response.data.note_id, "Note ID should be returned");
            assert(typeof response.data.attached === 'boolean', "Attached status should be returned");
            
            this.testResults.push({ test: "Relayer Integration", status: "PASS" });
            console.log("✅ Relayer integration test passed");
            
        } catch (error) {
            this.testResults.push({ test: "Relayer Integration", status: "FAIL", error: error.message });
            console.log("❌ Relayer integration test failed:", error.message);
        }
    }

    printResults() {
        console.log("\n📊 Test Results Summary:");
        console.log("========================");
        
        const passed = this.testResults.filter(r => r.status === "PASS").length;
        const failed = this.testResults.filter(r => r.status === "FAIL").length;
        
        this.testResults.forEach(result => {
            const status = result.status === "PASS" ? "✅" : "❌";
            console.log(`${status} ${result.test}: ${result.status}`);
            if (result.error) {
                console.log(`   Error: ${result.error}`);
            }
        });
        
        console.log(`\nTotal: ${passed + failed} tests, ${passed} passed, ${failed} failed`);
        
        if (failed === 0) {
            console.log("\n🎉 All tests passed!");
        } else {
            console.log("\n⚠️  Some tests failed. Please check the errors above.");
        }
    }
}

// Helper function for assertions
function assert(condition, message) {
    if (!condition) {
        throw new Error(message);
    }
}

async function main() {
    const test = new IntegrationTest();
    
    try {
        await test.setup();
        await test.runTests();
    } catch (error) {
        console.error("❌ Integration test failed:", error);
        process.exit(1);
    }
}

main().catch(console.error);
