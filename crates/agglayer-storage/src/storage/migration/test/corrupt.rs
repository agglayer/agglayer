use agglayer_types::NetworkId;
use rocksdb::DB as RocksDB;

use crate::{
    columns::ColumnSchema, storage::migration::migration_cf::MigrationRecordColumn,
    tests::TempDBDir,
};

use super::super::{Builder, DBOpenError};
use super::sample::*;

#[test_log::test]
fn default_cf_not_empty() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let db_path = &temp_dir.path;

    // Phase 1: Create a raw RocksDB database with data in the default column family
    {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        let db = RocksDB::open(&opts, db_path)?;
        db.put(b"some_key", b"some_value")?;
        drop(db);
    }

    // Phase 2: Try to open with Builder - should fail with DefaultCFNotEmpty
    {
        let result = Builder::open_sample(db_path);

        match result {
            Err(DBOpenError::DefaultCfNotEmpty) => (),
            Err(other) => panic!("Expected DefaultCFNotEmpty error, got: {other:?}"),
            Ok(_) => panic!("Expected DefaultCFNotEmpty error, but open succeeded"),
        }
    }

    Ok(())
}

#[test_log::test]
fn migration_record_gap() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let db_path = &temp_dir.path;

    // Phase 1: Create a database with a corrupted migration record.
    {
        let db = Builder::open_sample(db_path)?
            .sample_migrate_v0_v1()?
            .finalize(CFS_V1)?;

        // Verify migration record contains 4 steps at the end of phase 1
        let migration_record_count = db.keys::<MigrationRecordColumn>()?.count();
        assert_eq!(migration_record_count, 3);

        // Delete the migration record at step 1 to create a gap
        db.delete::<MigrationRecordColumn>(&1_u32)?;
    }

    // Phase 2: Try to open - should fail with MigrationRecordGap
    {
        let result = Builder::open_sample(db_path);

        match result {
            Err(DBOpenError::MigrationRecordGap(step)) => {
                assert_eq!(step, 1, "Gap should be detected at step 1");
            }
            other => panic!("Expected MigrationRecordGap error, got: {other:?}"),
        }
    }

    Ok(())
}

#[test_log::test]
fn unexpected_schema() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let db_path = &temp_dir.path;

    // Phase 1: Create a database with an unexpected column family using raw RocksDB
    {
        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let db = RocksDB::open_cf(&opts, db_path, ["unexpected_cf"])?;
        drop(db);
    }

    // Phase 2: Try to open with Builder - should fail with UnexpectedSchema
    {
        let result = Builder::open_sample(db_path);

        match result {
            Err(DBOpenError::UnexpectedSchema) => (),
            other => panic!("Expected UnexpectedSchema error, got: {other:?}"),
        }
    }

    Ok(())
}

#[test_log::test]
fn write_to_readonly_cf_during_migration() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let db_path = &temp_dir.path;

    // Phase 1: Initialize database with V0 schema and add some data
    {
        let db = Builder::open_sample(db_path)?.finalize(CFS_V0)?;
        db.put::<NetworkInfoV0Column>(&NetworkId::new(42), &DATA_V0[0].1)?;
    }

    // Phase 2: Try to migrate but write to the old (read-only) CF
    {
        let result = Builder::open_sample(db_path)?
            .add_cfs([NetworkInfoV1Column::COLUMN_FAMILY_NAME], |db| {
                // This should FAIL - trying to write to old CF during migration
                let v0_value = &DATA_V0[1].1;
                db.put::<NetworkInfoV0Column>(&NetworkId::new(42), v0_value)
            });

        match result {
            Err(DBOpenError::Migration(migration_err)) => {
                // Check that the error is WritingReadOnlyCf
                match migration_err.details {
                    crate::storage::DBMigrationErrorDetails::WritingReadOnlyCf(cf) => {
                        assert_eq!(NetworkInfoV0Column::COLUMN_FAMILY_NAME, cf);
                    }
                    err => panic!("Unexpected error {err:?}"),
                }
            }
            Err(err) => panic!("Unexpected error {err:?}"),
            Ok(_) => panic!("Expected error"),
        }
    }

    Ok(())
}
