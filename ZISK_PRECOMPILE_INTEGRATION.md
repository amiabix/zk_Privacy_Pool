# ZisK Precompile Integration Guide

## Current Status: Syscalls Not Directly Accessible

The ZisK syscalls module is private and not directly accessible from user code. This document outlines the integration strategy and workarounds.

## Available ZisK Precompiles

### Hash Functions
- **Keccak-256** (`0xf1`) - 167,000 cost units
- **SHA-256** (`0xf9`) - 9,000 cost units

### Arithmetic Operations
- **Arith256** (`0xf2`) - 1,200 cost units (256-bit arithmetic)
- **Arith256Mod** (`0xf3`) - 1,200 cost units (256-bit modular arithmetic)

### Elliptic Curve Operations
- **Secp256k1Add** (`0xf4`) - 1,200 cost units (elliptic curve addition)
- **Secp256k1Dbl** (`0xf5`) - 1,200 cost units (elliptic curve doubling)
- **Bn254CurveAdd** (`0xfa`) - 1,200 cost units (BN254 curve addition)
- **Bn254CurveDbl** (`0xfb`) - 1,200 cost units (BN254 curve doubling)
- **Bn254ComplexAdd** (`0xfc`) - 1,200 cost units (BN254 complex arithmetic)
- **Bn254ComplexSub** (`0xfd`) - 1,200 cost units (BN254 complex subtraction)
- **Bn254ComplexMul** (`0xfe`) - 1,200 cost units (BN254 complex multiplication)

## Integration Strategies

### Strategy 1: Direct Syscall Access (Ideal)
```rust
use ziskos::syscalls::*;

fn hash_with_keccak(data: &[u8]) -> [u8; 32] {
    let mut state = [0u64; 25];
    // Prepare data...
    syscall_keccak_f(&mut state);
    // Convert result...
}
```

**Status**: Not currently possible (syscalls module is private)

### Strategy 2: Custom Precompile Wrapper (Recommended)
```rust
// Create a wrapper module for ZisK precompiles
mod zisk_precompiles {
    use ziskos::ziskos_syscall;
    
    pub fn keccak256(data: &[u8]) -> [u8; 32] {
        // Convert data to u64 array
        let mut state = [0u64; 25];
        // Prepare state...
        
        // Call precompile via syscall
        ziskos_syscall!(0x800, &mut state); // Keccak CSR address
        
        // Convert result back to bytes
        // ...
    }
    
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        // Similar implementation for SHA-256
        // ...
    }
}
```

### Strategy 3: Assembly Integration (Advanced)
```rust
// Use inline assembly to call precompiles directly
unsafe fn call_keccak(state: *mut [u64; 25]) {
    asm!(
        "ecall",
        in("a0") 0xf1,  // Keccak precompile ID
        in("a1") state,
        options(nostack)
    );
}
```

## Implementation Plan

### Phase 1: Research and Setup
1. **Research ZisK precompile access patterns**
   - Check ZisK documentation for syscall examples
   - Look for community implementations
   - Contact ZisK team for guidance

2. **Create precompile wrapper module**
   - Implement wrapper functions for each precompile
   - Add proper error handling
   - Create unit tests

### Phase 2: Hash Function Integration
1. **Replace SHA-256 with ZisK precompile**
   ```rust
   // Current implementation
   fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
       use sha2::{Digest, Sha256};
       let mut hasher = Sha256::new();
       hasher.update(&left);
       hasher.update(&right);
       hasher.finalize().into()
   }
   
   // Target implementation
   fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
       let mut data = Vec::new();
       data.extend_from_slice(&left);
       data.extend_from_slice(&right);
       zisk_precompiles::sha256(&data)
   }
   ```

2. **Optimize for constraint cost**
   - Use SHA-256 (9,000 units) instead of Keccak (167,000 units)
   - Batch operations where possible
   - Minimize precompile calls

### Phase 3: Elliptic Curve Operations
1. **Implement BN254 curve operations**
   ```rust
   fn bn254_point_add(p1: &[u8; 64], p2: &[u8; 64]) -> [u8; 64] {
       // Convert points to u64 arrays
       // Call Bn254CurveAdd precompile
       // Convert result back to bytes
   }
   ```

2. **Build RedJubjub signature verification**
   - Use BN254 curve operations
   - Implement signature verification logic
   - Replace placeholder verification

### Phase 4: Pedersen Commitments
1. **Implement Pedersen commitment generation**
   ```rust
   fn pedersen_commit(value: u64, blinding: [u8; 32]) -> [u8; 32] {
       // Use BN254 curve operations
       // Generate commitment: C = v*G + r*H
       // Where G, H are generators, v is value, r is blinding
   }
   ```

2. **Add commitment verification**
   - Verify commitment structure
   - Check value ranges
   - Validate blinding factors

## Testing Strategy

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sha256_precompile() {
        let data = b"hello world";
        let result = zisk_precompiles::sha256(data);
        // Verify against known hash
        assert_eq!(result, expected_hash);
    }
    
    #[test]
    fn test_bn254_point_add() {
        let p1 = [1u8; 64];
        let p2 = [2u8; 64];
        let result = bn254_point_add(&p1, &p2);
        // Verify point addition
        assert!(is_valid_point(&result));
    }
}
```

### Integration Tests
```rust
#[test]
fn test_merkle_tree_with_precompiles() {
    let mut tree = MerkleTree::new();
    // Add UTXOs using precompile hashing
    // Verify tree operations
    // Check constraint costs
}
```

### Performance Tests
```rust
#[test]
fn test_constraint_costs() {
    // Measure constraint costs for different operations
    // Compare with standard implementations
    // Optimize for production use
}
```

## Error Handling

### Precompile Errors
```rust
#[derive(Debug)]
pub enum PrecompileError {
    InvalidInput,
    PrecompileFailed,
    InvalidOutput,
}

impl std::fmt::Display for PrecompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PrecompileError::InvalidInput => write!(f, "Invalid input for precompile"),
            PrecompileError::PrecompileFailed => write!(f, "Precompile execution failed"),
            PrecompileError::InvalidOutput => write!(f, "Invalid output from precompile"),
        }
    }
}
```

### Graceful Degradation
```rust
fn hash_with_fallback(data: &[u8]) -> Result<[u8; 32], PrecompileError> {
    // Try ZisK precompile first
    match zisk_precompiles::sha256(data) {
        Ok(hash) => Ok(hash),
        Err(_) => {
            // Fallback to standard implementation
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data);
            Ok(hasher.finalize().into())
        }
    }
}
```

## Performance Considerations

### Constraint Cost Optimization
- **SHA-256**: 9,000 units (preferred for hashing)
- **Keccak-256**: 167,000 units (avoid unless necessary)
- **Arithmetic**: 1,200 units each (use for calculations)
- **Curve operations**: 1,200 units each (use for cryptography)

### Batching Operations
```rust
fn batch_hash_utxos(utxos: &[UTXO]) -> Vec<[u8; 32]> {
    // Process multiple UTXOs in single precompile call
    // Reduce constraint costs
    // Improve performance
}
```

### Memory Management
```rust
fn efficient_hash(data: &[u8]) -> [u8; 32] {
    // Reuse buffers to avoid allocations
    // Minimize data copying
    // Optimize for ZisK constraints
}
```

## Next Steps

1. **Research ZisK precompile access patterns**
2. **Implement precompile wrapper module**
3. **Replace hash functions with ZisK precompiles**
4. **Add elliptic curve operations**
5. **Implement Pedersen commitments**
6. **Add comprehensive testing**
7. **Optimize for production use**

## Resources

- [ZisK Documentation](https://0xpolygonhermez.github.io/zisk/)
- [ZisK GitHub Repository](https://github.com/0xPolygonHermez/zisk)
- [RISC-V Assembly Reference](https://riscv.org/wp-content/uploads/2017/05/riscv-spec-v2.2.pdf)
- [BN254 Curve Specification](https://tools.ietf.org/html/draft-irtf-cfrg-pairing-friendly-curves-03)
