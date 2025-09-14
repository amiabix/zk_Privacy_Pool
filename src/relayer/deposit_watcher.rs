use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::Mutex;
use web3::types::{FilterBuilder, Log, Address, H256, U256, BlockNumber};
use web3::transports::Http;
use web3::Web3;
use hex;
use crate::utxo::converter::{ETHToUTXOConverter, IndexedUTXO};
use crate::crypto::CryptoUtils;
use crate::database::DatabaseManager; // your RocksDB wrapper

/// Relayer config
#[derive(Debug, Clone)]
pub struct RelayerConfig {
    pub rpc_url: String,
    pub pool_address: Address,
    pub confirmations: u64,
    pub poll_interval_ms: u64,
}

pub struct DepositWatcher {
    web3: Web3<Http>,
    cfg: RelayerConfig,
    converter: Arc<Mutex<ETHToUTXOConverter>>,
    db: Arc<Mutex<DatabaseManager>>,
}

impl DepositWatcher {
    pub fn new(cfg: RelayerConfig, converter: Arc<Mutex<ETHToUTXOConverter>>, db: Arc<Mutex<DatabaseManager>>) -> Result<Self> {
        let transport = Http::new(&cfg.rpc_url)?;
        Ok(Self {
            web3: Web3::new(transport),
            cfg,
            converter,
            db,
        })
    }

    /// Poll loop — production should use websocket subscription (log subscription) + fallback to polling
    pub async fn run_poll_loop(self: Arc<Self>) -> Result<()> {
        loop {
            if let Err(e) = self.process_new_logs().await {
                log::error!("error in deposit watcher loop: {:?}", e);
            }
            tokio::time::sleep(std::time::Duration::from_millis(self.cfg.poll_interval_ms)).await;
        }
    }

    async fn process_new_logs(&self) -> Result<()> {
        // Build event signature for Deposited event:
        // event DepositIndexed(address indexed depositor, bytes32 indexed commitment, uint256 value, address asset, bytes32 txHash, uint64 blockNumber, uint32 logIndex)
        // IMPORTANT: Use the exact ABI types and ordering as emitted by the contract
        let topic0 = web3::helpers::keccak256("DepositIndexed(address,bytes32,uint256,address,bytes32,uint64,uint32)".as_bytes());

        let filter = FilterBuilder::default()
            .address(vec![self.cfg.pool_address])
            .topic0(Some(H256::from_slice(&topic0)))
            .from_block(BlockNumber::Earliest)
            .to_block(BlockNumber::Latest)
            .build();

        let logs = self.web3.eth().logs(filter).await.context("fetch logs")?;

        // head block for confirmations
        let head_block = self.web3.eth().block_number().await?.as_u64();

        for log in logs.into_iter() {
            // extract block number, skip if not enough confirmations
            let log_block = match log.block_number {
                Some(bn) => bn.as_u64(),
                None => continue,
            };

            if head_block.saturating_sub(log_block) < self.cfg.confirmations {
                // not confirmed yet
                continue;
            }

            // idempotency: check DB if this txHash+logIndex already processed
            let tx = log.transaction_hash.unwrap_or_else(|| H256::zero());
            let id = format!("{}:{}", hex::encode(tx.as_bytes()), log.log_index.unwrap_or_default().as_u64());
            {
                let mut db = self.db.lock().await;
                if db.get_processed_flag(&id)? {
                    // already handled
                    continue;
                }
            }

            // parse topics & data: topics[1] = depositor, topics[2] = commitment
            if log.topics.len() < 3 {
                log::warn!("skipping malformed deposit event: topics < 3");
                continue;
            }

            let commitment = {
                let mut b = [0u8; 32];
                b.copy_from_slice(log.topics[2].as_bytes());
                b
            };

            // decode non-indexed data using ABI (your DepositIndexed's data encoding must be known)
            // Here we assume `data` encodes: value(uint256) | asset(address) | txHash(bytes32) | blockNumber(uint64) | logIndex(uint32)
            // If you emit only a subset, decode accordingly.
            let value = decode_value_from_log(&log.data.0)?;
            // NOTE: owner_pubkey must be provided by depositor to relayer (or left zero if wallet keeps secret). We'll accept optional mapping.

            // Owner pubkey: try to fetch a previously uploaded encrypted note where commitment matches
            let owner_pubkey = {
                let mut db = self.db.lock().await;
                if let Some(enc_note) = db.get_encrypted_note_by_commitment(&commitment)? {
                    enc_note.owner_pubkey
                } else {
                    // zero pubkey placeholder; wallet must keep secret locally
                    [0u8; 32]
                }
            };

            // create UTXO object but DO NOT invent secret/blinding if depositor must hold them locally
            let indexed_utxo = {
                let conv = self.converter.lock().await;
                conv.create_utxo_from_onchain(
                    H256::from(commitment),
                    U256::from(value),
                    owner_pubkey,
                    tx,
                    log_block,
                    log.log_index.unwrap_or_default().as_u32(),
                ).await?
            };

            // insert into merkle and persist
            {
                let mut conv = self.converter.lock().await;
                let leaf_index = conv.insert_utxo(indexed_utxo.clone()).await?;
                // persist processed flag & mapping
                let mut db = self.db.lock().await;
                db.mark_processed(&id)?;
                db.put_utxo_mapping(&commitment, &indexed_utxo, leaf_index)?;
            }

            log::info!("Inserted commitment {} at leaf {}", hex::encode(commitment), "TODO"); // replace with actual leaf index
        }

        Ok(())
    }
}

// helper to decode value from data blob when ABI layout known (quick naive decode)
fn decode_value_from_log(data: &[u8]) -> Result<u128> {
    // big-endian uint256 at offset 0..32
    if data.len() < 32 {
        anyhow::bail!("log data too short");
    }
    let mut buf = [0u8; 32];
    buf.copy_from_slice(&data[0..32]);
    let value = u128::from_be_bytes(buf[16..32].try_into().unwrap_or([0u8; 16])); // careful
    Ok(value)
}
