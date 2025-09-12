// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {PrivacyPool} from './PrivacyPool.sol';

/**
 * @title ETHPrivacyPool
 * @notice Privacy pool implementation for ETH
 */
contract ETHPrivacyPool is PrivacyPool {
  constructor(
    address _entrypoint,
    address _withdrawalVerifier,
    address _ragequitVerifier
  ) PrivacyPool(_entrypoint, _withdrawalVerifier, _ragequitVerifier, address(0)) {}

  /**
   * @notice Handle receiving ETH
   * @param _sender The address of the user sending funds
   * @param _value The amount of ETH being received
   */
  function _pull(address _sender, uint256 _value) internal override {
    // ETH is sent with the transaction, no need to transfer
    require(msg.value >= _value, "Insufficient ETH sent");
  }

  /**
   * @notice Handle sending ETH
   * @param _recipient The address of the user receiving funds
   * @param _value The amount of ETH being sent
   */
  function _push(address _recipient, uint256 _value) internal override {
    (bool success, ) = payable(_recipient).call{value: _value}("");
    require(success, "ETH transfer failed");
  }

  /**
   * @notice Receive ETH
   */
  receive() external payable {}
}
