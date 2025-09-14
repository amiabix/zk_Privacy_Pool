//! Note Generation Script
//! 
//! This script creates encrypted notes, uploads them to the relayer,
//! and submits deposit transactions to the blockchain.

import { ethers } from "ethers";
import { Note, EncryptedNote } from "../utxo/note";
import { Ecies } from "../crypto/ecies";

// Configuration
const RELAYER_URL = "http://localhost:3000";
const RPC_URL = "http://localhost:8545";
const PRIVACY_POOL_ADDRESS = "0x2279B7A0a67DB372996a5FaB50D91eAA73d2eBe6";

// Privacy Pool ABI (simplified)
const PRIVACY_POOL_ABI = [
    "function deposit(bytes32 commitment) external payable",
    "event Deposited(address indexed depositor, bytes32 indexed commitment, uint256 label, uint256 value, bytes32 indexed precommitmentHash, uint256 blockNumber, uint256 transactionHash, uint256 logIndex, bytes32 merkleRoot)"
];

interface GenNoteOptions {
    value: string; // ETH amount in wei
    recipientPubkey: string; // Recipient's public key (hex)
    chainId: number;
    poolAddress: string;
}

async function generateNote(options: GenNoteOptions): Promise<Note> {
    // Parse recipient public key
    const recipientPubkeyBytes = ethers.utils.arrayify(options.recipientPubkey);
    if (recipientPubkeyBytes.length !== 32) {
        throw new Error("Recipient public key must be 32 bytes");
    }
    
    // Create note
    const note = new Note(
        1, // version
        options.chainId,
        options.poolAddress,
        BigInt(options.value),
        recipientPubkeyBytes as [u8; 32]
    );
    
    return note;
}

async function encryptNote(note: Note, recipientPubkey: string): Promise<EncryptedNote> {
    const recipientPubkeyBytes = ethers.utils.arrayify(recipientPubkey);
    if (recipientPubkeyBytes.length !== 33) {
        throw new Error("Recipient public key must be 33 bytes (compressed)");
    }
    
    const recipientPubkeyArray = new Uint8Array(33);
    recipientPubkeyArray.set(recipientPubkeyBytes);
    
    // Encrypt note
    const encryptedNote = Ecies.encryptNote(note, recipientPubkeyArray as [u8; 33]);
    
    return encryptedNote;
}

async function uploadNoteToRelayer(encryptedNote: EncryptedNote): Promise<string> {
    const response = await fetch(`${RELAYER_URL}/upload_note`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            encrypted_note: encryptedNote
        })
    });
    
    if (!response.ok) {
        throw new Error(`Failed to upload note: ${response.statusText}`);
    }
    
    const result = await response.json();
    return result.note_id;
}

async function submitDepositTransaction(commitment: string, value: string, signer: ethers.Signer): Promise<string> {
    const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
    const contract = new ethers.Contract(PRIVACY_POOL_ADDRESS, PRIVACY_POOL_ABI, signer);
    
    // Submit deposit transaction
    const tx = await contract.deposit(commitment, { value: value });
    
    // Wait for confirmation
    const receipt = await tx.wait();
    
    return receipt.transactionHash;
}

async function attachTransactionToNote(noteId: string, txHash: string, outputIndex: number): Promise<void> {
    const response = await fetch(`${RELAYER_URL}/attach_tx`, {
        method: "POST",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({
            note_id: noteId,
            tx_hash: txHash,
            output_index: outputIndex
        })
    });
    
    if (!response.ok) {
        throw new Error(`Failed to attach transaction: ${response.statusText}`);
    }
}

async function main() {
    try {
        // Parse command line arguments
        const args = process.argv.slice(2);
        if (args.length < 4) {
            console.log("Usage: npm run gen-note <value_wei> <recipient_pubkey> <private_key> <chain_id>");
            console.log("Example: npm run gen-note 1000000000000000000 0x02... 0x1234... 1");
            process.exit(1);
        }
        
        const [valueWei, recipientPubkey, privateKey, chainId] = args;
        
        // Create signer
        const provider = new ethers.providers.JsonRpcProvider(RPC_URL);
        const signer = new ethers.Wallet(privateKey, provider);
        
        console.log("Generating note...");
        
        // Generate note
        const note = await generateNote({
            value: valueWei,
            recipientPubkey: recipientPubkey,
            chainId: parseInt(chainId),
            poolAddress: PRIVACY_POOL_ADDRESS
        });
        
        console.log("Note generated:", note.note_id);
        console.log("Commitment:", ethers.utils.hexlify(note.commitment));
        
        // Encrypt note
        console.log("Encrypting note...");
        const encryptedNote = await encryptNote(note, recipientPubkey);
        
        // Upload to relayer
        console.log("Uploading to relayer...");
        const noteId = await uploadNoteToRelayer(encryptedNote);
        console.log("Note uploaded with ID:", noteId);
        
        // Submit deposit transaction
        console.log("Submitting deposit transaction...");
        const txHash = await submitDepositTransaction(
            ethers.utils.hexlify(note.commitment),
            valueWei,
            signer
        );
        console.log("Deposit transaction:", txHash);
        
        // Attach transaction to note
        console.log("Attaching transaction to note...");
        await attachTransactionToNote(noteId, txHash, 0);
        console.log("Transaction attached successfully");
        
        console.log("\n Note generation complete!");
        console.log("Note ID:", noteId);
        console.log("Transaction Hash:", txHash);
        console.log("Commitment:", ethers.utils.hexlify(note.commitment));
        
    } catch (error) {
        console.error(" Error:", error);
        process.exit(1);
    }
}

// Run if called directly
if (require.main === module) {
    main();
}

export {
    generateNote,
    encryptNote,
    uploadNoteToRelayer,
    submitDepositTransaction,
    attachTransactionToNote
};
