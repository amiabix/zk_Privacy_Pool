//! Privacy Pool API Server
//! 
//! Main entry point for the Privacy Pool REST API server.
//! Integrates with existing UTXO and Merkle tree modules.

use privacy_pool_zkvm::api::{ApiServer, ApiServerBuilder};
use std::env;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();
    
    // Get configuration from environment variables
    let bind_addr = env::var("BIND_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string());
    
    let max_request_size = env::var("MAX_REQUEST_SIZE")
        .unwrap_or_else(|_| "1048576".to_string()) // 1MB default
        .parse()
        .unwrap_or(1024 * 1024);
    
    let request_timeout = env::var("REQUEST_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .unwrap_or(30);
    
    let enable_logging = env::var("ENABLE_LOGGING")
        .unwrap_or_else(|_| "true".to_string())
        .parse()
        .unwrap_or(true);
    
    println!("🔐 Privacy Pool ZKVM API Server");
    println!("===============================");
    println!();
    
    // Build and start server
    let server = ApiServerBuilder::new()
        .bind(&bind_addr)?
        .max_request_size(max_request_size)
        .request_timeout(request_timeout)
        .logging(enable_logging)
        .build()?;
    
    // Start the server
    server.start().await?;
    
    Ok(())
}