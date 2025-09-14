//! API Handlers for Privacy Pool Operations
//!
//! HTTP request handlers that integrate with existing UTXO and Merkle tree modules
//! and VERIFY REAL BLOCKCHAIN DEPOSITS before creating UTXOs.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use reqwest;
use serde_json::{json, Value};
use std::str::FromStr;

use crate::api::types::*;
use crate::utxo::CanonicalUTXO;
use crate::relayer::RealDepositEvent;
use crate::privacy::PrivacyPool;

/// Simplified application state using in-memory storage
#[derive(Clone)]
pub struct AppState {
    /// In-memory UTXO storage (utxo_id -> UTXO)
    pub utxos: Arc<Mutex<HashMap<[u8; 32], CanonicalUTXO>>>,
    
    /// Owner to UTXOs mapping (owner_commitment -> list of utxo_ids)
    pub owner_utxos: Arc<Mutex<HashMap<[u8; 32], Vec<[u8; 32]>>>>,
    
    /// Asset balances (owner_commitment -> asset_id -> balance_info)
    pub balances: Arc<Mutex<HashMap<[u8; 32], HashMap<[u8; 20], (u128, u32)>>>>,
    
    /// Tree state
    pub tree_root: Arc<Mutex<[u8; 32]>>,
    pub tree_version: Arc<Mutex<u64>>,
    
    /// Privacy pool instance
    pub privacy_pool: Arc<Mutex<PrivacyPool>>,
    
    /// Configuration
    pub config: AppConfig,
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub tree_depth: u8,
    pub tree_salt: u64,
    pub version: String,
    pub sepolia_rpc_url: String,
    pub contract_address: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tree_depth: 32,
            tree_salt: rand::random::<u64>(),
            version: "0.1.0".to_string(),
            sepolia_rpc_url: "https://eth-sepolia.g.alchemy.com/v2/wdp1FpAvY5GBD-wstEpHlsIY37WcgKgI".to_string(),
            contract_address: "0x19B8743Df3E8997489b50F455a1cAe3536C0ee31".to_string(),
        }
    }
}

impl AppState {
    /// Create new application state
    pub fn new() -> Result<Self> {
        let config = AppConfig::default();
        let privacy_pool = PrivacyPool::new([0u8; 32]); // Default scope
        
        Ok(Self {
            utxos: Arc::new(Mutex::new(HashMap::new())),
            owner_utxos: Arc::new(Mutex::new(HashMap::new())),
            balances: Arc::new(Mutex::new(HashMap::new())),
            tree_root: Arc::new(Mutex::new([0u8; 32])),
            tree_version: Arc::new(Mutex::new(0)),
            privacy_pool: Arc::new(Mutex::new(privacy_pool)),
            config,
        })
    }
}

/// Create API router with all endpoints
pub fn create_router() -> Result<Router> {
    let state = AppState::new()?;
    
    Ok(Router::new()
        .route("/api/health", get(health_check))
        .route("/api/deposit", post(process_deposit))
        .route("/api/balance/:owner", get(get_balance))
        .route("/api/utxos/:owner", get(get_owner_utxos))
        .route("/api/utxo/:utxo_id", get(get_utxo_details))
        .route("/api/tree/stats", get(get_tree_stats))
        .route("/api/tree/root", get(get_tree_root))
        .with_state(state))
}

/// Health check endpoint
pub async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let tree_version = *state.tree_version.lock().unwrap();
    let utxo_count = state.utxos.lock().unwrap().len();
    
    Json(HealthResponse {
        status: "healthy".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        version: state.config.version.clone(),
        database_status: "in-memory".to_string(),
        tree_status: format!("version: {}, utxos: {}", tree_version, utxo_count),
    })
}

/// Process a single ETH deposit - VERIFIES REAL BLOCKCHAIN TRANSACTION
pub async fn process_deposit(
    State(state): State<AppState>,
    Json(request): Json<DepositRequest>,
) -> std::result::Result<Json<DepositResponse>, (StatusCode, Json<ErrorResponse>)> {
    println!("🔍 VERIFYING BLOCKCHAIN TRANSACTION: {}", request.tx_hash);

    // STEP 1: VERIFY THE TRANSACTION EXISTS ON BLOCKCHAIN
    let transaction_data = match verify_transaction_on_blockchain(
        &request.tx_hash,
        &state.config.sepolia_rpc_url,
        &state.config.contract_address
    ).await {
        Ok(data) => data,
        Err(e) => {
            println!("❌ BLOCKCHAIN VERIFICATION FAILED: {}", e);
            return Err(api_error("BLOCKCHAIN_VERIFICATION_FAILED", &e.to_string()));
        }
    };

    println!("✅ TRANSACTION VERIFIED ON BLOCKCHAIN");
    println!("   - Value: {} ETH", transaction_data.value_eth);
    println!("   - From: {}", transaction_data.from_address);
    println!("   - To Contract: {}", transaction_data.to_address);
    println!("   - Block: {}", transaction_data.block_number);

    // STEP 2: CREATE VERIFIED DEPOSIT EVENT
    let depositor_address = web3::types::Address::from_str(&transaction_data.from_address)
        .map_err(|e| anyhow!("Invalid depositor address: {}", e))?;

    let commitment_hash = web3::types::H256::from_slice(
        &hex::decode(&request.commitment.strip_prefix("0x").unwrap_or(&request.commitment))
            .map_err(|e| anyhow!("Invalid commitment format: {}", e))?
    );

    let tx_hash_bytes = hex::decode(&request.tx_hash.strip_prefix("0x").unwrap_or(&request.tx_hash))
        .map_err(|e| anyhow!("Invalid transaction hash format: {}", e))?;
    let transaction_hash = web3::types::H256::from_slice(&tx_hash_bytes);

    let deposit_event = RealDepositEvent {
        depositor: depositor_address,
        commitment: commitment_hash,
        value: web3::types::U256::from_dec_str(&transaction_data.value_wei).unwrap_or(web3::types::U256::zero()),
        block_number: transaction_data.block_number,
        transaction_hash,
        label: request.label.map(|l| web3::types::U256::from_dec_str(&l).unwrap_or(web3::types::U256::zero())).unwrap_or(web3::types::U256::zero()),
        precommitment_hash: request.precommitment_hash.map(|ph| {
            let ph_bytes = hex::decode(&ph.strip_prefix("0x").unwrap_or(&ph)).unwrap_or_default();
            web3::types::H256::from_slice(&ph_bytes)
        }).unwrap_or(web3::types::H256::zero()),
        log_index: 0,
    };

    // STEP 3: Generate UTXO from VERIFIED deposit
    let utxo = match create_utxo_from_verified_deposit(&deposit_event, &state) {
        Ok(utxo) => utxo,
        Err(e) => return Err(api_error("UTXO_CREATION_FAILED", &e.to_string())),
    };

    // Calculate tree position
    let tree_position = crate::canonical_spec::generate_tree_index(
        utxo.utxo_id,
        state.config.tree_salt
    );

    // Get leaf hash
    let leaf_hash = match utxo.leaf_hash() {
        Ok(hash) => hash,
        Err(e) => return Err(api_error("LEAF_HASH_FAILED", &e.to_string())),
    };

    // STEP 4: Update in-memory storage with VERIFIED data
    {
        let mut utxos = state.utxos.lock().unwrap();
        utxos.insert(utxo.utxo_id, utxo.clone());

        let mut owner_utxos = state.owner_utxos.lock().unwrap();
        owner_utxos.entry(utxo.owner_commitment)
            .or_insert_with(Vec::new)
            .push(utxo.utxo_id);

        let mut balances = state.balances.lock().unwrap();
        let owner_balances = balances.entry(utxo.owner_commitment)
            .or_insert_with(HashMap::new);
        let (current_balance, current_count) = owner_balances.entry(utxo.asset_id)
            .or_insert((0, 0));
        *current_balance += utxo.amount;
        *current_count += 1;

        // Update tree version
        let mut tree_version = state.tree_version.lock().unwrap();
        *tree_version += 1;

        // Simple tree root update (in production this would be proper SMT)
        let mut tree_root = state.tree_root.lock().unwrap();
        *tree_root = crate::canonical_spec::generate_node_hash(*tree_root, leaf_hash);
    }

    println!("🎉 UTXO CREATED FROM VERIFIED BLOCKCHAIN DEPOSIT!");

    let response = DepositResponse {
        success: true,
        utxo_id: utils::hash_to_hex(utxo.utxo_id),
        new_root: utils::hash_to_hex(*state.tree_root.lock().unwrap()),
        tree_position,
        leaf_hash: utils::hash_to_hex(leaf_hash),
        root_version: *state.tree_version.lock().unwrap(),
        processed_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    Ok(Json(response))
}

/// Get balance for an owner  
pub async fn get_balance(
    State(state): State<AppState>,
    Path(owner_hex): Path<String>,
) -> Result<Json<BalanceInfo>, (StatusCode, Json<ErrorResponse>)> {
    let owner_commitment = match utils::hex_to_hash(&owner_hex) {
        Ok(hash) => hash,
        Err(_) => return Err(api_error("INVALID_OWNER", "Invalid owner commitment format")),
    };
    
    let asset_id = [0u8; 20]; // ETH
    
    let balances = state.balances.lock().unwrap();
    let (balance, utxo_count) = balances
        .get(&owner_commitment)
        .and_then(|owner_balances| owner_balances.get(&asset_id))
        .copied()
        .unwrap_or((0, 0));
    
    Ok(Json(BalanceInfo {
        balance: balance.to_string(),
        utxo_count,
        last_updated_block: 0,
        asset_id: utils::asset_id_to_hex(asset_id),
    }))
}

/// Get UTXOs for an owner
pub async fn get_owner_utxos(
    State(state): State<AppState>,
    Path(owner_hex): Path<String>,
    Query(query): Query<UTXOQuery>,
) -> Result<Json<UTXOListResponse>, (StatusCode, Json<ErrorResponse>)> {
    let owner_commitment = match utils::hex_to_hash(&owner_hex) {
        Ok(hash) => hash,
        Err(_) => return Err(api_error("INVALID_OWNER", "Invalid owner commitment format")),
    };
    
    let owner_utxos = state.owner_utxos.lock().unwrap();
    let utxos_map = state.utxos.lock().unwrap();
    
    let utxo_ids = owner_utxos.get(&owner_commitment).cloned().unwrap_or_default();
    let limit = query.limit.unwrap_or(100);
    
    let mut utxo_infos = Vec::new();
    for (i, utxo_id) in utxo_ids.iter().enumerate() {
        if i >= limit {
            break;
        }
        
        if let Some(utxo) = utxos_map.get(utxo_id) {
            let tree_position = crate::canonical_spec::generate_tree_index(
                utxo.utxo_id, 
                state.config.tree_salt
            );
            
            utxo_infos.push(UTXOInfo {
                utxo_id: utils::hash_to_hex(utxo.utxo_id),
                amount: utxo.amount.to_string(),
                asset_id: utils::asset_id_to_hex(utxo.asset_id),
                created_block: utxo.created_block,
                tree_position,
                lock_expiry: if utxo.lock_expiry > 0 { Some(utxo.lock_expiry) } else { None },
                lock_flags: utxo.lock_flags,
                is_spent: false,
            });
        }
    }
    
    Ok(Json(UTXOListResponse {
        total_count: utxo_infos.len(),
        utxos: utxo_infos,
        next_cursor: None,
    }))
}

/// Get specific UTXO details
pub async fn get_utxo_details(
    State(state): State<AppState>,
    Path(utxo_id_hex): Path<String>,
) -> Result<Json<UTXOInfo>, (StatusCode, Json<ErrorResponse>)> {
    let utxo_id = match utils::hex_to_hash(&utxo_id_hex) {
        Ok(hash) => hash,
        Err(_) => return Err(api_error("INVALID_UTXO_ID", "Invalid UTXO ID format")),
    };
    
    let utxos = state.utxos.lock().unwrap();
    let utxo = match utxos.get(&utxo_id) {
        Some(utxo) => utxo,
        None => return Err(api_error("UTXO_NOT_FOUND", "UTXO not found")),
    };
    
    let tree_position = crate::canonical_spec::generate_tree_index(
        utxo.utxo_id, 
        state.config.tree_salt
    );
    
    Ok(Json(UTXOInfo {
        utxo_id: utils::hash_to_hex(utxo.utxo_id),
        amount: utxo.amount.to_string(),
        asset_id: utils::asset_id_to_hex(utxo.asset_id),
        created_block: utxo.created_block,
        tree_position,
        lock_expiry: if utxo.lock_expiry > 0 { Some(utxo.lock_expiry) } else { None },
        lock_flags: utxo.lock_flags,
        is_spent: false,
    }))
}

/// Get tree statistics
pub async fn get_tree_stats(State(state): State<AppState>) -> Json<TreeStatsResponse> {
    let utxo_count = state.utxos.lock().unwrap().len() as u64;
    let tree_version = *state.tree_version.lock().unwrap();
    let tree_root = *state.tree_root.lock().unwrap();
    
    Json(TreeStatsResponse {
        current_root: utils::hash_to_hex(tree_root),
        root_version: tree_version,
        depth: state.config.tree_depth,
        total_utxos: utxo_count,
        total_nodes: utxo_count,
        tree_salt: state.config.tree_salt,
    })
}

/// Get current tree root
pub async fn get_tree_root(State(state): State<AppState>) -> Json<serde_json::Value> {
    let tree_root = *state.tree_root.lock().unwrap();
    let tree_version = *state.tree_version.lock().unwrap();
    
    Json(serde_json::json!({
        "root": utils::hash_to_hex(tree_root),
        "version": tree_version,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

// Helper functions

#[derive(Debug, Clone)]
struct BlockchainTransactionData {
    from_address: String,
    to_address: String,
    value_wei: String,
    value_eth: String,
    block_number: u64,
    gas_used: String,
    status: String,
}

/// VERIFY TRANSACTION ON BLOCKCHAIN - This is the critical fix!
async fn verify_transaction_on_blockchain(
    tx_hash: &str,
    rpc_url: &str,
    expected_contract_address: &str,
) -> Result<BlockchainTransactionData> {
    let client = reqwest::Client::new();

    // Call eth_getTransactionByHash
    let request_body = json!({
        "jsonrpc": "2.0",
        "method": "eth_getTransactionByHash",
        "params": [tx_hash],
        "id": 1
    });

    let response = client
        .post(rpc_url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to call RPC: {}", e))?;

    let response_json: Value = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse RPC response: {}", e))?;

    let tx_data = response_json["result"]
        .as_object()
        .ok_or_else(|| anyhow!("Transaction not found"))?;

    // Extract transaction details
    let from_address = tx_data["from"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing from address"))?
        .to_string();

    let to_address = tx_data["to"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing to address"))?
        .to_string();

    let value_hex = tx_data["value"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing value"))?;

    let block_number_hex = tx_data["blockNumber"]
        .as_str()
        .ok_or_else(|| anyhow!("Transaction not mined yet"))?;

    // Verify the transaction is to our contract
    if to_address.to_lowercase() != expected_contract_address.to_lowercase() {
        return Err(anyhow!(
            "Transaction is not to our contract. Expected: {}, Got: {}",
            expected_contract_address,
            to_address
        ));
    }

    // Convert hex values
    let value_wei = u128::from_str_radix(
        value_hex.strip_prefix("0x").unwrap_or(value_hex),
        16
    ).map_err(|e| anyhow!("Invalid value format: {}", e))?;

    let block_number = u64::from_str_radix(
        block_number_hex.strip_prefix("0x").unwrap_or(block_number_hex),
        16
    ).map_err(|e| anyhow!("Invalid block number format: {}", e))?;

    // Convert wei to ETH for display
    let value_eth = format!("{:.6}", value_wei as f64 / 1_000_000_000_000_000_000.0);

    // Get transaction receipt to verify it succeeded
    let receipt_request = json!({
        "jsonrpc": "2.0",
        "method": "eth_getTransactionReceipt",
        "params": [tx_hash],
        "id": 2
    });

    let receipt_response = client
        .post(rpc_url)
        .json(&receipt_request)
        .send()
        .await
        .map_err(|e| anyhow!("Failed to get transaction receipt: {}", e))?;

    let receipt_json: Value = receipt_response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse receipt response: {}", e))?;

    let receipt = receipt_json["result"]
        .as_object()
        .ok_or_else(|| anyhow!("Transaction receipt not found"))?;

    let status = receipt["status"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing transaction status"))?;

    if status != "0x1" {
        return Err(anyhow!("Transaction failed (status: {})", status));
    }

    let gas_used = receipt["gasUsed"]
        .as_str()
        .unwrap_or("0x0")
        .to_string();

    Ok(BlockchainTransactionData {
        from_address,
        to_address,
        value_wei: value_wei.to_string(),
        value_eth,
        block_number,
        gas_used,
        status: status.to_string(),
    })
}

/// Create UTXO from VERIFIED deposit event
fn create_utxo_from_verified_deposit(deposit: &RealDepositEvent, _state: &AppState) -> Result<CanonicalUTXO> {
    let owner_commitment = derive_owner_commitment(deposit)?;

    let utxo = CanonicalUTXO::new_eth(
        deposit.transaction_hash.0,
        0,
        deposit.block_number,
        rand::random::<u64>(),
        deposit.value.as_u128(),
        owner_commitment,
    );

    Ok(utxo)
}

/// Derive privacy-preserving owner commitment
fn derive_owner_commitment(deposit: &RealDepositEvent) -> Result<[u8; 32]> {
    use sha3::{Keccak256, Digest};

    let mut hasher = Keccak256::new();
    hasher.update(b"OWNER_COMMITMENT");
    hasher.update(deposit.depositor.as_bytes());
    hasher.update(deposit.commitment.as_bytes());
    hasher.update(&deposit.block_number.to_be_bytes());

    Ok(hasher.finalize().into())
}

/// Create API error response
fn api_error(error_code: &str, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: error_code.to_string(),
            message: message.to_string(),
            details: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }),
    )
}

/// Utility functions for hex conversions
mod utils {
    pub fn hash_to_hex(hash: [u8; 32]) -> String {
        format!("0x{}", hex::encode(hash))
    }
    
    pub fn hex_to_hash(hex_str: &str) -> Result<[u8; 32], Box<dyn std::error::Error>> {
        let clean_hex = hex_str.strip_prefix("0x").unwrap_or(hex_str);
        let bytes = hex::decode(clean_hex)?;
        if bytes.len() != 32 {
            return Err(format!("Expected 32 bytes, got {}", bytes.len()).into());
        }
        let mut array = [0u8; 32];
        array.copy_from_slice(&bytes);
        Ok(array)
    }
    
    pub fn asset_id_to_hex(asset_id: [u8; 20]) -> String {
        format!("0x{}", hex::encode(asset_id))
    }
}