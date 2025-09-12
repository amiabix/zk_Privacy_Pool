import { ethers } from 'ethers';
import { MerkleTree } from 'merkletreejs';
import { poseidon } from 'circomlibjs';

/**
 * Enhanced Merkle Tree Service for Privacy Pool
 * Handles commitment insertion and UTXO binding to owners
 */

export interface DepositEvent {
  depositor: string;
  commitment: string;
  value: string;
  blockNumber: number;
  transactionHash: string;
  logIndex: number;
}

export interface UTXO {
  commitment: string;
  value: bigint;
  secret: string;
  nullifier: string;
  owner: string;
  leafIndex: number;
  height: number;
}

export interface MerkleProof {
  siblings: string[];
  pathIndices: boolean[];
  root: string;
  leafIndex: number;
}

export interface TreeStats {
  totalUTXOs: number;
  treeDepth: number;
  rootHash: string;
  nextLeafIndex: number;
}

export class EnhancedMerkleTreeService {
  private utxos: UTXO[] = [];
  private commitmentToIndex: Map<string, number> = new Map();
  private ownerToUTXOs: Map<string, number[]> = new Map();
  private nextLeafIndex: number = 0;
  private merkleTree: MerkleTree;
  private lastProcessedBlock: number = 0;

  constructor() {
    this.merkleTree = new MerkleTree([], ethers.utils.keccak256, {
      sortPairs: true,
    });
  }

  /**
   * Process a deposit event and insert commitment into tree
   */
  async processDepositEvent(
    event: DepositEvent,
    secret?: string,
    nullifier?: string
  ): Promise<number> {
    // Generate secret and nullifier if not provided
    const utxoSecret = secret || this.generateSecret(event.blockNumber, event.value);
    const utxoNullifier = nullifier || this.generateNullifier(utxoSecret, this.nextLeafIndex);

    // Create UTXO with proper commitment binding
    const utxo: UTXO = {
      commitment: event.commitment,
      value: BigInt(event.value),
      secret: utxoSecret,
      nullifier: utxoNullifier,
      owner: event.depositor,
      leafIndex: this.nextLeafIndex,
      height: event.blockNumber,
    };

    // Check for duplicate commitment
    if (this.commitmentToIndex.has(event.commitment)) {
      throw new Error('Commitment already exists in tree');
    }

    const index = this.utxos.length;

    // Add to UTXO list
    this.utxos.push(utxo);

    // Update mappings
    this.commitmentToIndex.set(event.commitment, index);
    const ownerUTXOs = this.ownerToUTXOs.get(event.depositor) || [];
    ownerUTXOs.push(index);
    this.ownerToUTXOs.set(event.depositor, ownerUTXOs);

    // Update Merkle tree
    await this.updateMerkleTree();

    // Increment next leaf index
    this.nextLeafIndex++;

    this.lastProcessedBlock = event.blockNumber;

    console.log(`✅ Inserted commitment ${event.commitment} for owner ${event.depositor}`);
    console.log(`   Value: ${ethers.utils.formatEther(event.value)} ETH`);
    console.log(`   Leaf Index: ${utxo.leafIndex}`);

    return index;
  }

  /**
   * Get UTXO by commitment hash
   */
  getUTXOByCommitment(commitment: string): UTXO | undefined {
    const index = this.commitmentToIndex.get(commitment);
    return index !== undefined ? this.utxos[index] : undefined;
  }

  /**
   * Get all UTXOs owned by a specific address
   */
  getUTXOsByOwner(owner: string): UTXO[] {
    const indices = this.ownerToUTXOs.get(owner) || [];
    return indices.map(index => this.utxos[index]);
  }

  /**
   * Generate Merkle proof for a UTXO
   */
  generateMerkleProof(leafIndex: number): MerkleProof {
    if (leafIndex >= this.utxos.length) {
      throw new Error('Leaf index out of bounds');
    }

    const utxo = this.utxos[leafIndex];
    const leafHash = this.hashUTXO(utxo);
    
    const proof = this.merkleTree.getProof(leafHash);
    const siblings = proof.map(p => p.data.toString('hex'));
    const pathIndices = proof.map(p => p.position === 'right');

    return {
      siblings,
      pathIndices,
      root: this.merkleTree.getRoot().toString('hex'),
      leafIndex,
    };
  }

  /**
   * Verify Merkle proof
   */
  verifyMerkleProof(proof: MerkleProof, leaf: string): boolean {
    try {
      const leafBuffer = Buffer.from(leaf, 'hex');
      const rootBuffer = Buffer.from(proof.root, 'hex');
      
      return this.merkleTree.verify(proof.siblings.map(s => Buffer.from(s, 'hex')), leafBuffer, rootBuffer);
    } catch (error) {
      console.error('Error verifying Merkle proof:', error);
      return false;
    }
  }

  /**
   * Get current tree root
   */
  getRoot(): string {
    return this.merkleTree.getRoot().toString('hex');
  }

  /**
   * Get tree statistics
   */
  getTreeStats(): TreeStats {
    return {
      totalUTXOs: this.utxos.length,
      treeDepth: this.merkleTree.getDepth(),
      rootHash: this.getRoot(),
      nextLeafIndex: this.nextLeafIndex,
    };
  }

  /**
   * Get all UTXOs
   */
  getAllUTXOs(): UTXO[] {
    return [...this.utxos];
  }

  /**
   * Get last processed block
   */
  getLastProcessedBlock(): number {
    return this.lastProcessedBlock;
  }

  /**
   * Update Merkle tree after insertion
   */
  private async updateMerkleTree(): Promise<void> {
    const leaves = this.utxos.map(utxo => this.hashUTXO(utxo));
    this.merkleTree = new MerkleTree(leaves, ethers.utils.keccak256, {
      sortPairs: true,
    });
  }

  /**
   * Hash a UTXO for Merkle tree
   */
  private hashUTXO(utxo: UTXO): Buffer {
    const data = ethers.utils.solidityPack(
      ['bytes32', 'uint256', 'address', 'uint256'],
      [utxo.commitment, utxo.value.toString(), utxo.owner, utxo.leafIndex.toString()]
    );
    return Buffer.from(ethers.utils.keccak256(data).slice(2), 'hex');
  }

  /**
   * Generate secret for UTXO
   */
  private generateSecret(blockNumber: number, value: string): string {
    const data = ethers.utils.solidityPack(
      ['uint256', 'uint256', 'uint256', 'string'],
      [blockNumber, value, this.nextLeafIndex, 'privacy_pool_secret']
    );
    return ethers.utils.keccak256(data);
  }

  /**
   * Generate nullifier for UTXO
   */
  private generateNullifier(secret: string, leafIndex: number): string {
    const data = ethers.utils.solidityPack(
      ['bytes32', 'uint256', 'string'],
      [secret, leafIndex, 'privacy_pool_nullifier']
    );
    return ethers.utils.keccak256(data);
  }

  /**
   * Verify UTXO ownership
   */
  verifyOwnership(utxo: UTXO, owner: string): boolean {
    return utxo.owner.toLowerCase() === owner.toLowerCase();
  }

  /**
   * Get commitment hash for verification
   */
  getCommitmentHash(value: bigint, secret: string, nullifier: string, owner: string): string {
    const data = ethers.utils.solidityPack(
      ['uint256', 'bytes32', 'bytes32', 'address'],
      [value.toString(), secret, nullifier, owner]
    );
    return ethers.utils.keccak256(data);
  }
}

/**
 * Relayer service for handling deposit events
 */
export class RelayerService {
  private treeService: EnhancedMerkleTreeService;
  private provider: ethers.providers.Provider;
  private contract: ethers.Contract;

  constructor(
    provider: ethers.providers.Provider,
    contractAddress: string,
    contractABI: any
  ) {
    this.treeService = new EnhancedMerkleTreeService();
    this.provider = provider;
    this.contract = new ethers.Contract(contractAddress, contractABI, provider);
  }

  /**
   * Listen for deposit events and process them
   */
  async startListening(): Promise<void> {
    console.log('🔍 Starting to listen for deposit events...');

    this.contract.on('Deposited', async (
      depositor: string,
      commitment: string,
      label: string,
      value: string,
      precommitmentHash: string,
      event: any
    ) => {
      try {
        const depositEvent: DepositEvent = {
          depositor,
          commitment,
          value,
          blockNumber: event.blockNumber,
          transactionHash: event.transactionHash,
          logIndex: event.logIndex,
        };

        await this.processDepositEvent(depositEvent);
      } catch (error) {
        console.error('Error processing deposit event:', error);
      }
    });

    console.log('✅ Relayer service started');
  }

  /**
   * Process a deposit event
   */
  async processDepositEvent(event: DepositEvent): Promise<number> {
    return await this.treeService.processDepositEvent(event);
  }

  /**
   * Get tree service
   */
  getTreeService(): EnhancedMerkleTreeService {
    return this.treeService;
  }

  /**
   * Get Merkle proof for a commitment
   */
  async getMerkleProof(commitment: string): Promise<MerkleProof> {
    const utxo = this.treeService.getUTXOByCommitment(commitment);
    if (!utxo) {
      throw new Error('UTXO not found for commitment');
    }

    return this.treeService.generateMerkleProof(utxo.leafIndex);
  }

  /**
   * Get UTXOs for an owner
   */
  getOwnerUTXOs(owner: string): UTXO[] {
    return this.treeService.getUTXOsByOwner(owner);
  }

  /**
   * Get tree statistics
   */
  getTreeStats(): TreeStats {
    return this.treeService.getTreeStats();
  }
}
