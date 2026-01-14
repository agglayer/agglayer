use thiserror::Error;

use crate::storage::DBError;

#[derive(Debug, Error)]
pub enum DBOpenError {
    #[error(transparent)]
    Migration(#[from] DBMigrationError),

    #[error(transparent)]
    Database(#[from] DBError),

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

    #[error("Columns {cf_names:?} are in the migration schema but not in the final schema")]
    UnexpectedColumnsInSchema { cf_names: Vec<String> },

    #[error("Column {cf_name:?} is in the final schema but was never added during migration")]
    MissingColumnInMigrationPlan { cf_name: String },

    #[error("Backup operation failed: {0}")]
    Backup(#[source] rocksdb::Error),

    #[error("Backup created but no backup info returned")]
    BackupInfoMissing,
}

#[derive(Debug, Error)]
#[error("Migration failed at step {step_no}")]
pub struct DBMigrationError {
    pub step_no: u32,
    #[source]
    pub details: DBMigrationErrorDetails,
}

#[derive(Debug, Error)]
pub enum DBMigrationErrorDetails {
    #[error(transparent)]
    Database(#[from] DBError),

    #[error("Writing in a read-only column family {cf_name:?}")]
    WritingReadOnlyCf { cf_name: String },

    #[error("Custom migration error")]
    Custom(#[source] eyre::Error),

    #[error("Duplicate column family in migration plan: {cf_name}")]
    DuplicateColumnInMigrationPlan { cf_name: String },

    #[error("Column family not found in current schema: {cf_name}")]
    ColumnNotFoundInSchema { cf_name: String },
}
