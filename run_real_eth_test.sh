#!/bin/bash

# Real ETH Deposit Test Automation Script
# This script sets up Anvil, deploys contracts, and runs the real ETH deposit test

set -e  # Exit on any error

echo "🚀 Starting Real ETH Deposit Test Automation"
echo "=============================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if required tools are installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command -v node &> /dev/null; then
        print_error "Node.js is not installed. Please install Node.js first."
        exit 1
    fi
    
    if ! command -v npm &> /dev/null; then
        print_error "npm is not installed. Please install npm first."
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    if ! command -v anvil &> /dev/null; then
        print_warning "Anvil not found. Installing via npm..."
        npm install -g @foundry-rs/anvil
    fi
    
    print_success "All dependencies are available"
}

# Start Anvil in the background
start_anvil() {
    print_status "Starting Anvil local blockchain..."
    
    # Kill any existing anvil processes
    pkill -f anvil || true
    sleep 2
    
    # Start Anvil in the background
    anvil --host 0.0.0.0 --port 8545 --chain-id 31337 --accounts 10 --balance 10000 > anvil.log 2>&1 &
    ANVIL_PID=$!
    
    # Wait for Anvil to start
    print_status "Waiting for Anvil to start..."
    sleep 5
    
    # Check if Anvil is running
    if ! kill -0 $ANVIL_PID 2>/dev/null; then
        print_error "Failed to start Anvil"
        cat anvil.log
        exit 1
    fi
    
    print_success "Anvil started with PID: $ANVIL_PID"
    echo $ANVIL_PID > anvil.pid
}

# Deploy contracts
deploy_contracts() {
    print_status "Deploying smart contracts..."
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        print_status "Installing npm dependencies..."
        npm install
    fi
    
    # Deploy contracts
    print_status "Deploying contracts to Anvil..."
    npx hardhat run scripts/deploy.js --network anvil
    
    if [ $? -eq 0 ]; then
        print_success "Contracts deployed successfully"
    else
        print_error "Contract deployment failed"
        exit 1
    fi
}

# Run the Rust test
run_rust_test() {
    print_status "Running Rust integration test..."
    
    # Compile the project first
    print_status "Compiling Rust project..."
    cargo build --release
    
    if [ $? -ne 0 ]; then
        print_error "Rust compilation failed"
        exit 1
    fi
    
    # Run the test
    print_status "Executing real ETH deposit test..."
    cargo test --lib test_real_eth_deposits_and_utxo_creation -- --nocapture
    
    if [ $? -eq 0 ]; then
        print_success "Rust test completed successfully"
    else
        print_error "Rust test failed"
        exit 1
    fi
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    
    # Kill Anvil if it's running
    if [ -f "anvil.pid" ]; then
        ANVIL_PID=$(cat anvil.pid)
        if kill -0 $ANVIL_PID 2>/dev/null; then
            print_status "Stopping Anvil (PID: $ANVIL_PID)..."
            kill $ANVIL_PID
            sleep 2
        fi
        rm -f anvil.pid
    fi
    
    # Clean up log files
    rm -f anvil.log
    
    print_success "Cleanup completed"
}

# Trap to ensure cleanup on exit
trap cleanup EXIT

# Main execution
main() {
    echo
    print_status "Starting Real ETH Deposit Test Automation"
    echo
    
    # Step 1: Check dependencies
    check_dependencies
    echo
    
    # Step 2: Start Anvil
    start_anvil
    echo
    
    # Step 3: Deploy contracts
    deploy_contracts
    echo
    
    # Step 4: Run Rust test
    run_rust_test
    echo
    
    print_success "🎉 Real ETH Deposit Test completed successfully!"
    print_status "Check the output above for detailed results"
    echo
}

# Run main function
main "$@"
