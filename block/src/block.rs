use balance::Balance;
use chrono::Utc;
use common::{AppResult, error::AppError};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tx::tx::Tx;
use wallet::{Wallet, verify_signature};

pub const BLOCK_VERSION: u32 = 1;
pub const GENESIS_SIGN: &str = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
pub const GENESIS_TIMESTMAP: i64 = 1009227600;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub version: u32,
    pub hash: String,
    pub height: u64,
    pub prev_hash: String,
    pub v_merkle_root: String,
    pub txs_merkle_root: String,
    pub timestamp: i64,
    pub validator: String,
    pub signature: String,
    pub validators: Vec<Balance>,
    pub txs: Vec<Tx>,
}

impl Block {
    pub const GENESIS_HASH: &str =
        "0000000000000000000000000000000000000000000000000000000000000000";
    pub const GENESIS_VALIDATOR: &str =
        "000000000000000000000000000000000000000000000000000000000000000000";

    pub fn new(
        validator: &Wallet,
        prev_hash: String,
        height: u64,
        validators: Vec<Balance>,
        txs: Vec<Tx>,
    ) -> AppResult<Self> {
        let txs_merkle_root = Self::txs_merkle_root_str(&txs);
        let v_merkle_root = Self::v_merkle_root_str(&validators);
        let timestamp = Utc::now().timestamp();
        let validator_addr = validator.address_str();
        let mut block = Self {
            version: BLOCK_VERSION,
            height,
            prev_hash,
            v_merkle_root,
            txs_merkle_root,
            timestamp,
            validator: validator_addr,
            signature: String::default(),
            hash: String::default(),
            validators,
            txs,
        };
        let hash = block.hash();
        block.signature = validator.sign(&hash)?;
        block.hash = block.hash_str();
        Ok(block)
    }

    pub fn genesis(txs: Vec<Tx>) -> Self {
        let txs_merkle_root = Self::txs_merkle_root_str(&txs);
        let validators = vec![Balance::new(
            Self::GENESIS_VALIDATOR.to_string(),
            BigUint::default(),
        )];
        let v_merkle_root = Self::v_merkle_root_str(&validators);
        let mut genesis = Self {
            version: BLOCK_VERSION,
            height: 0,
            prev_hash: Self::GENESIS_HASH.to_string(),
            v_merkle_root,
            txs_merkle_root,
            timestamp: GENESIS_TIMESTMAP,
            validator: Self::GENESIS_VALIDATOR.to_string(),
            validators,
            signature: GENESIS_SIGN.to_string(),
            hash: String::default(),
            txs,
        };
        genesis.hash = genesis.hash_str();
        genesis
    }

    pub fn validate(&self) -> AppResult<()> {
        let hash = self.hash();
        if self.hash != self.hash_str() {
            return Err(AppError::InvalidBlockHash);
        }
        let merkle_root = Self::v_merkle_root_str(&self.validators);
        if merkle_root != self.v_merkle_root {
            return Err(AppError::InvalidTxsMerkle);
        }
        let merkle_root = Self::txs_merkle_root_str(&self.txs);
        if merkle_root != self.txs_merkle_root {
            return Err(AppError::InvalidTxsMerkle);
        }
        verify_signature(&self.validator, &self.signature, &hash)?;
        Ok(())
    }

    fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.version.to_be_bytes());
        hasher.update(self.height.to_be_bytes());
        hasher.update(self.prev_hash.as_bytes());
        hasher.update(Self::v_merkle_root(&self.validators));
        hasher.update(Self::txs_merkle_root(&self.txs));
        hasher.update(self.timestamp.to_be_bytes());
        hasher.update(self.validator.as_bytes());
        hasher.finalize().into()
    }

    pub fn hash_str(&self) -> String {
        bs58::encode(self.hash()).into_string()
    }

    fn txs_merkle_root(txs: &[Tx]) -> [u8; 32] {
        let tx_hashes: Vec<[u8; 32]> = txs.iter().map(|tx| tx.hash()).collect();
        let merkle_root =
            rs_merkle::MerkleTree::<rs_merkle::algorithms::Sha256>::from_leaves(&tx_hashes);
        merkle_root.root().unwrap_or([0u8; 32])
    }

    fn txs_merkle_root_str(txs: &[Tx]) -> String {
        bs58::encode(Self::txs_merkle_root(txs)).into_string()
    }

    fn v_merkle_root(validators: &[Balance]) -> [u8; 32] {
        let tx_hashes: Vec<[u8; 32]> = validators.iter().map(|v| v.hash()).collect();
        let merkle_root =
            rs_merkle::MerkleTree::<rs_merkle::algorithms::Sha256>::from_leaves(&tx_hashes);
        merkle_root.root().unwrap_or([0u8; 32])
    }

    fn v_merkle_root_str(validators: &[Balance]) -> String {
        bs58::encode(Self::v_merkle_root(validators)).into_string()
    }
}
