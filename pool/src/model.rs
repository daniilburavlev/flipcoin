use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use tx::tx::Tx;
use utxo::Utxo;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppState {
    pub txs: HashMap<String, Tx>,
    pub utxo: HashMap<String, HashSet<Utxo>>,
    pub stakes: HashMap<String, HashSet<Utxo>>,
}

impl AppState {
    pub fn new(
        txs: HashMap<String, Tx>,
        utxo: HashMap<String, HashSet<Utxo>>,
        stakes: HashMap<String, HashSet<Utxo>>,
    ) -> Self {
        Self { txs, utxo, stakes }
    }
}
