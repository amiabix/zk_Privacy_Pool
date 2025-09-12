// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "./PoseidonT4.sol";

/**
 * @title MerkleTree
 * @dev On-chain Merkle tree for commitment tracking
 * @notice Maintains a sparse Merkle tree for efficient commitment storage and verification
 */
contract MerkleTree {
    using PoseidonT4 for uint256[4];

    // Tree configuration
    uint256 public constant TREE_DEPTH = 32;
    uint256 public constant MAX_LEAVES = 2**TREE_DEPTH;
    
    // Tree state
    uint256 public nextLeafIndex;
    mapping(uint256 => uint256) public tree;
    mapping(uint256 => bool) public filledSubtrees;
    mapping(uint256 => bool) public zeros;
    
    // Commitment tracking
    mapping(uint256 => bool) public commitments;
    mapping(uint256 => address) public commitmentToOwner;
    mapping(address => uint256[]) public ownerToCommitments;
    
    // Events
    event LeafInserted(uint256 indexed leafIndex, uint256 indexed commitment, address indexed owner);
    event TreeUpdated(uint256 indexed newRoot, uint256 indexed leafCount);
    
    // Zero values for each level
    uint256[TREE_DEPTH] public ZEROS;
    
    constructor() {
        // Initialize zero values
        ZEROS[0] = 0;
        for (uint256 i = 1; i < TREE_DEPTH; i++) {
            ZEROS[i] = hashLeftRight(ZEROS[i-1], ZEROS[i-1]);
        }
    }
    
    /**
     * @dev Insert a commitment into the Merkle tree
     * @param commitment The commitment hash to insert
     * @param owner The owner of the commitment
     * @return leafIndex The index where the commitment was inserted
     */
    function insertCommitment(uint256 commitment, address owner) external returns (uint256) {
        require(nextLeafIndex < MAX_LEAVES, "MerkleTree: Tree is full");
        require(!commitments[commitment], "MerkleTree: Commitment already exists");
        
        uint256 leafIndex = nextLeafIndex;
        
        // Store commitment and owner mapping
        commitments[commitment] = true;
        commitmentToOwner[commitment] = owner;
        ownerToCommitments[owner].push(commitment);
        
        // Insert into tree
        _insertLeaf(commitment, leafIndex);
        
        nextLeafIndex++;
        
        emit LeafInserted(leafIndex, commitment, owner);
        emit TreeUpdated(getRoot(), nextLeafIndex);
        
        return leafIndex;
    }
    
    /**
     * @dev Check if a commitment exists in the tree
     * @param commitment The commitment hash to check
     * @return exists True if the commitment exists
     */
    function hasCommitment(uint256 commitment) external view returns (bool) {
        return commitments[commitment];
    }
    
    /**
     * @dev Get the owner of a commitment
     * @param commitment The commitment hash
     * @return owner The owner address
     */
    function getCommitmentOwner(uint256 commitment) external view returns (address) {
        require(commitments[commitment], "MerkleTree: Commitment does not exist");
        return commitmentToOwner[commitment];
    }
    
    /**
     * @dev Get all commitments for an owner
     * @param owner The owner address
     * @return commitmentList Array of commitment hashes
     */
    function getOwnerCommitments(address owner) external view returns (uint256[] memory) {
        return ownerToCommitments[owner];
    }
    
    /**
     * @dev Get the current Merkle root
     * @return root The current root hash
     */
    function getRoot() public view returns (uint256) {
        if (nextLeafIndex == 0) {
            return ZEROS[TREE_DEPTH - 1];
        }
        return tree[0];
    }
    
    /**
     * @dev Generate a Merkle proof for a leaf
     * @param leafIndex The index of the leaf
     * @return proof Array of sibling hashes
     * @return path Array of path indices
     */
    function getMerkleProof(uint256 leafIndex) external view returns (uint256[] memory proof, bool[] memory path) {
        require(leafIndex < nextLeafIndex, "MerkleTree: Leaf index out of bounds");
        
        proof = new uint256[](TREE_DEPTH);
        path = new bool[](TREE_DEPTH);
        
        uint256 currentIndex = leafIndex;
        
        for (uint256 i = 0; i < TREE_DEPTH; i++) {
            bool isRight = currentIndex % 2 == 1;
            uint256 siblingIndex = isRight ? currentIndex - 1 : currentIndex + 1;
            
            if (siblingIndex < nextLeafIndex) {
                proof[i] = tree[siblingIndex];
            } else {
                proof[i] = ZEROS[i];
            }
            
            path[i] = isRight;
            currentIndex /= 2;
        }
    }
    
    /**
     * @dev Verify a Merkle proof
     * @param leaf The leaf hash
     * @param proof Array of sibling hashes
     * @param path Array of path indices
     * @return valid True if the proof is valid
     */
    function verifyMerkleProof(
        uint256 leaf,
        uint256[] memory proof,
        bool[] memory path
    ) external view returns (bool) {
        require(proof.length == TREE_DEPTH, "MerkleTree: Invalid proof length");
        require(path.length == TREE_DEPTH, "MerkleTree: Invalid path length");
        
        uint256 current = leaf;
        
        for (uint256 i = 0; i < TREE_DEPTH; i++) {
            if (path[i]) {
                current = hashLeftRight(proof[i], current);
            } else {
                current = hashLeftRight(current, proof[i]);
            }
        }
        
        return current == getRoot();
    }
    
    /**
     * @dev Get tree statistics
     * @return leafCount Number of leaves in the tree
     * @return root Current root hash
     * @return depth Tree depth
     */
    function getTreeStats() external view returns (uint256 leafCount, uint256 root, uint256 depth) {
        return (nextLeafIndex, getRoot(), TREE_DEPTH);
    }
    
    /**
     * @dev Internal function to insert a leaf into the tree
     * @param leaf The leaf hash
     * @param leafIndex The index of the leaf
     */
    function _insertLeaf(uint256 leaf, uint256 leafIndex) internal {
        uint256 currentIndex = leafIndex;
        uint256 currentLevel = 0;
        
        // Insert leaf at the bottom level
        tree[leafIndex] = leaf;
        
        // Update tree bottom-up
        while (currentLevel < TREE_DEPTH - 1) {
            bool isRight = currentIndex % 2 == 1;
            uint256 siblingIndex = isRight ? currentIndex - 1 : currentIndex + 1;
            
            uint256 left = isRight ? tree[siblingIndex] : tree[currentIndex];
            uint256 right = isRight ? tree[currentIndex] : tree[siblingIndex];
            
            uint256 parentIndex = currentIndex / 2;
            tree[parentIndex] = hashLeftRight(left, right);
            
            currentIndex = parentIndex;
            currentLevel++;
        }
    }
    
    /**
     * @dev Hash two values together
     * @param left Left value
     * @param right Right value
     * @return hash The hash of the two values
     */
    function hashLeftRight(uint256 left, uint256 right) internal pure returns (uint256) {
        uint256[4] memory inputs = [left, right, 0, 0];
        return PoseidonT4.poseidon(inputs);
    }
}
