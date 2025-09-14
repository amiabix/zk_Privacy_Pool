//! Standalone Privacy Pool API Server
//! REST API server that connects the React frontend with basic UTXO functionality

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
use serde::{Deserialize, Serialize};

/// Simple application state
#[derive(Clone)]
pub struct AppState {
    pub merkle_root: String,
    pub total_utxos: u32,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            merkle_root: "0x04735efbce809c030d37ba49c991137ee9bae0681dd865766a2c50dd1c301282".to_string(),
            total_utxos: 0,
        }
    }
}

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "privacy-pool-api",
        "version": "1.0.0"
    }))
}

/// Get UTXOs for a specific owner
async fn get_utxos(
    Path(owner): Path<String>,
    Query(params): Query<UTXOQuery>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let limit = params.limit.unwrap_or(100);
    let after_block = params.after_block;
    
    // Mock UTXO data
    let mock_utxos = vec![
        json!({
            "utxo_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "amount": "1.5",
            "owner_commitment": owner,
            "created_block": 12345,
            "tree_position": 1,
            "commitment": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "nullifier": "0x1111111111111111111111111111111111111111111111111111111111111111",
            "is_spent": false
        }),
        json!({
            "utxo_id": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "amount": "0.5",
            "owner_commitment": owner,
            "created_block": 12346,
            "tree_position": 2,
            "commitment": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "nullifier": "0x2222222222222222222222222222222222222222222222222222222222222222",
            "is_spent": false
        }),
    ];
    
    Ok(Json(json!({
        "utxos": mock_utxos,
        "total_count": mock_utxos.len(),
        "owner": owner,
        "limit": limit,
        "after_block": after_block
    })))
}

/// Get balance for a specific owner
async fn get_balance(
    Path(owner): Path<String>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Mock balance data
    let balance = json!({
        "balance": "2.0",
        "utxo_count": 2,
        "last_updated_block": 12346,
        "asset_id": "ETH"
    });
    
    Ok(Json(json!({
        "balance": balance,
        "owner": owner
    })))
}

/// Process a new deposit
async fn process_deposit(
    State(state): State<AppState>,
    Json(deposit): Json<DepositRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    // Mock deposit processing
    let response = json!({
        "success": true,
        "deposit_event": {
            "depositor": deposit.depositor,
            "amount": deposit.amount,
            "commitment": format!("0x{:064x}", rand::random::<u64>()),
            "block_number": deposit.block_number,
            "transaction_hash": deposit.transaction_hash,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            "utxo_id": format!("0x{:064x}", rand::random::<u64>()),
            "status": "processed"
        },
        "utxo_id": format!("0x{:064x}", rand::random::<u64>()),
        "tree_position": 1,
        "merkle_root": state.merkle_root
    });
    
    Ok(Json(response))
}

/// Get tree statistics
async fn get_tree_stats(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let stats = json!({
        "current_root": state.merkle_root,
        "root_version": 1,
        "total_utxos": state.total_utxos,
        "total_nodes": 3,
        "tree_depth": 32,
        "last_updated": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    });
    
    Ok(Json(json!({
        "stats": stats
    })))
}

/// Get current Merkle tree root
async fn get_tree_root(
    State(state): State<AppState>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Ok(Json(json!({
        "root": state.merkle_root,
        "version": 1,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    })))
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
    commitment: Option<String>,
    block_number: u64,
    transaction_hash: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!(" Starting Standalone Privacy Pool API Server...");
    
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
    
    println!(" Standalone Privacy Pool API Server running on http://{}", addr);
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
