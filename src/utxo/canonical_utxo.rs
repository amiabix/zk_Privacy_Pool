//! Canonical UTXO Implementation
//! 
//! This module implements the grade UTXO format with exact
//! byte-level serialization according to the canonical specification.

use serde::{Serialize, Deserialize};
use crate::canonical_spec::{self, utxo_format, cf_prefixes};
use anyhow::{Result, anyhow, bail};
use std::io::{Cursor, Write, Read};

/// Lock flags bit definitions
pub mod lock_flags {
    pub const TIMELOCK_PRESENT: u8 = 0x01;
    pub const SCRIPT_PRESENT: u8 = 0x02;
    pub const WITHDRAWAL_LOCK: u8 = 0x04;
    pub const RESERVED_FLAG_1: u8 = 0x08;
    pub const RESERVED_FLAG_2: u8 = 0x10;
    pub const RESERVED_FLAG_3: u8 = 0x20;
    pub const RESERVED_FLAG_4: u8 = 0x40;
    pub const RESERVED_FLAG_5: u8 = 0x80;
}

/// Enhanced UTXO structure following canonical specification
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalUTXO {
    /// UTXO identifier (32 bytes)
    pub utxo_id: [u8; 32],
    
    /// Asset identifier (H160 contract address or 0x00*20 for ETH)
    pub asset_id: [u8; 20],
    
    /// Amount in smallest unit (u128 big-endian)
    pub amount: u128,
    
    /// Privacy-preserving owner commitment (not raw address)
    pub owner_commitment: [u8; 32],
    
    /// Block number when UTXO was created
    pub created_block: u64,
    
    /// Timelock expiration (block number or timestamp)
    pub lock_expiry: u64,
    
    /// Lock flags (see lock_flags module)
    pub lock_flags: u8,
    
    /// Additional lock data (scripts, metadata)
    pub lock_data: Vec<u8>,
}

impl CanonicalUTXO {
    /// Create new UTXO with minimal parameters
    pub fn new(
        txid: [u8; 32],
        vout: u32,
        created_block: u64,
        entropy: u64,
        asset_id: [u8; 20],
        amount: u128,
        owner_commitment: [u8; 32],
    ) -> Self {
        let utxo_id = canonical_spec::generate_utxo_id(txid, vout, created_block, entropy);
        
        Self {
            utxo_id,
            asset_id,
            amount,
            owner_commitment,
            created_block,
            lock_expiry: 0,
            lock_flags: 0,
            lock_data: Vec::new(),
        }
    }

    /// Create UTXO for native ETH
    pub fn new_eth(
        txid: [u8; 32],
        vout: u32,
        created_block: u64,
        entropy: u64,
        amount: u128,
        owner_commitment: [u8; 32],
    ) -> Self {
        Self::new(
            txid,
            vout,
            created_block,
            entropy,
            utxo_format::ETH_ASSET_ID,
            amount,
            owner_commitment,
        )
    }

    /// Create UTXO with timelock
    pub fn with_timelock(mut self, lock_expiry: u64) -> Self {
        self.lock_expiry = lock_expiry;
        self.lock_flags |= lock_flags::TIMELOCK_PRESENT;
        self
    }

    /// Create UTXO with script data
    pub fn with_script(mut self, script_data: Vec<u8>) -> Self {
        self.lock_data = script_data;
        if !self.lock_data.is_empty() {
            self.lock_flags |= lock_flags::SCRIPT_PRESENT;
        }
        self
    }

    /// Check if this is an ETH UTXO
    pub fn is_eth(&self) -> bool {
        self.asset_id == utxo_format::ETH_ASSET_ID
    }

    /// Check if timelock is active
    pub fn has_timelock(&self) -> bool {
        self.lock_flags & lock_flags::TIMELOCK_PRESENT != 0
    }

    /// Check if script is present
    pub fn has_script(&self) -> bool {
        self.lock_flags & lock_flags::SCRIPT_PRESENT != 0
    }

    /// Check if timelock has expired
    pub fn is_timelock_expired(&self, current_block_or_time: u64) -> bool {
        if !self.has_timelock() {
            return true; // No timelock means always spendable
        }
        current_block_or_time >= self.lock_expiry
    }

    /// Serialize to canonical binary format
    /// 
    /// Format:
    /// - magic (4 bytes BE): 0x55545830 ("UTX0")
    /// - version (2 bytes BE): current version
    /// - flags (2 bytes BE): extensibility flags
    /// - utxo_id (32 bytes): repeated for cross-check
    /// - asset_id (20 bytes): contract address or ETH
    /// - _reserved_1 (4 bytes): alignment padding
    /// - amount (16 bytes BE): u128 amount
    /// - owner_commitment (32 bytes): privacy commitment
    /// - created_block (8 bytes BE): creation block
    /// - lock_expiry (8 bytes BE): timelock expiry
    /// - lock_flags (1 byte): flag bits
    /// - _reserved_2 (3 bytes): padding
    /// - lock_data_len (4 bytes BE): script length
    /// - lock_data (variable, padded to 8-byte boundary)
    /// - checksum (4 bytes BE): CRC32 of all preceding data
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let lock_data_padded_len = canonical_spec::align8(self.lock_data.len());
        let total_size = utxo_format::MIN_SIZE + lock_data_padded_len;
        
        let mut buffer = Vec::with_capacity(total_size);
        let mut cursor = Cursor::new(&mut buffer);

        // Magic number (4 bytes BE)
        cursor.write_all(&utxo_format::MAGIC.to_be_bytes())?;
        
        // Version (2 bytes BE)
        cursor.write_all(&utxo_format::VERSION.to_be_bytes())?;
        
        // Flags (2 bytes BE) - currently unused, reserved for future
        cursor.write_all(&0u16.to_be_bytes())?;
        
        // UTXO ID (32 bytes)
        cursor.write_all(&self.utxo_id)?;
        
        // Asset ID (20 bytes)
        cursor.write_all(&self.asset_id)?;
        
        // Reserved padding 1 (4 bytes)
        cursor.write_all(&[0u8; 4])?;
        
        // Amount (16 bytes BE)
        cursor.write_all(&self.amount.to_be_bytes())?;
        
        // Owner commitment (32 bytes)
        cursor.write_all(&self.owner_commitment)?;
        
        // Created block (8 bytes BE)
        cursor.write_all(&self.created_block.to_be_bytes())?;
        
        // Lock expiry (8 bytes BE)
        cursor.write_all(&self.lock_expiry.to_be_bytes())?;
        
        // Lock flags (1 byte)
        cursor.write_all(&[self.lock_flags])?;
        
        // Reserved padding 2 (3 bytes)
        cursor.write_all(&[0u8; 3])?;
        
        // Lock data length (4 bytes BE)
        cursor.write_all(&(self.lock_data.len() as u32).to_be_bytes())?;
        
        // Lock data (padded to 8-byte boundary)
        cursor.write_all(&self.lock_data)?;
        let padding_needed = lock_data_padded_len - self.lock_data.len();
        if padding_needed > 0 {
            cursor.write_all(&vec![0u8; padding_needed])?;
        }

        // Calculate checksum over all data so far
        // Drop cursor to release mutable borrow
        drop(cursor);
        let checksum = canonical_spec::calculate_crc32(&buffer);
        
        // Checksum (4 bytes BE)
        buffer.extend_from_slice(&checksum.to_be_bytes());
        
        Ok(buffer)
    }

    /// Deserialize from canonical binary format
    pub fn deserialize(data: &[u8]) -> Result<Self> {
        if data.len() < utxo_format::MIN_SIZE {
            bail!("UTXO data too short: {} bytes", data.len());
        }

        let mut cursor = Cursor::new(data);
        
        // Helper function to read u32 big-endian
        let read_u32_be = |cursor: &mut Cursor<&[u8]>| -> Result<u32> {
            let mut buf = [0u8; 4];
            cursor.read_exact(&mut buf)?;
            Ok(u32::from_be_bytes(buf))
        };
        
        // Helper function to read u16 big-endian
        let read_u16_be = |cursor: &mut Cursor<&[u8]>| -> Result<u16> {
            let mut buf = [0u8; 2];
            cursor.read_exact(&mut buf)?;
            Ok(u16::from_be_bytes(buf))
        };
        
        // Helper function to read u64 big-endian
        let read_u64_be = |cursor: &mut Cursor<&[u8]>| -> Result<u64> {
            let mut buf = [0u8; 8];
            cursor.read_exact(&mut buf)?;
            Ok(u64::from_be_bytes(buf))
        };
        
        // Helper function to read u128 big-endian
        let read_u128_be = |cursor: &mut Cursor<&[u8]>| -> Result<u128> {
            let mut buf = [0u8; 16];
            cursor.read_exact(&mut buf)?;
            Ok(u128::from_be_bytes(buf))
        };
        
        // Helper function to read bytes
        let read_bytes = |cursor: &mut Cursor<&[u8]>, len: usize| -> Result<Vec<u8>> {
            let mut buf = vec![0u8; len];
            cursor.read_exact(&mut buf)?;
            Ok(buf)
        };

        // Magic number
        let magic = read_u32_be(&mut cursor)?;
        if magic != utxo_format::MAGIC {
            bail!("Invalid UTXO magic: 0x{:08x}", magic);
        }

        // Version
        let version = read_u16_be(&mut cursor)?;
        if version != utxo_format::VERSION {
            bail!("Unsupported UTXO version: {}", version);
        }

        // Flags (currently unused)
        let _flags = read_u16_be(&mut cursor)?;

        // UTXO ID
        let utxo_id_bytes = read_bytes(&mut cursor, 32)?;
        let utxo_id: [u8; 32] = utxo_id_bytes.try_into()
            .map_err(|_| anyhow!("Invalid UTXO ID length"))?;

        // Asset ID
        let asset_id_bytes = read_bytes(&mut cursor, 20)?;
        let asset_id: [u8; 20] = asset_id_bytes.try_into()
            .map_err(|_| anyhow!("Invalid asset ID length"))?;

        // Reserved padding 1
        let _reserved1 = read_bytes(&mut cursor, 4)?;

        // Amount
        let amount = read_u128_be(&mut cursor)?;

        // Owner commitment
        let owner_commitment_bytes = read_bytes(&mut cursor, 32)?;
        let owner_commitment: [u8; 32] = owner_commitment_bytes.try_into()
            .map_err(|_| anyhow!("Invalid owner commitment length"))?;

        // Created block
        let created_block = read_u64_be(&mut cursor)?;

        // Lock expiry
        let lock_expiry = read_u64_be(&mut cursor)?;

        // Lock flags
        let mut lock_flags_buf = [0u8; 1];
        cursor.read_exact(&mut lock_flags_buf)?;
        let lock_flags = lock_flags_buf[0];

        // Reserved padding 2
        let _reserved2 = read_bytes(&mut cursor, 3)?;

        // Lock data length
        let lock_data_len = read_u32_be(&mut cursor)? as usize;
        
        // Validate lock data length is reasonable
        if lock_data_len > 1024 * 1024 {  // 1MB max
            bail!("Lock data too large: {} bytes", lock_data_len);
        }

        // Lock data (with padding)
        let lock_data_padded_len = canonical_spec::align8(lock_data_len);
        let lock_data_padded = read_bytes(&mut cursor, lock_data_padded_len)?;
        let lock_data = lock_data_padded[..lock_data_len].to_vec();

        // Verify checksum
        let expected_checksum = read_u32_be(&mut cursor)?;
        let data_for_checksum = &data[..data.len() - 4]; // All except checksum
        let actual_checksum = canonical_spec::calculate_crc32(data_for_checksum);
        
        if expected_checksum != actual_checksum {
            bail!("UTXO checksum mismatch: expected 0x{:08x}, got 0x{:08x}", 
                  expected_checksum, actual_checksum);
        }

        Ok(Self {
            utxo_id,
            asset_id,
            amount,
            owner_commitment,
            created_block,
            lock_expiry,
            lock_flags,
            lock_data,
        })
    }

    /// Generate leaf hash for this UTXO
    pub fn leaf_hash(&self) -> Result<[u8; 32]> {
        let serialized = self.serialize()?;
        Ok(canonical_spec::generate_leaf_hash(&serialized))
    }

    /// Get total serialized size including padding
    pub fn serialized_size(&self) -> usize {
        utxo_format::MIN_SIZE + canonical_spec::align8(self.lock_data.len())
    }

    /// Validate UTXO structure
    pub fn validate(&self) -> Result<()> {
        // Check amount is non-zero for active UTXOs
        if self.amount == 0 {
            bail!("UTXO amount cannot be zero");
        }

        // Validate lock flags consistency
        if self.has_timelock() && self.lock_expiry == 0 {
            bail!("Timelock flag set but expiry is zero");
        }

        if self.has_script() && self.lock_data.is_empty() {
            bail!("Script flag set but lock_data is empty");
        }

        if !self.has_script() && !self.lock_data.is_empty() {
            bail!("Lock data present but script flag not set");
        }

        // Validate lock data size
        if self.lock_data.len() > 1024 * 1024 {  // 1MB max
            bail!("Lock data too large: {} bytes", self.lock_data.len());
        }

        Ok(())
    }

    /// Create database key for cf_utxos
    pub fn db_key(&self) -> Vec<u8> {
        let mut key = Vec::with_capacity(33);
        key.push(cf_prefixes::UTXOS);
        key.extend_from_slice(&self.utxo_id);
        key
    }

    /// Create database key for cf_owner_index
    pub fn owner_index_key(&self) -> Vec<u8> {
        let mut key = Vec::with_capacity(73); // 1 + 32 + 8 + 32
        key.push(cf_prefixes::OWNER_INDEX);
        key.extend_from_slice(&self.owner_commitment);
        key.extend_from_slice(&self.created_block.to_be_bytes());
        key.extend_from_slice(&self.utxo_id);
        key
    }

    /// Create database value for cf_owner_index
    pub fn owner_index_value(&self) -> Vec<u8> {
        let mut value = Vec::with_capacity(37); // 16 + 20 + 1
        value.extend_from_slice(&self.amount.to_be_bytes());
        value.extend_from_slice(&self.asset_id);
        value.push(self.lock_flags);
        value
    }
}

/// UTXO validation errors
#[derive(Debug, thiserror::Error)]
pub enum UTXOError {
    #[error("Invalid UTXO format: {0}")]
    InvalidFormat(String),
    
    #[error("UTXO checksum mismatch")]
    ChecksumMismatch,
    
    #[error("Unsupported UTXO version: {0}")]
    UnsupportedVersion(u16),
    
    #[error("UTXO data too short: {0} bytes")]
    DataTooShort(usize),
    
    #[error("Lock data too large: {0} bytes")]
    LockDataTooLarge(usize),
    
    #[error("Invalid lock flags: {0}")]
    InvalidLockFlags(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_creation() {
        let txid = [1u8; 32];
        let vout = 0;
        let created_block = 12345;
        let entropy = 67890;
        let amount = 1_000_000_000_000_000_000u128; // 1 ETH in wei
        let owner_commitment = [2u8; 32];

        let utxo = CanonicalUTXO::new_eth(
            txid, vout, created_block, entropy, amount, owner_commitment
        );

        assert!(utxo.is_eth());
        assert_eq!(utxo.amount, amount);
        assert_eq!(utxo.owner_commitment, owner_commitment);
        assert_eq!(utxo.created_block, created_block);
        assert!(!utxo.has_timelock());
        assert!(!utxo.has_script());
    }

    #[test]
    fn test_utxo_serialization_roundtrip() {
        let txid = [1u8; 32];
        let vout = 0;
        let created_block = 12345;
        let entropy = 67890;
        let amount = 1_000_000_000_000_000_000u128;
        let owner_commitment = [2u8; 32];

        let utxo = CanonicalUTXO::new_eth(
            txid, vout, created_block, entropy, amount, owner_commitment
        );

        let serialized = utxo.serialize().unwrap();
        let deserialized = CanonicalUTXO::deserialize(&serialized).unwrap();

        assert_eq!(utxo, deserialized);
    }

    #[test]
    fn test_utxo_with_timelock() {
        let txid = [1u8; 32];
        let vout = 0;
        let created_block = 12345;
        let entropy = 67890;
        let amount = 1_000_000_000_000_000_000u128;
        let owner_commitment = [2u8; 32];
        let lock_expiry = 54321;

        let utxo = CanonicalUTXO::new_eth(
            txid, vout, created_block, entropy, amount, owner_commitment
        ).with_timelock(lock_expiry);

        assert!(utxo.has_timelock());
        assert_eq!(utxo.lock_expiry, lock_expiry);
        assert!(!utxo.is_timelock_expired(lock_expiry - 1));
        assert!(utxo.is_timelock_expired(lock_expiry));
        assert!(utxo.is_timelock_expired(lock_expiry + 1));

        // Test serialization roundtrip
        let serialized = utxo.serialize().unwrap();
        let deserialized = CanonicalUTXO::deserialize(&serialized).unwrap();
        assert_eq!(utxo, deserialized);
    }

    #[test]
    fn test_utxo_with_script() {
        let txid = [1u8; 32];
        let vout = 0;
        let created_block = 12345;
        let entropy = 67890;
        let amount = 1_000_000_000_000_000_000u128;
        let owner_commitment = [2u8; 32];
        let script_data = vec![1, 2, 3, 4, 5];

        let utxo = CanonicalUTXO::new_eth(
            txid, vout, created_block, entropy, amount, owner_commitment
        ).with_script(script_data.clone());

        assert!(utxo.has_script());
        assert_eq!(utxo.lock_data, script_data);

        // Test serialization roundtrip
        let serialized = utxo.serialize().unwrap();
        let deserialized = CanonicalUTXO::deserialize(&serialized).unwrap();
        assert_eq!(utxo, deserialized);
    }

    #[test]
    fn test_utxo_validation() {
        let txid = [1u8; 32];
        let vout = 0;
        let created_block = 12345;
        let entropy = 67890;
        let amount = 1_000_000_000_000_000_000u128;
        let owner_commitment = [2u8; 32];

        let valid_utxo = CanonicalUTXO::new_eth(
            txid, vout, created_block, entropy, amount, owner_commitment
        );
        assert!(valid_utxo.validate().is_ok());

        // Test zero amount
        let mut invalid_utxo = valid_utxo.clone();
        invalid_utxo.amount = 0;
        assert!(invalid_utxo.validate().is_err());

        // Test inconsistent timelock flag
        let mut invalid_utxo = valid_utxo.clone();
        invalid_utxo.lock_flags = lock_flags::TIMELOCK_PRESENT;
        invalid_utxo.lock_expiry = 0;
        assert!(invalid_utxo.validate().is_err());
    }

    #[test]
    fn test_leaf_hash_generation() {
        let txid = [1u8; 32];
        let vout = 0;
        let created_block = 12345;
        let entropy = 67890;
        let amount = 1_000_000_000_000_000_000u128;
        let owner_commitment = [2u8; 32];

        let utxo = CanonicalUTXO::new_eth(
            txid, vout, created_block, entropy, amount, owner_commitment
        );

        let leaf_hash = utxo.leaf_hash().unwrap();
        assert_eq!(leaf_hash.len(), 32);

        // Should be deterministic
        let leaf_hash2 = utxo.leaf_hash().unwrap();
        assert_eq!(leaf_hash, leaf_hash2);
    }

    #[test]
    fn test_database_keys() {
        let txid = [1u8; 32];
        let vout = 0;
        let created_block = 12345;
        let entropy = 67890;
        let amount = 1_000_000_000_000_000_000u128;
        let owner_commitment = [2u8; 32];

        let utxo = CanonicalUTXO::new_eth(
            txid, vout, created_block, entropy, amount, owner_commitment
        );

        let db_key = utxo.db_key();
        assert_eq!(db_key[0], cf_prefixes::UTXOS);
        assert_eq!(db_key.len(), 33); // 1 + 32

        let owner_key = utxo.owner_index_key();
        assert_eq!(owner_key[0], cf_prefixes::OWNER_INDEX);
        assert_eq!(owner_key.len(), 73); // 1 + 32 + 8 + 32

        let owner_value = utxo.owner_index_value();
        assert_eq!(owner_value.len(), 37); // 16 + 20 + 1
    }
}