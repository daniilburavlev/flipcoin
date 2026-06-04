use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("invalid secret key")]
    InvalidSecretKey,
    #[error("invalid block hash")]
    InvalidBlockHash,
    #[error("invalid password hash")]
    InvalidPasswordHash,
    #[error("encryption")]
    Encryption,
    #[error("DB error")]
    DB,
    #[error("cannot serialize: {0}")]
    Ser(String),
    #[error("cannot deserialize: {0}")]
    Des(String),
    #[error("duplicate UTXO")]
    DuplicateUTXO,
    #[error("fee should be >= 0")]
    InvalidFee,
    #[error("recalculated fee not match")]
    FeeNotEq,
    #[error("wrong signature")]
    InvalidSig,
    #[error("not all inputs signed")]
    InvalidSigAmount,
    #[error("invalid public key")]
    InvalidPublicKey,
    #[error("utxo doesn't exist")]
    UnexistedUtxo,
    #[error("at least 1 vin should exists")]
    EmptyTxVin,
    #[error("at least 1 vout should exists")]
    EmptyTxVout,
    #[error("invalid txs merkle")]
    InvalidTxsMerkle,
    #[error("invalid block prev hash")]
    InvalidBlockPrevHash,
    #[error("invalid block height")]
    InvalidBlockHeight,
    #[error("utxo already exists")]
    UtxoAlreadyExists,
    #[error("block late")]
    BlockLate,
    #[error("block not found")]
    BlockNotFound,
    #[error("decode error")]
    Decode,
    #[error("invalid validators")]
    InvalidValidators,
    #[error("dialing error: {0}")]
    Dial(String),
    #[error("listening error: {0}")]
    Listening(String),
    #[error("network")]
    Network,
    #[error("IO: {0}")]
    IO(#[from] std::io::Error),
    #[error("scheduler")]
    Scheduler,
    #[error("error occurred")]
    Other,
    #[error("tx not found")]
    TxNotFound,
    #[error("invalid keystore")]
    InvalidKeystore,
    #[error("invalid paht")]
    InvalidPath,
    #[error("{0}")]
    Decoding(String),
    #[error("{0}")]
    AddTx(String),
}
