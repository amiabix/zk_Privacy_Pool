# Privacy Pool ZKVM Frontend

A React frontend for the Privacy Pool ZKVM system that allows users to connect their MetaMask wallet and interact with the privacy pool.

## Features

- 🔗 MetaMask wallet connection
- 💰 ETH balance display
- 📥 Deposit ETH into privacy pool
- 🔍 View private UTXOs
- 📤 Spend UTXOs (with ZK proof generation)
- 📱 Responsive design

## Setup

1. Install dependencies:
```bash
cd frontend
npm install
```

2. Start development server:
```bash
npm run dev
```

3. Open http://localhost:3000 in your browser

## Configuration

Update the contract address in `src/App.jsx`:
```javascript
const CONTRACT_ADDRESS = "YOUR_DEPLOYED_CONTRACT_ADDRESS"
```

## Requirements

- MetaMask browser extension
- Node.js 16+
- Your privacy pool contract deployed on the network

## Development

The frontend uses:
- React 18
- Vite for build tooling
- Ethers.js for Ethereum interaction
- CSS for styling

## Integration

This frontend connects to your Rust backend through:
1. Smart contract events for deposit/withdrawal
2. REST API calls for UTXO queries
3. WebSocket for real-time updates

Make sure your backend is running and the contract is deployed before using the frontend.
