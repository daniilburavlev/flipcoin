use common::AppResult;
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use wallet::Wallet;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Vote {
    pub hash: String,
    pub voter: String,
    pub block_height: u64,
    pub block_hash: String,
    pub stake: BigUint,
    pub validator: String,
    pub signature: String,
}

impl Vote {
    pub fn init(block_height: u64, block_hash: String, validator: String, stake: BigUint) -> Self {
        Self {
            hash: String::default(),
            voter: String::default(),
            block_height,
            block_hash,
            validator,
            stake,
            signature: String::default(),
        }
    }

    pub fn approve(mut self, wallet: &Wallet) -> AppResult<Self> {
        self.voter = wallet.address_str();
        self.hash = self.hash_str();
        self.signature = wallet.sign(&self.hash())?;
        Ok(self)
    }

    fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(self.voter.as_bytes());
        hasher.update(self.block_height.to_be_bytes());
        hasher.update(self.block_hash.as_bytes());
        hasher.update(self.validator.as_bytes());
        hasher.update(self.stake.to_bytes_be());
        hasher.finalize().into()
    }

    pub fn hash_str(&self) -> String {
        bs58::encode(self.hash()).into_string()
    }
}
