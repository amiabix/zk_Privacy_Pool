include "../node_modules/circomlib/circuits/poseidon.circom";

template CommitmentHasher() {
    signal input value;
    signal input secret[2];
    signal input nullifier[2];
    signal input owner[2];
    
    signal output commitment[2];
    signal output nullifierHash[2];
    
    component poseidon = Poseidon(4, 3, 8, 57);
    
    poseidon.inputs[0] <== value;
    poseidon.inputs[1] <== secret[0];
    poseidon.inputs[2] <== nullifier[0];
    poseidon.inputs[3] <== owner[0];
    
    commitment[0] <== poseidon.out;
    
    component poseidon2 = Poseidon(4, 3, 8, 57);
    poseidon2.inputs[0] <== secret[1];
    poseidon2.inputs[1] <== nullifier[1];
    poseidon2.inputs[2] <== owner[1];
    poseidon2.inputs[3] <== value;
    
    commitment[1] <== poseidon2.out;
    
    component nullifierPoseidon = Poseidon(2, 3, 8, 57);
    nullifierPoseidon.inputs[0] <== nullifier[0];
    nullifierPoseidon.inputs[1] <== nullifier[1];
    
    nullifierHash[0] <== nullifierPoseidon.out;
    nullifierHash[1] <== nullifier[0] + nullifier[1];
}

component main = CommitmentHasher();
