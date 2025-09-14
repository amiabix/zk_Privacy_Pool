#!/usr/bin/env node

/**
 * Complete Privacy Pool Demo
 * 
 * This script demonstrates the entire privacy pool flow:
 * 1. Deploy contracts
 * 2. Start relayer (simulated)
 * 3. Create and encrypt note
 * 4. Upload to relayer
 * 5. Submit deposit transaction
 * 6. Process deposit (relayer)
 * 7. Update Merkle root
 * 8. Test withdrawal
 */

const { ethers } = require("hardhat");
const axios = require('axios');
const { poseidon } = require('poseidon-lite');
const { randomBytes } = require('crypto');

class PrivacyPoolDemo {
    constructor() {
        this.contracts = {};
        this.accounts = {};
        this.relayerUrl = process.env.RELAYER_URL || 'http://localhost:3000';
    }

    async setup() {
        console.log("🚀 Setting up Privacy Pool Demo...");
        
        // Get accounts
        const [deployer, operator, user1, user2] = await ethers.getSigners();
        this.accounts = { deployer, operator, user1, user2 };
        
        console.log("📋 Accounts:");
        console.log("  Deployer:", deployer.address);
        console.log("  Operator:", operator.address);
        console.log("  User1:", user1.address);
        console.log("  User2:", user2.address);
        
        // Deploy contracts
        await this.deployContracts();
        
        console.log("✅ Setup completed");
    }

    async deployContracts() {
        console.log("\n📦 Deploying contracts...");
        
        // Deploy MockVerifier
        const MockVerifier = await ethers.getContractFactory("MockVerifier");
        this.contracts.verifier = await MockVerifier.deploy();
        await this.contracts.verifier.deployed();
        console.log("  MockVerifier:", this.contracts.verifier.address);
        
        // Deploy PrivacyPool
        const PrivacyPool = await ethers.getContractFactory("PrivacyPool");
        this.contracts.privacyPool = await PrivacyPool.deploy(
            this.accounts.deployer.address,
            this.accounts.operator.address,
            this.contracts.verifier.address
        );
        await this.contracts.privacyPool.deployed();
        console.log("  PrivacyPool:", this.contracts.privacyPool.address);
    }

    async createNote(value, recipientPubkey) {
        console.log("\n📝 Creating note...");
        
        // Generate random secret and blinding
        const secret = randomBytes(32);
        const blinding = randomBytes(32);
        
        // Compute commitment using Poseidon
        const commitment = this.computeCommitment(recipientPubkey, value, secret, blinding);
        
        const note = {
            version: 1,
            chain_id: 31337, // Hardhat local
            pool_address: this.contracts.privacyPool.address,
            value: ethers.utils.parseEther(value).toString(),
            owner_enc_pk: recipientPubkey,
            secret: '0x' + secret.toString('hex'),
            blinding: '0x' + blinding.toString('hex'),
            commitment: '0x' + commitment.toString('hex')
        };
        
        console.log("  Note created:");
        console.log("    Commitment:", note.commitment);
        console.log("    Value:", value, "ETH");
        console.log("    Owner:", recipientPubkey);
        
        return { note, secret, blinding };
    }

    computeCommitment(ownerPk, value, secret, blinding) {
        // Convert inputs to field elements
        const ownerPkBytes = Buffer.from(ownerPk.slice(2), 'hex');
        const valueBytes = Buffer.from(ethers.utils.parseEther(value).toHexString().slice(2), 'hex');
        
        // Pad to 32 bytes
        const paddedOwnerPk = Buffer.alloc(32);
        ownerPkBytes.copy(paddedOwnerPk, 32 - ownerPkBytes.length);
        
        const paddedValue = Buffer.alloc(32);
        valueBytes.copy(paddedValue, 32 - valueBytes.length);

        // Compute Poseidon hash
        const inputs = [
            BigInt('0x' + paddedOwnerPk.toString('hex')),
            BigInt('0x' + paddedValue.toString('hex')),
            BigInt('0x' + secret.toString('hex')),
            BigInt('0x' + blinding.toString('hex'))
        ];

        const hash = poseidon(inputs);
        return Buffer.from(hash.toString(16).padStart(64, '0'), 'hex');
    }

    async encryptNote(note) {
        console.log("\n🔐 Encrypting note...");
        
        // In a real implementation, this would use proper ECIES encryption
        const noteJson = JSON.stringify(note);
        const ciphertext = Buffer.from(noteJson).toString('hex');
        
        const encryptedNote = {
            ephemeral_pubkey: '0x' + randomBytes(33).toString('hex'),
            nonce: '0x' + randomBytes(24).toString('hex'),
            ciphertext: '0x' + ciphertext,
            commitment: note.commitment,
            owner_enc_pk: note.owner_enc_pk
        };
        
        console.log("  Note encrypted successfully");
        return encryptedNote;
    }

    async uploadNote(encryptedNote) {
        console.log("\n📤 Uploading note to relayer...");
        
        try {
            const response = await axios.post(`${this.relayerUrl}/notes/upload`, {
                encrypted_note: encryptedNote
            });
            
            console.log("  Note uploaded successfully");
            console.log("    Note ID:", response.data.note_id);
            console.log("    Attached:", response.data.attached);
            
            return response.data;
        } catch (error) {
            console.log("  ⚠️  Relayer not available, simulating upload");
            return { note_id: "simulated_note_id", attached: false };
        }
    }

    async submitDeposit(commitment, value) {
        console.log("\n💰 Submitting deposit transaction...");
        
        const tx = await this.contracts.privacyPool
            .connect(this.accounts.user1)
            .depositETH(commitment, { 
                value: ethers.utils.parseEther(value) 
            });
        
        console.log("  Transaction submitted:", tx.hash);
        
        const receipt = await tx.wait();
        console.log("  Transaction confirmed in block:", receipt.blockNumber);
        
        // Extract event data
        const depositEvent = receipt.events.find(e => e.event === 'DepositIndexed');
        if (depositEvent) {
            console.log("  Deposit event emitted:");
            console.log("    Depositor:", depositEvent.args.depositor);
            console.log("    Commitment:", depositEvent.args.commitment);
            console.log("    Value:", ethers.utils.formatEther(depositEvent.args.value), "ETH");
        }
        
        return tx.hash;
    }

    async processDeposit(txHash, commitment) {
        console.log("\n🔄 Processing deposit (relayer simulation)...");
        
        // In a real implementation, the relayer would:
        // 1. Listen for DepositIndexed events
        // 2. Wait for confirmations
        // 3. Insert commitment into Merkle tree
        // 4. Update database
        
        console.log("  Simulating relayer processing...");
        console.log("    TX Hash:", txHash);
        console.log("    Commitment:", commitment);
        console.log("    Status: Processed (simulated)");
        
        // Simulate Merkle tree insertion
        const leafIndex = Math.floor(Math.random() * 1000);
        console.log("    Leaf Index:", leafIndex);
        
        return leafIndex;
    }

    async updateMerkleRoot() {
        console.log("\n🌳 Updating Merkle root...");
        
        // Generate a new Merkle root (simulated)
        const newRoot = ethers.utils.keccak256(
            ethers.utils.toUtf8Bytes("new-merkle-root-" + Date.now())
        );
        
        const tx = await this.contracts.privacyPool
            .connect(this.accounts.operator)
            .publishMerkleRoot(newRoot);
        
        const receipt = await tx.wait();
        console.log("  Merkle root updated:", newRoot);
        console.log("  Gas used:", receipt.gasUsed.toString());
        
        return newRoot;
    }

    async testWithdrawal(secret, leafIndex, merkleRoot) {
        console.log("\n📤 Testing withdrawal...");
        
        // Generate nullifier
        const nullifier = ethers.utils.keccak256(
            ethers.utils.concat([
                ethers.utils.toUtf8Bytes("PRIVPOOL_NULL_V1"),
                secret,
                ethers.utils.defaultAbiCoder.encode(['uint64'], [leafIndex])
            ])
        );
        
        const recipient = this.accounts.user2.address;
        const amount = ethers.utils.parseEther("0.05");
        const asset = ethers.constants.AddressZero;
        
        // Mock proof and public signals
        const proof = "0x" + "00".repeat(128);
        const publicSignals = [
            nullifier,
            ethers.BigNumber.from(recipient),
            amount,
            ethers.BigNumber.from(asset),
            merkleRoot
        ];
        
        console.log("  Withdrawal details:");
        console.log("    Nullifier:", nullifier);
        console.log("    Recipient:", recipient);
        console.log("    Amount:", ethers.utils.formatEther(amount), "ETH");
        
        const tx = await this.contracts.privacyPool
            .connect(this.accounts.user1)
            .withdraw(nullifier, recipient, amount, asset, proof, publicSignals);
        
        const receipt = await tx.wait();
        console.log("  Withdrawal successful!");
        console.log("  Gas used:", receipt.gasUsed.toString());
        
        // Verify nullifier is marked as used
        const isNullifierUsed = await this.contracts.privacyPool.nullifiers(nullifier);
        console.log("  Nullifier used:", isNullifierUsed);
        
        return tx.hash;
    }

    async runDemo() {
        console.log("🎭 Privacy Pool Complete Demo");
        console.log("=============================");
        
        try {
            await this.setup();
            
            // Step 1: Create note
            const recipientPubkey = '0x' + randomBytes(33).toString('hex');
            const { note, secret, blinding } = await this.createNote("0.1", recipientPubkey);
            
            // Step 2: Encrypt note
            const encryptedNote = await this.encryptNote(note);
            
            // Step 3: Upload to relayer
            const uploadResult = await this.uploadNote(encryptedNote);
            
            // Step 4: Submit deposit
            const txHash = await this.submitDeposit(note.commitment, "0.1");
            
            // Step 5: Process deposit (relayer)
            const leafIndex = await this.processDeposit(txHash, note.commitment);
            
            // Step 6: Update Merkle root
            const merkleRoot = await this.updateMerkleRoot();
            
            // Step 7: Test withdrawal
            const withdrawalTx = await this.testWithdrawal(secret, leafIndex, merkleRoot);
            
            console.log("\n🎉 Demo completed successfully!");
            console.log("\n📊 Summary:");
            console.log("  Contract Address:", this.contracts.privacyPool.address);
            console.log("  Deposit TX:", txHash);
            console.log("  Withdrawal TX:", withdrawalTx);
            console.log("  Merkle Root:", merkleRoot);
            console.log("  Leaf Index:", leafIndex);
            
        } catch (error) {
            console.error("❌ Demo failed:", error);
            process.exit(1);
        }
    }
}

// Run demo
async function main() {
    const demo = new PrivacyPoolDemo();
    await demo.runDemo();
}

main().catch(console.error);
