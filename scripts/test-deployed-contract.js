const { ethers } = require("hardhat");

async function main() {
    console.log("🧪 Testing deployed PrivacyPoolFixed contract...");
    
    const contractAddress = "0x19B8743Df3E8997489b50F455a1cAe3536C0ee31";
    const [signer] = await ethers.getSigners();
    
    // Connect to deployed contract
    const PrivacyPoolFixed = await ethers.getContractFactory("PrivacyPoolFixed");
    const contract = PrivacyPoolFixed.attach(contractAddress);
    
    console.log("📍 Contract Address:", contractAddress);
    console.log("👤 Testing with account:", signer.address);
    
    try {
        // Test view functions
        console.log("\n📊 Testing view functions...");
        
        const merkleRoot = await contract.getCurrentMerkleRoot();
        console.log("✅ Merkle Root:", merkleRoot);
        
        const contractBalance = await contract.getContractBalance();
        console.log("✅ Contract Balance:", ethers.formatEther(contractBalance), "ETH");
        
        const totalDeposits = await contract.getTotalDeposits();
        console.log("✅ Total Deposits:", ethers.formatEther(totalDeposits), "ETH");
        
        // Test function signatures (should not be ambiguous)
        console.log("\n🔍 Testing function signatures...");
        
        // This should work - no ambiguity
        const depositAutoFragment = contract.interface.getFunction("depositAuto");
        console.log("✅ depositAuto() signature:", depositAutoFragment.format());
        
        const depositWithCommitmentFragment = contract.interface.getFunction("depositWithCommitment");
        console.log("✅ depositWithCommitment(bytes32) signature:", depositWithCommitmentFragment.format());
        
        const getContractBalanceFragment = contract.interface.getFunction("getContractBalance");
        console.log("✅ getContractBalance() signature:", getContractBalanceFragment.format());
        
        const getUserBalanceFragment = contract.interface.getFunction("getUserBalance");
        console.log("✅ getUserBalance(address) signature:", getUserBalanceFragment.format());
        
        // Test preview commitment
        const previewCommitment = await contract.previewCommitment(
            signer.address,
            ethers.parseEther("0.1")
        );
        console.log("✅ Preview commitment:", previewCommitment);
        
        // Test commitment usage check
        const isUsed = await contract.isCommitmentUsed(previewCommitment);
        console.log("✅ Is preview commitment used:", isUsed);
        
        console.log("\n🎉 All tests passed! Contract is working correctly.");
        console.log("\n📱 Frontend Integration:");
        console.log("   Contract Address: " + contractAddress);
        console.log("   Use: contract.depositAuto({ value: ethers.parseEther('0.1') })");
        console.log("   Use: contract.getContractBalance()");
        
    } catch (error) {
        console.error("❌ Test failed:", error.message);
        throw error;
    }
}

main()
    .then(() => {
        console.log("\n✅ Contract testing completed successfully!");
        process.exit(0);
    })
    .catch((error) => {
        console.error("❌ Contract testing failed:", error);
        process.exit(1);
    });