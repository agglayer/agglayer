#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Trying to access an unknown ColumnFamily")]
    ColumnFamilyNotFound,

    #[error(r#"Serialization error: {0}
        This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    Serialization(#[from] bincode::Error),

    #[error(r#"An unexpected error occurred: {0}
        This is a critical bug that needs to be reported on `https://github.com/agglayer/agglayer/issues`"#)]
    Unexpected(String),

    #[error("No certificate found")]
    NoCertificate,

    #[error("No proof found")]
    NoProof,
}
