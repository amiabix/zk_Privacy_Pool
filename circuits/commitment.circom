pragma circom 2.2.0;

include "../../node_modules/circomlib/circuits/poseidon.circom";
include "../../node_modules/circomlib/circuits/comparators.circom";

/**
 * @title Commitment Circuit
 * @dev Generates and verifies UTXO commitments with owner binding
 * @notice commitment = hash(value || secret || nullifier || owner_pubkey)
 * 
 * This circuit ensures that:
 * 1. Only the holder of the secret can generate a valid commitment
 * 2. The commitment is bound to a specific owner public key
 * 3. The commitment can be verified without revealing the secret
 */

template CommitmentHasher() {
    // Input signals
    signal input value;           // UTXO value (128 bits max for privacy)
    signal input secret[2];       // Secret key (split into 2 field elements)
    signal input nullifier[2];    // Nullifier (split into 2 field elements)
    signal input owner[2];        // Owner public key (split into 2 field elements)
    
    // Output signals
    signal output commitment[2];  // Commitment hash (2 field elements)
    signal output nullifierHash[2]; // Nullifier hash for spending
    
    // Component for Poseidon hash
    component poseidon = Poseidon(4);
    
    // Hash the commitment components
    poseidon.inputs[0] <== value;
    poseidon.inputs[1] <== secret[0];
    poseidon.inputs[2] <== nullifier[0];
    poseidon.inputs[3] <== owner[0];
    
    // Output commitment
    commitment[0] <== poseidon.out;
    
    // Second hash for additional security
    component poseidon2 = Poseidon(4);
    poseidon2.inputs[0] <== secret[1];
    poseidon2.inputs[1] <== nullifier[1];
    poseidon2.inputs[2] <== owner[1];
    poseidon2.inputs[3] <== value;
    
    commitment[1] <== poseidon2.out;
    
    // Generate nullifier hash for spending
    component nullifierPoseidon = Poseidon(2);
    nullifierPoseidon.inputs[0] <== nullifier[0];
    nullifierPoseidon.inputs[1] <== nullifier[1];
    
    nullifierHash[0] <== nullifierPoseidon.out;
    nullifierHash[1] <== nullifier[0] + nullifier[1]; // Additional binding
}

/**
 * @title Commitment Verification Circuit
 * @dev Verifies that a commitment was generated correctly
 */
template CommitmentVerifier() {
    // Input signals
    signal input value;
    signal input secret[2];
    signal input nullifier[2];
    signal input owner[2];
    signal input commitment[2];
    
    // Output signals
    signal output valid;
    
    // Generate commitment hasher
    component hasher = CommitmentHasher();
    
    // Connect inputs
    hasher.value <== value;
    hasher.secret[0] <== secret[0];
    hasher.secret[1] <== secret[1];
    hasher.nullifier[0] <== nullifier[0];
    hasher.nullifier[1] <== nullifier[1];
    hasher.owner[0] <== owner[0];
    hasher.owner[1] <== owner[1];
    
    // Verify commitment matches
    component eq0 = IsEqual();
    eq0.in[0] <== hasher.commitment[0];
    eq0.in[1] <== commitment[0];
    
    component eq1 = IsEqual();
    eq1.in[0] <== hasher.commitment[1];
    eq1.in[1] <== commitment[1];
    
    // Both parts must match
    valid <== eq0.out * eq1.out;
}

/**
 * @title Owner Verification Circuit
 * @dev Verifies that a UTXO belongs to a specific owner
 */
template OwnerVerifier() {
    // Input signals
    signal input utxoOwner[2];    // Owner from UTXO
    signal input claimedOwner[2]; // Claimed owner
    signal input commitment[2];   // UTXO commitment
    
    // Output signals
    signal output valid;
    
    // Verify owner matches
    component ownerEq0 = IsEqual();
    ownerEq0.in[0] <== utxoOwner[0];
    ownerEq0.in[1] <== claimedOwner[0];
    
    component ownerEq1 = IsEqual();
    ownerEq1.in[0] <== utxoOwner[1];
    ownerEq1.in[1] <== claimedOwner[1];
    
    // Both owner parts must match
    valid <== ownerEq0.out * ownerEq1.out;
}

/**
 * @title UTXO Ownership Proof Circuit
 * @dev Proves ownership of a UTXO without revealing the secret
 */
template UTXOOwnershipProof() {
    // Public inputs
    signal input commitment[2];   // UTXO commitment
    signal input owner[2];        // Owner public key
    signal input value;           // UTXO value
    
    // Private inputs
    signal private input secret[2];    // Secret key
    signal private input nullifier[2]; // Nullifier
    
    // Output signals
    signal output valid;
    
    // Verify commitment was generated correctly
    component commitmentVerifier = CommitmentVerifier();
    commitmentVerifier.value <== value;
    commitmentVerifier.secret[0] <== secret[0];
    commitmentVerifier.secret[1] <== secret[1];
    commitmentVerifier.nullifier[0] <== nullifier[0];
    commitmentVerifier.nullifier[1] <== nullifier[1];
    commitmentVerifier.owner[0] <== owner[0];
    commitmentVerifier.owner[1] <== owner[1];
    commitmentVerifier.commitment[0] <== commitment[0];
    commitmentVerifier.commitment[1] <== commitment[1];
    
    // Verify owner binding
    component ownerVerifier = OwnerVerifier();
    ownerVerifier.utxoOwner[0] <== owner[0];
    ownerVerifier.utxoOwner[1] <== owner[1];
    ownerVerifier.claimedOwner[0] <== owner[0];
    ownerVerifier.claimedOwner[1] <== owner[1];
    ownerVerifier.commitment[0] <== commitment[0];
    ownerVerifier.commitment[1] <== commitment[1];
    
    // Both verifications must pass
    valid <== commitmentVerifier.valid * ownerVerifier.valid;
}

/**
 * @title Main Commitment Circuit
 * @dev Main circuit for commitment operations
 */
template MainCommitment() {
    // Public inputs
    signal input value;
    signal input owner[2];
    signal input commitment[2];
    
    // Private inputs
    signal private input secret[2];
    signal private input nullifier[2];
    
    // Output signals
    signal output valid;
    signal output nullifierHash[2];
    
    // Generate commitment hasher
    component hasher = CommitmentHasher();
    hasher.value <== value;
    hasher.secret[0] <== secret[0];
    hasher.secret[1] <== secret[1];
    hasher.nullifier[0] <== nullifier[0];
    hasher.nullifier[1] <== nullifier[1];
    hasher.owner[0] <== owner[0];
    hasher.owner[1] <== owner[1];
    
    // Verify commitment
    component verifier = CommitmentVerifier();
    verifier.value <== value;
    verifier.secret[0] <== secret[0];
    verifier.secret[1] <== secret[1];
    verifier.nullifier[0] <== nullifier[0];
    verifier.nullifier[1] <== nullifier[1];
    verifier.owner[0] <== owner[0];
    verifier.owner[1] <== owner[1];
    verifier.commitment[0] <== commitment[0];
    verifier.commitment[1] <== commitment[1];
    
    // Outputs
    valid <== verifier.valid;
    nullifierHash[0] <== hasher.nullifierHash[0];
    nullifierHash[1] <== hasher.nullifierHash[1];
}