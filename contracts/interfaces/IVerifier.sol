// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

/**
 * @title IVerifier
 * @notice Interface for ZK proof verifiers
 */
interface IVerifier {
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
  ) external view returns (bool);
}
