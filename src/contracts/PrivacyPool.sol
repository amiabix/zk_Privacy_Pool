// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract PrivacyPool is ReentrancyGuard, Ownable {
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
     * @dev Deposit ETH and create a commitment
     * @param commitment The commitment hash for the deposit
     */
    function deposit(bytes32 commitment) external payable nonReentrant {
        require(msg.value > 0, "Deposit amount must be greater than 0");
        require(!commitments[commitment], "Commitment already exists");
        
        commitments[commitment] = true;
        totalDeposits += msg.value;
        
        // Update merkle root (simplified - in production, this would be done by a relayer)
        bytes32 oldRoot = merkleRoot;
        merkleRoot = keccak256(abi.encodePacked(oldRoot, commitment, block.timestamp));
        
        emit Deposited(msg.sender, commitment, msg.value, block.timestamp);
        emit MerkleRootUpdated(oldRoot, merkleRoot, block.timestamp);
    }
    
    /**
     * @dev Simple deposit function for frontend (generates commitment automatically)
     */
    function deposit() external payable nonReentrant {
        require(msg.value > 0, "Deposit amount must be greater than 0");
        
        // Generate commitment from depositor address and amount
        bytes32 commitment = keccak256(abi.encodePacked(msg.sender, msg.value, block.timestamp, block.number));
        require(!commitments[commitment], "Commitment collision - please try again");
        
        commitments[commitment] = true;
        totalDeposits += msg.value;
        
        // Update merkle root (simplified - in production, this would be done by a relayer)
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
     * @dev Get contract balance
     */
    function getBalance() external view returns (uint256) {
        return address(this).balance;
    }
    
    /**
     * @dev Get balance for a specific account (for frontend compatibility)
     */
    function getBalance(address account) external view returns (uint256) {
        // This is a simplified implementation - in production, you'd track individual balances
        // For now, return 0 as we don't track individual balances in this simple implementation
        return 0;
    }
    
    /**
     * @dev Check if a commitment exists
     */
    function isCommitmentUsed(bytes32 commitment) external view returns (bool) {
        return commitments[commitment];
    }
    
    /**
     * @dev Check if a nullifier has been used
     */
    function isNullifierUsed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }
    
    /**
     * @dev Emergency function to withdraw all funds (only owner)
     */
    function emergencyWithdraw() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "No funds to withdraw");
        
        payable(owner()).transfer(balance);
    }
}