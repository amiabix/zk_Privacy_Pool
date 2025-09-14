// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;


import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "@openzeppelin/contracts/security/Pausable.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";

interface IVerifier {
    // Verifier must accept (proof, publicSignals) and return bool
    function verifyProof(bytes calldata proof, uint256[] calldata publicSignals) external view returns (bool);
}

contract PrivacyPool is ReentrancyGuard, AccessControl, Pausable {
    bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");
    bytes32 public constant PAUSER_ROLE   = keccak256("PAUSER_ROLE");

    // Events
    event DepositIndexed(
        address indexed depositor,
        bytes32 indexed commitment,
        uint256 value,
        address indexed asset,         // address(0) for ETH
        bytes32 txHash,
        uint64  blockNumber,
        uint32  logIndex
    );

    event MerkleRootPublished(bytes32 indexed oldRoot, bytes32 indexed newRoot, address indexed operator, uint256 blockNumber);
    event Withdrawn(address indexed recipient, uint256 amount, address asset, bytes32 indexed nullifier, uint256 blockNumber);
    event EmergencyWithdrawal(address indexed to, uint256 amount, address asset, address indexed admin);

    // State
    bytes32 public merkleRoot;
    IVerifier public verifier;

    // Nullifier set: nullifier -> true if spent
    mapping(bytes32 => bool) public nullifiers;

    // Token whitelist optional (can restrict accepted tokens)
    mapping(address => bool) public tokenAllowed;

    // On-chain accounting
    uint256 public totalDeposits;
    uint256 public totalWithdrawals;

    // Admin
    address public admin;

    // Constructor: admin + operator
    constructor(address _admin, address initialOperator, address _verifier) {
        require(_admin != address(0), "admin required");
        require(initialOperator != address(0), "operator required");
        require(_verifier != address(0), "verifier required");

        admin = _admin;
        verifier = IVerifier(_verifier);

        _setupRole(DEFAULT_ADMIN_ROLE, _admin);
        _setupRole(OPERATOR_ROLE, initialOperator);
        _setupRole(PAUSER_ROLE, _admin);

        merkleRoot = bytes32(0);
    }

    // ---- Deposit functions ----
    // deposit ETH with precomputed commitment (client forms commitment locally)
    function depositWithCommitment(bytes32 commitment) external payable whenNotPaused nonReentrant {
        require(msg.value > 0, "zero value");
        require(commitment != bytes32(0), "invalid commitment");
        
        // Update accounting
        totalDeposits += msg.value;
        
        // emit event only. Relayer will include commitment into tree off-chain.
        emit DepositIndexed(msg.sender, commitment, msg.value, address(0), bytes32(bytes20(msg.sender)), uint64(block.number), uint32(0));
    }

    // deposit ERC20 token
    function depositERC20(address token, bytes32 commitment, uint256 amount) external whenNotPaused nonReentrant {
        require(amount > 0, "zero amount");
        require(commitment != bytes32(0), "invalid commitment");
        require(token != address(0), "invalid token");
        
        // optional whitelist check - allow all tokens by default
        if (tokenAllowed[token]) {
            // Token is whitelisted
        } else {
            // Allow all tokens by default, or revert if whitelist enforced
            // revert("token not allowed");
        }

        // transfer tokens in
        require(IERC20(token).transferFrom(msg.sender, address(this), amount), "transfer failed");
        
        // Update accounting
        totalDeposits += amount;

        emit DepositIndexed(msg.sender, commitment, amount, token, bytes32(bytes20(msg.sender)), uint64(block.number), uint32(0));
    }

    // ---- Operator publishes new root after off-chain insertion ----
    function publishMerkleRoot(bytes32 newRoot) external onlyRole(OPERATOR_ROLE) whenNotPaused {
        require(newRoot != bytes32(0), "invalid root");
        bytes32 old = merkleRoot;
        merkleRoot = newRoot;
        emit MerkleRootPublished(old, newRoot, msg.sender, block.number);
    }

    // ---- Withdraw (requires zk-proof) ----
    // publicSignals must contain: [nullifier, recipient (as uint256), amount, assetAddress (uint256), merkleRoot, ...]
    function withdraw(bytes32 nullifier, address payable recipient, uint256 amount, address asset, bytes calldata proof, uint256[] calldata publicSignals) external nonReentrant whenNotPaused {
        require(!nullifiers[nullifier], "nullifier used");
        require(amount > 0, "zero amount");
        require(recipient != address(0), "invalid recipient");
        
        // Verify that merkleRoot used in proof equals current merkle root — verifier must validate publicSignals.
        // Verifier must also ensure nullifier derivation and amount correctness.
        require(verifier.verifyProof(proof, publicSignals), "invalid proof");

        // mark spent before external effects
        nullifiers[nullifier] = true;
        
        // Update accounting
        totalWithdrawals += amount;

        if (asset == address(0)) {
            // ETH
            (bool ok, ) = recipient.call{value: amount}("");
            require(ok, "ETH transfer failed");
        } else {
            // ERC20
            require(IERC20(asset).transfer(recipient, amount), "token transfer failed");
        }

        emit Withdrawn(recipient, amount, asset, nullifier, block.number);
    }

    // ---- Admin/operator functions ----
    function setVerifier(address _verifier) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(_verifier != address(0), "invalid");
        verifier = IVerifier(_verifier);
    }

    function setOperator(address op, bool enabled) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(op != address(0), "invalid");
        if (enabled) {
            grantRole(OPERATOR_ROLE, op);
        } else {
            revokeRole(OPERATOR_ROLE, op);
        }
    }

    function setTokenAllowed(address token, bool allowed) external onlyRole(DEFAULT_ADMIN_ROLE) {
        tokenAllowed[token] = allowed;
    }

    function pause() external onlyRole(PAUSER_ROLE) { _pause(); }
    function unpause() external onlyRole(PAUSER_ROLE) { _unpause(); }

    function emergencyWithdrawAll(address payable to, address asset) external onlyRole(DEFAULT_ADMIN_ROLE) {
        if (asset == address(0)) {
            uint256 bal = address(this).balance;
            if (bal > 0) {
                (bool ok, ) = to.call{value: bal}("");
                require(ok, "eth transfer failed");
                emit EmergencyWithdrawal(to, bal, asset, msg.sender);
            }
        } else {
            uint256 bal = IERC20(asset).balanceOf(address(this));
            if (bal > 0) {
                require(IERC20(asset).transfer(to, bal), "token transfer failed");
                emit EmergencyWithdrawal(to, bal, asset, msg.sender);
            }
        }
    }

    // ---- View functions ----
    
    /**
     * @dev Get current Merkle root
     */
    function getMerkleRoot() external view returns (bytes32) {
        return merkleRoot;
    }
    
    /**
     * @dev Check if nullifier is used
     */
    function isNullifierUsed(bytes32 nullifier) external view returns (bool) {
        return nullifiers[nullifier];
    }
    
    /**
     * @dev Get accounting totals
     */
    function getAccounting() external view returns (uint256, uint256) {
        return (totalDeposits, totalWithdrawals);
    }
    
    /**
     * @dev Check accounting invariant
     * @return true if contract_balance + totalWithdrawals == totalDeposits
     */
    function checkAccountingInvariant() external view returns (bool) {
        uint256 contractBalance = address(this).balance;
        return contractBalance + totalWithdrawals == totalDeposits;
    }
    
    /**
     * @dev Get contract balance
     */
    function getContractBalance() external view returns (uint256) {
        return address(this).balance;
    }
    
    /**
     * @dev Get token balance
     */
    function getTokenBalance(address token) external view returns (uint256) {
        return IERC20(token).balanceOf(address(this));
    }

    // fallback
    receive() external payable {}
}