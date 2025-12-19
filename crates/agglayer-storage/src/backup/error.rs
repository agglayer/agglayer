#[derive(Debug, thiserror::Error)]
pub enum BackupError {
    #[error("Unable to send backup request")]
    UnableToSendBackupRequest,

    #[error("RocksDB error: {0}")]
    RocksDB(#[from] rocksdb::Error),

    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),
}
