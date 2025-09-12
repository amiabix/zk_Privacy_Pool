// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

/**
 * @title Entrypoint
 * @notice Entrypoint contract for managing privacy pools
 */
contract Entrypoint {
  /// @notice The latest ASP root
  uint256 public latestRoot;

  /// @notice Emitted when a new root is submitted
  event RootSubmitted(uint256 indexed root);

  /**
   * @notice Submit a new root
   * @param _root The new root to submit
   */
  function submitRoot(uint256 _root) external {
    latestRoot = _root;
    emit RootSubmitted(_root);
  }
}
