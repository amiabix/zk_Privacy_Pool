include "../node_modules/circomlib/circuits/poseidon.circom";

template CompleteSpending() {
    // Public inputs
    signal input root;
    signal input nullifier;
    signal input newCommitment;
    signal input value;
    
    // Private inputs
    signal input secret[2];
    signal input owner[2];
    signal input leafIndex;
    signal input siblings[32];
    
    // Outputs
    signal output valid;
    
    // 1. Generate commitment from UTXO data
    component commitmentGen = Poseidon(4, 3, 8, 57);
    commitmentGen.inputs[0] <== value;
    commitmentGen.inputs[1] <== secret[0];
    commitmentGen.inputs[2] <== secret[1];
    commitmentGen.inputs[3] <== owner[0];
    
    // 2. Generate nullifier from secret
    component nullifierGen = Poseidon(2, 3, 8, 57);
    nullifierGen.inputs[0] <== secret[0];
    nullifierGen.inputs[1] <== secret[1];
    
    // 3. Verify nullifier matches input
    signal nullifierMatch;
    nullifierMatch <== (nullifierGen.out - nullifier == 0);
    
    // 4. Verify Merkle tree membership
    signal currentHash;
    currentHash <== commitmentGen.out;
    
    for (var i = 0; i < 32; i++) {
        component hasher = Poseidon(2, 3, 8, 57);
        hasher.inputs[0] <== currentHash;
        hasher.inputs[1] <== siblings[i];
        currentHash <== hasher.out;
    }
    
    // 5. Verify root matches
    signal rootMatch;
    rootMatch <== (currentHash - root == 0);
    
    // 6. Verify value is positive
    signal valuePositive;
    valuePositive <== (value > 0);
    
    // 7. Verify new commitment is different from old
    signal commitmentDiff;
    commitmentDiff <== (commitmentGen.out - newCommitment != 0);
    
    // Final validation: all checks must pass
    valid <== nullifierMatch * rootMatch * valuePositive * commitmentDiff;
}

component main = CompleteSpending();
