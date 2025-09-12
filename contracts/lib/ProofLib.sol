// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

/**
 * @title ProofLib
 * @notice Library for handling ZK proofs
 */
library ProofLib {
  /**
   * @notice Withdrawal proof structure
   */
  struct WithdrawProof {
    uint256[2] pA;
    uint256[2][2] pB;
    uint256[2] pC;
    uint256[] pubSignals;
  }

  /**
   * @notice Ragequit proof structure
   */
  struct RagequitProof {
    uint256[2] pA;
    uint256[2][2] pB;
    uint256[2] pC;
    uint256[] pubSignals;
  }

  /**
   * @notice Get the context from withdrawal proof
   * @param _proof The withdrawal proof
   * @return The context value
   */
  function context(WithdrawProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[5];
  }

  /**
   * @notice Get the state root from withdrawal proof
   * @param _proof The withdrawal proof
   * @return The state root
   */
  function stateRoot(WithdrawProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[1];
  }

  /**
   * @notice Get the state tree depth from withdrawal proof
   * @param _proof The withdrawal proof
   * @return The state tree depth
   */
  function stateTreeDepth(WithdrawProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[2];
  }

  /**
   * @notice Get the ASP root from withdrawal proof
   * @param _proof The withdrawal proof
   * @return The ASP root
   */
  function ASPRoot(WithdrawProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[3];
  }

  /**
   * @notice Get the ASP tree depth from withdrawal proof
   * @param _proof The withdrawal proof
   * @return The ASP tree depth
   */
  function ASPTreeDepth(WithdrawProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[4];
  }

  /**
   * @notice Get the withdrawn value from withdrawal proof
   * @param _proof The withdrawal proof
   * @return The withdrawn value
   */
  function withdrawnValue(WithdrawProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[0];
  }

  /**
   * @notice Get the existing nullifier hash from withdrawal proof
   * @param _proof The withdrawal proof
   * @return The existing nullifier hash
   */
  function existingNullifierHash(WithdrawProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[6];
  }

  /**
   * @notice Get the new commitment hash from withdrawal proof
   * @param _proof The withdrawal proof
   * @return The new commitment hash
   */
  function newCommitmentHash(WithdrawProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[7];
  }

  /**
   * @notice Get the label from ragequit proof
   * @param _proof The ragequit proof
   * @return The label
   */
  function label(RagequitProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[0];
  }

  /**
   * @notice Get the commitment hash from ragequit proof
   * @param _proof The ragequit proof
   * @return The commitment hash
   */
  function commitmentHash(RagequitProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[1];
  }

  /**
   * @notice Get the value from ragequit proof
   * @param _proof The ragequit proof
   * @return The value
   */
  function value(RagequitProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[2];
  }

  /**
   * @notice Get the nullifier hash from ragequit proof
   * @param _proof The ragequit proof
   * @return The nullifier hash
   */
  function nullifierHash(RagequitProof memory _proof) internal pure returns (uint256) {
    return _proof.pubSignals[3];
  }
}
