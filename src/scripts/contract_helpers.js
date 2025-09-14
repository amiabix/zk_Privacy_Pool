//! Contract Helper Functions
//! 
//! Utility functions to interact with the Privacy Pool contract
//! Resolves function ambiguities and provides easy-to-use interfaces

const { ethers } = require('ethers');

/**
 * Privacy Pool Contract Helper
 * Handles function ambiguities and provides clean interfaces
 */
class PrivacyPoolHelper {
    constructor(contractAddress, contractABI, provider) {
        this.contract = new ethers.Contract(contractAddress, contractABI, provider);
    }

    /**
     * Make a simple deposit with auto-generated commitment
     * @param {string} amount - Amount in ETH (e.g., "0.1")
     * @param {object} signer - Ethers signer object
     * @returns {Promise<object>} Transaction response
     */
    async depositAutoCommitment(amount, signer) {
        const contractWithSigner = this.contract.connect(signer);
        
        // Use the parameterless deposit function explicitly
        const tx = await contractWithSigner["deposit()"]({
            value: ethers.parseEther(amount),
            gasLimit: 300000 // Set appropriate gas limit
        });
        
        return tx;
    }

    /**
     * Make a deposit with manual commitment
     * @param {string} commitment - 32-byte commitment hash (0x prefixed)
     * @param {string} amount - Amount in ETH (e.g., "0.1")
     * @param {object} signer - Ethers signer object
     * @returns {Promise<object>} Transaction response
     */
    async depositWithCommitment(commitment, amount, signer) {
        const contractWithSigner = this.contract.connect(signer);
        
        // Use the commitment-based deposit function explicitly
        const tx = await contractWithSigner["deposit(bytes32)"](commitment, {
            value: ethers.parseEther(amount),
            gasLimit: 300000
        });
        
        return tx;
    }

    /**
     * Generate a random commitment hash
     * @param {string} depositor - Depositor address
     * @param {string} amount - Amount in wei
     * @returns {string} 32-byte commitment hash
     */
    generateCommitment(depositor, amount) {
        const random = ethers.randomBytes(32);
        const timestamp = Math.floor(Date.now() / 1000);
        
        return ethers.keccak256(
            ethers.AbiCoder.defaultAbiCoder().encode(
                ["address", "uint256", "uint256", "bytes32"],
                [depositor, amount, timestamp, random]
            )
        );
    }

    /**
     * Get contract balance
     * @returns {Promise<string>} Balance in ETH
     */
    async getBalance() {
        const balance = await this.contract.getBalance();
        return ethers.formatEther(balance);
    }

    /**
     * Get current Merkle root
     * @returns {Promise<string>} Current Merkle root hash
     */
    async getMerkleRoot() {
        return await this.contract.merkleRoot();
    }

    /**
     * Check if a commitment exists
     * @param {string} commitment - Commitment hash to check
     * @returns {Promise<boolean>} True if commitment exists
     */
    async commitmentExists(commitment) {
        return await this.contract.commitments(commitment);
    }

    /**
     * Check if a nullifier has been used
     * @param {string} nullifier - Nullifier hash to check
     * @returns {Promise<boolean>} True if nullifier has been used
     */
    async nullifierUsed(nullifier) {
        return await this.contract.nullifiers(nullifier);
    }
}

/**
 * React/Frontend helper functions
 */
const ContractHelpers = {
    /**
     * Simple deposit function for React components
     * @param {object} contract - Ethers contract instance
     * @param {string} amount - Amount in ETH
     * @param {object} signer - Ethers signer
     * @returns {Promise<object>} Transaction result
     */
    async simpleDeposit(contract, amount, signer) {
        try {
            const contractWithSigner = contract.connect(signer);
            
            // Use explicit function signature to avoid ambiguity
            const tx = await contractWithSigner["deposit()"]({
                value: ethers.parseEther(amount),
                gasLimit: 300000
            });
            
            console.log("Transaction sent:", tx.hash);
            const receipt = await tx.wait();
            console.log("Transaction confirmed:", receipt.transactionHash);
            
            return {
                success: true,
                txHash: receipt.transactionHash,
                blockNumber: receipt.blockNumber
            };
        } catch (error) {
            console.error("Deposit failed:", error);
            return {
                success: false,
                error: error.message
            };
        }
    },

    /**
     * Get deposit events from the contract
     * @param {object} contract - Ethers contract instance
     * @param {number} fromBlock - Starting block number
     * @returns {Promise<Array>} Array of deposit events
     */
    async getDepositEvents(contract, fromBlock = 0) {
        const filter = contract.filters.Deposited();
        const events = await contract.queryFilter(filter, fromBlock);
        
        return events.map(event => ({
            depositor: event.args.depositor,
            commitment: event.args.commitment,
            amount: ethers.formatEther(event.args.amount),
            timestamp: event.args.timestamp.toString(),
            blockNumber: event.blockNumber,
            txHash: event.transactionHash
        }));
    }
};

module.exports = {
    PrivacyPoolHelper,
    ContractHelpers
};