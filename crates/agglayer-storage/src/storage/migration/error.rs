use thiserror::Error;

use crate::storage::DbError;

#[derive(Debug, Error)]
pub enum DbOpenError {
    #[error(transparent)]
    Migration(#[from] DbMigrationError),

    #[error(transparent)]
    Database(#[from] DbError),

    #[error("Unexpected database schema")]
    UnexpectedSchema,

    #[error("Default column family is not empty")]
    DefaultCfNotEmpty,

    #[error("Migration record gap detected at step {0}")]
    MigrationRecordGap(u32),

    #[error(
        "Fewer migration steps declared in the code than recorded in database (declared: \
         {declared}, recorded: {recorded}). This indicates existing migration steps were removed \
         from the code, or an older version of agglayer-node is being used, which is not allowed."
    )]
    FewerStepsDeclared { declared: u32, recorded: u32 },
}

#[derive(Debug, Error)]
#[error("Migration failed at step {step}")]
pub struct DbMigrationError {
    pub step: u32,
    #[source]
    pub details: DbMigrationErrorDetails,
}

#[derive(Debug, Error)]
pub enum DbMigrationErrorDetails {
    #[error(transparent)]
    Database(#[from] DbError),

    #[error("Writing in a read-only column family {0:?}")]
    WritingReadOnlyCf(String),

    #[error("Custom migration error")]
    Custom(#[source] eyre::Error),
}
