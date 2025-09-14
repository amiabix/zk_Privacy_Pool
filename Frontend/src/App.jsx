import React, { useState, useEffect } from 'react'
import { ethers } from 'ethers'

// Privacy Pool Contract ABI (simplified)
const PRIVACY_POOL_ABI = [
  "function depositAuto() external payable",
  "function depositWithCommitment(bytes32 commitment) external payable",
  "function withdraw(bytes32 nullifier, address recipient, uint256 amount) external",
  "function getContractBalance() external view returns (uint256)",
  "function getUserBalance(address account) external view returns (uint256)",
  "function isCommitmentUsed(bytes32 commitment) external view returns (bool)",
  "function isNullifierUsed(bytes32 nullifier) external view returns (bool)",
  "function merkleRoot() external view returns (bytes32)",
  "function totalDeposits() external view returns (uint256)",
  "function totalWithdrawals() external view returns (uint256)",
  "function previewCommitment(address depositor, uint256 amount) external view returns (bytes32)",
  "event Deposited(address indexed depositor, bytes32 indexed commitment, uint256 value, uint256 timestamp)",
  "event Withdrawn(address indexed recipient, bytes32 indexed nullifier, uint256 value, uint256 timestamp)",
  "event MerkleRootUpdated(bytes32 indexed oldRoot, bytes32 indexed newRoot, uint256 timestamp)"
]

// Contract address (you'll need to update this with your deployed contract)
const CONTRACT_ADDRESS = "0x19B8743Df3E8997489b50F455a1cAe3536C0ee31" // Sepolia deployed contract

function App() {
  const [account, setAccount] = useState(null)
  const [provider, setProvider] = useState(null)
  const [signer, setSigner] = useState(null)
  const [contract, setContract] = useState(null)
  const [balance, setBalance] = useState('0')
  const [ethBalance, setEthBalance] = useState('0')
  const [isConnected, setIsConnected] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState(null)
  const [utxos, setUtxos] = useState([])
  const [depositAmount, setDepositAmount] = useState('')
  const [currentNetwork, setCurrentNetwork] = useState(null)
  const [isCorrectNetwork, setIsCorrectNetwork] = useState(false)
  const [apiConnected, setApiConnected] = useState(false)
  const [treeStats, setTreeStats] = useState(null)

  // Check if MetaMask is installed
  const isMetaMaskInstalled = () => {
    return typeof window.ethereum !== 'undefined'
  }

  // Check API connection and load tree stats
  const checkApiConnection = async () => {
    try {
      const response = await fetch('http://localhost:8080/health')
      if (response.ok) {
        const health = await response.json()
        setApiConnected(true)
        console.log(' API Connected:', health)
        
        // Load tree stats
        const statsResponse = await fetch('http://localhost:8080/api/tree/stats')
        if (statsResponse.ok) {
          const statsData = await statsResponse.json()
          setTreeStats(statsData.stats)
          console.log(' Tree Stats:', statsData.stats)
        }
      } else {
        setApiConnected(false)
        console.log(' API not responding')
      }
    } catch (err) {
      console.log(' API server not available:', err.message)
      setApiConnected(false)
    }
  }

  // Connect to MetaMask
  const connectWallet = async () => {
    try {
      setError(null)
      setIsLoading(true)

      if (!isMetaMaskInstalled()) {
        throw new Error('MetaMask is not installed. Please install MetaMask to continue.')
      }

      // Request account access
      const accounts = await window.ethereum.request({
        method: 'eth_requestAccounts'
      })

      if (accounts.length === 0) {
        throw new Error('No accounts found')
      }

      const account = accounts[0]
      setAccount(account)

      // Create provider and signer
      const provider = new ethers.BrowserProvider(window.ethereum)
      const signer = await provider.getSigner()
      
      setProvider(provider)
      setSigner(signer)

      // Check network
      const network = await provider.getNetwork()
      setCurrentNetwork(network)
      const isAnvil = network.chainId === 31337n
      const isSepolia = network.chainId === 11155111n
      setIsCorrectNetwork(isAnvil || isSepolia)

      // Create contract instance (only if on supported network)
      let contract = null
      if (isAnvil || isSepolia) {
        contract = new ethers.Contract(CONTRACT_ADDRESS, PRIVACY_POOL_ABI, signer)
        setContract(contract)
      } else {
        setContract(null)
      }

      setIsConnected(true)

      // Load initial data
      await loadWalletData(provider, contract, account)
      
      // Check API connection
      await checkApiConnection()

    } catch (err) {
      setError(err.message)
      console.error('Error connecting wallet:', err)
    } finally {
      setIsLoading(false)
    }
  }

  // Load wallet data
  const loadWalletData = async (provider, contract, account) => {
    try {
      // Get ETH balance
      const ethBalance = await provider.getBalance(account)
      setEthBalance(ethers.formatEther(ethBalance))

      // Get contract balance (only if contract exists and we're on the right network)
      try {
        const network = await provider.getNetwork()
        console.log('Current network:', network.chainId.toString())
        
        // Check if we're on the correct network (Anvil: 31337, Sepolia: 11155111)
        if (network.chainId === 31337n) {
          const contractBalance = await contract.getContractBalance()
          setBalance(ethers.formatEther(contractBalance))
        } else if (network.chainId === 11155111n) {
          // Sepolia network - get balance from API
          try {
            const response = await fetch(`http://localhost:8080/api/balance/${account}`)
            if (response.ok) {
              const data = await response.json()
              setBalance(data.balance.balance)
            } else {
              setBalance('0')
            }
          } catch (err) {
            console.log('API not available, using contract balance')
            setBalance('0')
          }
        } else {
          console.log('Not on supported network, contract balance not available')
          setBalance('0')
        }
      } catch (err) {
        console.log('Contract balance method not available:', err.message)
        setBalance('0')
      }

      // Load UTXOs from API
      try {
        const response = await fetch(`http://localhost:8080/api/utxos/${account}`)
        if (response.ok) {
          const data = await response.json()
          setUtxos(data.utxos.map(utxo => ({
            id: utxo.utxo_id,
            amount: utxo.amount,
            commitment: utxo.commitment,
            created_block: utxo.created_block,
            tree_position: utxo.tree_position,
            is_spent: utxo.is_spent
          })))
        } else {
          // Fallback to mock data if API not available
          setUtxos([
            {
              id: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
              amount: '1.5',
              commitment: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
            },
            {
              id: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
              amount: '0.5',
              commitment: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
            }
          ])
        }
      } catch (err) {
        console.log('API not available, using mock data')
        setUtxos([
          {
            id: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef',
            amount: '1.5',
            commitment: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890'
          },
          {
            id: '0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890',
            amount: '0.5',
            commitment: '0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef'
          }
        ])
      }

    } catch (err) {
      console.error('Error loading wallet data:', err)
      setError('Failed to load wallet data')
    }
  }

  // Handle deposit
  const handleDeposit = async () => {
    if (!contract || !depositAmount) return

    try {
      setError(null)
      setIsLoading(true)

      const amount = ethers.parseEther(depositAmount)
      
      const tx = await contract.depositAuto({
        value: amount
      })

      console.log('Deposit transaction:', tx.hash)
      
      // Wait for transaction to be mined
      const receipt = await tx.wait()
      
      // Send deposit data to API
      try {
        const depositData = {
          depositor: account,
          amount: ethers.parseEther(depositAmount).toString(), // Convert to wei string
          commitment: "0x0000000000000000000000000000000000000000000000000000000000000000", // Placeholder - API will generate
          block_number: receipt.blockNumber,
          transaction_hash: tx.hash
        }
        
        const response = await fetch('http://localhost:8080/api/deposit', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(depositData)
        })
        
        if (response.ok) {
          const result = await response.json()
          console.log('Deposit processed by API:', result)
        }
      } catch (apiErr) {
        console.log('API not available, deposit recorded on-chain only')
      }
      
      // Reload wallet data
      await loadWalletData(provider, contract, account)
      
      setDepositAmount('')
      alert('Deposit successful!')

    } catch (err) {
      setError(err.message)
      console.error('Error depositing:', err)
    } finally {
      setIsLoading(false)
    }
  }

  // Handle withdrawal (simplified)
  const handleWithdraw = async (utxoId, amount) => {
    if (!contract) return

    try {
      setError(null)
      setIsLoading(true)

      // This is a simplified withdrawal - in reality you'd need ZK proofs
      console.log('Withdrawing UTXO:', utxoId, 'Amount:', amount)
      
      // For now, just show a message
      alert('Withdrawal requires ZK proof generation (not implemented in this demo)')

    } catch (err) {
      setError(err.message)
      console.error('Error withdrawing:', err)
    } finally {
      setIsLoading(false)
    }
  }

  // Listen for account changes
  useEffect(() => {
    if (isMetaMaskInstalled()) {
      window.ethereum.on('accountsChanged', (accounts) => {
        if (accounts.length === 0) {
          // User disconnected
          setAccount(null)
          setIsConnected(false)
          setProvider(null)
          setSigner(null)
          setContract(null)
          setBalance('0')
          setEthBalance('0')
          setUtxos([])
        } else {
          // User switched accounts
          setAccount(accounts[0])
          if (provider && contract) {
            loadWalletData(provider, contract, accounts[0])
          }
        }
      })

      window.ethereum.on('chainChanged', () => {
        // Reload the page when chain changes
        window.location.reload()
      })
    }
  }, [provider, contract])

  return (
    <div className="container">
      <div className="header">
        <h1> Privacy Pool ZKVM</h1>
        <p>Deposit ETH publicly, spend privately with zero-knowledge proofs</p>
      </div>

      <div className="card">
        <div className="wallet-section">
          {!isConnected ? (
            <div>
              <h2>Connect Your Wallet</h2>
              <p>Connect to MetaMask to start using the Privacy Pool</p>
              <button 
                className="button" 
                onClick={connectWallet}
                disabled={isLoading}
              >
                {isLoading ? 'Connecting...' : 'Connect MetaMask'}
              </button>
            </div>
          ) : (
            <div>
              <h2>Wallet Connected</h2>
              <div className="wallet-info">
                <div className="wallet-address">
                  <strong>Address:</strong> {account}
                </div>
                <div className="balance">
                  ETH Balance: {ethBalance} ETH
                </div>
                {currentNetwork && (
                  <div className="balance">
                    Network: {currentNetwork.name} (Chain ID: {currentNetwork.chainId.toString()})
                  </div>
                )}
                {isCorrectNetwork && (
                  <div className="balance">
                    Privacy Pool Balance: {balance} ETH
                  </div>
                )}
              </div>

              <div className={`status ${isCorrectNetwork ? 'connected' : 'error'}`}>
                {isCorrectNetwork ? 
                  (currentNetwork?.chainId === 31337n ? ' Connected to Privacy Pool (Anvil)' : ' Connected to Privacy Pool (Sepolia)') : 
                  ' Switch to Anvil or Sepolia network to use Privacy Pool'
                }
              </div>
              
              <div className={`status ${apiConnected ? 'connected' : 'error'}`}>
                {apiConnected ? 
                  ' Rust API Server Connected' : 
                  ' Rust API Server Not Available (using mock data)'
                }
              </div>
              
              {!isCorrectNetwork && (
                <div style={{ display: 'flex', gap: '1rem', justifyContent: 'center' }}>
                  <button 
                    className="button secondary"
                    onClick={async () => {
                      try {
                        await window.ethereum.request({
                          method: 'wallet_switchEthereumChain',
                          params: [{ chainId: '0x7A69' }], // 31337 in hex
                        });
                      } catch (switchError) {
                        if (switchError.code === 4902) {
                          try {
                            await window.ethereum.request({
                              method: 'wallet_addEthereumChain',
                              params: [{
                                chainId: '0x7A69',
                                chainName: 'Anvil',
                                rpcUrls: ['http://127.0.0.1:8545'],
                                nativeCurrency: {
                                  name: 'Ethereum',
                                  symbol: 'ETH',
                                  decimals: 18
                                }
                              }]
                            });
                          } catch (addError) {
                            console.error('Error adding Anvil network:', addError);
                          }
                        }
                      }
                    }}
                  >
                    Switch to Anvil
                  </button>
                  <button 
                    className="button secondary"
                    onClick={async () => {
                      try {
                        await window.ethereum.request({
                          method: 'wallet_switchEthereumChain',
                          params: [{ chainId: '0xaa36a7' }], // 11155111 in hex
                        });
                      } catch (switchError) {
                        console.error('Error switching to Sepolia:', switchError);
                      }
                    }}
                  >
                    Switch to Sepolia
                  </button>
                </div>
              )}
            </div>
          )}

          {error && (
            <div className="error">
              {error}
            </div>
          )}
        </div>
      </div>

      {isConnected && isCorrectNetwork && currentNetwork?.chainId === 31337n && (
        <>
          <div className="card">
            <h2>Deposit ETH</h2>
            <p>Convert your ETH into private UTXOs</p>
            <div style={{ display: 'flex', gap: '1rem', marginTop: '1rem' }}>
              <input
                type="number"
                placeholder="Amount in ETH"
                value={depositAmount}
                onChange={(e) => setDepositAmount(e.target.value)}
                style={{
                  padding: '0.75rem',
                  border: '1px solid #ddd',
                  borderRadius: '8px',
                  flex: 1
                }}
              />
              <button 
                className="button"
                onClick={handleDeposit}
                disabled={isLoading || !depositAmount}
              >
                {isLoading ? 'Depositing...' : 'Deposit'}
              </button>
            </div>
          </div>

          <div className="card">
            <h2>Your UTXOs</h2>
            <p>Private transaction outputs you can spend</p>
            <div className="utxo-list">
              {utxos.length === 0 ? (
                <div className="loading">No UTXOs found</div>
              ) : (
                utxos.map((utxo, index) => (
                  <div key={index} className="utxo-item">
                    <div className="utxo-amount">{utxo.amount} ETH</div>
                    <div className="utxo-id">ID: {utxo.id}</div>
                    <div className="utxo-id">Commitment: {utxo.commitment}</div>
                    <button 
                      className="button secondary"
                      onClick={() => handleWithdraw(utxo.id, utxo.amount)}
                      disabled={isLoading}
                      style={{ marginTop: '0.5rem' }}
                    >
                      Spend UTXO
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>
        </>
      )}

      {isConnected && isCorrectNetwork && currentNetwork?.chainId === 11155111n && (
        <>
          <div className="card">
            <h2>Deposit ETH (Sepolia)</h2>
            <p>Convert your Sepolia ETH into private UTXOs</p>
            <div style={{ display: 'flex', gap: '1rem', marginTop: '1rem' }}>
              <input
                type="number"
                placeholder="Amount in ETH"
                value={depositAmount}
                onChange={(e) => setDepositAmount(e.target.value)}
                style={{
                  padding: '0.75rem',
                  border: '1px solid #ddd',
                  borderRadius: '8px',
                  flex: 1
                }}
              />
              <button 
                className="button"
                onClick={handleDeposit}
                disabled={isLoading || !depositAmount}
              >
                {isLoading ? 'Depositing...' : 'Deposit'}
              </button>
            </div>
          </div>

          <div className="card">
            <h2>Your UTXOs (Sepolia)</h2>
            <p>Private transaction outputs you can spend</p>
            <div className="utxo-list">
              {utxos.length === 0 ? (
                <div className="loading">No UTXOs found</div>
              ) : (
                utxos.map((utxo, index) => (
                  <div key={index} className="utxo-item">
                    <div className="utxo-amount">{utxo.amount} ETH</div>
                    <div className="utxo-id">ID: {utxo.id}</div>
                    <div className="utxo-id">Commitment: {utxo.commitment}</div>
                    <button 
                      className="button secondary"
                      onClick={() => handleWithdraw(utxo.id, utxo.amount)}
                      disabled={isLoading}
                      style={{ marginTop: '0.5rem' }}
                    >
                      Spend UTXO
                    </button>
                  </div>
                ))
              )}
            </div>
          </div>

          <div className="card">
            <h2>Contract Information</h2>
            <div style={{ marginTop: '1rem' }}>
              <p><strong>Contract Address:</strong> 0x76c0e1372EEe04EE2BdbBD7eDa7C3B2102009026</p>
              <p><strong>Network:</strong> Sepolia Testnet</p>
              <p><strong>Owner:</strong> 0x6a685bF2E7C92a51b8B0fBadE91Cda085E304B46</p>
              <p><strong>Merkle Root:</strong> 0x04735efbce809c030d37ba49c991137ee9bae0681dd865766a2c50dd1c301282</p>
            </div>
          </div>
        </>
      )}

      {apiConnected && treeStats && (
        <div className="card">
          <h2>Merkle Tree Statistics</h2>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '1rem', marginTop: '1rem' }}>
            <div>
              <h3>Current Root</h3>
              <p style={{ fontFamily: 'monospace', fontSize: '0.9rem', wordBreak: 'break-all' }}>
                {treeStats.current_root}
              </p>
            </div>
            <div>
              <h3>Total UTXOs</h3>
              <p style={{ fontSize: '1.5rem', fontWeight: 'bold', color: '#667eea' }}>
                {treeStats.total_utxos}
              </p>
            </div>
            <div>
              <h3>Tree Depth</h3>
              <p style={{ fontSize: '1.5rem', fontWeight: 'bold', color: '#667eea' }}>
                {treeStats.tree_depth}
              </p>
            </div>
            <div>
              <h3>Root Version</h3>
              <p style={{ fontSize: '1.5rem', fontWeight: 'bold', color: '#667eea' }}>
                {treeStats.root_version}
              </p>
            </div>
          </div>
        </div>
      )}

      <div className="card">
        <h2>How It Works</h2>
        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))', gap: '1rem', marginTop: '1rem' }}>
          <div>
            <h3>1. Deposit</h3>
            <p>Send ETH to the privacy pool contract. Your deposit creates a private UTXO.</p>
          </div>
          <div>
            <h3>2. Privacy</h3>
            <p>Your UTXO is stored in a Merkle tree. Only you know which UTXO is yours.</p>
          </div>
          <div>
            <h3>3. Spend</h3>
            <p>Generate a zero-knowledge proof to spend your UTXO without revealing details.</p>
          </div>
          <div>
            <h3>4. Withdraw</h3>
            <p>Withdraw to any address with complete privacy protection.</p>
          </div>
        </div>
      </div>
    </div>
  )
}

export default App
