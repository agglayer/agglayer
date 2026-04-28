use super::sample::*;
use crate::{storage::migration::Builder, tests::TempDBDir};

#[test_log::test]
fn sample_migration() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let db_path = &temp_dir.path;

    // Step 1: Initialize database with V0 schema and add V0 data
    {
        let db = Builder::open_sample(db_path)?.finalize(CFS_V0)?;

        // Insert V0 data
        for (key, value) in &DATA_V0 {
            db.put::<NetworkInfoV0Column>(key, value)?;
        }

        // Verify V0 data
        for (key, expected_value) in &DATA_V0 {
            let value = db.get::<NetworkInfoV0Column>(key)?;
            assert_eq!(value.as_ref(), Some(expected_value));
        }
    }

    // Step 2: Open database and migrate to V1, then add V1 data
    {
        let db = Builder::open_sample(db_path)?
            .sample_migrate_v0_v1()?
            .finalize(CFS_V1)?;

        // Insert new V1 data
        for (key, value) in &DATA_V1[DATA_V1_NEW_START..] {
            db.put::<NetworkInfoV1Column>(key, value)?;
        }

        // Verify all V1 data (migrated + new)
        for (key, expected_value) in &DATA_V1 {
            let value = db.get::<NetworkInfoV1Column>(key)?;
            assert_eq!(value.as_ref(), Some(expected_value));
        }
    }

    // Step 3: Open database and migrate to V2, then add V2 data
    {
        let db = Builder::open_sample(db_path)?
            .sample_migrate_v0_v1()?
            .sample_migrate_v1_v2()?
            .finalize(CFS_V2)?;

        // Insert new V2 cool data
        for (key, value) in &DATA_V2_COOL[DATA_V2_COOL_NEW_START..] {
            db.put::<NetworkInfoV2CoolColumn>(key, value)?;
        }

        // Insert new V2 uncool data
        for (key, value) in &DATA_V2_UNCOOL[DATA_V2_UNCOOL_NEW_START..] {
            db.put::<NetworkInfoV2UncoolColumn>(key, value)?;
        }

        // Verify all V2 cool data
        for (key, expected_value) in &DATA_V2_COOL {
            let value = db.get::<NetworkInfoV2CoolColumn>(key)?;
            assert_eq!(value.as_ref(), Some(expected_value));
        }

        // Verify all V2 uncool data
        for (key, expected_value) in &DATA_V2_UNCOOL {
            let value = db.get::<NetworkInfoV2UncoolColumn>(key)?;
            assert_eq!(value.as_ref(), Some(expected_value));
        }
    }

    Ok(())
}
