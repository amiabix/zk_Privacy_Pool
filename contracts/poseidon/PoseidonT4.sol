// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

/**
 * @title PoseidonT4
 * @notice Poseidon hash function for 4 inputs
 * @dev Simplified implementation for testing
 */
library PoseidonT4 {
  /**
   * @notice Hash 3 inputs using Poseidon (for compatibility)
   * @param inputs Array of 3 inputs
   * @return The hash result
   */
  function hash(uint256[3] memory inputs) internal pure returns (uint256) {
    // Simplified Poseidon implementation
    // In production, use the actual Poseidon implementation
    return uint256(keccak256(abi.encodePacked(inputs[0], inputs[1], inputs[2]))) % 21888242871839275222246405745257275088548364400416034343698204186575808495617;
  }

  /**
   * @notice Hash 4 inputs using Poseidon
   * @param inputs Array of 4 inputs
   * @return The hash result
   */
  function hash(uint256[4] memory inputs) internal pure returns (uint256) {
    // Simplified Poseidon implementation
    // In production, use the actual Poseidon implementation
    return uint256(keccak256(abi.encodePacked(inputs[0], inputs[1], inputs[2], inputs[3]))) % 21888242871839275222246405745257275088548364400416034343698204186575808495617;
  }
}
