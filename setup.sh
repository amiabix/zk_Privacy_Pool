#!/bin/bash

# Privacy Pool Verifier Setup Script
# This script sets up the development environment for the smart contract

set -e

echo "🚀 Privacy Pool Verifier Setup"
echo "================================"

# Check if Node.js is installed
if ! command -v node &> /dev/null; then
    echo "❌ Node.js is not installed. Please install Node.js first."
    echo "   Visit: https://nodejs.org/"
    exit 1
fi

echo "✅ Node.js found: $(node --version)"

# Check if npm is installed
if ! command -v npm &> /dev/null; then
    echo "❌ npm is not installed. Please install npm first."
    exit 1
fi

echo "✅ npm found: $(npm --version)"

# Install dependencies
echo ""
echo "📦 Installing dependencies..."
npm install

# Create necessary directories
echo ""
echo "📁 Creating directories..."
mkdir -p deployments
mkdir -p artifacts
mkdir -p cache

# Copy environment file if it doesn't exist
if [ ! -f .env ]; then
    echo ""
    echo "📄 Creating .env file..."
    cp env.example .env
    echo "⚠️  Please edit .env file with your configuration"
else
    echo "✅ .env file already exists"
fi

# Compile contracts
echo ""
echo "🔨 Compiling contracts..."
npm run compile

# Run tests
echo ""
echo "🧪 Running tests..."
npm test

echo ""
echo "🎉 Setup completed successfully!"
echo ""
echo "Next steps:"
echo "1. Edit .env file with your private key and RPC URLs"
echo "2. Deploy to Plasma network: npm run deploy:plasma"
echo "3. Generate test proof data: node scripts/generate-test-proof.js"
echo "4. Test proof verification: node scripts/verify-proof.js"
echo ""
echo "For more information, see contracts/README.md"
