use std::hash::Hash;

use common::{deserialize_biguint, serialize_biguint};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TxType {
    Transfer,
    Stake,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Utxo {
    pub vout: u64,
    pub tx_id: String,
    #[serde(
        deserialize_with = "deserialize_biguint",
        serialize_with = "serialize_biguint"
    )]
    pub amount: BigUint,
    pub owner: String,
    pub block_height: u64,
    pub vt: TxType,
}

impl Utxo {
    pub fn hash_bytes(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.vout.to_be_bytes());
        hasher.update(self.tx_id.as_bytes());
        hasher.update(self.amount.to_bytes_be());
        hasher.update(self.owner.as_bytes());
        hasher.update(self.block_height.to_be_bytes());
        hasher.finalize().into()
    }
}

impl PartialEq<Self> for Utxo {
    fn eq(&self, other: &Self) -> bool {
        self.tx_id == other.tx_id && self.vout == other.vout
    }
}

impl Eq for Utxo {}

impl Hash for Utxo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.tx_id.hash(state);
        self.vout.hash(state);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct TxOutput {
    pub to: String,
    #[serde(
        deserialize_with = "deserialize_biguint",
        serialize_with = "serialize_biguint"
    )]
    pub amount: BigUint,
    pub vt: TxType,
}

impl TxOutput {
    pub fn transfer(to: String, amount: BigUint) -> Self {
        Self::new(to, amount, TxType::Transfer)
    }

    pub fn stake(to: String, amount: BigUint) -> Self {
        Self::new(to, amount, TxType::Stake)
    }

    fn new(to: String, amount: BigUint, vt: TxType) -> Self {
        Self { to, amount, vt }
    }
}
