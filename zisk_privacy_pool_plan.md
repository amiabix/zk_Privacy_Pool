# ZisK Privacy Pool Implementation Plan

## 🎯 **Project Overview**

Building a true privacy-preserving mixing pool using **ZisK zkVM** and **ZeroPool's architecture**. This combines the best of both worlds:
- **ZeroPool's proven privacy pool logic** (multi-user, Merkle trees, nullifiers)
- **ZisK's efficient zkVM execution** (Rust-native, fast proving)

## 🚀 **Phase 1: ZisK Environment Setup**

### **1.1 Install ZisK Toolchain**

```bash
# Install ZisK CLI tools
cargo install cargo-zisk

# Verify installation
cargo-zisk --version
```

### **1.2 Create Project Structure**

```bash
# Create new ZisK project
cargo-zisk new privacy-pool-zkvm
cd privacy-pool-zkvm

# Project structure:
privacy-pool-zkvm/
├── Cargo.toml
├── src/
│   ├── main.rs              # ZisK entry point
│   ├── privacy_pool.rs      # Core privacy pool logic
│   ├── transaction.rs       # Transaction handling
│   ├── merkle_tree.rs       # Merkle tree operations
│   ├── zk_proofs.rs         # ZK proof generation
│   └── state.rs             # State management
├── build/
│   └── input.bin            # Input data for ZisK
└── tests/
    └── integration_tests.rs
```

### **1.3 Configure Cargo.toml**

```toml
[package]
name = "privacy-pool-zkvm"
version = "0.1.0"
edition = "2021"
default-run = "privacy-pool-zkvm"

[dependencies]
ziskos = { git = "https://github.com/0xPolygonHermez/zisk.git" }
# Crypto dependencies
sha2 = "0.10.8"
poseidon-hash = "0.1"
merkle-tree = "0.1"
# Serialization
bincode = "1.3"
serde = { version = "1.0", features = ["derive"] }
# Utilities
byteorder = "1.5.0"
anyhow = "1.0"
thiserror = "1.0"
```

## 🏗️ **Phase 2: Core Architecture Design**

### **2.1 ZisK Entry Point (main.rs)**

```rust
#![no_main]
ziskos::entrypoint!(main);

use privacy_pool_zkvm::privacy_pool::PrivacyPool;
use privacy_pool_zkvm::transaction::PrivacyTransaction;
use ziskos::{read_input, set_output};

fn main() {
    // Read transaction data from input.bin
    let input: Vec<u8> = read_input();
    
    // Deserialize transaction
    let transaction: PrivacyTransaction = bincode::deserialize(&input)
        .expect("Failed to deserialize transaction");
    
    // Process transaction in ZisK zkVM
    let mut pool = PrivacyPool::new();
    let result = pool.process_transaction(transaction);
    
    // Output result
    let output = bincode::serialize(&result).expect("Failed to serialize result");
    for (i, chunk) in output.chunks(4).enumerate() {
        let val = u32::from_le_bytes(chunk.try_into().unwrap_or([0; 4]));
        set_output(i as u32, val);
    }
}
```

### **2.2 Core Data Structures (privacy_pool.rs)**

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPool {
    pub merkle_tree: MerkleTree<Commitment>,
    pub nullifiers: HashSet<NullifierHash>,
    pub pool_index: u64,
    pub pool_balance: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commitment {
    pub value: u64,
    pub secret: [u8; 32],
    pub nullifier: [u8; 32],
    pub hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullifierHash([u8; 32]);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Deposit { amount: u64, recipient: [u8; 32] },
    Withdraw { amount: u64, recipient: [u8; 32] },
    Transfer { amount: u64, recipient: [u8; 32] },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyTransaction {
    pub tx_type: TransactionType,
    pub nullifier: [u8; 32],
    pub out_commit: [u8; 32],
    pub merkle_proof: MerkleProof,
    pub zk_proof: ZkProof,
    pub private_inputs: PrivateInputs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivateInputs {
    pub value: u64,
    pub secret: [u8; 32],
    pub nullifier: [u8; 32],
    pub new_secret: Option<[u8; 32]>,
    pub new_nullifier: Option<[u8; 32]>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MerkleProof {
    pub siblings: Vec<[u8; 32]>,
    pub path: Vec<bool>,
    pub root: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZkProof {
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<u64>,
}
```

## 🔧 **Phase 3: Implementation Details**

### **3.1 Merkle Tree Operations (merkle_tree.rs)**

```rust
use sha2::{Digest, Sha256};

pub struct MerkleTree<T> {
    pub leaves: Vec<T>,
    pub root: [u8; 32],
    pub depth: usize,
}

impl<T: Clone + Serialize> MerkleTree<T> {
    pub fn new() -> Self {
        Self {
            leaves: Vec::new(),
            root: [0u8; 32],
            depth: 0,
        }
    }
    
    pub fn insert(&mut self, leaf: T) -> Result<[u8; 32], Error> {
        self.leaves.push(leaf);
        self.update_root()?;
        Ok(self.root)
    }
    
    pub fn generate_proof(&self, index: usize) -> Result<MerkleProof, Error> {
        // Generate Merkle proof for leaf at index
        // This will be proven in ZisK zkVM
    }
    
    fn update_root(&mut self) -> Result<(), Error> {
        // Update Merkle tree root
        // Implement efficient tree updates
    }
}

fn hash_leaf<T: Serialize>(leaf: &T) -> [u8; 32] {
    let serialized = bincode::serialize(leaf).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&serialized);
    hasher.finalize().into()
}
```

### **3.2 ZK Proof Generation (zk_proofs.rs)**

```rust
use poseidon_hash::PoseidonHash;

pub struct ZkProofGenerator {
    poseidon: PoseidonHash,
}

impl ZkProofGenerator {
    pub fn new() -> Self {
        Self {
            poseidon: PoseidonHash::new(),
        }
    }
    
    pub fn generate_commitment(&self, value: u64, secret: &[u8; 32], nullifier: &[u8; 32]) -> [u8; 32] {
        // Generate commitment using Poseidon hash
        // This runs inside ZisK zkVM
        let input = [value.to_le_bytes(), *secret, *nullifier].concat();
        self.poseidon.hash(&input)
    }
    
    pub fn generate_nullifier_hash(&self, nullifier: &[u8; 32]) -> [u8; 32] {
        // Generate nullifier hash
        self.poseidon.hash(nullifier)
    }
    
    pub fn verify_merkle_proof(&self, proof: &MerkleProof, leaf: &[u8; 32]) -> bool {
        // Verify Merkle proof
        // This is proven in ZisK zkVM
        let mut current = *leaf;
        for (sibling, is_right) in proof.siblings.iter().zip(proof.path.iter()) {
            if *is_right {
                current = self.poseidon.hash(&[current, *sibling].concat());
            } else {
                current = self.poseidon.hash(&[*sibling, current].concat());
            }
        }
        current == proof.root
    }
}
```

### **3.3 Transaction Processing (privacy_pool.rs)**

```rust
impl PrivacyPool {
    pub fn new() -> Self {
        Self {
            merkle_tree: MerkleTree::new(),
            nullifiers: HashSet::new(),
            pool_index: 0,
            pool_balance: 0,
        }
    }
    
    pub fn process_transaction(&mut self, tx: PrivacyTransaction) -> Result<TransactionResult, Error> {
        // This entire function runs inside ZisK zkVM
        // All operations are proven
        
        // 1. Verify nullifier not used
        let nullifier_hash = self.zk_proofs.generate_nullifier_hash(&tx.nullifier);
        if self.nullifiers.contains(&NullifierHash(nullifier_hash)) {
            return Err(Error::DoubleSpend);
        }
        
        // 2. Verify Merkle proof
        let commitment = self.zk_proofs.generate_commitment(
            tx.private_inputs.value,
            &tx.private_inputs.secret,
            &tx.private_inputs.nullifier,
        );
        
        if !self.zk_proofs.verify_merkle_proof(&tx.merkle_proof, &commitment) {
            return Err(Error::InvalidMerkleProof);
        }
        
        // 3. Process transaction type
        match tx.tx_type {
            TransactionType::Deposit { amount, recipient } => {
                self.process_deposit(amount, recipient, commitment)?;
            },
            TransactionType::Withdraw { amount, recipient } => {
                self.process_withdraw(amount, recipient, tx.private_inputs)?;
            },
            TransactionType::Transfer { amount, recipient } => {
                self.process_transfer(amount, recipient, tx.private_inputs)?;
            },
        }
        
        // 4. Update state
        self.nullifiers.insert(NullifierHash(nullifier_hash));
        self.pool_index += 1;
        
        Ok(TransactionResult::Success)
    }
    
    fn process_deposit(&mut self, amount: u64, recipient: [u8; 32], commitment: [u8; 32]) -> Result<(), Error> {
        // Add commitment to Merkle tree
        self.merkle_tree.insert(Commitment {
            value: amount,
            secret: [0; 32], // Will be set by user
            nullifier: [0; 32], // Will be set by user
            hash: commitment,
        })?;
        
        self.pool_balance += amount;
        Ok(())
    }
    
    fn process_withdraw(&mut self, amount: u64, recipient: [u8; 32], private_inputs: PrivateInputs) -> Result<(), Error> {
        // Verify sufficient balance
        if private_inputs.value < amount {
            return Err(Error::InsufficientBalance);
        }
        
        // Handle partial withdrawal
        if let (Some(new_secret), Some(new_nullifier)) = (private_inputs.new_secret, private_inputs.new_nullifier) {
            let new_commitment = self.zk_proofs.generate_commitment(
                private_inputs.value - amount,
                &new_secret,
                &new_nullifier,
            );
            self.merkle_tree.insert(Commitment {
                value: private_inputs.value - amount,
                secret: new_secret,
                nullifier: new_nullifier,
                hash: new_commitment,
            })?;
        }
        
        self.pool_balance -= amount;
        Ok(())
    }
}
```

## 🧪 **Phase 4: Testing & Development**

### **4.1 Create Test Input (build/input.bin)**

```rust
// Create test transaction
let test_tx = PrivacyTransaction {
    tx_type: TransactionType::Deposit { 
        amount: 1000, 
        recipient: [1u8; 32] 
    },
    nullifier: [2u8; 32],
    out_commit: [3u8; 32],
    merkle_proof: MerkleProof {
        siblings: vec![[4u8; 32]],
        path: vec![true],
        root: [5u8; 32],
    },
    zk_proof: ZkProof {
        proof_data: vec![6u8; 100],
        public_inputs: vec![1000, 1],
    },
    private_inputs: PrivateInputs {
        value: 1000,
        secret: [7u8; 32],
        nullifier: [2u8; 32],
        new_secret: None,
        new_nullifier: None,
    },
};

// Serialize and save to input.bin
let serialized = bincode::serialize(&test_tx).unwrap();
std::fs::write("build/input.bin", serialized).unwrap();
```

### **4.2 Build and Test**

```bash
# Build for ZisK
cargo-zisk build --release

# Test in emulator
cargo-zisk run --release -i build/input.bin

# Generate proof
cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm -k $HOME/.zisk/provingKey
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm -i build/input.bin -o proof -a -y
```

## 📊 **Phase 5: Performance Optimization**

### **5.1 ZisK-Specific Optimizations**

1. **Memory Management**: Optimize for ZisK's memory constraints
2. **Proof Generation**: Use concurrent proof generation with MPI
3. **GPU Acceleration**: Enable GPU support for faster proving
4. **Constraint Optimization**: Minimize ZisK execution steps

### **5.2 Concurrent Proof Generation**

```bash
# Multi-process proof generation
mpirun --bind-to none -np 4 -x OMP_NUM_THREADS=8 -x RAYON_NUM_THREADS=8 \
    cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm \
    -i build/input.bin -o proof -a -y
```

## 🎯 **Success Metrics**

- [ ] **Functional**: Basic deposit/withdraw works in ZisK
- [ ] **Private**: ZK proofs provide actual privacy
- [ ] **Multi-user**: Multiple users can use the pool
- [ ] **Efficient**: Fast proof generation (< 1 minute)
- [ ] **Scalable**: Handles 100+ users per pool
- [ ] **Secure**: No double-spending, proper state management

## 🚀 **Next Steps**

1. **Set up ZisK environment** (Week 1)
2. **Implement core structures** (Week 2)
3. **Add Merkle tree operations** (Week 3)
4. **Implement transaction processing** (Week 4)
5. **Test and optimize** (Week 5-6)

This plan gives you a complete roadmap from ZeroPool's architecture to a ZisK-based privacy pool!
