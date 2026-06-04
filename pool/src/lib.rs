use std::collections::{HashMap, HashSet};

use futures::lock::{Mutex, MutexGuard};
use tx::tx::Tx;
use utxo::{TxType, Utxo};

pub use crate::model::AppState;

mod model;

pub const TX_LIMIT: usize = 100_000;

pub struct MemPoolLock {
    txs: HashMap<String, Tx>,
    utxo: HashMap<String, HashSet<Utxo>>,
    stakes: HashMap<String, HashSet<Utxo>>,
}

impl MemPoolLock {
    fn new() -> Self {
        Self {
            txs: HashMap::new(),
            utxo: HashMap::new(),
            stakes: HashMap::new(),
        }
    }

    pub fn contains_utxo(&self, utxo: &Utxo) -> bool {
        let Some(utxos) = self.utxo.get(&utxo.owner) else {
            return false;
        };
        utxos.contains(utxo)
    }

    pub fn contains_tx(&self, tx: &Tx) -> bool {
        let Some(existed) = self.txs.get(&tx.tx_id) else {
            return false;
        };
        existed == tx
    }

    pub fn add_utxo(&mut self, utxo: Utxo) {
        let utxos = match &utxo.vt {
            TxType::Transfer => self.utxo.entry(utxo.owner.clone()).or_default(),
            TxType::Stake => self.stakes.entry(utxo.owner.clone()).or_default(),
        };
        utxos.insert(utxo);
    }

    pub fn remove_utxo(&mut self, utxo: &Utxo) {
        if let Some(utxos) = self.utxo.get_mut(&utxo.owner) {
            utxos.remove(utxo);
        }
    }

    pub fn add_tx(&mut self, tx: Tx) {
        self.txs.insert(tx.tx_id.clone(), tx);
    }

    pub fn restore(&mut self, state: AppState) {
        self.txs.extend(state.txs);
        self.utxo.extend(state.utxo);
        self.stakes.extend(state.stakes);
    }

    pub fn remove_tx(&mut self, tx: &Tx) {
        self.txs.remove(&tx.tx_id);
    }

    pub fn get_txs(&self) -> Vec<Tx> {
        let mut txs = Vec::with_capacity(self.txs.len());
        for tx in self.txs.values() {
            txs.push(tx.clone());
        }
        txs
    }

    pub fn borrow_stakes(&self) -> &HashMap<String, HashSet<Utxo>> {
        &self.stakes
    }

    pub fn borrow_stakes_by_addr(&self, address: &str) -> Option<&HashSet<Utxo>> {
        self.stakes.get(address)
    }

    pub fn get_state(&self) -> AppState {
        AppState::new(self.txs.clone(), self.utxo.clone(), self.stakes.clone())
    }

    pub fn get_utxos(&self, address: String) -> Vec<Utxo> {
        if let Some(found) = self.utxo.get(&address).cloned() {
            return found.into_iter().collect();
        }
        vec![]
    }
}

pub struct MemPool {
    lock: Mutex<MemPoolLock>,
}

impl Default for MemPool {
    fn default() -> Self {
        Self {
            lock: Mutex::new(MemPoolLock::new()),
        }
    }
}

impl MemPool {
    pub async fn lock(&self) -> MutexGuard<'_, MemPoolLock> {
        self.lock.lock().await
    }
}
