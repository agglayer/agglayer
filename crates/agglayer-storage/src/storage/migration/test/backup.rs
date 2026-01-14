use std::path::Path;

use agglayer_types::{Height, NetworkId};
use rocksdb::backup::{BackupEngineInfo, RestoreOptions};

use super::sample::*;
use crate::{storage::migration::Builder, tests::TempDBDir};

#[test_log::test]
fn backup_before_migrate() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();
    let db_path = &temp_dir.path;
    let backup_path = &backup_dir.path;

    // Initialize database with backup before migration
    let db = Builder::sample_builder()
        .finalize(CFS_V0)?
        .open(db_path)?
        .backup(backup_path)?
        .migrate()?;

    // Insert V0 data
    for (key, value) in &DATA_V0 {
        db.put::<NetworkInfoV0Column>(key, value)?;
    }

    // Verify V0 data
    for (key, expected_value) in &DATA_V0 {
        let value = db.get::<NetworkInfoV0Column>(key)?;
        assert_eq!(value.as_ref(), Some(expected_value));
    }

    // Verify backup directory structure was created
    assert!(backup_path.exists(), "Backup directory should exist");

    // Verify backup was created by checking backup info
    let backup_info = get_backup_info(backup_path)?;
    assert_eq!(backup_info.len(), 1, "Should have 1 backup version");
    assert_eq!(backup_info[0].backup_id, 1);

    Ok(())
}

#[test_log::test]
fn backup_contains_data() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();
    let restore_dir = TempDBDir::new();
    let db_path = &temp_dir.path;
    let backup_path = &backup_dir.path;
    let restore_path = &restore_dir.path;

    // Step 1: Initialize database and add initial data
    {
        let db = Builder::sample_builder()
            .finalize(CFS_V0)?
            .open(db_path)?
            .migrate()?;

        // Insert V0 data
        for (key, value) in &DATA_V0 {
            db.put::<NetworkInfoV0Column>(key, value)?;
        }
    }

    // Step 2: Create backup after initial data is written
    {
        let _db = Builder::sample_builder()
            .finalize(CFS_V0)?
            .open(db_path)?
            .backup(backup_path)?
            .migrate()?;
    }

    // Step 3: Add more data AFTER backup was created
    {
        let db = Builder::sample_builder()
            .finalize(CFS_V0)?
            .open(db_path)?
            .migrate()?;

        // Insert additional data that should NOT be in the backup
        let extra_key = NetworkId::new(999);
        let extra_value = NetworkInfoV0 {
            height: Height::new(9999),
            num_beans: 9999,
            num_failures: 9999,
        };
        db.put::<NetworkInfoV0Column>(&extra_key, &extra_value)?;

        // Verify the extra data is in the current database
        let value = db.get::<NetworkInfoV0Column>(&extra_key)?;
        assert_eq!(value.as_ref(), Some(&extra_value));
    }

    // Step 4: Restore backup to a new location
    {
        let mut engine = backup_engine(backup_path)?;

        std::fs::create_dir_all(restore_path)?;
        engine.restore_from_latest_backup(
            restore_path,
            restore_path,
            &RestoreOptions::default(),
        )?;
    }

    // Step 5: Open restored database and verify data
    {
        let db = Builder::sample_builder()
            .finalize(CFS_V0)?
            .open(restore_path)?
            .migrate()?;

        // Verify all original V0 data was restored
        for (key, expected_value) in &DATA_V0 {
            let value = db.get::<NetworkInfoV0Column>(key)?;
            assert_eq!(
                value.as_ref(),
                Some(expected_value),
                "Restored data should match original"
            );
        }

        // Verify the extra data added AFTER backup is NOT in the restored database
        let extra_key = NetworkId::new(999);
        let value = db.get::<NetworkInfoV0Column>(&extra_key)?;
        assert_eq!(
            value, None,
            "Data added after backup should not be in restored database"
        );
    }

    Ok(())
}

#[test_log::test]
fn multiple_backups_versioned() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();
    let db_path = &temp_dir.path;
    let backup_path = &backup_dir.path;

    // Create first backup
    {
        Builder::sample_builder()
            .finalize(CFS_V0)?
            .open(db_path)?
            .backup(backup_path)?
            .migrate()?;
    }

    // Create second backup
    {
        Builder::sample_builder()
            .finalize(CFS_V0)?
            .open(db_path)?
            .backup(backup_path)?
            .migrate()?;
    }

    // Verify multiple backup versions exist
    let backup_info = get_backup_info(backup_path)?;
    assert_eq!(backup_info.len(), 2, "Backup count does not match");

    // Verify backup IDs are sequential
    assert_eq!(backup_info[0].backup_id, 1);
    assert_eq!(backup_info[1].backup_id, 2);

    Ok(())
}

#[test_log::test]
fn backup_if_migration_needed() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let backup_dir = TempDBDir::new();
    let db_path = &temp_dir.path;
    let backup_path = &backup_dir.path;

    // First opening: migrations are needed, backup should be created
    {
        let _db = Builder::sample_builder()
            .finalize(CFS_V0)?
            .open(db_path)?
            .backup_if_migration_needed(backup_path)?
            .migrate()?;
    }

    // Get backup info after first opening
    let backup_info_after_first = get_backup_info(backup_path)?;
    assert_eq!(
        backup_info_after_first.len(),
        1,
        "Should have 1 backup after first opening"
    );

    // Second opening: no migrations needed, backup should NOT be created
    {
        let _db = Builder::sample_builder()
            .finalize(CFS_V0)?
            .open(db_path)?
            .backup_if_migration_needed(backup_path)?
            .migrate()?;
    }

    // Verify backup info is unchanged (no new backup created)
    let backup_info_after_second = get_backup_info(backup_path)?;
    assert_eq!(
        backup_info_after_first.len(),
        backup_info_after_second.len()
    );
    assert_eq!(
        backup_info_after_first[0].backup_id,
        backup_info_after_second[0].backup_id
    );
    assert_eq!(
        backup_info_after_first[0].timestamp,
        backup_info_after_second[0].timestamp
    );

    Ok(())
}

fn backup_engine(path: &Path) -> eyre::Result<rocksdb::backup::BackupEngine> {
    let env = rocksdb::Env::new()?;
    let opts = rocksdb::backup::BackupEngineOptions::new(path)?;
    Ok(rocksdb::backup::BackupEngine::open(&opts, &env)?)
}

fn get_backup_info(backup_path: &Path) -> eyre::Result<Vec<BackupEngineInfo>> {
    Ok(backup_engine(backup_path)?.get_backup_info())
}
