//! Working Privacy Pool API Server
//! 
//! Connects frontend to actual Rust UTXO functionality using existing working modules

use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::{CorsLayer, Any};
use std::sync::Arc;
use tokio::sync::Mutex;

use privacy_pool_zkvm::{
    utxo::{UTXOIndex, ETHToUTXOConverter},
    merkle::EnhancedMerkleTree,
    relayer::{DepositEvent, BlockchainConfig},
    privacy::PrivacyPool,
    utils::*,
};

/// Application state with working modules
#[derive(Clone)]
pub struct AppState {
    pub utxo_index: Arc<UTXOIndex>,
    pub merkle_tree: Arc<EnhancedMerkleTree>,
    pub utxo_converter: Arc<Mutex<ETHToUTXOConverter>>,
    pub privacy_pool: Arc<PrivacyPool>,
    pub tree_root: Arc<tokio::sync::Mutex<[u8; 32]>>,
    pub tree_version: Arc<tokio::sync::Mutex<u64>>,
}

impl AppState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let utxo_index = Arc::new(UTXOIndex::new());
        let merkle_tree = Arc::new(EnhancedMerkleTree::new());
        let blockchain_config = BlockchainConfig::default();
        let privacy_pool_contract = privacy_pool_zkvm::utxo::converter::PrivacyPoolContract::new(blockchain_config)?;
        let utxo_converter = Arc::new(Mutex::new(ETHToUTXOConverter::new(privacy_pool_contract)));
        let privacy_pool = Arc::new(PrivacyPool::new([0u8; 32])); // Default scope
        
        Ok(Self {
            utxo_index,
            merkle_tree,
            utxo_converter,
            privacy_pool,
            tree_root: Arc::new(tokio::sync::Mutex::new([0u8; 32])),
            tree_version: Arc::new(tokio::sync::Mutex::new(0)),
        })
    }
}

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let tree_version = *state.tree_version.lock().await;
    let utxo_count = state.utxo_index.count();
    
    Json(json!({
        "status": "healthy",
        "service": "privacy-pool-api",
        "version": "1.0.0",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "tree_version": tree_version,
        "utxo_count": utxo_count
    }))
}

/// Get UTXOs for a specific owner
async fn get_utxos(
    Path(owner): Path<String>,
    Query(params): Query<UTXOQuery>,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = params.limit.unwrap_or(100);
    
    // Convert owner string to bytes (simplified for demo)
    let owner_bytes = owner.as_bytes();
    let mut owner_hash = [0u8; 32];
    if owner_bytes.len() >= 32 {
        owner_hash.copy_from_slice(&owner_bytes[..32]);
    } else {
        // Pad with zeros for shorter addresses
        owner_hash[..owner_bytes.len()].copy_from_slice(owner_bytes);
    }
    
    // Get UTXOs from index
    let utxos = state.utxo_index.get_address_utxos(&owner_hash);
    let mut utxo_list = Vec::new();
    
    for (i, indexed_utxo) in utxos.iter().enumerate() {
        if i >= limit as usize {
            break;
        }
        
        utxo_list.push(json!({
            "utxo_id": format!("0x{}", hex::encode(indexed_utxo.id.tx_hash)),
            "amount": indexed_utxo.value.to_string(),
            "owner_commitment": owner,
            "created_block": indexed_utxo.height,
            "tree_position": indexed_utxo.height,
            "commitment": format!("0x{}", hex::encode(indexed_utxo.id.tx_hash)),
            "nullifier": format!("0x{}", hex::encode([0u8; 32])), // Placeholder
            "is_spent": indexed_utxo.spent_in_tx.is_some()
        }));
    }
    
    Ok(Json(json!({
        "utxos": utxo_list,
        "total_count": utxo_list.len(),
        "owner": owner,
        "limit": limit
    })))
}

/// Get balance for a specific owner
async fn get_balance(
    Path(owner): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Convert owner string to bytes
    let owner_bytes = owner.as_bytes();
    let mut owner_hash = [0u8; 32];
    if owner_bytes.len() >= 32 {
        owner_hash.copy_from_slice(&owner_bytes[..32]);
    } else {
        owner_hash[..owner_bytes.len()].copy_from_slice(owner_bytes);
    }
    
    // Get UTXOs and calculate balance
    let utxos = state.utxo_index.get_address_utxos(&owner_hash);
    let total_balance: u64 = utxos.iter().map(|utxo| utxo.value).sum();
    
    Ok(Json(json!({
        "balance": {
            "balance": total_balance.to_string(),
            "utxo_count": utxos.len() as u32,
            "last_updated_block": 0,
            "asset_id": "ETH"
        },
        "owner": owner
    })))
}

/// Process a new deposit
async fn process_deposit(
    State(state): State<AppState>,
    Json(deposit): Json<DepositRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Store clones for later use
    let depositor_clone = deposit.depositor.clone();
    let commitment_clone = deposit.commitment.clone();
    let transaction_hash_clone = deposit.transaction_hash.clone();
    let amount_clone = deposit.amount.clone();

    // Create a mock deposit event from the request
    let deposit_event = DepositEvent {
        depositor: deposit.depositor,
        commitment: deposit.commitment,
        value: deposit.amount.parse().unwrap_or_default(),
        block_number: deposit.block_number,
        transaction_hash: deposit.transaction_hash,
        label: deposit.label.unwrap_or_default().parse().unwrap_or_default(),
        log_index: 0,
        precommitment_hash: deposit.precommitment_hash.unwrap_or_default(),
        merkle_root: "0x0000000000000000000000000000000000000000000000000000000000000000".to_string(),
    };
    
    // Process the deposit using the converter
    // For API server, generate a dummy private key (in production, this should come from the depositor)
    let dummy_private_key = [0u8; 32]; // In production, this should be provided by the client
    match state.utxo_converter.lock().await.process_real_eth_deposit(&deposit_event, &dummy_private_key).await {
        Ok(result) => {
            // Update tree version
            let mut tree_version = state.tree_version.lock().await;
            *tree_version += 1;
            
            // Update tree root (simplified)
            let mut tree_root = state.tree_root.lock().await;
            *tree_root = sha256(&format!("{}{}", hex::encode(*tree_root), *tree_version).as_bytes());
            
            Ok(Json(json!({
                "success": true,
                "deposit_event": {
                    "depositor": depositor_clone,
                    "amount": amount_clone,
                    "commitment": commitment_clone,
                    "block_number": deposit.block_number,
                    "transaction_hash": transaction_hash_clone,
                    "timestamp": std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    "utxo_count": result.len(),
                    "status": "processed"
                },
                "utxo_count": result.len(),
                "tree_position": 1,
                "merkle_root": format!("0x{}", hex::encode(*tree_root))
            })))
        }
        Err(e) => {
            Err((StatusCode::BAD_REQUEST, Json(json!({
                "error": "DEPOSIT_FAILED",
                "message": e.to_string(),
                "timestamp": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            }))))
        }
    }
}

/// Get tree statistics
async fn get_tree_stats(
    State(state): State<AppState>,
) -> Json<Value> {
    let tree_version = *state.tree_version.lock().await;
    let tree_root = *state.tree_root.lock().await;
    let utxo_count = state.utxo_index.count();
    
    Json(json!({
        "stats": {
            "current_root": format!("0x{}", hex::encode(tree_root)),
            "root_version": tree_version,
            "total_utxos": utxo_count,
            "total_nodes": utxo_count,
            "tree_depth": 32,
            "last_updated": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        }
    }))
}

/// Get current Merkle tree root
async fn get_tree_root(
    State(state): State<AppState>,
) -> Json<Value> {
    let tree_root = *state.tree_root.lock().await;
    let tree_version = *state.tree_version.lock().await;
    
    Json(json!({
        "root": format!("0x{}", hex::encode(tree_root)),
        "version": tree_version,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

/// Query parameters for UTXO listing
#[derive(serde::Deserialize)]
struct UTXOQuery {
    limit: Option<u32>,
    after_block: Option<u64>,
    asset_id: Option<String>,
}

/// Deposit request from frontend
#[derive(serde::Deserialize)]
struct DepositRequest {
    depositor: String,
    amount: String,
    commitment: String,
    block_number: u64,
    transaction_hash: String,
    label: Option<String>,
    precommitment_hash: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!(" Starting Working Privacy Pool API Server...");
    
    // Create application state
    let app_state = AppState::new()?;
    
    // Create router
    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        
        // UTXO endpoints
        .route("/api/utxos/:owner", get(get_utxos))
        .route("/api/balance/:owner", get(get_balance))
        
        // Deposit endpoints
        .route("/api/deposit", post(process_deposit))
        
        // Tree endpoints
        .route("/api/tree/root", get(get_tree_root))
        .route("/api/tree/stats", get(get_tree_stats))
        
        // Add CORS middleware
        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
        )
        .with_state(app_state);
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = TcpListener::bind(addr).await?;
    
    println!(" Working Privacy Pool API Server running on http://{}", addr);
    println!(" Frontend can connect to: http://localhost:8080");
    println!(" Health check: http://localhost:8080/health");
    println!(" API endpoints:");
    println!("  GET  /api/utxos/:owner     - Get UTXOs for owner");
    println!("  GET  /api/balance/:owner   - Get balance for owner");
    println!("  POST /api/deposit          - Process new deposit");
    println!("  GET  /api/tree/stats       - Get tree statistics");
    println!("  GET  /api/tree/root        - Get current Merkle root");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
