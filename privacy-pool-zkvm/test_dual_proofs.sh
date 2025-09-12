#!/bin/bash

echo "🎯 Testing Dual Proof Architecture"
echo "=================================="

# Create build directory
mkdir -p build

echo ""
echo "🔐 Testing Individual Transaction Proofs"
echo "========================================"

# 1. Create individual transaction test data
echo "Creating individual transaction test..."
cargo run --bin create_individual_tx_test

# 2. Build individual transaction proof program
echo "Building individual transaction proof program..."
cargo-zisk build --release --bin individual_tx

# 3. Test individual transaction in emulator
echo "Testing individual transaction in emulator..."
cargo-zisk run --release --bin individual_tx -i build/individual_input.bin

# 4. Generate individual transaction proof
echo "Generating individual transaction proof..."
cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/individual_tx -k $HOME/.zisk/provingKey
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/individual_tx -i build/individual_input.bin -o proof_individual -a -y

echo ""
echo "🏦 Testing Pool Accounting Proofs"
echo "================================="

# 1. Create pool batch test data
echo "Creating pool batch test..."
cargo run --bin create_pool_batch_test

# 2. Build pool accounting proof program
echo "Building pool accounting proof program..."
cargo-zisk build --release --bin pool_accounting

# 3. Test pool accounting in emulator
echo "Testing pool accounting in emulator..."
cargo-zisk run --release --bin pool_accounting -i build/pool_batch_input.bin

# 4. Generate pool accounting proof
echo "Generating pool accounting proof..."
cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/pool_accounting -k $HOME/.zisk/provingKey
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/pool_accounting -i build/pool_batch_input.bin -o proof_pool -a -y

echo ""
echo "✅ Dual Proof Architecture Test Complete!"
echo "========================================="
echo "Individual TX Proof: proof_individual/"
echo "Pool Accounting Proof: proof_pool/"
echo ""
echo "📊 Summary:"
echo "- Individual proofs: Fast, user-controlled, fully private"
echo "- Pool proofs: Comprehensive, operator-controlled, batch efficient"
echo "- Combined: Maximum security with practical efficiency"
