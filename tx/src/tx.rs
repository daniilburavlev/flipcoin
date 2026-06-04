use std::{cmp::Ordering, collections::HashSet};

use chrono::Utc;
use common::{AppResult, error::AppError};
use common::{deserialize_biguint, serialize_biguint};
use num_bigint::BigUint;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use utxo::{TxOutput, Utxo};
use wallet::{Wallet, verify_signature};

pub struct UtxoFrom<'a>(pub &'a Tx, pub u64);

impl<'a> From<UtxoFrom<'a>> for Vec<Utxo> {
    fn from(value: UtxoFrom<'a>) -> Self {
        let mut utxos = Vec::with_capacity(value.0.vout.len());
        for (i, out) in value.0.vout.iter().enumerate() {
            let utxo = Utxo {
                vout: i as u64,
                tx_id: value.0.tx_id.clone(),
                amount: out.amount.clone(),
                owner: out.to.clone(),
                block_height: value.1,
                vt: out.vt.clone(),
            };
            utxos.push(utxo);
        }
        utxos
    }
}

impl From<Tx> for Vec<Utxo> {
    fn from(tx: Tx) -> Self {
        let mut vout = vec![];
        let tx_id = tx.tx_id;
        for (i, v) in tx.vout.into_iter().enumerate() {
            vout.push(Utxo {
                vout: i as u64,
                tx_id: tx_id.clone(),
                amount: v.amount,
                owner: v.to,
                block_height: 0,
                vt: v.vt,
            });
        }
        vout
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct Tx {
    pub tx_id: String,
    pub vin: Vec<Utxo>,
    pub vout: Vec<TxOutput>,
    pub signatures: Vec<String>,
    #[serde(
        deserialize_with = "deserialize_biguint",
        serialize_with = "serialize_biguint"
    )]
    pub fee: BigUint,
    pub timestamp: i64,
}

impl Tx {
    pub fn new(from: &Wallet, vin: Vec<Utxo>, vout: Vec<TxOutput>) -> AppResult<Self> {
        Self::validate_vin(&vin)?;
        let fee = Self::calc_fee(&vin, &vout)?;
        let mut signatures = vec![];
        for v in vin.iter() {
            let signature = from.sign(&v.hash_bytes())?;
            signatures.push(signature);
        }
        let mut tx = Self {
            tx_id: String::default(),
            vin,
            vout,
            signatures,
            fee,
            timestamp: Utc::now().timestamp(),
        };
        tx.tx_id = tx.hash_str();
        Ok(tx)
    }

    pub fn transfer(
        from: &Wallet,
        mut utxos: Vec<Utxo>,
        to: String,
        amount: BigUint,
        fee: BigUint,
    ) -> AppResult<Self> {
        utxos.sort_by(|a, b| a.amount.cmp(&b.amount));
        let mut sum = BigUint::default();
        let mut vin = vec![];
        let mut vout = vec![TxOutput::transfer(to.clone(), amount.clone())];
        let required = amount + fee;
        for utxo in utxos {
            sum += utxo.amount.clone();
            vin.push(utxo);
            if sum > required {
                vout.push(TxOutput::transfer(from.address_str(), sum - required));
                break;
            } else if sum == required {
                break;
            }
        }
        Self::new(from, vin, vout)
    }

    pub fn validate(&self) -> AppResult<()> {
        if self.vin.is_empty() {
            return Err(AppError::EmptyTxVin);
        }
        if self.vout.is_empty() {
            return Err(AppError::EmptyTxVout);
        }
        Self::validate_vin(&self.vin)?;
        let fee = Self::calc_fee(&self.vin, &self.vout)?;
        if fee != self.fee {
            return Err(AppError::FeeNotEq);
        }
        if self.signatures.len() != self.vin.len() {
            return Err(AppError::InvalidSigAmount);
        }
        for (sig, v) in self.signatures.iter().zip(self.vin.iter()) {
            let hash = v.hash_bytes();
            if !verify_signature(&v.owner, sig, &hash)? {
                return Err(AppError::InvalidSig);
            }
        }
        Ok(())
    }

    fn validate_vin(vin: &[Utxo]) -> AppResult<()> {
        let mut existed = HashSet::<(&str, u64)>::new();
        for i in vin {
            if existed.contains(&(i.tx_id.as_str(), i.vout)) {
                return Err(AppError::DuplicateUTXO);
            }
            existed.insert((i.tx_id.as_str(), i.vout));
        }
        Ok(())
    }

    fn calc_fee(vin: &[Utxo], vout: &[TxOutput]) -> AppResult<BigUint> {
        let mut sum_in = BigUint::default();
        for v in vin {
            sum_in += v.amount.clone();
        }
        let mut sum_out = BigUint::default();
        for v in vout {
            sum_out += v.amount.clone();
        }
        if sum_in <= sum_out {
            return Err(AppError::InvalidFee);
        }
        Ok(sum_in - sum_out)
    }

    pub fn hash(&self) -> [u8; 32] {
        let mut hasher = Sha256::new();
        for input in self.vin.iter() {
            hasher.update(input.tx_id.as_bytes());
            hasher.update(input.vout.to_be_bytes());
        }
        for output in self.vout.iter() {
            hasher.update(output.to.as_bytes());
            hasher.update(output.amount.to_bytes_be());
        }
        hasher.update(self.timestamp.to_be_bytes());
        hasher.finalize().into()
    }

    pub fn hash_str(&self) -> String {
        bs58::encode(self.hash()).into_string()
    }
}

impl PartialOrd for Tx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Tx {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match other.fee.cmp(&self.fee) {
            Ordering::Equal => self.tx_id.cmp(&other.tx_id),
            o => o,
        }
    }
}
