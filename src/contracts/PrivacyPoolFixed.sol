// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/**
 * @title Privacy Pool Contract (Fixed Version)
 * @dev Privacy pool with unambiguous function names
 * @notice This version fixes function name collisions by using distinct names
 */
contract PrivacyPoolFixed is ReentrancyGuard, Ownable {
    // Events
    event Deposited(address indexed depositor, bytes32 indexed commitment, uint256 value, uint256 timestamp);
    event Withdrawn(address indexed recipient, bytes32 indexed nullifier, uint256 value, uint256 timestamp);
    event MerkleRootUpdated(bytes32 indexed oldRoot, bytes32 indexed newRoot, uint256 timestamp);
    
    // State variables
    mapping(bytes32 => bool) public commitments;
    mapping(bytes32 => bool) public nullifiers;
    bytes32 public merkleRoot;
    uint256 public totalDeposits;
    uint256 public totalWithdrawals;
    
    // Merkle tree depth (32 levels)
    uint256 constant TREE_DEPTH = 32;
    
    constructor() Ownable(msg.sender) {
        // Initialize with empty merkle root
        merkleRoot = keccak256(abi.encodePacked("PRIVACY_POOL_INIT"));
    }
    
    /**
     * @dev Deposit ETH with a specific commitment hash
     * @param commitment The commitment hash for the deposit
     */
    function depositWithCommitment(bytes32 commitment) external payable nonReentrant {
        require(msg.value > 0, "Deposit amount must be greater than 0");
        require(!commitments[commitment], "Commitment already exists");
        
        commitments[commitment] = true;
        totalDeposits += msg.value;
        
        // Update merkle root (simplified - in this would be done by a relayer)
        bytes32 oldRoot = merkleRoot;
        merkleRoot = keccak256(abi.encodePacked(oldRoot, commitment, block.timestamp));
        
        emit Deposited(msg.sender, commitment, msg.value, block.timestamp);
        emit MerkleRootUpdated(oldRoot, merkleRoot, block.timestamp);
    }
    
    /**
     * @dev Simple deposit function with auto-generated commitment
     * @notice This function automatically generates a commitment for the user
     */
    function depositAuto() external payable nonReentrant {
        require(msg.value > 0, "Deposit amount must be greater than 0");
        
        // Generate commitment from depositor address, amount, timestamp, and block number
        bytes32 commitment = keccak256(abi.encodePacked(
            msg.sender, 
            msg.value, 
            block.timestamp, 
            block.number,
            blockhash(block.number - 1) // Add some randomness
        ));
        
        // Ensure no collision (very unlikely but good practice)
        require(!commitments[commitment], "Commitment collision - please try again");
        
        commitments[commitment] = true;
        totalDeposits += msg.value;
        
        // Update merkle root
        bytes32 oldRoot = merkleRoot;
        merkleRoot = keccak256(abi.encodePacked(oldRoot, commitment, block.timestamp));
        
        emit Deposited(msg.sender, commitment, msg.value, block.timestamp);
        emit MerkleRootUpdated(oldRoot, merkleRoot, block.timestamp);
    }
    
    /**
     * @dev Withdraw ETH using a nullifier
     * @param nullifier The nullifier to prevent double-spending
     * @param recipient The address to receive the funds
     * @param amount The amount to withdraw
     */
    function withdraw(
        bytes32 nullifier,
        address recipient,
        uint256 amount
    ) external nonReentrant {
        require(!nullifiers[nullifier], "Nullifier already used");
        require(amount > 0, "Withdrawal amount must be greater than 0");
        require(amount <= address(this).balance, "Insufficient contract balance");
        
        nullifiers[nullifier] = true;
        totalWithdrawals += amount;
        
        // Transfer funds
        payable(recipient).transfer(amount);
        
        emit Withdrawn(recipient, nullifier, amount, block.timestamp);
    }
    
    /**
     * @dev Get total contract balance
     */
    function getContractBalance() external view returns (uint256) {
        return address(this).balance;
    }
    
    /**
     * @dev Get user balance (simplified - returns 0 for now)
     * @param account The account to check balance for
     * @notice In a full implementation, this would track individual UTXO balances
     */
    function getUserBalance(address account) external view returns (uint256) {
        // This is a simplified implementation
        // In production, you'd track individual balances through UTXOs
        return 0;
    }
    
    /**
     * @dev Check if a commitment exists
     * @param commitment The commitment to check
     */
    function isCommitmentUsed(bytes32 commitment) external view returns (bool) {
        return commitments[commitment];
    }
    
    /**
     * @dev Check if a nullifier has been used
     * @param nullifier The nullifier to check
     */
    function isNullifierUsed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }
    
    /**
     * @dev Get current merkle root
     */
    function getCurrentMerkleRoot() external view returns (bytes32) {
        return merkleRoot;
    }
    
    /**
     * @dev Get total number of deposits
     */
    function getTotalDeposits() external view returns (uint256) {
        return totalDeposits;
    }
    
    /**
     * @dev Get total number of withdrawals
     */
    function getTotalWithdrawals() external view returns (uint256) {
        return totalWithdrawals;
    }
    
    /**
     * @dev Emergency function to withdraw all funds (only owner)
     */
    function emergencyWithdraw() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "No funds to withdraw");
        
        payable(owner()).transfer(balance);
    }
    
    /**
     * @dev Generate a commitment preview (for frontend)
     * @param depositor The depositor address
     * @param amount The deposit amount
     * @return The commitment that would be generated
     */
    function previewCommitment(address depositor, uint256 amount) external view returns (bytes32) {
        return keccak256(abi.encodePacked(
            depositor,
            amount,
            block.timestamp,
            block.number,
            blockhash(block.number - 1)
        ));
    }
}