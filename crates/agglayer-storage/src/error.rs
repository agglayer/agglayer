#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("Trying to access an unknown ColumnFamily")]
    ColumnFamilyNotFound,

    #[error("Serialization error: {0}")]
    Serialization(#[from] bincode::Error),
    #[error(r#"An unexpected error occured: {0}
        This is a critical bug that need to be report on `https://github.com/agglayer/agglayer/issues`"#)]
    Unexpected(String),
}
