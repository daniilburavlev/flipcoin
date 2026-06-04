use std::sync::Arc;

use balance::Balance;
use block::block::Block;
use common::{AppResult, error::AppError};
use num_bigint::BigUint;
use sha2::{Digest, Sha256};
use state::State;
use wallet::Wallet;

pub struct Slot {
    pub validator: String,
    pub validators: Vec<Balance>,
    pub stake: BigUint,
}

impl Slot {
    pub fn new(validator: String, validators: Vec<Balance>, stake: BigUint) -> Self {
        Self {
            validator,
            validators,
            stake,
        }
    }

    pub fn genesis() -> Self {
        let stake = BigUint::from(1000u64);
        Self {
            validator: Block::GENESIS_VALIDATOR.to_string(),
            validators: vec![Balance::new(
                Block::GENESIS_VALIDATOR.to_string(),
                stake.clone(),
            )],
            stake: BigUint::from(1000u64),
        }
    }
}

pub struct Validator {
    wallet: Wallet,
    state: Arc<State>,
}

impl Validator {
    pub fn new(wallet: Wallet, state: &Arc<State>) -> Self {
        Self {
            wallet,
            state: Arc::clone(state),
        }
    }

    pub async fn current_validator(&self) -> AppResult<Slot> {
        let last_block = self.load_last_block().await?;
        let hash = last_block.hash;
        let holders = self.state.get_holders().await;
        get_validator(hash, holders)
    }

    pub async fn slot_validator(&self, block: &Block) -> AppResult<Slot> {
        if block.height == 0 {
            return Ok(Slot::genesis());
        }
        let hash = block.hash.clone();
        let holders = block.validators.clone();
        get_validator(hash, holders)
    }

    pub async fn create_block(&self) -> AppResult<(Option<Block>, u64, BigUint)> {
        let slot = self.current_validator().await?;
        let block = self.load_last_block().await?;
        if slot.validator != self.wallet.address_str() {
            return Ok((None, block.height + 1, slot.stake));
        }
        let txs = self.state.get_pending_txs().await;
        let block = Block::new(
            &self.wallet,
            block.hash,
            block.height + 1,
            slot.validators,
            txs,
        )?;
        self.state.add_block(&slot.validator, &block).await?;
        let height = block.height;
        Ok((Some(block), height, slot.stake))
    }

    async fn load_last_block(&self) -> AppResult<Block> {
        self.state
            .get_last_block()
            .await?
            .ok_or(AppError::BlockNotFound)
    }
}

fn get_validator(hash: String, holders: Vec<Balance>) -> AppResult<Slot> {
    let mut total = BigUint::default();
    for holder in holders.iter() {
        total += holder.amount.clone();
    }
    let roll = hash_to_bigint(hash)?;
    let target = roll % total.clone();
    let mut cumulative = BigUint::default();
    for holder in holders.iter() {
        cumulative += holder.amount.clone();
        if cumulative > target {
            return Ok(Slot::new(holder.address.clone(), holders, total));
        }
    }
    Err(AppError::InvalidValidators)
}

fn hash_to_bigint(hash: String) -> AppResult<BigUint> {
    let hash: Vec<u8> = bs58::decode(hash)
        .into_vec()
        .map_err(|_| AppError::Decode)?;
    let hash = Sha256::digest(&hash);
    Ok(BigUint::from_bytes_be(&hash))
}
