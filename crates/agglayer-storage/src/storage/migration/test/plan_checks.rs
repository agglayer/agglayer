use super::sample::*;
use crate::storage::{Builder, DBMigrationError, DBMigrationErrorDetails, DBOpenError};

#[test]
fn duplicate_column_in_migration_plan() {
    let error = Builder::sample_builder()
        .add_cfs(CFS_V1, |_db| Ok(()))
        .unwrap()
        .add_cfs(CFS_V1, |_db| Ok(()))
        .unwrap_err();

    assert!(
        matches!(
            error,
            DBOpenError::Migration(DBMigrationError {
                step_no: 2,
                details: DBMigrationErrorDetails::DuplicateColumnInMigrationPlan { .. }
            })
        ),
        "Unexpected error {error:?}"
    );
}

#[test]
fn column_not_found_in_schema() {
    let error = Builder::sample_builder().drop_cfs(CFS_V1).unwrap_err();

    assert!(
        matches!(
            error,
            DBOpenError::Migration(DBMigrationError {
                step_no: 1,
                details: DBMigrationErrorDetails::ColumnNotFoundInSchema { .. }
            })
        ),
        "Unexpected error {error:?}"
    );
}

#[test]
fn missing_column_in_migration_plan() {
    let error = Builder::sample_builder().finalize(CFS_V1).unwrap_err();

    assert!(
        matches!(error, DBOpenError::MissingColumnInMigrationPlan { .. }),
        "Unexpected error {error:?}"
    );
}

#[test]
fn unexpected_columns_in_schema() {
    let error = Builder::sample_builder().finalize(&[]).unwrap_err();

    assert!(
        matches!(
            error,
            DBOpenError::UnexpectedColumnsInSchema { ref cf_names }
            if cf_names.len() == 1 && cf_names[0] == "network_info_v0"
        ),
        "Unexpected error {error:?}"
    );
}
