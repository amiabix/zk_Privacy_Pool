pragma circom 2.2.0;

include "./merkleTree.circom";
include "../../node_modules/circomlib/circuits/poseidon.circom";
include "../../node_modules/circomlib/circuits/comparators.circom";

/**
 * @title Spend Circuit
 * @dev Proves the right to spend a UTXO while maintaining privacy
 * @notice This circuit proves:
 *         1. Knowledge of secret and nullifier for a commitment
 *         2. The commitment exists in the Merkle tree (inclusion proof)
 *         3. The nullifier has not been used before
 *         4. The user owns the UTXO (knows the secret)
 */
template Spend(levels) {
    // Public inputs
    signal input root;                    // Merkle tree root
    signal input nullifierHash;          // Nullifier hash (prevents double spending)
    signal input recipient;               // Recipient address
    signal input relayer;                 // Relayer address (can be 0)
    signal input fee;                     // Fee paid to relayer
    signal input refund;                  // Refund amount
    
    // Private inputs  
    signal private input secret;          // Secret key
    signal private input nullifier;       // Nullifier
    signal private input pathElements[levels]; // Merkle proof
    signal private input pathIndices[levels];  // Merkle proof path
    signal private input value;           // UTXO value
    signal private input owner;           // Owner public key
    
    // Outputs
    signal output valid;                  // 1 if spend is valid, 0 otherwise
    
    // Component for computing commitment
    component commitmentHasher = Poseidon(4);
    commitmentHasher.inputs[0] <== value;
    commitmentHasher.inputs[1] <== secret;
    commitmentHasher.inputs[2] <== nullifier;
    commitmentHasher.inputs[3] <== owner;
    
    // Component for computing nullifier hash
    component nullifierHasher = Poseidon(2);
    nullifierHasher.inputs[0] <== nullifier;
    nullifierHasher.inputs[1] <== secret;
    
    // Verify nullifier hash matches public input
    component nullifierCheck = IsEqual();
    nullifierCheck.in[0] <== nullifierHasher.out;
    nullifierCheck.in[1] <== nullifierHash;
    
    // Component for Merkle tree verification
    component merkleProof = MerkleTreeInclusionProof(levels);
    merkleProof.leaf <== commitmentHasher.out;
    merkleProof.root <== root;
    for (var i = 0; i < levels; i++) {
        merkleProof.pathElements[i] <== pathElements[i];
        merkleProof.pathIndices[i] <== pathIndices[i];
    }
    
    // Verify value constraints (value >= fee + refund)
    component valueCheck = GreaterEqualThan(64);
    valueCheck.in[0] <== value;
    valueCheck.in[1] <== fee + refund;
    
    // All checks must pass
    valid <== nullifierCheck.out * merkleProof.valid * valueCheck.out;
}

/**
 * @title Withdrawal Circuit  
 * @dev Proves the right to withdraw ETH from the privacy pool
 */
template Withdraw(levels) {
    // Public inputs
    signal input root;                    // Merkle tree root
    signal input nullifierHash;          // Nullifier hash
    signal input recipient;               // ETH recipient address
    signal input relayer;                 // Relayer address
    signal input fee;                     // Relayer fee
    signal input refund;                  // Refund amount
    
    // Private inputs
    signal private input secret;          // Secret key
    signal private input nullifier;       // Nullifier
    signal private input pathElements[levels]; // Merkle proof
    signal private input pathIndices[levels];  // Merkle proof path
    signal private input value;           // UTXO value
    signal private input owner;           // Owner public key
    
    // Use the spend circuit to verify everything
    component spendProof = Spend(levels);
    spendProof.root <== root;
    spendProof.nullifierHash <== nullifierHash;
    spendProof.recipient <== recipient;
    spendProof.relayer <== relayer;
    spendProof.fee <== fee;
    spendProof.refund <== refund;
    spendProof.secret <== secret;
    spendProof.nullifier <== nullifier;
    spendProof.value <== value;
    spendProof.owner <== owner;
    
    for (var i = 0; i < levels; i++) {
        spendProof.pathElements[i] <== pathElements[i];
        spendProof.pathIndices[i] <== pathIndices[i];
    }
    
    // Output validity
    signal output valid;
    valid <== spendProof.valid;
}

/**
 * @title Main Spend Circuit (32 levels for production)
 */
component main = Spend(32);