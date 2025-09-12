// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

/**
 * @title Verifier
 * @notice Mock verifier for testing purposes
 * @dev In production, this would be the actual Groth16 verifier
 */
contract Verifier {
  /**
   * @notice Verify a proof
   * @param _pA The proof A
   * @param _pB The proof B
   * @param _pC The proof C
   * @param _pubSignals The public signals
   * @return Whether the proof is valid
   */
  function verifyProof(
    uint256[2] memory _pA,
    uint256[2][2] memory _pB,
    uint256[2] memory _pC,
    uint256[] memory _pubSignals
  ) external pure returns (bool) {
    // Mock implementation - always returns true for testing
    // In production, this would verify the actual Groth16 proof
    return true;
  }
}
