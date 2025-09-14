const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("PrivacyPoolFixed", function () {
    let privacyPool;
    let owner;
    let addr1;
    let addr2;
    let addrs;

    beforeEach(async function () {
        // Get the ContractFactory and Signers
        [owner, addr1, addr2, ...addrs] = await ethers.getSigners();

        // Deploy the contract
        const PrivacyPoolFixed = await ethers.getContractFactory("PrivacyPoolFixed");
        privacyPool = await PrivacyPoolFixed.deploy();
        await privacyPool.deployed();
    });

    describe("Deployment", function () {
        it("Should set the right owner", async function () {
            expect(await privacyPool.owner()).to.equal(owner.address);
        });

        it("Should initialize with correct merkle root", async function () {
            const expectedRoot = ethers.keccak256(
                ethers.utils.defaultAbiCoder.encode(["string"], ["PRIVACY_POOL_INIT"])
            );
            expect(await privacyPool.getCurrentMerkleRoot()).to.equal(expectedRoot);
        });

        it("Should start with zero balances", async function () {
            expect(await privacyPool.getContractBalance()).to.equal(0);
            expect(await privacyPool.getTotalDeposits()).to.equal(0);
            expect(await privacyPool.getTotalWithdrawals()).to.equal(0);
        });
    });

    describe("Auto Deposits", function () {
        it("Should allow auto deposit", async function () {
            const depositAmount = ethers.parseEther("1.0");
            
            await expect(privacyPool.connect(addr1).depositAuto({ value: depositAmount }))
                .to.emit(privacyPool, "Deposited")
                .withArgs(addr1.address, ethers.anyValue, depositAmount, ethers.anyValue);
                
            expect(await privacyPool.getContractBalance()).to.equal(depositAmount);
            expect(await privacyPool.getTotalDeposits()).to.equal(depositAmount);
        });

        it("Should reject zero value deposits", async function () {
            await expect(privacyPool.connect(addr1).depositAuto({ value: 0 }))
                .to.be.revertedWith("Deposit amount must be greater than 0");
        });

        it("Should update merkle root on deposit", async function () {
            const initialRoot = await privacyPool.getCurrentMerkleRoot();
            
            await privacyPool.connect(addr1).depositAuto({ value: ethers.parseEther("1.0") });
            
            const newRoot = await privacyPool.getCurrentMerkleRoot();
            expect(newRoot).to.not.equal(initialRoot);
        });
    });

    describe("Commitment Deposits", function () {
        it("Should allow deposit with custom commitment", async function () {
            const commitment = ethers.keccak256(ethers.utils.toUtf8Bytes("test_commitment"));
            const depositAmount = ethers.parseEther("1.0");
            
            await expect(privacyPool.connect(addr1).depositWithCommitment(commitment, { value: depositAmount }))
                .to.emit(privacyPool, "Deposited")
                .withArgs(addr1.address, commitment, depositAmount, ethers.anyValue);
        });

        it("Should reject duplicate commitments", async function () {
            const commitment = ethers.keccak256(ethers.utils.toUtf8Bytes("test_commitment"));
            const depositAmount = ethers.parseEther("1.0");
            
            await privacyPool.connect(addr1).depositWithCommitment(commitment, { value: depositAmount });
            
            await expect(privacyPool.connect(addr2).depositWithCommitment(commitment, { value: depositAmount }))
                .to.be.revertedWith("Commitment already exists");
        });
    });

    describe("Balance Functions", function () {
        it("Should return correct contract balance", async function () {
            const amount1 = ethers.parseEther("1.0");
            const amount2 = ethers.parseEther("0.5");
            
            await privacyPool.connect(addr1).depositAuto({ value: amount1 });
            await privacyPool.connect(addr2).depositAuto({ value: amount2 });
            
            expect(await privacyPool.getContractBalance()).to.equal(amount1 + amount2);
        });

        it("Should return zero user balance (simplified implementation)", async function () {
            expect(await privacyPool.getUserBalance(addr1.address)).to.equal(0);
        });
    });

    describe("Utility Functions", function () {
        it("Should check commitment usage correctly", async function () {
            const commitment = ethers.keccak256(ethers.utils.toUtf8Bytes("test_commitment"));
            
            expect(await privacyPool.isCommitmentUsed(commitment)).to.be.false;
            
            await privacyPool.connect(addr1).depositWithCommitment(commitment, { value: ethers.parseEther("1.0") });
            
            expect(await privacyPool.isCommitmentUsed(commitment)).to.be.true;
        });

        it("Should preview commitment correctly", async function () {
            const amount = ethers.parseEther("1.0");
            
            // This should not revert
            const previewCommitment = await privacyPool.previewCommitment(addr1.address, amount);
            expect(previewCommitment).to.not.equal("0x0000000000000000000000000000000000000000000000000000000000000000");
        });
    });

    describe("Function Name Disambiguation", function () {
        it("Should have distinct function names with no ambiguity", async function () {
            // Test that we can call both deposit functions without ambiguity
            const commitment = ethers.keccak256(ethers.utils.toUtf8Bytes("test"));
            const amount = ethers.parseEther("1.0");
            
            // This should work without ambiguity errors
            await privacyPool.connect(addr1).depositAuto({ value: amount });
            await privacyPool.connect(addr2).depositWithCommitment(commitment, { value: amount });
            
            // Both balance functions should work
            const contractBalance = await privacyPool.getContractBalance();
            const userBalance = await privacyPool.getUserBalance(addr1.address);
            
            expect(contractBalance).to.equal(amount * 2n);
            expect(userBalance).to.equal(0); // Simplified implementation
        });
    });
});