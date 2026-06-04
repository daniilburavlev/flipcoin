use block::block::Block;
use chain::{Chain, model::PeerInfo};
use common::error::AppError;
use rpc::ApiResult;
use tx::tx::Tx;
use utxo::Utxo;

pub struct AppState {
    chain: Chain,
}

impl AppState {
    pub fn new(chain: Chain) -> Self {
        Self { chain }
    }

    pub(crate) async fn add_tx(&self, tx: Tx) -> ApiResult<()> {
        self.chain.add_tx(tx.clone()).await?;
        Ok(())
    }

    pub(crate) async fn get_txs(&self) -> ApiResult<Vec<Tx>> {
        Ok(self.chain.get_txs().await?)
    }

    pub(crate) async fn get_block(&self, height: u64) -> ApiResult<Block> {
        if let Some(block) = self.chain.get_block(height).await? {
            Ok(block)
        } else {
            Err(AppError::BlockNotFound.into())
        }
    }

    pub(crate) async fn get_block_by_hash(&self, hash: String) -> ApiResult<Block> {
        if let Some(block) = self.chain.get_block_by_hash(hash).await? {
            Ok(block)
        } else {
            Err(AppError::BlockNotFound.into())
        }
    }

    pub(crate) async fn get_tx(&self, tx_id: String) -> ApiResult<Tx> {
        if let Some(tx) = self.chain.get_tx(tx_id).await? {
            Ok(tx)
        } else {
            Err(AppError::TxNotFound.into())
        }
    }

    pub fn get_info(&self) -> PeerInfo {
        self.chain.get_info()
    }

    pub async fn get_state(&self) -> pool::AppState {
        self.chain.get_state().await
    }

    pub async fn get_utxos(&self, address: String) -> Vec<Utxo> {
        self.chain.get_utxos(address).await
    }
}
