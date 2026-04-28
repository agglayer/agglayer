use super::sample::*;
use crate::{storage::migration::Builder, tests::TempDBDir};

#[rstest::rstest]
#[test_log::test]
fn partial_migration(
    // fail steps are:
    // 0..=4: just before adding item at index N
    // 5: after adding all items but before the migration step is recorded
    #[values(0, 1, 2, 3, 4, 5)] fail_step_no: u32,

    // what kind of failure to issue
    #[values("panic", "return(simulated_fail)")] fail_mode: &str,
) -> Result<(), eyre::Error> {
    let scenario = fail::FailScenario::setup();

    let temp_dir = TempDBDir::new();
    let db_path = &temp_dir.path;

    // Phase 1: Initialize V0 database with data
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

    // Phase 2: Attempt migration, failing after the set number of iterations.
    {
        fail::cfg(
            "sample_migrate",
            &format!("{fail_step_no}*off->{fail_mode}"),
        )
        .expect("Failed to configure failpoint");

        let result = std::panic::catch_unwind(|| {
            Builder::sample_builder()
                .sample_migrate_v0_v1()?
                .finalize(CFS_V1)?
                .open(db_path)?
                .migrate()
        });

        match result {
            Err(_) => (),
            Ok(Err(_)) => (),
            _ => panic!("Expected migration to fail"),
        }
    }

    // Phase 3: Disable failpoint and retry migration - should complete successfully
    {
        fail::cfg("sample_migrate", "off").expect("Failed to disable failpoint");

        let db = Builder::sample_builder()
            .sample_migrate_v0_v1()?
            .finalize(CFS_V1)?
            .open(db_path)?
            .migrate()?;

        // Verify all V1 data is correct (should be the migrated V0 data)
        for (key, expected_value) in &DATA_V1[..DATA_V1_NEW_START] {
            let value = db.get::<NetworkInfoV1Column>(key)?;
            assert_eq!(
                value.as_ref(),
                Some(expected_value),
                "Data mismatch for key {key:?}",
            );
        }
    }

    scenario.teardown();

    Ok(())
}
