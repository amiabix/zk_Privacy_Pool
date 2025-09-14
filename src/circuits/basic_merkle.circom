include "../node_modules/circomlib/circuits/poseidon.circom";

template BasicMerkle() {
    signal input leaf;
    signal input sibling;
    signal input root;
    
    signal output valid;
    
    component poseidon = Poseidon(2, 3, 8, 57);
    poseidon.inputs[0] <== leaf;
    poseidon.inputs[1] <== sibling;
    
    valid <== poseidon.out - root;
}

component main = BasicMerkle();
