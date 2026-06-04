use balance::Balance;
use serde::{Deserialize, Serialize};
use tx::tx::Tx;

use crate::block::Block;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct BlockData {
    pub hash: String,
    pub version: u32,
    pub height: u64,
    pub prev_hash: String,
    pub v_merkle_root: String,
    pub txs_merkle_root: String,
    pub timestamp: i64,
    pub validator: String,
    pub validators: Vec<Balance>,
    pub signature: String,
}

impl From<&Block> for BlockData {
    fn from(block: &Block) -> Self {
        Self {
            hash: block.hash.clone(),
            version: block.version,
            height: block.height,
            prev_hash: block.prev_hash.clone(),
            v_merkle_root: block.v_merkle_root.clone(),
            txs_merkle_root: block.txs_merkle_root.clone(),
            timestamp: block.timestamp,
            validator: block.validator.clone(),
            validators: block.validators.clone(),
            signature: block.signature.clone(),
        }
    }
}

pub struct FromBlockData(pub BlockData, pub Vec<Tx>);

impl From<FromBlockData> for Block {
    fn from(value: FromBlockData) -> Self {
        Self {
            version: value.0.version,
            hash: value.0.hash,
            height: value.0.height,
            prev_hash: value.0.prev_hash,
            v_merkle_root: value.0.v_merkle_root,
            txs_merkle_root: value.0.txs_merkle_root,
            timestamp: value.0.timestamp,
            validator: value.0.validator,
            validators: value.0.validators,
            signature: value.0.signature,
            txs: value.1,
        }
    }
}
