// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

/**
 * Mock Verifier for testing purposes
 * In production, this would be replaced with a real Groth16/Plonk verifier
 */
contract MockVerifier {
    // Mock verification - always returns true for testing
    // In production, this would verify actual ZK proofs
    function verifyProof(bytes calldata proof, uint256[] calldata publicSignals) 
        external 
        pure 
        returns (bool) 
    {
        // Basic validation of public signals format
        require(publicSignals.length >= 5, "Invalid public signals length");
        
        // Check that nullifier is not zero
        require(publicSignals[0] != 0, "Nullifier cannot be zero");
        
        // Check that amount is positive
        require(publicSignals[2] > 0, "Amount must be positive");
        
        // Check that merkle root is not zero
        require(publicSignals[4] != 0, "Merkle root cannot be zero");
        
        // For testing, always return true
        // In production, this would verify the actual proof
        return true;
    }
}
