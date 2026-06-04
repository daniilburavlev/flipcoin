use common::AppResult;
use db::DB;

use crate::block_data::BlockData;

pub const BLOCK_CF: &str = "block";
pub const BLOCK_BY_HASH: &str = "blockbh";
pub const LAST_HASH: &str = "last";

pub struct BlockStorage {
    db: DB,
}

impl BlockStorage {
    pub fn new(db: DB) -> Self {
        Self { db }
    }

    pub async fn save(&self, block: &BlockData) -> AppResult<()> {
        self.db
            .put(block.height.to_string(), &block, BLOCK_CF)
            .await?;
        self.db
            .put(block.hash.clone(), &block.height, BLOCK_BY_HASH)
            .await?;
        self.db
            .put(LAST_HASH.to_string(), &block.height, BLOCK_BY_HASH)
            .await?;
        Ok(())
    }

    pub async fn get_by_height(&self, height: u64) -> AppResult<Option<BlockData>> {
        self.db.get(height.to_string(), BLOCK_CF).await
    }

    pub async fn get_by_hash(&self, hash: String) -> AppResult<Option<BlockData>> {
        match self.db.get(hash, BLOCK_BY_HASH).await? {
            Some(height) => self.get_by_height(height).await,
            None => Ok(None),
        }
    }

    pub async fn get_last(&self) -> AppResult<Option<BlockData>> {
        self.get_by_hash(LAST_HASH.to_string()).await
    }
}
