use db::DB;
use num_bigint::BigUint;
use tempfile::tempdir;
use tx::{
    tx::Tx,
    tx_storage::{TX_BY_BLOCK, TX_BY_WALLET, TX_CF, TxStorage},
};
use utxo::{TxOutput, TxType, Utxo};
use wallet::Wallet;

#[tokio::test]
async fn save_get() {
    let dir = tempdir().unwrap();
    let db = DB::open(dir.path(), vec![TX_CF, TX_BY_BLOCK, TX_BY_WALLET])
        .await
        .unwrap();
    let storage = TxStorage::new(db);
    let wallet = Wallet::default();
    let utxo = Utxo {
        vout: 0,
        tx_id: "hash".to_string(),
        amount: BigUint::from(10u64),
        owner: wallet.address_str(),
        block_height: 0,
        vt: TxType::Transfer,
    };
    let to = Wallet::default();
    let out = TxOutput {
        to: to.address_str(),
        amount: BigUint::from(9u64),
        vt: TxType::Transfer,
    };
    let tx = Tx::new(&wallet, vec![utxo], vec![out]).unwrap();
    storage
        .save_all(std::slice::from_ref(&tx), 0)
        .await
        .unwrap();

    let found = storage
        .get_by_idx(tx.tx_id.to_string())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(tx, found);

    let found = storage.get_by_block(0).await.unwrap();
    assert_eq!(std::slice::from_ref(&tx), found);

    let found = storage.get_by_wallet(wallet.address_str()).await.unwrap();
    assert_eq!(std::slice::from_ref(&tx), found);

    let found = storage.get_by_wallet(to.address_str()).await.unwrap();
    assert_eq!(std::slice::from_ref(&tx), found);
}
