use super::sample::*;
use crate::{
    storage::{Builder, DBOpenError},
    tests::TempDBDir,
};

#[test_log::test]
fn fewer_steps_declared_than_recorded() -> Result<(), eyre::Error> {
    let temp_dir = TempDBDir::new();
    let db_path = &temp_dir.path;

    // Phase 1: Initialize database and run both migrations (v0->v1 and v1->v2)
    {
        Builder::sample_builder()
            .sample_migrate_v0_v1()?
            .sample_migrate_v1_v2()?
            .finalize(CFS_V2)?
            .open(db_path)?
            .migrate()?;
    }

    // Phase 2: Try to open with only first migration - should fail with
    // FewerStepsDeclared
    {
        let result = Builder::sample_builder()
            .sample_migrate_v0_v1()?
            .finalize(CFS_V1)
            .and_then(|plan| plan.open(db_path));

        match result {
            Err(DBOpenError::FewerStepsDeclared { declared, recorded }) => {
                assert!(
                    declared < recorded,
                    "Declared steps should be less than recorded steps"
                );
            }
            Err(other) => {
                panic!("Expected FewerStepsDeclared error, got: {other:?}")
            }
            Ok(_) => {
                panic!("Expected FewerStepsDeclared error, but open succeeded")
            }
        }
    }

    Ok(())
}
