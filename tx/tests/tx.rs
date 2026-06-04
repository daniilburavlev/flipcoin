use num_bigint::BigUint;
use tx::tx::Tx;
use utxo::{TxOutput, TxType, Utxo};
use wallet::Wallet;

#[test]
fn transfer() {
    let from = Wallet::default();
    let utxos = vec![
        Utxo {
            vout: 0,
            tx_id: "hash".to_string(),
            amount: BigUint::from(100u64),
            owner: from.address_str(),
            block_height: 0,
            vt: TxType::Transfer,
        },
        Utxo {
            vout: 1,
            tx_id: "hash".to_string(),
            amount: BigUint::from(100u64),
            owner: from.address_str(),
            block_height: 1,
            vt: TxType::Transfer,
        },
    ];

    let tx = Tx::transfer(
        &from,
        utxos,
        "to".to_string(),
        BigUint::from(100u64),
        BigUint::from(20u64),
    )
    .unwrap();
    assert_eq!(
        tx.vout,
        vec![
            TxOutput::transfer("to".to_string(), BigUint::from(100u64)),
            TxOutput::transfer(from.address_str(), BigUint::from(80u64))
        ]
    );
}
