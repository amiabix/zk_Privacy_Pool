#!/bin/bash

echo "🚀 Running ZisK Privacy Pool Multi-User Test"
echo "=============================================="

# Step 1: Create test data
echo "📝 Creating multi-user test data..."
cargo run --bin create_multi_user_test

# Step 2: Build for ZisK
echo "🔨 Building for ZisK..."
cargo-zisk build --release

# Step 3: Test in emulator
echo "🧪 Testing in ZisK emulator..."
cargo-zisk run --release -i build/input.bin -m

# Step 4: Generate proof (optional - takes longer)
echo "🔐 Generating ZK proof..."
cargo-zisk rom-setup -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm -k $HOME/.zisk/provingKey
cargo-zisk prove -e target/riscv64ima-zisk-zkvm-elf/release/privacy-pool-zkvm -i build/input.bin -o proof -a -y

echo "✅ Test completed!"
echo "Check the output above for transaction results and pool statistics."
