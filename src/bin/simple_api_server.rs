//! Simple Privacy Pool API Server
//! 
//! Lightweight API server with mock data for quick frontend testing.
//! Uses the same endpoint structure as the full API server.

use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Query},
    Json,
};
use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use serde::Deserialize;

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

/// Health check endpoint
async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "healthy",
        "service": "privacy-pool-simple-api",
        "version": "0.1.0",
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

/// Get UTXOs for a specific owner
async fn get_utxos(
    Path(owner): Path<String>,
    Query(params): Query<UTXOQuery>,
) -> Json<Value> {
    let limit = params.limit.unwrap_or(100);
    
    // Mock UTXO data for demonstration
    let mock_utxos = vec![
        json!({
            "utxo_id": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "amount": "1500000000000000000", // 1.5 ETH in wei
            "asset_id": "0x0000000000000000000000000000000000000000",
            "created_block": 12345,
            "tree_position": 1,
            "lock_expiry": null,
            "lock_flags": 0,
            "is_spent": false
        }),
        json!({
            "utxo_id": "0xabcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890",
            "amount": "500000000000000000", // 0.5 ETH in wei
            "asset_id": "0x0000000000000000000000000000000000000000",
            "created_block": 12346,
            "tree_position": 2,
            "lock_expiry": null,
            "lock_flags": 0,
            "is_spent": false
        }),
    ];
    
    Json(json!({
        "utxos": mock_utxos,
        "total_count": mock_utxos.len(),
        "next_cursor": null
    }))
}

/// Get balance for a specific owner
async fn get_balance(Path(owner): Path<String>) -> Json<Value> {
    Json(json!({
        "balance": "2000000000000000000", // 2.0 ETH in wei
        "utxo_count": 2,
        "last_updated_block": 12346,
        "asset_id": "0x0000000000000000000000000000000000000000"
    }))
}

/// Process a new deposit
async fn process_deposit(Json(deposit): Json<DepositRequest>) -> Json<Value> {
    // Generate mock response
    let utxo_id = format!("0x{:064x}", rand::random::<u64>());
    let leaf_hash = format!("0x{:064x}", rand::random::<u64>());
    let new_root = format!("0x{:064x}", rand::random::<u64>());
    
    Json(json!({
        "success": true,
        "utxo_id": utxo_id,
        "new_root": new_root,
        "tree_position": rand::random::<u64>() % 1000,
        "leaf_hash": leaf_hash,
        "root_version": 1,
        "processed_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

/// Get tree statistics
async fn get_tree_stats() -> Json<Value> {
    Json(json!({
        "current_root": "0x04735efbce809c030d37ba49c991137ee9bae0681dd865766a2c50dd1c301282",
        "root_version": 1,
        "depth": 32,
        "total_utxos": 42,
        "total_nodes": 84,
        "tree_salt": 12345678
    }))
}

/// Get current Merkle tree root
async fn get_tree_root() -> Json<Value> {
    Json(json!({
        "root": "0x04735efbce809c030d37ba49c991137ee9bae0681dd865766a2c50dd1c301282",
        "version": 1,
        "timestamp": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();
    
    println!("🚀 Simple Privacy Pool API Server");
    println!("=================================");
    println!("📡 Quick mock server for frontend development");
    println!();
    
    // Create router with mock endpoints
    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/deposit", post(process_deposit))
        .route("/api/balance/:owner", get(get_balance))
        .route("/api/utxos/:owner", get(get_utxos))
        .route("/api/tree/stats", get(get_tree_stats))
        .route("/api/tree/root", get(get_tree_root))
        .layer(CorsLayer::permissive());
    
    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3001));
    let listener = TcpListener::bind(addr).await?;
    
    println!("✅ Server running on: http://{}", addr);
    println!("📋 Available endpoints:");
    println!("   GET  /api/health          - Health check");
    println!("   POST /api/deposit         - Process ETH deposit (mock)");
    println!("   GET  /api/balance/:owner  - Get owner balance (mock)");
    println!("   GET  /api/utxos/:owner    - Get owner UTXOs (mock)");
    println!("   GET  /api/tree/stats      - Get tree statistics (mock)");
    println!("   GET  /api/tree/root       - Get current tree root (mock)");
    println!();
    println!("🔧 Use this server for frontend development and testing");
    println!("🔗 Example: curl http://localhost:3001/api/health");
    
    axum::serve(listener, app).await?;
    
    Ok(())
}