//! Hash Utilities
//! Common hash functions and utilities for the privacy pool

use sha2::{Sha256, Digest};
use sha3::{Keccak256, Digest as Sha3Digest};

/// Hash a byte slice using SHA-256
pub fn sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash a byte slice using Keccak-256
pub fn keccak256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Keccak256::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Hash two 32-byte arrays together
pub fn hash_pair(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(&left);
    data.extend_from_slice(&right);
    sha256(&data)
}

/// Hash multiple byte slices together
pub fn hash_multiple(data: &[&[u8]]) -> [u8; 32] {
    let mut combined = Vec::new();
    for slice in data {
        combined.extend_from_slice(slice);
    }
    sha256(&combined)
}
