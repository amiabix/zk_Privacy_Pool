// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {ProofLib} from '../lib/ProofLib.sol';

/**
 * @title IPrivacyPool
 * @notice Interface for Privacy Pool operations
 */
interface IPrivacyPool {
  /**
   * @notice Emitted when a user deposits funds
   * @param depositor The address of the depositor
   * @param commitment The commitment hash
   * @param label The label for the commitment
   * @param value The amount deposited
   * @param precommitmentHash The precommitment hash
   */
  event Deposited(
    address indexed depositor,
    uint256 indexed commitment,
    uint256 indexed label,
    uint256 value,
    uint256 precommitmentHash
  );

  /**
   * @notice Emitted when a user withdraws funds
   * @param processooor The address of the processooor
   * @param value The amount withdrawn
   * @param nullifierHash The nullifier hash
   * @param newCommitmentHash The new commitment hash
   */
  event Withdrawn(
    address indexed processooor,
    uint256 value,
    uint256 nullifierHash,
    uint256 newCommitmentHash
  );

  /**
   * @notice Emitted when a user ragequits
   * @param ragequitter The address of the ragequitter
   * @param commitmentHash The commitment hash
   * @param label The label
   * @param value The amount ragequit
   */
  event Ragequit(
    address indexed ragequitter,
    uint256 commitmentHash,
    uint256 label,
    uint256 value
  );

  /**
   * @notice Emitted when the pool dies
   */
  event PoolDied();

  /**
   * @notice Deposits funds into the privacy pool
   * @param _depositor The address of the depositor
   * @param _value The amount to deposit
   * @param _precommitmentHash The precommitment hash
   * @return _commitment The commitment hash
   */
  function deposit(
    address _depositor,
    uint256 _value,
    uint256 _precommitmentHash
  ) external payable returns (uint256 _commitment);

  /**
   * @notice Withdraws funds from the privacy pool
   * @param _withdrawal The withdrawal data
   * @param _proof The withdrawal proof
   */
  function withdraw(
    Withdrawal memory _withdrawal,
    ProofLib.WithdrawProof memory _proof
  ) external;

  /**
   * @notice Ragequits from the privacy pool
   * @param _proof The ragequit proof
   */
  function ragequit(ProofLib.RagequitProof memory _proof) external;

  /**
   * @notice Winds down the pool
   */
  function windDown() external;

  /**
   * @notice Withdrawal data structure
   */
  struct Withdrawal {
    address processooor;
  }
}
