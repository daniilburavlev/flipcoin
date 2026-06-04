use common::{deserialize_biguint, serialize_biguint};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Balance {
    pub address: String,
    #[serde(
        deserialize_with = "deserialize_biguint",
        serialize_with = "serialize_biguint"
    )]
    pub amount: BigUint,
}

impl Balance {
    pub fn new(address: String, amount: BigUint) -> Self {
        Self { address, amount }
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.address.as_bytes());
        hasher.update(self.amount.to_bytes_be());
        hasher.finalize().into()
    }
}
