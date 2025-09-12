# ZisK Privacy Pool Implementation

A privacy-preserving mixing pool implementation using ZisK zkVM, based on ZeroPool's architecture.

## 🎯 **Features**

- **Multi-user support** - Multiple users can deposit/withdraw
- **True privacy** - ZK proofs provide actual anonymity
- **Merkle tree state** - Efficient state management
- **Nullifier system** - Prevents double-spending
- **ZisK zkVM** - Fast proving with Rust-native development

## 🚀 **Quick Start**

### 1. Build the Project

```bash
# Build for ZisK
cargo-zisk build --release
```

### 2. Create Test Input

```bash
# Create test transaction data
cargo run --bin create_test_input
```

### 3. Test in Emulator

```bash
# Test in ZisK emulator
cargo-zisk run --release -i build/input.bin
```

### 4. Generate Proof

```bash
# Setup ROM (first time only)
cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm -k $HOME/.zisk/provingKey

# Generate proof
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm -i build/input.bin -o proof -a -y
```

## 📁 **Project Structure**

```
privacy-pool-zkvm/
├── src/
│   ├── main.rs              # ZisK entry point
│   ├── lib.rs               # Module exports
│   ├── privacy_pool.rs      # Core privacy pool logic
│   ├── transaction.rs       # Transaction types and structures
│   ├── merkle_tree.rs       # Merkle tree operations
│   └── zk_proofs.rs         # ZK proof generation
├── build/
│   └── input.bin            # Input data for ZisK
├── create_test_input.rs     # Test data generator
└── Cargo.toml
```

## 🔧 **Key Components**

### **PrivacyPool** (`privacy_pool.rs`)
- Main privacy pool logic
- Transaction processing
- State management
- Multi-user coordination

### **Transaction Types** (`transaction.rs`)
- `Deposit` - Add funds to the pool
- `Withdraw` - Remove funds from the pool
- `Transfer` - Internal pool transfers

### **Merkle Tree** (`merkle_tree.rs`)
- Efficient state storage
- Proof generation
- Tree updates

### **ZK Proofs** (`zk_proofs.rs`)
- Commitment generation
- Nullifier hashing
- Merkle proof verification

## 🧪 **Testing**

### **Unit Tests**
```bash
cargo test
```

### **Integration Tests**
```bash
cargo-zisk run --release -i build/input.bin
```

### **Performance Testing**
```bash
# With metrics
cargo-zisk run --release -i build/input.bin -m

# With statistics
cargo-zisk run --release -i build/input.bin -x
```

## 🚀 **Performance Optimization**

### **Concurrent Proof Generation**
```bash
# Multi-process proof generation
mpirun --bind-to none -np 4 -x OMP_NUM_THREADS=8 -x RAYON_NUM_THREADS=8 \
    cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm \
    -i build/input.bin -o proof -a -y
```

### **GPU Acceleration**
```bash
# Build with GPU support
cargo build --release --features gpu

# Run with GPU
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm \
    -i build/input.bin -o proof -a -y --gpu
```

## 📊 **Expected Performance**

- **Proof Generation**: < 1 minute for basic transactions
- **Throughput**: 100+ transactions per hour
- **Memory Usage**: ~25GB per proof process
- **Scalability**: Supports 100+ users per pool

## 🔒 **Security Features**

- **Double-spend prevention** via nullifier system
- **Privacy preservation** through ZK proofs
- **State consistency** via Merkle tree verification
- **Transaction integrity** through cryptographic commitments

## 🔗 **Smart Contract Integration**

### **On-Chain Verification**
The project includes a complete smart contract system for on-chain verification of ZisK proofs:

- **PrivacyPoolVerifier.sol**: Main verification contract
- **Plasma Network Deployment**: Deploy to Polygon Plasma
- **Proof Verification**: On-chain ZK proof validation
- **State Management**: Merkle root and balance tracking

### **Quick Start - Smart Contract**

```bash
# Setup development environment
./setup.sh

# Deploy to Plasma network
npm run deploy:plasma

# Generate test proof data
node scripts/generate-test-proof.js

# Test proof verification
node scripts/verify-proof.js
```

### **Contract Features**
- ✅ **Proof Verification**: Verify ZisK zkVM proofs on-chain
- ✅ **State Management**: Track Merkle root and pool balance
- ✅ **Nullifier Tracking**: Prevent double-spending
- ✅ **Replay Protection**: Prevent proof replay attacks
- ✅ **Event Logging**: Comprehensive event system

## 🎯 **Next Steps**

1. **Deploy Smart Contract** to Plasma network
2. **Integrate ZisK Verifier** for production use
3. **Add Poseidon hash** for better ZK performance
4. **Implement batch processing** for multiple transactions
5. **Add multi-pool support** for different denominations
6. **Add relayer system** for transaction coordination

## 📚 **References**

- [ZisK Documentation](https://0xpolygonhermez.github.io/zisk/getting_started/writing_programs.html)
- [ZeroPool Architecture](https://github.com/zeropoolnetwork/zeropool-substrate)
- [Privacy Pool Concepts](https://vitalik.ca/general/2022/11/22/poe.html)
