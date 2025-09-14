pragma circom 2.2.0;

include "../node_modules/circomlib/circuits/poseidon.circom";
include "../node_modules/circomlib/circuits/comparators.circom";

template SpendingProof(depth) {
    // Public inputs
    signal input root;
    signal input nullifier;
    signal input newCommitment;
    signal input recipient;
    signal input value;
    
    // Private inputs
    signal input secret[2];
    signal input nullifierSecret[2];
    signal input owner[2];
    signal input leafIndex;
    signal input siblings[depth];
    signal input pathIndices[depth];
    
    // Outputs
    signal output valid;
    
    // 1. Verify UTXO ownership (secret key knowledge)
    component ownershipCheck = Poseidon(2, 3, 8, 57);
    ownershipCheck.inputs[0] <== secret[0];
    ownershipCheck.inputs[1] <== secret[1];
    
    // 2. Generate commitment from UTXO data
    component commitmentGen = Poseidon(4, 3, 8, 57);
    commitmentGen.inputs[0] <== value;
    commitmentGen.inputs[1] <== secret[0];
    commitmentGen.inputs[2] <== nullifierSecret[0];
    commitmentGen.inputs[3] <== owner[0];
    
    // 3. Generate nullifier from secret
    component nullifierGen = Poseidon(2, 3, 8, 57);
    nullifierGen.inputs[0] <== nullifierSecret[0];
    nullifierGen.inputs[1] <== nullifierSecret[1];
    
    // 4. Verify nullifier matches input
    component nullifierCheck = IsEqual();
    nullifierCheck.in[0] <== nullifierGen.out;
    nullifierCheck.in[1] <== nullifier;
    
    // 5. Verify Merkle tree membership
    signal currentHash;
    currentHash <== commitmentGen.out;
    
    for (var i = 0; i < depth; i++) {
        component hasher = Poseidon(2, 3, 8, 57);
        hasher.inputs[0] <== currentHash;
        hasher.inputs[1] <== siblings[i];
        currentHash <== hasher.out;
    }
    
    // 6. Verify root matches
    component rootCheck = IsEqual();
    rootCheck.in[0] <== currentHash;
    rootCheck.in[1] <== root;
    
    // 7. Verify value is positive
    component valueCheck = IsZero();
    valueCheck.in[0] <== value;
    
    // 8. Verify new commitment is different from old
    component commitmentDiff = IsEqual();
    commitmentDiff.in[0] <== commitmentGen.out;
    commitmentDiff.in[1] <== newCommitment;
    
    // Final validation: all checks must pass
    valid <== ownershipCheck.out * nullifierCheck.out * rootCheck.out * (1 - valueCheck.out) * (1 - commitmentDiff.out);
}

template IsEqual() {
    signal input in[2];
    signal output out;
    out <== (in[0] - in[1] == 0);
}

template IsZero() {
    signal input in[0];
    signal output out;
    out <== (in[0] == 0);
}

component main = SpendingProof(32);
