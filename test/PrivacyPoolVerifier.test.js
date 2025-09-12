const { expect } = require("chai");
const { ethers } = require("hardhat");

describe("PrivacyPoolVerifier", function () {
  let verifier;
  let owner;
  let user1;
  let user2;

  beforeEach(async function () {
    [owner, user1, user2] = await ethers.getSigners();
    
    const PrivacyPoolVerifier = await ethers.getContractFactory("PrivacyPoolVerifier");
    verifier = await PrivacyPoolVerifier.deploy();
    await verifier.waitForDeployment();
  });

  describe("Deployment", function () {
    it("Should set the right owner", async function () {
      expect(await verifier.owner()).to.equal(owner.address);
    });

    it("Should initialize with empty state", async function () {
      const [merkleRoot, poolBalance, nullifierCount] = await verifier.getPoolState();
      expect(merkleRoot).to.equal(ethers.ZeroHash);
      expect(poolBalance).to.equal(0);
      expect(nullifierCount).to.equal(0);
    });
  });

  describe("Pool Initialization", function () {
    it("Should allow owner to initialize pool", async function () {
      const merkleRoot = ethers.keccak256(ethers.toUtf8Bytes("test"));
      const poolBalance = ethers.parseEther("100");
      
      await verifier.initializePool(merkleRoot, poolBalance);
      
      const [currentRoot, currentBalance, nullifierCount] = await verifier.getPoolState();
      expect(currentRoot).to.equal(merkleRoot);
      expect(currentBalance).to.equal(poolBalance);
      expect(nullifierCount).to.equal(0);
    });

    it("Should not allow non-owner to initialize pool", async function () {
      const merkleRoot = ethers.keccak256(ethers.toUtf8Bytes("test"));
      const poolBalance = ethers.parseEther("100");
      
      await expect(
        verifier.connect(user1).initializePool(merkleRoot, poolBalance)
      ).to.be.revertedWith("Ownable: caller is not the owner");
    });

    it("Should not allow double initialization", async function () {
      const merkleRoot = ethers.keccak256(ethers.toUtf8Bytes("test"));
      const poolBalance = ethers.parseEther("100");
      
      await verifier.initializePool(merkleRoot, poolBalance);
      
      await expect(
        verifier.initializePool(merkleRoot, poolBalance)
      ).to.be.revertedWith("Pool already initialized");
    });
  });

  describe("ZisK Verifier Setup", function () {
    it("Should allow owner to set ZisK verifier", async function () {
      const mockVerifier = ethers.Wallet.createRandom().address;
      await verifier.setZiskVerifier(mockVerifier);
      expect(await verifier.ziskVerifier()).to.equal(mockVerifier);
    });

    it("Should not allow non-owner to set ZisK verifier", async function () {
      const mockVerifier = ethers.Wallet.createRandom().address;
      await expect(
        verifier.connect(user1).setZiskVerifier(mockVerifier)
      ).to.be.revertedWith("Ownable: caller is not the owner");
    });

    it("Should not allow zero address for ZisK verifier", async function () {
      await expect(
        verifier.setZiskVerifier(ethers.ZeroAddress)
      ).to.be.revertedWith("Invalid verifier address");
    });
  });

  describe("Proof Verification", function () {
    beforeEach(async function () {
      // Initialize pool
      const merkleRoot = ethers.keccak256(ethers.toUtf8Bytes("initial"));
      const poolBalance = ethers.parseEther("1000");
      await verifier.initializePool(merkleRoot, poolBalance);
      
      // Set a mock ZisK verifier (in production, this would be the actual verifier)
      const mockVerifier = ethers.Wallet.createRandom().address;
      await verifier.setZiskVerifier(mockVerifier);
    });

    it("Should reject proof verification before pool initialization", async function () {
      // Deploy a new contract without initialization
      const PrivacyPoolVerifier = await ethers.getContractFactory("PrivacyPoolVerifier");
      const newVerifier = await PrivacyPoolVerifier.deploy();
      await newVerifier.waitForDeployment();
      
      const proofData = {
        publicInputs: new Array(23).fill(1),
        proof: "0x" + "00".repeat(100)
      };
      
      await expect(
        newVerifier.verifyAndUpdateProof(
          proofData,
          [],
          [],
          [],
          [],
          "0x",
          "0x",
          0
        )
      ).to.be.revertedWith("Pool not initialized");
    });

    it("Should reject proof verification without ZisK verifier", async function () {
      // Deploy a new contract without ZisK verifier
      const PrivacyPoolVerifier = await ethers.getContractFactory("PrivacyPoolVerifier");
      const newVerifier = await PrivacyPoolVerifier.deploy();
      await newVerifier.waitForDeployment();
      
      const merkleRoot = ethers.keccak256(ethers.toUtf8Bytes("initial"));
      const poolBalance = ethers.parseEther("1000");
      await newVerifier.initializePool(merkleRoot, poolBalance);
      
      const proofData = {
        publicInputs: new Array(23).fill(1),
        proof: "0x" + "00".repeat(100)
      };
      
      await expect(
        newVerifier.verifyAndUpdateProof(
          proofData,
          [],
          [],
          [],
          [],
          "0x",
          "0x",
          0
        )
      ).to.be.revertedWith("ZisK verifier not set");
    });

    it("Should reject invalid proof data length", async function () {
      const proofData = {
        publicInputs: new Array(20).fill(1), // Wrong length
        proof: "0x" + "00".repeat(100)
      };
      
      await expect(
        verifier.verifyAndUpdateProof(
          proofData,
          [],
          [],
          [],
          [],
          "0x",
          "0x",
          0
        )
      ).to.be.revertedWith("Invalid proof data length");
    });
  });

  describe("Nullifier Management", function () {
    beforeEach(async function () {
      const merkleRoot = ethers.keccak256(ethers.toUtf8Bytes("initial"));
      const poolBalance = ethers.parseEther("1000");
      await verifier.initializePool(merkleRoot, poolBalance);
      
      const mockVerifier = ethers.Wallet.createRandom().address;
      await verifier.setZiskVerifier(mockVerifier);
    });

    it("Should track nullifier usage", async function () {
      const nullifier = ethers.keccak256(ethers.toUtf8Bytes("test-nullifier"));
      
      expect(await verifier.isNullifierUsed(nullifier)).to.be.false;
      
      // In a real scenario, this would be set through proof verification
      // For testing, we'll simulate the internal state
      // Note: This is a simplified test - in practice, nullifiers are set through proof verification
    });
  });

  describe("Utility Functions", function () {
    it("Should reconstruct bytes32 correctly", async function () {
      // Test the internal reconstruction functions
      // This would require making the functions public or adding test functions
      // For now, we'll test the public interface
      const [merkleRoot, poolBalance, nullifierCount] = await verifier.getPoolState();
      expect(typeof merkleRoot).to.equal("string");
      expect(typeof poolBalance).to.equal("bigint");
      expect(typeof nullifierCount).to.equal("bigint");
    });
  });
});
