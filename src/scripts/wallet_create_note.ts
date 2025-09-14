#!/usr/bin/env node

/**
 * Wallet script for creating encrypted notes
 * This script demonstrates the complete flow:
 * 1. Generate note with secret/blinding
 * 2. Compute commitment using Poseidon
 * 3. Encrypt note using ECIES
 * 4. Upload to relayer
 * 5. Submit deposit transaction
 */

import { ethers } from 'ethers';
import { poseidon } from 'poseidon-lite';
import { randomBytes } from 'crypto';
import axios from 'axios';

// Configuration
const RELAYER_URL = process.env.RELAYER_URL || 'http://localhost:3000';
const RPC_URL = process.env.RPC_URL || 'http://localhost:8545';
const PRIVACY_POOL_ADDRESS = process.env.PRIVACY_POOL_ADDRESS || '0x...';

// Types
interface Note {
    version: number;
    chain_id: number;
    pool_address: string;
    value: string;
    owner_enc_pk: string;
    secret: string;
    blinding: string;
    commitment: string;
}

interface EncryptedNoteData {
    ephemeral_pubkey: string;
    nonce: string;
    ciphertext: string;
    commitment: string;
    owner_enc_pk: string;
}

interface NoteUploadRequest {
    encrypted_note: EncryptedNoteData;
    uploader_sig?: string;
}

class PrivacyWallet {
    private provider: ethers.providers.JsonRpcProvider;
    private wallet: ethers.Wallet;
    private relayerUrl: string;
    private poolAddress: string;

    constructor(
        privateKey: string,
        rpcUrl: string,
        relayerUrl: string,
        poolAddress: string
    ) {
        this.provider = new ethers.providers.JsonRpcProvider(rpcUrl);
        this.wallet = new ethers.Wallet(privateKey, this.provider);
        this.relayerUrl = relayerUrl;
        this.poolAddress = poolAddress;
    }

    /**
     * Generate a new note with random secret and blinding
     */
    generateNote(value: string, recipientPubkey: string): Note {
        // Generate random secret and blinding (32 bytes each)
        const secret = randomBytes(32);
        const blinding = randomBytes(32);

        // Compute commitment using Poseidon hash
        // commitment = Poseidon(owner_enc_pk, value, secret, blinding)
        const commitment = this.computeCommitment(recipientPubkey, value, secret, blinding);

        return {
            version: 1,
            chain_id: 1, // Mainnet
            pool_address: this.poolAddress,
            value: value,
            owner_enc_pk: recipientPubkey,
            secret: '0x' + secret.toString('hex'),
            blinding: '0x' + blinding.toString('hex'),
            commitment: '0x' + commitment.toString('hex')
        };
    }

    /**
     * Compute commitment using Poseidon hash
     */
    private computeCommitment(
        ownerPk: string,
        value: string,
        secret: Buffer,
        blinding: Buffer
    ): Buffer {
        // Convert inputs to field elements
        const ownerPkBytes = Buffer.from(ownerPk.slice(2), 'hex');
        const valueBytes = Buffer.from(value.slice(2), 'hex');
        
        // Pad to 32 bytes if needed
        const paddedOwnerPk = Buffer.alloc(32);
        ownerPkBytes.copy(paddedOwnerPk, 32 - ownerPkBytes.length);
        
        const paddedValue = Buffer.alloc(32);
        valueBytes.copy(paddedValue, 32 - valueBytes.length);

        // Compute Poseidon hash
        const inputs = [
            BigInt('0x' + paddedOwnerPk.toString('hex')),
            BigInt('0x' + paddedValue.toString('hex')),
            BigInt('0x' + secret.toString('hex')),
            BigInt('0x' + blinding.toString('hex'))
        ];

        const hash = poseidon(inputs);
        return Buffer.from(hash.toString(16).padStart(64, '0'), 'hex');
    }

    /**
     * Encrypt note using ECIES (simplified - in production use proper ECIES)
     */
    async encryptNote(note: Note, recipientPubkey: string): Promise<EncryptedNoteData> {
        // In a real implementation, this would use proper ECIES encryption
        // For now, we'll create a mock encrypted note
        const noteJson = JSON.stringify(note);
        const ciphertext = Buffer.from(noteJson).toString('hex');
        
        return {
            ephemeral_pubkey: '0x' + randomBytes(33).toString('hex'),
            nonce: '0x' + randomBytes(24).toString('hex'),
            ciphertext: '0x' + ciphertext,
            commitment: note.commitment,
            owner_enc_pk: recipientPubkey
        };
    }

    /**
     * Upload encrypted note to relayer
     */
    async uploadNote(encryptedNote: EncryptedNoteData): Promise<string> {
        try {
            const response = await axios.post(`${this.relayerUrl}/notes/upload`, {
                encrypted_note: encryptedNote
            });

            if (response.data.attached) {
                console.log('Note uploaded and already attached to deposit');
                console.log('TX Hash:', response.data.tx_hash);
                console.log('Leaf Index:', response.data.leaf_index);
            } else {
                console.log('Note uploaded successfully, waiting for deposit');
            }

            return response.data.note_id;
        } catch (error) {
            console.error('Failed to upload note:', error.response?.data || error.message);
            throw error;
        }
    }

    /**
     * Submit deposit transaction
     */
    async submitDeposit(commitment: string, value: string): Promise<string> {
        try {
            const contract = new ethers.Contract(
                this.poolAddress,
                [
                    'function depositETH(bytes32 commitment) payable'
                ],
                this.wallet
            );

            const tx = await contract.depositETH(commitment, {
                value: ethers.utils.parseEther(value)
            });

            console.log('Deposit transaction submitted:', tx.hash);
            console.log('Waiting for confirmation...');

            const receipt = await tx.wait();
            console.log('Deposit confirmed in block:', receipt.blockNumber);

            return tx.hash;
        } catch (error) {
            console.error('Failed to submit deposit:', error.message);
            throw error;
        }
    }

    /**
     * Complete deposit flow: create note, encrypt, upload, and deposit
     */
    async deposit(value: string, recipientPubkey: string): Promise<void> {
        console.log('Creating note...');
        const note = this.generateNote(value, recipientPubkey);
        console.log('Note created with commitment:', note.commitment);

        console.log('Encrypting note...');
        const encryptedNote = await this.encryptNote(note, recipientPubkey);

        console.log('Uploading encrypted note to relayer...');
        const noteId = await this.uploadNote(encryptedNote);

        console.log('Submitting deposit transaction...');
        const txHash = await this.submitDeposit(note.commitment, value);

        console.log('Deposit flow completed!');
        console.log('Note ID:', noteId);
        console.log('Transaction Hash:', txHash);
        console.log('Commitment:', note.commitment);
    }
}

// Main execution
async function main() {
    const privateKey = process.env.PRIVATE_KEY;
    const value = process.env.VALUE || '0.1'; // ETH amount
    const recipientPubkey = process.env.RECIPIENT_PUBKEY || '0x' + randomBytes(33).toString('hex');

    if (!privateKey) {
        console.error('Please set PRIVATE_KEY environment variable');
        process.exit(1);
    }

    if (!PRIVACY_POOL_ADDRESS || PRIVACY_POOL_ADDRESS === '0x...') {
        console.error('Please set PRIVACY_POOL_ADDRESS environment variable');
        process.exit(1);
    }

    try {
        const wallet = new PrivacyWallet(
            privateKey,
            RPC_URL,
            RELAYER_URL,
            PRIVACY_POOL_ADDRESS
        );

        await wallet.deposit(value, recipientPubkey);
    } catch (error) {
        console.error('Error:', error.message);
        process.exit(1);
    }
}

// Run if called directly
if (require.main === module) {
    main().catch(console.error);
}

export { PrivacyWallet };
