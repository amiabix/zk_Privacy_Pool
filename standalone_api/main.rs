//! Standalone Working Privacy Pool API Server
//! 
//! Connects frontend to Rust functionality without problematic database dependencies

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
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Simple UTXO representation for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleUTXO {
    pub utxo_id: String,
    pub amount: u64,
    pub owner: String,
    pub created_block: u64,
    pub commitment: String,
    pub nullifier: String,
    pub is_spent: bool,
}

/// Application state with in-memory storage
#[derive(Clone)]
pub struct AppState {
    pub utxos: Arc<tokio::sync::Mutex<HashMap<String, SimpleUTXO>>>,
    pub owner_utxos: Arc<tokio::sync::Mutex<HashMap<String, Vec<String>>>>,
    pub balances: Arc<tokio::sync::Mutex<HashMap<String, u64>>>,
    pub tree_root: Arc<tokio::sync::Mutex<String>>,
    pub tree_version: Arc<tokio::sync::Mutex<u64>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            utxos: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            owner_utxos: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            balances: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            tree_root: Arc::new(tokio::sync::Mutex::new("0x04735efbce809c030d37ba49c991137ee9bae0681dd865766a2c50dd1c301282".to_string())),
            tree_version: Arc::new(tokio::sync::Mutex::new(0)),
        }
    }
}

/// Health check endpoint
async fn health_check(State(state): State<AppState>) -> Json<Value> {
    let tree_version = *state.tree_version.lock().await;
    let utxo_count = state.utxos.lock().await.len();
    
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
    
    let owner_utxos = state.owner_utxos.lock().await;
    let utxos_map = state.utxos.lock().await;
    
    let utxo_ids = owner_utxos.get(&owner).cloned().unwrap_or_default();
    let mut utxo_list = Vec::new();
    
    for (i, utxo_id) in utxo_ids.iter().enumerate() {
        if i >= limit as usize {
            break;
        }
        
        if let Some(utxo) = utxos_map.get(utxo_id) {
            utxo_list.push(json!({
                "utxo_id": utxo.utxo_id,
                "amount": utxo.amount.to_string(),
                "owner_commitment": utxo.owner,
                "created_block": utxo.created_block,
                "tree_position": utxo.created_block,
                "commitment": utxo.commitment,
                "nullifier": utxo.nullifier,
                "is_spent": utxo.is_spent
            }));
        }
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
    let balances = state.balances.lock().await;
    let balance = balances.get(&owner).copied().unwrap_or(0);
    
    let owner_utxos = state.owner_utxos.lock().await;
    let utxo_count = owner_utxos.get(&owner).map(|v| v.len()).unwrap_or(0);
    
    Ok(Json(json!({
        "balance": {
            "balance": balance.to_string(),
            "utxo_count": utxo_count as u32,
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
    // Generate UTXO ID
    let utxo_id = format!("0x{:064x}", rand::random::<u64>());
    let amount = deposit.amount.parse::<u64>().unwrap_or(0);
    
    // Create UTXO
    let utxo = SimpleUTXO {
        utxo_id: utxo_id.clone(),
        amount,
        owner: deposit.depositor.clone(),
        created_block: deposit.block_number,
        commitment: deposit.commitment.clone(),
        nullifier: format!("0x{:064x}", rand::random::<u64>()),
        is_spent: false,
    };
    
    // Update storage
    {
        let mut utxos = state.utxos.lock().await;
        utxos.insert(utxo_id.clone(), utxo.clone());
        
        let mut owner_utxos = state.owner_utxos.lock().await;
        owner_utxos.entry(deposit.depositor.clone())
            .or_insert_with(Vec::new)
            .push(utxo_id.clone());
        
        let mut balances = state.balances.lock().await;
        *balances.entry(deposit.depositor.clone()).or_insert(0) += amount;
        
        // Update tree version
        let mut tree_version = state.tree_version.lock().await;
        *tree_version += 1;
        
        // Update tree root (simplified)
        let mut tree_root = state.tree_root.lock().await;
        *tree_root = format!("0x{:064x}", rand::random::<u64>());
    }
    
    Ok(Json(json!({
        "success": true,
        "deposit_event": {
            "depositor": deposit.depositor,
            "amount": deposit.amount,
            "commitment": deposit.commitment,
            "block_number": deposit.block_number,
            "transaction_hash": deposit.transaction_hash,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "utxo_id": utxo_id,
            "status": "processed"
        },
        "utxo_id": utxo_id,
        "tree_position": 1,
        "merkle_root": *state.tree_root.lock().await
    })))
}

/// Get tree statistics
async fn get_tree_stats(
    State(state): State<AppState>,
) -> Json<Value> {
    let tree_version = *state.tree_version.lock().await;
    let tree_root = state.tree_root.lock().await.clone();
    let utxo_count = state.utxos.lock().await.len();
    
    Json(json!({
        "stats": {
            "current_root": tree_root,
            "root_version": tree_version,
            "total_utxos": utxo_count as u64,
            "total_nodes": utxo_count as u64,
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
    let tree_root = state.tree_root.lock().await.clone();
    let tree_version = *state.tree_version.lock().await;
    
    Json(json!({
        "root": tree_root,
        "version": tree_version,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

/// Query parameters for UTXO listing
#[derive(Deserialize)]
struct UTXOQuery {
    limit: Option<u32>,
    after_block: Option<u64>,
    asset_id: Option<String>,
}

/// Deposit request from frontend
#[derive(Deserialize)]
struct DepositRequest {
    depositor: String,
    amount: String,
    commitment: String,
    block_number: u64,
    transaction_hash: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 Starting Standalone Working Privacy Pool API Server...");
    
    // Create application state
    let app_state = AppState::new();
    
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
    
    println!("✅ Standalone Working Privacy Pool API Server running on http://{}", addr);
    println!("📡 Frontend can connect to: http://localhost:8080");
    println!("🔗 Health check: http://localhost:8080/health");
    println!("📊 API endpoints:");
    println!("  GET  /api/utxos/:owner     - Get UTXOs for owner");
    println!("  GET  /api/balance/:owner   - Get balance for owner");
    println!("  POST /api/deposit          - Process new deposit");
    println!("  GET  /api/tree/stats       - Get tree statistics");
    println!("  GET  /api/tree/root        - Get current Merkle root");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
