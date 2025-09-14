include "../node_modules/circomlib/circuits/poseidon.circom";

template MerkleTreeProof(depth) {
    signal input leaf;
    signal input leafIndex;
    signal input siblings[depth];
    signal input root;
    
    signal output valid;
    
    signal current;
    current <== leaf;
    
    component poseidon = Poseidon(2, 3, 8, 57);
    poseidon.inputs[0] <== current;
    poseidon.inputs[1] <== siblings[0];
    
    component eq = IsEqual();
    eq.in[0] <== poseidon.out;
    eq.in[1] <== root;
    
    valid <== eq.out;
}

template IsEqual() {
    signal input in[2];
    signal output out;
    
    signal diff;
    diff <== in[0] - in[1];
    
    component isZero = IsZero();
    isZero.in <== diff;
    
    out <== isZero.out;
}

template IsZero() {
    signal input in;
    signal output out;
    
    signal inv;
    inv <== in != 0 ? 1/in : 0;
    
    out <== 1 - (in * inv);
}

component main = MerkleTreeProof(32);
