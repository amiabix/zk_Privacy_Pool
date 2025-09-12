// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

/**
 * @title IEntrypoint
 * @notice Interface for Entrypoint contract
 */
interface IEntrypoint {
  /**
   * @notice Get the latest root
   * @return The latest root
   */
  function latestRoot() external view returns (uint256);
}
