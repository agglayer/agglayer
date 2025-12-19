use super::MigrationRecord;
use crate::schema::ColumnSchema;

const MIGRATION_RECORD_CF: &str = "migration_record_v0_cf";

pub struct MigrationRecordColumn;

impl ColumnSchema for MigrationRecordColumn {
    type Key = u32;
    type Value = MigrationRecord;

    const COLUMN_FAMILY_NAME: &'static str = MIGRATION_RECORD_CF;
}

