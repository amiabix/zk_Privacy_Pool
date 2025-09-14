const { ethers } = require("hardhat");

async function main() {
  console.log("🔍 Checking actual contract balance...");
  
  const contractAddress = "0x19B8743Df3E8997489b50F455a1cAe3536C0ee31";
  const [deployer] = await ethers.getSigners();
  
  console.log("👤 Checking with account:", deployer.address);
  
  // Get contract instance
  const PrivacyPoolFixed = await ethers.getContractFactory("PrivacyPoolFixed");
  const contract = PrivacyPoolFixed.attach(contractAddress);
  
  // Check actual ETH balance of contract
  const contractBalance = await ethers.provider.getBalance(contractAddress);
  console.log("💰 Contract ETH Balance:", ethers.formatEther(contractBalance), "ETH");
  
  // Check contract's internal balance tracking
  const totalDeposits = await contract.totalDeposits();
  console.log("📊 Total Deposits (contract):", ethers.formatEther(totalDeposits), "ETH");
  
  // Check user's balance from contract
  const userBalance = await contract.getUserBalance(deployer.address);
  console.log("👤 User Balance (contract):", ethers.formatEther(userBalance), "ETH");
  
  // Check merkle root
  const merkleRoot = await contract.merkleRoot();
  console.log("🌳 Merkle Root:", merkleRoot);
  
  console.log("\n🎯 SUMMARY:");
  console.log("- ETH in contract:", ethers.formatEther(contractBalance), "ETH");
  console.log("- Contract tracking:", ethers.formatEther(totalDeposits), "ETH");
  console.log("- Your balance:", ethers.formatEther(userBalance), "ETH");
}

main().catch(console.error);
