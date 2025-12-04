use rocksdb::DB as RocksDB;

use crate::storage::migration::migration_cf::MigrationRecordColumn;
use crate::tests::TempDBDir;

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
            Err(DBOpenError::DefaultCFNotEmpty) => (),
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
        assert_eq!(migration_record_count, 4);

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
