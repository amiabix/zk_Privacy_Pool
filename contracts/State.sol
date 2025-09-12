// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import {Constants} from './lib/Constants.sol';
import {ProofLib} from './lib/ProofLib.sol';
import {IPrivacyPool} from './interfaces/IPrivacyPool.sol';

/**
 * @title State
 * @notice Base contract for managing privacy pool state
 */
abstract contract State {
  using ProofLib for ProofLib.WithdrawProof;
  using ProofLib for ProofLib.RagequitProof;

  //////////////////////// ERRORS ////////////////////////

  error InvalidProcessooor();
  error ContextMismatch();
  error InvalidTreeDepth();
  error UnknownStateRoot();
  error IncorrectASPRoot();
  error PoolIsDead();
  error InvalidDepositValue();
  error InvalidProof();
  error OnlyOriginalDepositor();
  error InvalidCommitment();
  error OnlyEntrypoint();

  //////////////////////// STATE ////////////////////////

  /// @notice The entrypoint contract
  address public immutable ENTRYPOINT;

  /// @notice The withdrawal verifier contract
  address public immutable WITHDRAWAL_VERIFIER;

  /// @notice The ragequit verifier contract
  address public immutable RAGEQUIT_VERIFIER;

  /// @notice The pool asset
  address public immutable ASSET;

  /// @notice The pool scope
  bytes32 public immutable SCOPE;

  /// @notice Whether the pool is dead
  bool public dead;

  /// @notice The nonce counter
  uint256 public nonce;

  /// @notice The maximum tree depth
  uint256 public constant MAX_TREE_DEPTH = 32;

  /// @notice Mapping from label to depositor
  mapping(uint256 => address) public depositors;

  /// @notice Mapping from commitment to existence
  mapping(uint256 => bool) public commitments;

  /// @notice Mapping from nullifier to spent status
  mapping(uint256 => bool) public nullifiers;

  /// @notice Mapping from state root to existence
  mapping(uint256 => bool) public stateRoots;

  //////////////////////// MODIFIERS ////////////////////////

  modifier onlyEntrypoint() {
    if (msg.sender != ENTRYPOINT) revert OnlyEntrypoint();
    _;
  }

  //////////////////////// CONSTRUCTOR ////////////////////////

  constructor(
    address _asset,
    address _entrypoint,
    address _withdrawalVerifier,
    address _ragequitVerifier
  ) {
    ASSET = _asset;
    ENTRYPOINT = _entrypoint;
    WITHDRAWAL_VERIFIER = _withdrawalVerifier;
    RAGEQUIT_VERIFIER = _ragequitVerifier;
    SCOPE = keccak256(abi.encodePacked("PrivacyPool", _asset, _entrypoint));
  }

  //////////////////////// INTERNAL FUNCTIONS ////////////////////////

  /**
   * @notice Insert a commitment into the state
   * @param _commitment The commitment to insert
   */
  function _insert(uint256 _commitment) internal {
    if (commitments[_commitment]) revert InvalidCommitment();
    commitments[_commitment] = true;
  }

  /**
   * @notice Spend a nullifier
   * @param _nullifier The nullifier to spend
   */
  function _spend(uint256 _nullifier) internal {
    if (nullifiers[_nullifier]) revert InvalidCommitment();
    nullifiers[_nullifier] = true;
  }

  /**
   * @notice Check if a commitment is in state
   * @param _commitment The commitment to check
   * @return Whether the commitment exists
   */
  function _isInState(uint256 _commitment) internal view returns (bool) {
    return commitments[_commitment];
  }

  /**
   * @notice Check if a state root is known
   * @param _root The state root to check
   * @return Whether the state root is known
   */
  function _isKnownRoot(uint256 _root) internal view returns (bool) {
    return stateRoots[_root];
  }

  /**
   * @notice Add a known state root
   * @param _root The state root to add
   */
  function _addKnownRoot(uint256 _root) internal {
    stateRoots[_root] = true;
  }
}
