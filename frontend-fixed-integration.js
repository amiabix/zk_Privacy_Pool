// Fixed Frontend Integration for Privacy Pool
// This code uses the PrivacyPoolFixed contract with unambiguous function names

import { ethers } from 'ethers';

// Contract configuration
const CONTRACT_CONFIG = {
    // Update this address after deploying the fixed contract
    address: "YOUR_DEPLOYED_CONTRACT_ADDRESS_HERE",
    
    // Simplified ABI with only the functions we need (no ambiguity)
    abi: [
        // Deposit functions (now with different names)
        "function depositAuto() external payable",
        "function depositWithCommitment(bytes32 commitment) external payable",
        
        // Balance functions (now with different names)
        "function getContractBalance() external view returns (uint256)",
        "function getUserBalance(address account) external view returns (uint256)",
        
        // Utility functions
        "function getCurrentMerkleRoot() external view returns (bytes32)",
        "function getTotalDeposits() external view returns (uint256)",
        "function getTotalWithdrawals() external view returns (uint256)",
        "function isCommitmentUsed(bytes32 commitment) external view returns (bool)",
        "function previewCommitment(address depositor, uint256 amount) external view returns (bytes32)",
        
        // Events
        "event Deposited(address indexed depositor, bytes32 indexed commitment, uint256 value, uint256 timestamp)",
        "event MerkleRootUpdated(bytes32 indexed oldRoot, bytes32 indexed newRoot, uint256 timestamp)"
    ]
};

/**
 * Privacy Pool Client - Fixed Version
 * No more ambiguous function calls!
 */
export class PrivacyPoolClient {
    constructor(contractAddress, provider) {
        this.contractAddress = contractAddress || CONTRACT_CONFIG.address;
        this.provider = provider;
        this.contract = new ethers.Contract(this.contractAddress, CONTRACT_CONFIG.abi, provider);
    }

    /**
     * Make a simple deposit (auto-generates commitment)
     * @param {string} amount - Amount in ETH (e.g., "0.1")
     * @param {object} signer - Ethers signer object
     * @returns {Promise<object>} Transaction result
     */
    async depositSimple(amount, signer) {
        try {
            const contractWithSigner = this.contract.connect(signer);
            
            // Use the unambiguous function name
            const tx = await contractWithSigner.depositAuto({
                value: ethers.parseEther(amount),
                gasLimit: 300000
            });
            
            console.log("✅ Simple deposit transaction sent:", tx.hash);
            
            const receipt = await tx.wait();
            console.log("✅ Transaction confirmed:", receipt.transactionHash);
            
            // Extract event data
            const depositEvent = receipt.events?.find(e => e.event === 'Deposited');
            
            return {
                success: true,
                txHash: receipt.transactionHash,
                blockNumber: receipt.blockNumber,
                commitment: depositEvent?.args?.commitment || null,
                amount: ethers.formatEther(depositEvent?.args?.value || 0)
            };
            
        } catch (error) {
            console.error("❌ Simple deposit failed:", error);
            return {
                success: false,
                error: error.message
            };
        }
    }

    /**
     * Make a deposit with custom commitment
     * @param {string} commitment - 32-byte commitment hash (0x prefixed)
     * @param {string} amount - Amount in ETH (e.g., "0.1")
     * @param {object} signer - Ethers signer object
     * @returns {Promise<object>} Transaction result
     */
    async depositWithCommitment(commitment, amount, signer) {
        try {
            const contractWithSigner = this.contract.connect(signer);
            
            // Use the unambiguous function name
            const tx = await contractWithSigner.depositWithCommitment(commitment, {
                value: ethers.parseEther(amount),
                gasLimit: 300000
            });
            
            const receipt = await tx.wait();
            
            return {
                success: true,
                txHash: receipt.transactionHash,
                blockNumber: receipt.blockNumber,
                commitment: commitment,
                amount: amount
            };
            
        } catch (error) {
            console.error("❌ Commitment deposit failed:", error);
            return {
                success: false,
                error: error.message
            };
        }
    }

    /**
     * Get total contract balance
     * @returns {Promise<string>} Balance in ETH
     */
    async getContractBalance() {
        try {
            const balance = await this.contract.getContractBalance();
            return ethers.formatEther(balance);
        } catch (error) {
            console.error("❌ Failed to get contract balance:", error);
            return "0";
        }
    }

    /**
     * Get user balance (simplified implementation)
     * @param {string} userAddress - User's address
     * @returns {Promise<string>} Balance in ETH
     */
    async getUserBalance(userAddress) {
        try {
            const balance = await this.contract.getUserBalance(userAddress);
            return ethers.formatEther(balance);
        } catch (error) {
            console.error("❌ Failed to get user balance:", error);
            return "0";
        }
    }

    /**
     * Get current Merkle root
     * @returns {Promise<string>} Current Merkle root hash
     */
    async getCurrentMerkleRoot() {
        try {
            return await this.contract.getCurrentMerkleRoot();
        } catch (error) {
            console.error("❌ Failed to get Merkle root:", error);
            return "0x0";
        }
    }

    /**
     * Preview what commitment would be generated for a deposit
     * @param {string} depositorAddress - Depositor's address
     * @param {string} amount - Amount in ETH
     * @returns {Promise<string>} Preview commitment hash
     */
    async previewCommitment(depositorAddress, amount) {
        try {
            return await this.contract.previewCommitment(
                depositorAddress,
                ethers.parseEther(amount)
            );
        } catch (error) {
            console.error("❌ Failed to preview commitment:", error);
            return "0x0";
        }
    }

    /**
     * Check if a commitment is already used
     * @param {string} commitment - Commitment hash to check
     * @returns {Promise<boolean>} True if commitment is used
     */
    async isCommitmentUsed(commitment) {
        try {
            return await this.contract.isCommitmentUsed(commitment);
        } catch (error) {
            console.error("❌ Failed to check commitment:", error);
            return false;
        }
    }

    /**
     * Listen for deposit events
     * @param {function} callback - Callback function for new deposits
     */
    async listenForDeposits(callback) {
        this.contract.on("Deposited", (depositor, commitment, value, timestamp, event) => {
            callback({
                depositor,
                commitment,
                amount: ethers.formatEther(value),
                timestamp: timestamp.toNumber(),
                blockNumber: event.blockNumber,
                txHash: event.transactionHash
            });
        });
    }
}

// React Hook for Privacy Pool
export const usePrivacyPool = (contractAddress, provider, signer) => {
    const [client, setClient] = React.useState(null);
    const [loading, setLoading] = React.useState(false);
    const [balance, setBalance] = React.useState("0");

    React.useEffect(() => {
        if (provider) {
            const newClient = new PrivacyPoolClient(contractAddress, provider);
            setClient(newClient);
        }
    }, [contractAddress, provider]);

    const deposit = async (amount) => {
        if (!client || !signer) return { success: false, error: "No client or signer" };
        
        setLoading(true);
        try {
            const result = await client.depositSimple(amount, signer);
            if (result.success) {
                // Refresh balance after successful deposit
                const newBalance = await client.getContractBalance();
                setBalance(newBalance);
            }
            return result;
        } finally {
            setLoading(false);
        }
    };

    const refreshBalance = async () => {
        if (!client) return;
        
        try {
            const contractBalance = await client.getContractBalance();
            setBalance(contractBalance);
        } catch (error) {
            console.error("Failed to refresh balance:", error);
        }
    };

    return {
        client,
        deposit,
        balance,
        loading,
        refreshBalance
    };
};

// Usage Example for React Component:
/*
function DepositComponent() {
    const { deposit, balance, loading, refreshBalance } = usePrivacyPool(
        CONTRACT_CONFIG.address,
        provider,
        signer
    );
    
    const handleDeposit = async () => {
        const result = await deposit("0.1");
        
        if (result.success) {
            alert(`Deposit successful! TX: ${result.txHash}`);
        } else {
            alert(`Deposit failed: ${result.error}`);
        }
    };
    
    return (
        <div>
            <p>Pool Balance: {balance} ETH</p>
            <button onClick={handleDeposit} disabled={loading}>
                {loading ? "Depositing..." : "Deposit 0.1 ETH"}
            </button>
            <button onClick={refreshBalance}>Refresh Balance</button>
        </div>
    );
}
*/