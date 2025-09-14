# Privacy Pool ZKVM

A comprehensive privacy-preserving transaction system built with zero-knowledge virtual machine (zkVM) technology. This implementation enables users to deposit ETH publicly while maintaining complete privacy during transactions through UTXO-based architecture and zero-knowledge proofs.

## Overview

The Privacy Pool ZKVM system provides a complete solution for private transactions on Ethereum-compatible networks. Users can deposit ETH into a smart contract, receive private UTXOs stored in a Merkle tree, and spend these UTXOs using zero-knowledge proofs without revealing transaction details.

## Architecture

### Core Components

**Rust Backend**
- Canonical UTXO implementation with byte-level serialization
- Enhanced Merkle tree with efficient operations
- REST API server with blockchain verification
- Live deposit event processing
- Cryptographic utilities and ZisK precompiles

**Smart Contracts**
- Privacy pool contract for ETH deposits and withdrawals
- Commitment tracking and nullifier management
- Merkle root updates and state synchronization
- Multi-network support (Anvil, Sepolia)

**Zero-Knowledge Circuits**
- Circom circuits for commitment generation
- Noir circuits for withdrawal proofs
- Poseidon hashing for cryptographic operations
- Merkle proof verification

**User Interface**
- Web-based interface for system interaction
- Wallet integration capabilities
- UTXO management interface
- Multi-network support

## Key Features

### Privacy Mechanisms
- **UTXO-based Privacy**: Users deposit ETH publicly, receive private UTXOs
- **Merkle Tree Storage**: UTXOs stored in 32-level Merkle tree with privacy-preserving commitments
- **Zero-Knowledge Proofs**: Circom/Noir circuits for private spending without revealing details
- **Nullifier System**: Prevents double-spending without revealing UTXO information

### Blockchain Integration
- **Transaction Verification**: API server verifies actual blockchain deposits
- **Multi-Network Support**: Anvil (local) and Sepolia testnet compatibility
- **Event Processing**: Live deposit event monitoring and processing
- **State Synchronization**: Merkle tree updates with blockchain state changes

### Technical Specifications
- **Canonical Serialization**: Byte-level format specifications for all data structures
- **Database Integration**: RocksDB with column families for efficient storage
- **Error Handling**: Comprehensive error types and recovery mechanisms
- **API Documentation**: Well-documented REST endpoints with clear interfaces

## System Flow

### Deposit Process
1. User sends ETH to smart contract
2. Contract creates commitment and emits deposit event
3. API server verifies transaction on blockchain
4. System generates private UTXO and updates Merkle tree
5. User receives UTXO with privacy-preserving commitment

### Spending Process
1. User generates zero-knowledge proof proving UTXO ownership
2. Proof includes Merkle tree membership and nullifier generation
3. Smart contract verifies proof and updates state
4. User withdraws to any address with complete privacy

## Installation and Setup

### Prerequisites
- Rust 1.70+ (for backend)
- Node.js 16+ (for frontend and contracts)
- MetaMask browser extension
- Git

### Backend Setup

1. Clone the repository:
```bash
git clone <repository-url>
cd privacy-pool-zkvm
```

2. Install Rust dependencies:
```bash
cargo build
```

3. Run the API server:
```bash
cargo run --bin api_server
```

The API server will start on `http://localhost:3000` with the following endpoints:
- `GET /api/health` - Health check
- `POST /api/deposit` - Process ETH deposit
- `GET /api/balance/:owner` - Get owner balance
- `GET /api/utxos/:owner` - Get owner UTXOs
- `GET /api/tree/stats` - Get tree statistics

### Smart Contract Deployment

1. Install dependencies:
```bash
npm install
```

2. Compile contracts:
```bash
npm run compile
```

3. Deploy to local network:
```bash
npm run node
npm run deploy:local
```

4. Deploy to Sepolia testnet:
```bash
npm run deploy:sepolia
```

### Frontend Setup

1. Navigate to frontend directory:
```bash
cd Frontend
```

2. Install dependencies:
```bash
npm install
```

3. Start development server:
```bash
npm run dev
```

4. Open `http://localhost:3000` in your browser

## Configuration

### Environment Variables

Create a `.env` file in the project root:

```bash
# RPC URLs
SEPOLIA_RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
ANVIL_RPC_URL=http://127.0.0.1:8545

# Contract Addresses
PRIVACY_POOL_CONTRACT=0x19B8743Df3E8997489b50F455a1cAe3536C0ee31

# API Configuration
BIND_ADDR=127.0.0.1:3000
MAX_REQUEST_SIZE=1048576
REQUEST_TIMEOUT=30
```

### Network Configuration

The system supports multiple networks:

**Anvil (Local Development)**
- Chain ID: 31337
- RPC URL: http://127.0.0.1:8545
- Use for development and testing

**Sepolia Testnet**
- Chain ID: 11155111
- RPC URL: https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
- Use for testing with testnet ETH

## Usage

### Depositing ETH

1. Connect MetaMask wallet to supported network
2. Enter deposit amount in the frontend
3. Confirm transaction in MetaMask
4. System verifies transaction and creates private UTXO
5. UTXO appears in your private balance

### Viewing UTXOs

1. Navigate to "Your UTXOs" section
2. View all private UTXOs with amounts and commitments
3. Check tree position and creation block
4. Monitor spending status

### Spending UTXOs

1. Select UTXO to spend
2. System generates zero-knowledge proof
3. Proof includes Merkle tree membership verification
4. Transaction executes with complete privacy

## API Reference

### Deposit Endpoint

```http
POST /api/deposit
Content-Type: application/json

{
  "depositor": "0x...",
  "amount": "1000000000000000000",
  "commitment": "0x...",
  "block_number": 12345,
  "transaction_hash": "0x...",
  "label": "optional_label",
  "precommitment_hash": "0x..."
}
```

### UTXO Query Endpoint

```http
GET /api/utxos/:owner?limit=100&after_block=0&asset_id=ETH
```

### Balance Endpoint

```http
GET /api/balance/:owner
```

## Development

### Project Structure

```
privacy-pool-zkvm/
├── src/                    # Rust backend source
│   ├── api/               # REST API implementation
│   ├── utxo/              # UTXO system and management
│   ├── privacy/           # Privacy pool core logic
│   ├── merkle/            # Merkle tree implementation
│   ├── relayer/           # Blockchain integration
│   ├── circuits/          # Zero-knowledge circuits
│   └── contracts/         # Smart contracts
├── Frontend/              # React frontend application
├── scripts/               # Deployment and utility scripts
└── artifacts/             # Compiled contract artifacts
```

### Testing

Run the test suite:

```bash
# Rust tests
cargo test

# Smart contract tests
npm test

# Frontend tests
cd Frontend
npm test
```

### Building

Build all components:

```bash
# Backend
cargo build --release

# Smart contracts
npm run compile

# Frontend
cd Frontend
npm run build
```

## Security Considerations

### Current Implementation Status

The system implements a complete privacy pool architecture with the following security features:

- **Cryptographic Commitments**: Poseidon hashing for commitment generation
- **Merkle Tree Verification**: Proper Merkle proof validation
- **Nullifier Management**: Double-spend prevention through nullifier tracking
- **Transaction Verification**: Blockchain transaction validation

### Security Features

- **Access Control**: Owner-only functions for critical operations
- **Reentrancy Protection**: Prevents reentrancy attacks in smart contracts
- **Input Validation**: Comprehensive validation of all inputs
- **State Consistency**: Ensures valid state transitions

## Performance

### Gas Optimization

- **Smart Contract**: Optimized for gas efficiency with packed structs
- **Merkle Tree**: Efficient insertion and proof generation
- **API Server**: High-performance request handling with async operations

### Scalability

- **Database**: RocksDB with column families for efficient storage
- **Caching**: LRU cache for frequently accessed data
- **Batch Processing**: Parallel processing for bulk operations

## Monitoring and Events

### Key Events

**Smart Contract Events**
- `Deposited`: ETH deposit with commitment
- `Withdrawn`: UTXO withdrawal with nullifier
- `MerkleRootUpdated`: Tree state changes

**API Events**
- Deposit processing status
- UTXO creation and updates
- Tree state changes

### Monitoring

- Track deposit and withdrawal success rates
- Monitor gas usage patterns
- Watch for failed transactions
- Track nullifier usage and UTXO statistics

## Troubleshooting

### Common Issues

**Connection Issues**
- Ensure MetaMask is connected to correct network
- Check RPC URL configuration
- Verify contract address is correct

**Transaction Failures**
- Check sufficient ETH balance for gas
- Verify transaction parameters
- Ensure contract is properly deployed

**API Errors**
- Check API server is running
- Verify request format and parameters
- Check blockchain connectivity

### Debug Commands

```bash
# Check API server status
curl http://localhost:3000/api/health

# Check contract state
npx hardhat console --network sepolia
> const contract = await ethers.getContractAt("PrivacyPoolFixed", "0x...");
> await contract.getContractBalance();

# View logs
cargo run --bin api_server -- --log-level debug
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests for new functionality
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Support

For technical support and questions:

1. Check the troubleshooting section
2. Review the API documentation
3. Examine the test files for usage examples
4. Open an issue on GitHub

## Roadmap

### Planned Features

- **Enhanced Privacy**: Additional privacy-preserving features
- **Multi-Asset Support**: Support for ERC-20 tokens
- **Mobile Support**: Mobile application development
- **Advanced Analytics**: Enhanced monitoring and analytics

### Future Improvements

- **Gas Optimization**: Further gas usage improvements
- **Scalability**: Enhanced scalability solutions
- **Security**: Additional security features and audits
- **Documentation**: Expanded documentation and tutorials
