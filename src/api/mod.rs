//! API Module for Privacy Pool ZKVM
//! 
//! Provides HTTP REST API endpoints for frontend integration
//! while maintaining all existing UTXO and Merkle tree functionality.

pub mod handlers;
pub mod types;
pub mod server;
pub mod middleware;

// Re-export main types
pub use handlers::*;
pub use types::*;
pub use server::ApiServer;
pub use middleware::*;