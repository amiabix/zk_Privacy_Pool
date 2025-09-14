//! Integration Test for Encrypted Notes System
//! 
//! This module tests the complete encrypted notes flow from generation
//! to decryption and Merkle proof verification.

// Note: tempfile is already in dev-dependencies

/// Test the complete encrypted notes flow
#[tokio::test]
async fn test_complete_encrypted_notes_flow() -> Result<()> {
    // Setup test database
    let temp_dir = tempfile::TempDir::new()?;
    let db_config = DBConfig {
        db_path: temp_dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };
    
    let db = DatabaseManager::open(db_config.clone())?;
    
    // Create test key pairs
    let (sender_secret, sender_public) = Ecies::generate_keypair()?;
    let (recipient_secret, recipient_public) = Ecies::generate_keypair()?;
    
    // Create master key for recipient wallet
    let master_key = ExtendedPrivateKey::from_seed(b"test_master_seed")?;
    
    // Create relayer
    let mut relayer = EncryptedNotesRelayer::new(db.clone())?;
    
    // Create wallet note manager
    let mut wallet_manager = WalletNoteManager::new(db, master_key.clone(), "http://localhost:3000".to_string());
    wallet_manager.initialize()?;
    
    // Step 1: Create note
    let note = Note::new(
        1, // version
        1, // chain_id
        "0x1234567890123456789012345678901234567890".to_string(),
        1000000000000000000u64, // 1 ETH
        [0x42u8; 32], // owner_pk
    );
    
    assert!(note.verify());
    println!(" Note created: {}", note.note_id);
    
    // Step 2: Encrypt note
    let mut recipient_pubkey = [0u8; 33];
    recipient_pubkey.copy_from_slice(&recipient_public.to_encoded_point(false).as_bytes());
    
    let encrypted_note = Ecies::encrypt_note(&note, &recipient_pubkey)?;
    println!(" Note encrypted");
    
    // Step 3: Upload to relayer
    let note_id = relayer.upload_note(encrypted_note.clone())?;
    println!(" Note uploaded to relayer: {}", note_id);
    
    // Step 4: Simulate deposit transaction
    let tx_hash = "0xabcdef1234567890".to_string();
    let output_index = 0;
    
    relayer.attach_tx(&note_id, tx_hash.clone(), output_index)?;
    println!(" Transaction attached to note");
    
    // Step 5: Verify Merkle root was updated
    let merkle_root = relayer.get_merkle_root();
    assert_ne!(merkle_root, [0u8; 32]);
    println!(" Merkle root updated: {}", hex::encode(merkle_root));
    
    // Step 6: Test note discovery by wallet
    let mut scanner = NoteScanner::new(
        DatabaseManager::open(db_config.clone())?,
        master_key
    );
    
    // Simulate finding the note (in real scenario, this would come from relayer)
    let encrypted_entry = crate::relayer::EncryptedNoteEntry {
        note_id: note_id.clone(),
        ephemeral_pubkey: encrypted_note.ephemeral_pubkey,
        nonce: encrypted_note.nonce,
        ciphertext: encrypted_note.ciphertext,
        commitment: encrypted_note.commitment,
        uploaded_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        tx_hash: Some(tx_hash),
        output_index: Some(output_index),
        leaf_index: Some(0),
    };
    
    // Try to decrypt the note
    let mut seckey_bytes = [0u8; 32];
    seckey_bytes.copy_from_slice(&recipient_secret.to_be_bytes());
    
    if let Some(decrypted_note) = scanner.try_decrypt_note(&encrypted_entry).await? {
        assert_eq!(decrypted_note.note_id, note.note_id);
        assert_eq!(decrypted_note.value, note.value);
        assert_eq!(decrypted_note.commitment, note.commitment);
        println!(" Note successfully decrypted by wallet");
    } else {
        panic!("Failed to decrypt note");
    }
    
    // Step 7: Test Merkle proof generation
    if let Some(proof) = relayer.get_merkle_proof(&note.commitment)? {
        assert_eq!(proof.root, merkle_root);
        println!(" Merkle proof generated successfully");
    } else {
        panic!("Failed to generate Merkle proof");
    }
    
    println!("\n Complete encrypted notes flow test passed!");
    
    Ok(())
}

/// Test ECIES encryption/decryption roundtrip
#[test]
fn test_ecies_roundtrip() -> Result<()> {
    // Generate test key pair
    let (secret_key, public_key) = Ecies::generate_keypair()?;
    
    // Create test note
    let note = Note::new(
        1,
        1,
        "0x1234567890123456789012345678901234567890".to_string(),
        1000000000000000000u64,
        [0x42u8; 32],
    );
    
    // Serialize public key
    let mut pubkey_bytes = [0u8; 33];
    pubkey_bytes.copy_from_slice(&public_key.to_encoded_point(false).as_bytes());
    
    // Serialize secret key
    let mut seckey_bytes = [0u8; 32];
    seckey_bytes.copy_from_slice(&secret_key.to_be_bytes());
    
    // Encrypt note
    let encrypted = Ecies::encrypt_note(&note, &pubkey_bytes)?;
    
    // Decrypt note
    let decrypted = Ecies::decrypt_note(&encrypted, &seckey_bytes)?;
    
    // Verify roundtrip
    assert_eq!(note, decrypted);
    println!(" ECIES roundtrip test passed");
    
    Ok(())
}

/// Test note serialization
#[test]
fn test_note_serialization() -> Result<()> {
    let note = Note::new(
        1,
        1,
        "0x1234567890123456789012345678901234567890".to_string(),
        1000000000000000000u64,
        [0x42u8; 32],
    );
    
    // Test JSON serialization
    let json = note.to_json()?;
    let deserialized = Note::from_json(&json)?;
    
    assert_eq!(note, deserialized);
    println!(" Note serialization test passed");
    
    Ok(())
}

/// Test commitment computation
#[test]
fn test_commitment_computation() -> Result<()> {
    let value = 1000000000000000000u64;
    let owner_pk = [0x42u8; 32];
    let secret = [0x24u8; 32];
    let blinding = [0x12u8; 32];
    
    let commitment = Note::compute_commitment(value, &owner_pk, &secret, &blinding);
    
    // Verify commitment is deterministic
    let commitment2 = Note::compute_commitment(value, &owner_pk, &secret, &blinding);
    assert_eq!(commitment, commitment2);
    
    // Verify commitment changes with different inputs
    let commitment3 = Note::compute_commitment(value + 1, &owner_pk, &secret, &blinding);
    assert_ne!(commitment, commitment3);
    
    println!(" Commitment computation test passed");
    
    Ok(())
}

/// Test wallet note scanning
#[tokio::test]
async fn test_wallet_note_scanning() -> Result<()> {
    let temp_dir = tempfile::TempDir::new()?;
    let db_config = DBConfig {
        db_path: temp_dir.path().to_str().unwrap().to_string(),
        ..Default::default()
    };
    
    let db = DatabaseManager::open(db_config.clone())?;
    let master_key = ExtendedPrivateKey::from_seed(b"test_master_seed")?;
    
    let mut scanner = NoteScanner::new(db, master_key);
    
    // Test empty scanner
    assert!(scanner.get_local_notes().is_empty());
    
    // Test loading notes from database
    scanner.load_notes_from_db()?;
    
    println!(" Wallet note scanning test passed");
    
    Ok(())
}
