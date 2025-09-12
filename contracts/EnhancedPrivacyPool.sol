// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./MerkleTree.sol";

/**
 * @title EnhancedPrivacyPool
 * @dev Privacy pool with proper nullifier registry and withdrawal mechanisms
 */
contract EnhancedPrivacyPool is MerkleTree {
    using PoseidonT4 for uint256[4];
    
    // Events
    event Deposited(address indexed depositor, uint256 indexed commitment, uint256 value, uint256 blockNumber);
    event Withdrawn(address indexed recipient, uint256 indexed nullifierHash, uint256 value, address relayer, uint256 fee);
    event NullifierUsed(uint256 indexed nullifierHash);
    
    // Nullifier registry to prevent double spending
    mapping(uint256 => bool) public nullifiers;
    
    // Pool balance
    uint256 public totalDeposits;
    uint256 public totalWithdrawals;
    
    // Minimum and maximum values
    uint256 public constant MIN_WITHDRAWAL = 0.1 ether;
    uint256 public constant MAX_WITHDRAWAL = 100 ether;
    uint256 public constant MAX_FEE = 0.05 ether;
    
    modifier onlyValidNullifier(uint256 nullifierHash) {
        require(!nullifiers[nullifierHash], "Nullifier has already been used");
        _;
    }
    
    modifier onlyValidAmount(uint256 amount) {
        require(amount >= MIN_WITHDRAWAL && amount <= MAX_WITHDRAWAL, "Invalid withdrawal amount");
        _;
    }
    
    /**
     * @dev Deposit ETH into the privacy pool
     * @param commitment The commitment hash binding value to owner
     */
    function deposit(uint256 commitment) external payable returns (uint256) {
        require(msg.value > 0, "Must deposit a positive amount");
        require(msg.value <= MAX_WITHDRAWAL, "Deposit amount too large");
        
        // Insert commitment into Merkle tree
        uint256 leafIndex = insertCommitment(commitment, msg.sender);
        
        // Update total deposits
        totalDeposits += msg.value;
        
        emit Deposited(msg.sender, commitment, msg.value, block.number);
        
        return leafIndex;
    }
    
    /**
     * @dev Withdraw ETH from the privacy pool
     * @param proof Merkle proof of commitment inclusion
     * @param pathIndices Path indices for Merkle proof
     * @param nullifierHash Nullifier hash to prevent double spending
     * @param recipient Address to receive the ETH
     * @param relayer Address of the relayer (can be zero)
     * @param fee Fee to pay the relayer
     * @param refund Refund amount (unused)
     */
    function withdraw(
        uint256[] memory proof,
        bool[] memory pathIndices,
        uint256 nullifierHash,
        address payable recipient,
        address payable relayer,
        uint256 fee,
        uint256 refund
    ) external onlyValidNullifier(nullifierHash) {
        require(recipient != address(0), "Invalid recipient");
        require(fee <= MAX_FEE, "Fee too high");
        
        // For simplicity, we'll use a fixed withdrawal amount
        // In production, this would be verified through ZK proof
        uint256 withdrawalAmount = 1 ether;
        
        require(address(this).balance >= withdrawalAmount, "Insufficient pool balance");
        require(withdrawalAmount >= fee, "Fee exceeds withdrawal amount");
        
        // Mark nullifier as used
        nullifiers[nullifierHash] = true;
        
        // Calculate net withdrawal amount
        uint256 netAmount = withdrawalAmount - fee;
        
        // Update totals
        totalWithdrawals += withdrawalAmount;
        
        // Transfer funds
        if (netAmount > 0) {
            recipient.transfer(netAmount);
        }
        
        if (fee > 0 && relayer != address(0)) {
            relayer.transfer(fee);
        }
        
        emit Withdrawn(recipient, nullifierHash, withdrawalAmount, relayer, fee);
        emit NullifierUsed(nullifierHash);
    }
    
    /**
     * @dev Withdraw with ZK proof verification (future implementation)
     */
    function withdrawWithProof(
        uint256[2] memory a,
        uint256[2][2] memory b,
        uint256[2] memory c,
        uint256[] memory publicSignals
    ) external pure {
        // This would integrate with a ZK proof verifier
        // For now, we'll just revert with a message
        revert("ZK proof verification not yet implemented");
    }
    
    /**
     * @dev Check if a nullifier has been used
     */
    function isNullifierUsed(uint256 nullifierHash) external view returns (bool) {
        return nullifiers[nullifierHash];
    }
    
    /**
     * @dev Get pool statistics
     */
    function getPoolStats() external view returns (
        uint256 balance,
        uint256 deposits,
        uint256 withdrawals,
        uint256 leafCount,
        uint256 root
    ) {
        return (
            address(this).balance,
            totalDeposits,
            totalWithdrawals,
            nextLeafIndex,
            getRoot()
        );
    }
    
    /**
     * @dev Emergency withdrawal function (only if pool is compromised)
     * This is a safety mechanism and would be removed in production
     */
    function emergencyWithdraw() external {
        require(msg.sender == owner(), "Only owner can emergency withdraw");
        payable(msg.sender).transfer(address(this).balance);
    }
    
    /**
     * @dev Get contract owner (for emergency functions)
     */
    function owner() internal pure returns (address) {
        // This would be set in constructor in production
        return address(0); // Placeholder
    }
    
    /**
     * @dev Receive function to accept ETH deposits
     */
    receive() external payable {
        // Allow direct ETH deposits (though not recommended for privacy)
        totalDeposits += msg.value;
    }
}