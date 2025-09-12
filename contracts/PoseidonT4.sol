// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

/**
 * @title PoseidonT4
 * @dev Poseidon hash function implementation for 4 inputs
 * @notice This is a simplified implementation. In production, use the official Poseidon library.
 */
library PoseidonT4 {
    // Simplified round constants (in production, use proper Poseidon parameters)
    uint256 constant C0 = 0x0ee9a36a1d8a1ae1e6ee7d8bc4b5e0b7ef5e2b8a7c9c0c2a5f7e3d1b8f4c6e9a;
    uint256 constant C1 = 0x1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b;
    uint256 constant C2 = 0x2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c;
    uint256 constant C3 = 0x3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1a2b3c4d;
    
    // Field modulus (BN254 scalar field)
    uint256 constant FIELD_MODULUS = 21888242871839275222246405745257275088548364400416034343698204186575808495617;
    
    /**
     * @dev Hash function for 4 inputs
     * @param inputs Array of 4 field elements
     * @return Hash result
     */
    function poseidon(uint256[4] memory inputs) internal pure returns (uint256) {
        uint256[4] memory state;
        
        // Initialize state
        for (uint i = 0; i < 4; i++) {
            state[i] = inputs[i] % FIELD_MODULUS;
        }
        
        // Simplified round function (in production, implement full Poseidon)
        for (uint round = 0; round < 8; round++) {
            // Add round constants
            state[0] = addmod(state[0], C0, FIELD_MODULUS);
            state[1] = addmod(state[1], C1, FIELD_MODULUS);
            state[2] = addmod(state[2], C2, FIELD_MODULUS);
            state[3] = addmod(state[3], C3, FIELD_MODULUS);
            
            // S-box (x^5)
            for (uint i = 0; i < 4; i++) {
                uint256 x = state[i];
                uint256 x2 = mulmod(x, x, FIELD_MODULUS);
                uint256 x4 = mulmod(x2, x2, FIELD_MODULUS);
                state[i] = mulmod(x4, x, FIELD_MODULUS);
            }
            
            // MDS matrix multiplication (simplified)
            uint256[4] memory newState;
            newState[0] = addmod(addmod(state[0], state[1], FIELD_MODULUS), 
                                addmod(state[2], state[3], FIELD_MODULUS), FIELD_MODULUS);
            newState[1] = addmod(addmod(state[0], mulmod(state[1], 2, FIELD_MODULUS), FIELD_MODULUS),
                                addmod(state[2], state[3], FIELD_MODULUS), FIELD_MODULUS);
            newState[2] = addmod(addmod(state[0], state[1], FIELD_MODULUS),
                                addmod(mulmod(state[2], 2, FIELD_MODULUS), state[3], FIELD_MODULUS), FIELD_MODULUS);
            newState[3] = addmod(addmod(state[0], state[1], FIELD_MODULUS),
                                addmod(state[2], mulmod(state[3], 2, FIELD_MODULUS), FIELD_MODULUS), FIELD_MODULUS);
            
            state = newState;
        }
        
        return state[0];
    }
    
    /**
     * @dev Hash two values
     */
    function hash(uint256 left, uint256 right) internal pure returns (uint256) {
        uint256[4] memory inputs = [left, right, 0, 0];
        return poseidon(inputs);
    }
    
    /**
     * @dev Hash four values
     */
    function hash(uint256[4] memory inputs) internal pure returns (uint256) {
        return poseidon(inputs);
    }
}