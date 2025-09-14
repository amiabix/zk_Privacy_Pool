const { ethers } = require("ethers");

async function checkContract() {
  // Connect to Sepolia
  const provider = new ethers.JsonRpcProvider("https://eth-sepolia.g.alchemy.com/v2/wdp1FpAvY5GBD-wstEpHlsIY37WcgKgI");
  
  const contractAddress = "0x19B8743Df3E8997489b50F455a1cAe3536C0ee31";
  
  // Check actual ETH balance
  const balance = await provider.getBalance(contractAddress);
  console.log("💰 REAL Contract Balance:", ethers.formatEther(balance), "ETH");
  
  // Check if contract has any code
  const code = await provider.getCode(contractAddress);
  console.log("📄 Contract Code Length:", code.length);
  
  // Get recent transactions
  const blockNumber = await provider.getBlockNumber();
  console.log("🔢 Current Block:", blockNumber);
}

checkContract().catch(console.error);
