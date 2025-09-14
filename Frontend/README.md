# Privacy Pool ZKVM Frontend

A comprehensive Privacy Pool system that provides users with an intuitive interface to interact with privacy-preserving transactions with private UTXO management and note transfers. 

## Overview

The user interface serves as the primary interaction point for the Privacy Pool ZKVM system, allowing users to deposit ETH publicly while maintaining complete privacy during transactions. The interface provides access to core privacy pool functionality across different devices and networks.

## Features

### Wallet Integration
- MetaMask wallet connection with network detection
- Support for multiple networks (Anvil local, Sepolia testnet)
- Live wallet balance monitoring
- Automatic network switching capabilities

### Transaction Management
- ETH deposit functionality with live confirmation
- Private UTXO viewing and management
- Transaction history and status tracking
- Gas estimation and transaction optimization

### Privacy Features
- Private balance display without revealing individual UTXOs
- Secure commitment viewing and verification
- Nullifier tracking for spent UTXOs
- Merkle tree position information

### User Experience
- Responsive design for desktop and mobile devices
- Live updates and notifications
- Intuitive navigation and user interface
- Error handling and user feedback

## Technical Architecture

### Core Technologies
- **Web Interface**: Modern web-based user interface
- **Build Tools**: Fast build tooling and development server
- **Blockchain Integration**: Ethereum blockchain interaction
- **Web3 Utilities**: Blockchain interaction utilities
- **Responsive Design**: Modern styling with responsive design

### State Management
- Local state management for user interface
- Global application state management
- Blockchain interaction state handling
- Live state synchronization

### Network Integration
- Multi-network support with automatic detection
- RPC endpoint configuration
- Contract address management
- Transaction monitoring and confirmation

## Installation and Setup

### Prerequisites
- Node.js 16 or higher
- npm or yarn package manager
- MetaMask browser extension
- Privacy pool contract deployed on supported network

### Installation

1. Navigate to the frontend directory:
```bash
cd Frontend
```

2. Install dependencies:
```bash
npm install
```

3. Configure environment variables (optional):
```bash
cp .env.example .env
# Edit .env with your configuration
```

### Development

1. Start the development server:
```bash
npm run dev
```

2. Open your browser and navigate to `http://localhost:3000`

3. Connect your MetaMask wallet to the supported network

### Building for Deployment

1. Build the application:
```bash
npm run build
```

2. Preview the built application:
```bash
npm run preview
```

## Configuration

### Contract Address

Update the contract address in `src/App.jsx` to match your deployed contract:

```javascript
const CONTRACT_ADDRESS = "0x19B8743Df3E8997489b50F455a1cAe3536C0ee31" // Sepolia deployed contract
```

### Network Configuration

The application supports multiple networks with automatic detection:

**Anvil (Local Development)**
- Chain ID: 31337
- RPC URL: http://127.0.0.1:8545
- Contract: Deploy locally using Hardhat

**Sepolia Testnet**
- Chain ID: 11155111
- RPC URL: https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
- Contract: Deploy using provided scripts

### API Configuration

Configure the backend API endpoint in the application:

```javascript
const API_BASE_URL = "http://localhost:3000" // Default API server URL
```

## Usage Guide

### Connecting Your Wallet

1. Click "Connect MetaMask" button
2. Authorize the connection in MetaMask
3. Ensure you're on the correct network
4. Verify your wallet address is displayed

### Depositing ETH

1. Enter the amount of ETH you want to deposit
2. Click "Deposit" button
3. Confirm the transaction in MetaMask
4. Wait for transaction confirmation
5. View your new private UTXO in the interface

### Viewing UTXOs

1. Navigate to "Your UTXOs" section
2. View all your private UTXOs with details:
   - UTXO ID and commitment hash
   - Amount and creation block
   - Tree position and spending status
3. Monitor UTXO status and updates

### Spending UTXOs

1. Select a UTXO from your list
2. Click "Spend UTXO" button
3. System generates zero-knowledge proof
4. Confirm the spending transaction
5. UTXO is marked as spent and removed from your balance

## API Integration

### Backend Communication

The frontend communicates with the Rust backend through:

1. **REST API Calls**: UTXO queries, balance checks, deposit processing
2. **Smart Contract Events**: Live deposit and withdrawal notifications
3. **WebSocket Connections**: Live updates and state synchronization

### API Endpoints

**Health Check**
```http
GET /api/health
```

**Deposit Processing**
```http
POST /api/deposit
Content-Type: application/json
{
  "depositor": "0x...",
  "amount": "1000000000000000000",
  "commitment": "0x...",
  "block_number": 12345,
  "transaction_hash": "0x..."
}
```

**UTXO Queries**
```http
GET /api/utxos/:owner?limit=100&after_block=0
```

**Balance Information**
```http
GET /api/balance/:owner
```

## Error Handling

### Common Issues

**Wallet Connection Issues**
- Ensure MetaMask is installed and unlocked
- Check network connection and RPC endpoint
- Verify wallet permissions and account access

**Transaction Failures**
- Check sufficient ETH balance for gas fees
- Verify transaction parameters and network
- Ensure contract is properly deployed

**API Communication Errors**
- Verify backend server is running
- Check API endpoint configuration
- Ensure proper network connectivity

### Error Messages

The application provides clear error messages for:
- Wallet connection failures
- Transaction rejections
- Network connectivity issues
- API communication errors
- Invalid input parameters

## Security Considerations

### Client-Side Security
- No private keys stored in the application
- Secure communication with MetaMask
- Input validation and sanitization
- HTTPS enforcement in deployment

### Privacy Protection
- UTXO details only visible to owner
- Commitment hashes for privacy preservation
- No transaction linking information stored
- Secure nullifier generation

## Performance Optimization

### Loading Performance
- Lazy loading of components
- Optimized bundle size with Vite
- Efficient state management
- Minimal re-renders

### Network Optimization
- Efficient API calls and caching
- Batch operations where possible
- Live updates without polling
- Optimized transaction handling

## Browser Compatibility

### Supported Browsers
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+

### Required Features
- Web3 provider support (MetaMask)
- ES6+ JavaScript support
- CSS Grid and Flexbox support
- Local storage API

## Development

### Project Structure

```
Frontend/
├── src/
│   ├── App.jsx              # Main application component
│   ├── index.css            # Global styles
│   ├── main.jsx             # Application entry point
│   └── vite.config.js       # Vite configuration
├── public/                  # Static assets
├── package.json             # Dependencies and scripts
└── README.md               # This file
```

### Available Scripts

```bash
npm run dev          # Start development server
npm run build        # Build for deployment
npm run preview      # Preview built application
npm run lint         # Run ESLint
npm run test         # Run tests
```

### Contributing

1. Follow the existing code style and patterns
2. Add tests for new functionality
3. Update documentation as needed
4. Ensure responsive design compatibility
5. Test with different networks and wallets

## Troubleshooting

### Development Issues

**Build Failures**
- Check Node.js version compatibility
- Clear node_modules and reinstall
- Verify all dependencies are installed

**Runtime Errors**
- Check browser console for error messages
- Verify MetaMask is properly connected
- Ensure backend API is running

**Network Issues**
- Verify RPC endpoint configuration
- Check network connectivity
- Ensure contract is deployed on correct network

### Debug Mode

Enable debug logging by setting:
```javascript
const DEBUG = true // In App.jsx
```

This will provide additional console output for troubleshooting.

## License

MIT License - see LICENSE file for details.

## Support

For technical support and questions:

1. Check the troubleshooting section
2. Review the API documentation
3. Examine the source code for implementation details
4. Open an issue on GitHub for bug reports
5. Contact the development team for additional support
