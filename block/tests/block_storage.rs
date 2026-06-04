use block::{
    block::Block,
    block_data::BlockData,
    block_storage::{BLOCK_BY_HASH, BLOCK_CF, BlockStorage},
};
use db::DB;

#[tokio::test]
async fn save_get() {
    let file = tempfile::tempdir().unwrap();
    let db = DB::open(file.path(), vec![BLOCK_CF, BLOCK_BY_HASH])
        .await
        .unwrap();
    let storage = BlockStorage::new(db);
    let block = Block::genesis(vec![]);
    let block: BlockData = (&block).into();
    storage.save(&block).await.unwrap();

    let found = storage.get_by_height(0).await.unwrap().unwrap();
    assert_eq!(block, found);

    let found = storage
        .get_by_hash(block.hash.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(block, found);

    let found = storage.get_last().await.unwrap().unwrap();
    assert_eq!(block, found);
}
