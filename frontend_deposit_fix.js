// Frontend Deposit Fix for Ambiguous Function Error
// Copy this code into your React component

import { ethers } from 'ethers';

// Fixed deposit function that resolves the ambiguity
export const makeDeposit = async (amount, signer, contractAddress, contractABI) => {
    try {
        // Create contract instance
        const contract = new ethers.Contract(contractAddress, contractABI, signer);
        
        // SOLUTION: Use explicit function signature to avoid ambiguity
        // Instead of: contract.deposit()
        // Use: contract["deposit()"] for the parameterless version
        
        const tx = await contract["deposit()"]({
            value: ethers.parseEther(amount), // Convert ETH to wei
            gasLimit: 300000 // Explicit gas limit
        });
        
        console.log(" Transaction sent:", tx.hash);
        
        // Wait for transaction confirmation
        const receipt = await tx.wait();
        console.log(" Transaction confirmed:", receipt.transactionHash);
        
        return {
            success: true,
            txHash: receipt.transactionHash,
            blockNumber: receipt.blockNumber
        };
        
    } catch (error) {
        console.error(" Deposit failed:", error);
        
        // Handle specific error types
        if (error.code === 'INVALID_ARGUMENT') {
            console.error("Ambiguous function - use explicit signature");
        }
        
        return {
            success: false,
            error: error.message
        };
    }
};

// Alternative: If you want to provide your own commitment
export const makeDepositWithCommitment = async (commitment, amount, signer, contractAddress, contractABI) => {
    try {
        const contract = new ethers.Contract(contractAddress, contractABI, signer);
        
        // Use the commitment-based deposit function explicitly
        const tx = await contract["deposit(bytes32)"](commitment, {
            value: ethers.parseEther(amount),
            gasLimit: 300000
        });
        
        const receipt = await tx.wait();
        
        return {
            success: true,
            txHash: receipt.transactionHash,
            blockNumber: receipt.blockNumber
        };
        
    } catch (error) {
        console.error(" Deposit with commitment failed:", error);
        return {
            success: false,
            error: error.message
        };
    }
};

// Helper to generate a random commitment if needed
export const generateRandomCommitment = (depositorAddress, amount) => {
    const random = ethers.randomBytes(32);
    const timestamp = Math.floor(Date.now() / 1000);
    
    return ethers.keccak256(
        ethers.AbiCoder.defaultAbiCoder().encode(
            ["address", "uint256", "uint256", "bytes32"],
            [depositorAddress, ethers.parseEther(amount), timestamp, random]
        )
    );
};

// Usage example in your React component:
/*
const handleDeposit = async () => {
    const amount = "0.1"; // 0.1 ETH
    const result = await makeDeposit(amount, signer, contractAddress, contractABI);
    
    if (result.success) {
        alert(`Deposit successful! TX: ${result.txHash}`);
    } else {
        alert(`Deposit failed: ${result.error}`);
    }
};
*/