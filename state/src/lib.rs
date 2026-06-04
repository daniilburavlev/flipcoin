use std::collections::{HashMap, HashSet};

use balance::Balance;
use block::block::Block;
use common::{AppResult, error::AppError};
use num_bigint::BigUint;
use pool::{AppState, MemPool, MemPoolLock};
use storage::Storage;
use tx::tx::{Tx, UtxoFrom};
use utxo::Utxo;

pub struct State {
    mem_pool: MemPool,
    storage: Storage,
}

impl State {
    pub fn new(storage: Storage) -> AppResult<Self> {
        Ok(Self {
            mem_pool: MemPool::default(),
            storage,
        })
    }

    pub async fn add_genesis(&self, txs: Vec<Tx>) -> AppResult<()> {
        let mut lock = self.mem_pool.lock().await;
        move_txs(&mut lock, &txs, 0);
        self.storage.add_genesis(txs).await?;
        Ok(())
    }

    pub async fn add_tx(&self, tx: Tx) -> AppResult<()> {
        let mut lock = self.mem_pool.lock().await;
        process_tx(&mut lock, tx)?;
        Ok(())
    }

    pub async fn get_txs(&self) -> AppResult<Vec<Tx>> {
        self.storage.get_txs().await
    }

    pub async fn add_block(&self, validator: &str, block: &Block) -> AppResult<()> {
        let mut lock = self.mem_pool.lock().await;
        if validator != block.validator {
            return Err(AppError::InvalidValidators);
        }
        block.validate()?;
        let Some(last_block) = self.storage.get_last_block().await? else {
            return Err(AppError::BlockNotFound);
        };
        if last_block.hash != block.prev_hash {
            tracing::info!("{} {}", last_block.hash, block.prev_hash);
            return Err(AppError::InvalidBlockPrevHash);
        }
        let height = block.height;
        if last_block.height + 1 != height {
            return Err(AppError::InvalidBlockHeight);
        }
        let mut unexisted = vec![];
        for tx in &block.txs {
            if !lock.contains_tx(tx) {
                unexisted.push(tx);
            }
        }
        process_txs(&mut lock, unexisted)?;
        move_txs(&mut lock, &block.txs, height);
        self.storage.save_block(block).await?;
        Ok(())
    }

    pub async fn get_pending_txs(&self) -> Vec<Tx> {
        let lock = self.mem_pool.lock().await;
        lock.get_txs()
    }

    pub async fn get_holders(&self) -> Vec<Balance> {
        let lock = self.mem_pool.lock().await;
        let stakes = lock.borrow_stakes();
        calc_holders(stakes)
    }

    pub async fn get_last_block(&self) -> AppResult<Option<Block>> {
        self.storage.get_last_block().await
    }

    pub async fn get_block(&self, height: u64) -> AppResult<Option<Block>> {
        self.storage.get_block(height).await
    }

    pub async fn get_block_by_hash(&self, hash: String) -> AppResult<Option<Block>> {
        self.storage.get_block_by_hash(hash).await
    }

    pub async fn get_tx(&self, tx_id: String) -> AppResult<Option<Tx>> {
        self.storage.get_tx(tx_id).await
    }

    pub async fn get_stake(&self, address: String) -> Balance {
        let lock = self.mem_pool.lock().await;
        let mut total = BigUint::default();
        if let Some(stakes) = lock.borrow_stakes_by_addr(&address) {
            for stake in stakes {
                total += stake.amount.clone();
            }
        }
        Balance::new(address, total)
    }

    pub async fn get_state(&self) -> AppState {
        let lock = self.mem_pool.lock().await;
        lock.get_state()
    }

    pub async fn restore_pool(&self, state: AppState) {
        let mut lock = self.mem_pool.lock().await;
        lock.restore(state);
    }

    pub async fn restore_state(&self) -> AppResult<()> {
        let mut lock = self.mem_pool.lock().await;
        let mut height = 0;
        while let Some(block) = self.storage.get_block(height).await? {
            for tx in &block.txs {
                remove_utxo(&mut lock, &tx.vin);
            }
            move_txs(&mut lock, &block.txs, height);
            height += 1;
        }
        Ok(())
    }

    pub async fn get_utxos(&self, address: String) -> Vec<Utxo> {
        let lock = self.mem_pool.lock().await;
        lock.get_utxos(address)
    }
}

fn process_tx(lock: &mut MemPoolLock, tx: Tx) -> AppResult<()> {
    tx.validate()?;
    for utxo in tx.vin.iter() {
        if !lock.contains_utxo(utxo) {
            return Err(AppError::UnexistedUtxo);
        }
    }
    for utxo in tx.vin.iter() {
        lock.remove_utxo(utxo);
    }
    lock.add_tx(tx);
    Ok(())
}

fn remove_utxo(lock: &mut MemPoolLock, utxos: &[Utxo]) {
    for utxo in utxos.iter() {
        lock.remove_utxo(utxo);
    }
}

fn process_txs(lock: &mut MemPoolLock, txs: Vec<&Tx>) -> AppResult<()> {
    for tx in txs.iter() {
        tx.validate()?;
        for utxo in tx.vin.iter() {
            if !lock.contains_utxo(utxo) {
                return Err(AppError::UnexistedUtxo);
            }
        }
    }
    for tx in txs {
        tx.validate()?;
        for utxo in tx.vin.iter() {
            lock.remove_utxo(utxo);
        }
        lock.add_tx(tx.clone());
    }
    Ok(())
}

fn move_txs(lock: &mut MemPoolLock, txs: &[Tx], height: u64) {
    for tx in txs {
        lock.remove_tx(tx);
        let utxos: Vec<Utxo> = UtxoFrom(tx, height).into();
        for utxo in utxos {
            lock.add_utxo(utxo);
        }
    }
}

fn calc_holders(holders: &HashMap<String, HashSet<Utxo>>) -> Vec<Balance> {
    let mut balances = Vec::with_capacity(holders.len());
    for (address, utxo) in holders {
        let mut total = BigUint::default();
        for u in utxo {
            total += u.amount.clone();
        }
        let balance = Balance::new(address.clone(), total);
        balances.push(balance);
    }
    balances
}
