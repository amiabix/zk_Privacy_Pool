const { ethers } = require("hardhat");
const fs = require('fs');

async function main() {
    const PrivacyPoolFixed = await ethers.getContractFactory("PrivacyPoolFixed");
    const abi = JSON.stringify(PrivacyPoolFixed.interface.format('json'), null, 2);
    
    fs.writeFileSync('PrivacyPoolFixed-ABI.json', abi);
    console.log("ABI saved to PrivacyPoolFixed-ABI.json");
}

main().catch(console.error);