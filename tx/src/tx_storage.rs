use chrono::Utc;
use common::AppResult;
use db::DB;

pub const TX_CF: &str = "txs";
pub const TX_BY_BLOCK: &str = "txsbb";
pub const TX_BY_WALLET: &str = "txsbw";

use crate::tx::Tx;

pub struct TxStorage {
    db: DB,
}

impl TxStorage {
    pub fn new(db: DB) -> Self {
        Self { db }
    }

    pub async fn save(&self, tx: &Tx, block: u64) -> AppResult<()> {
        println!("{} {:?}", tx.hash_str(), tx);
        let now = Utc::now().timestamp();
        for from in tx.vin.iter() {
            self.db
                .put(format!("{}:{}", from.owner, now), &tx.tx_id, TX_BY_WALLET)
                .await?
        }
        for to in tx.vout.iter() {
            self.db
                .put(format!("{}:{}", to.to, now), &tx.tx_id, TX_BY_WALLET)
                .await?
        }
        self.db.put(tx.hash_str(), tx, TX_CF).await?;
        self.db
            .put(format!("{}:{}", block, now), &tx.tx_id, TX_BY_BLOCK)
            .await?;
        Ok(())
    }

    pub async fn save_all(&self, txs: &[Tx], block: u64) -> AppResult<()> {
        for tx in txs {
            self.save(tx, block).await?;
        }
        Ok(())
    }

    pub async fn get_by_idx(&self, tx_idx: String) -> AppResult<Option<Tx>> {
        self.db.get(tx_idx, TX_CF).await
    }

    pub async fn get_by_block(&self, height: u64) -> AppResult<Vec<Tx>> {
        let txs_ids: Vec<String> = self.db.prefix_all(height.to_string(), TX_BY_BLOCK).await?;
        self.db.multi_get(txs_ids, TX_CF).await
    }

    pub async fn get_by_wallet(&self, address: String) -> AppResult<Vec<Tx>> {
        let txs_ids: Vec<String> = self.db.prefix_all(address, TX_BY_WALLET).await?;
        self.db.multi_get(txs_ids, TX_CF).await
    }

    pub async fn get(&self) -> AppResult<Vec<Tx>> {
        let txs: Vec<Tx> = self.db.all(TX_CF).await?;
        for tx in &txs {
            println!("tx {} {:?}", tx.hash_str(), tx);
        }
        Ok(txs)
    }
}
