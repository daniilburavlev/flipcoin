use std::path::Path;

use block::{
    block::Block,
    block_data::{BlockData, FromBlockData},
    block_storage::BlockStorage,
};
use common::AppResult;
use db::DB;
use tx::{tx::Tx, tx_storage::TxStorage};
use utxo::Utxo;

pub struct Storage {
    block_storage: BlockStorage,
    tx_storage: TxStorage,
}

impl Storage {
    pub async fn new(path: &Path) -> AppResult<Self> {
        let db = DB::open(
            path,
            vec![
                block::block_storage::BLOCK_CF,
                block::block_storage::BLOCK_BY_HASH,
                tx::tx_storage::TX_CF,
                tx::tx_storage::TX_BY_BLOCK,
                tx::tx_storage::TX_BY_WALLET,
            ],
        )
        .await?;
        Ok(Self {
            block_storage: BlockStorage::new(db.clone()),
            tx_storage: TxStorage::new(db.clone()),
        })
    }

    pub async fn get_txs(&self) -> AppResult<Vec<Tx>> {
        self.tx_storage.get().await
    }

    pub async fn add_genesis(&self, txs: Vec<Tx>) -> AppResult<()> {
        let block = Block::genesis(txs);
        let block_data: BlockData = (&block).into();
        self.block_storage.save(&block_data).await?;
        self.tx_storage.save_all(&block.txs, block.height).await?;
        Ok(())
    }

    pub async fn get_block(&self, height: u64) -> AppResult<Option<Block>> {
        if let Some(block_data) = self.block_storage.get_by_height(height).await? {
            let txs = self.tx_storage.get_by_block(height).await?;
            tracing::info!("found txs: {:?}", txs);
            let block: Block = FromBlockData(block_data, txs).into();
            return Ok(Some(block));
        }
        Ok(None)
    }

    pub async fn get_txs_by_block(&self, height: u64) -> AppResult<Vec<Tx>> {
        self.tx_storage.get_by_block(height).await
    }

    pub async fn get_block_by_hash(&self, hash: String) -> AppResult<Option<Block>> {
        if let Some(block_data) = self.block_storage.get_by_hash(hash).await? {
            let txs = self.tx_storage.get_by_block(block_data.height).await?;
            let block: Block = FromBlockData(block_data, txs).into();
            return Ok(Some(block));
        }
        Ok(None)
    }

    pub async fn get_tx(&self, tx_idx: String) -> AppResult<Option<Tx>> {
        self.tx_storage.get_by_idx(tx_idx).await
    }

    pub async fn save_block(&self, block: &Block) -> AppResult<()> {
        let block_data: BlockData = block.into();
        self.block_storage.save(&block_data).await?;
        let txs = &block.txs;
        self.tx_storage.save_all(txs, block.height).await?;
        Ok(())
    }

    pub async fn get_last_block(&self) -> AppResult<Option<Block>> {
        if let Some(block_data) = self.block_storage.get_last().await? {
            let txs = self.tx_storage.get_by_block(block_data.height).await?;
            let block: Block = FromBlockData(block_data, txs).into();
            return Ok(Some(block));
        }
        Ok(None)
    }
}
