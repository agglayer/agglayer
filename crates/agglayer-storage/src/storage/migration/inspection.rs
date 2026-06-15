use std::{
    collections::BTreeSet,
    fmt,
    path::{Path, PathBuf},
};

use rocksdb::Options;

use super::migration_cf::MigrationRecordColumn;
use crate::{
    schema::{ColumnDescriptor, ColumnSchema},
    storage::DB,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaInspection {
    pub path: PathBuf,
    pub status: SchemaStatus,
    pub column_families: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SchemaStatus {
    Missing,
    Empty,
    Current,
    NeedsMigration,
    UnsupportedSchema,
    MigrationRecordGap(u32),
    FutureMigrationRecords { declared: u32, recorded: u32 },
    Unreadable(String),
}

impl fmt::Display for SchemaStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing => f.write_str("missing"),
            Self::Empty => f.write_str("empty"),
            Self::Current => f.write_str("current"),
            Self::NeedsMigration => f.write_str("needs migration"),
            Self::UnsupportedSchema => f.write_str("unsupported schema"),
            Self::MigrationRecordGap(step) => write!(f, "migration record gap at step {step}"),
            Self::FutureMigrationRecords { declared, recorded } => write!(
                f,
                "database has {recorded} migration records but this binary declares {declared}"
            ),
            Self::Unreadable(error) => write!(f, "unreadable: {error}"),
        }
    }
}

pub fn inspect_schema(
    path: &Path,
    cfs_v0: &[ColumnDescriptor],
    current_cfs: &[ColumnDescriptor],
    declared_steps: u32,
) -> SchemaInspection {
    let mut out = SchemaInspection {
        path: path.to_path_buf(),
        status: SchemaStatus::Missing,
        column_families: BTreeSet::new(),
    };

    match path.try_exists() {
        Ok(false) => return out,
        Ok(true) => {}
        Err(error) => {
            out.status = SchemaStatus::Unreadable(error.to_string());
            return out;
        }
    }

    if is_empty_dir(path) {
        out.status = SchemaStatus::Empty;
        return out;
    }

    let names = match rocksdb::DB::list_cf(&Options::default(), path) {
        Ok(names) => names,
        Err(error) => {
            out.status = SchemaStatus::Unreadable(error.to_string());
            return out;
        }
    };

    out.column_families = names
        .into_iter()
        .filter(|name| name != rocksdb::DEFAULT_COLUMN_FAMILY_NAME)
        .collect();

    let migration_cf = MigrationRecordColumn::COLUMN_FAMILY_NAME.to_string();
    let has_migration_records = out.column_families.contains(&migration_cf);
    let data_cfs = out
        .column_families
        .iter()
        .filter(|name| *name != &migration_cf)
        .cloned()
        .collect::<BTreeSet<_>>();
    let legacy = descriptor_names(cfs_v0);
    let current = descriptor_names(current_cfs);

    if !has_migration_records {
        out.status = if data_cfs == current {
            SchemaStatus::Current
        } else if data_cfs == legacy {
            SchemaStatus::NeedsMigration
        } else {
            SchemaStatus::UnsupportedSchema
        };
        return out;
    }

    let recorded_steps = match recorded_steps(path) {
        Ok(steps) => steps,
        Err(error) => {
            out.status = SchemaStatus::Unreadable(error.to_string());
            return out;
        }
    };

    for (expected, actual) in recorded_steps.iter().enumerate() {
        let expected = u32::try_from(expected).unwrap_or(u32::MAX);
        if *actual != expected {
            out.status = SchemaStatus::MigrationRecordGap(expected);
            return out;
        }
    }

    let recorded = u32::try_from(recorded_steps.len()).unwrap_or(u32::MAX);

    // Check for newer-version storage before rejecting unknown CF sets: a
    // database with more recorded steps than this binary declares was written
    // by a newer agglayer-node, which may have added column families this
    // binary does not recognize. Report it as a version mismatch (upgrade the
    // binary) rather than an unsupported schema or a migration requirement.
    if recorded > declared_steps {
        out.status = SchemaStatus::FutureMigrationRecords {
            declared: declared_steps,
            recorded,
        };
        return out;
    }

    // Records are valid and within the declared range, so the CF set must match
    // a schema this binary recognizes.
    if data_cfs != legacy && data_cfs != current {
        out.status = SchemaStatus::UnsupportedSchema;
        return out;
    }

    out.status = if data_cfs == current && recorded == declared_steps {
        SchemaStatus::Current
    } else {
        SchemaStatus::NeedsMigration
    };
    out
}

fn descriptor_names(cfs: &[ColumnDescriptor]) -> BTreeSet<String> {
    cfs.iter().map(|cf| cf.name().to_string()).collect()
}

fn is_empty_dir(path: &Path) -> bool {
    path.is_dir()
        && path
            .read_dir()
            .map(|mut entries| entries.next().is_none())
            .unwrap_or(false)
}

fn recorded_steps(path: &Path) -> Result<Vec<u32>, crate::storage::DBError> {
    let db = DB::open_cf_readonly(path, &[])?;
    let steps = db.keys::<MigrationRecordColumn>()?.collect();
    steps
}

#[cfg(test)]
mod tests {
    use super::{super::record::MigrationRecord, *};
    use crate::{
        columns::{metadata::MetadataColumn, network_info::NetworkInfoColumn},
        schema::{Codec as _, ColumnDescriptor},
        storage::{
            migration::{
                migration_cf::MigrationRecordColumn, open_migrated_or_create, DBOpenError,
            },
            Builder, DB,
        },
        tests::TempDBDir,
    };

    const LEGACY: &[ColumnDescriptor] = &[ColumnDescriptor::new::<MetadataColumn>()];
    const CURRENT: &[ColumnDescriptor] = &[
        ColumnDescriptor::new::<MetadataColumn>(),
        ColumnDescriptor::new::<NetworkInfoColumn>(),
    ];

    fn create_raw_db(path: &Path, names: impl IntoIterator<Item = &'static str>) {
        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);
        let descriptors = names
            .into_iter()
            .map(|name| rocksdb::ColumnFamilyDescriptor::new(name, rocksdb::Options::default()));
        let db = rocksdb::DB::open_cf_descriptors(&options, path, descriptors).unwrap();
        drop(db);
    }

    fn inspect(path: &Path) -> SchemaInspection {
        inspect_schema(path, LEGACY, CURRENT, 2)
    }

    fn open_current(path: &Path) -> Result<DB, DBOpenError> {
        Builder::open(path, LEGACY)
            .and_then(|builder| builder.ensure_cfs(CURRENT))
            .and_then(|builder| builder.finalize(CURRENT))
    }

    #[test]
    fn missing_path_is_missing() {
        let tmp = TempDBDir::new();
        let path = tmp.path.join("missing");

        assert_eq!(inspect(&path).status, SchemaStatus::Missing);
    }

    #[test]
    fn empty_path_is_empty() {
        let tmp = TempDBDir::new();

        assert_eq!(inspect(&tmp.path).status, SchemaStatus::Empty);
    }

    #[test]
    fn legacy_without_records_needs_migration() {
        let tmp = TempDBDir::new();
        create_raw_db(&tmp.path, [MetadataColumn::COLUMN_FAMILY_NAME]);

        assert_eq!(inspect(&tmp.path).status, SchemaStatus::NeedsMigration);
    }

    #[test]
    fn current_without_records_is_current() {
        let tmp = TempDBDir::new();
        create_raw_db(
            &tmp.path,
            [
                MetadataColumn::COLUMN_FAMILY_NAME,
                NetworkInfoColumn::COLUMN_FAMILY_NAME,
            ],
        );

        assert_eq!(inspect(&tmp.path).status, SchemaStatus::Current);
    }

    #[test]
    fn current_with_contiguous_records_is_current() {
        let tmp = TempDBDir::new();
        Builder::open(&tmp.path, LEGACY)
            .unwrap()
            .ensure_cfs(CURRENT)
            .unwrap()
            .finalize(CURRENT)
            .unwrap();

        assert_eq!(inspect(&tmp.path).status, SchemaStatus::Current);
    }

    #[test]
    fn open_migrated_or_create_allows_missing_storage_creation() {
        let tmp = TempDBDir::new();
        let path = tmp.path.join("missing");

        let db = open_migrated_or_create(&path, LEGACY, CURRENT, 2, open_current).unwrap();
        drop(db);

        assert_eq!(inspect(&path).status, SchemaStatus::Current);
    }

    #[test]
    fn open_migrated_or_create_rejects_existing_legacy_storage() {
        let tmp = TempDBDir::new();
        create_raw_db(&tmp.path, [MetadataColumn::COLUMN_FAMILY_NAME]);

        let error = match open_migrated_or_create(&tmp.path, LEGACY, CURRENT, 2, open_current) {
            Ok(_) => panic!("legacy storage unexpectedly opened as current"),
            Err(error) => error,
        };

        assert!(matches!(
            error,
            DBOpenError::StorageNeedsMigration {
                status: SchemaStatus::NeedsMigration,
                ..
            }
        ));
        assert_eq!(
            error.to_string(),
            format!(
                "Storage at {} needs migration (needs migration); run explicit storage migration \
                 before starting agglayer-node",
                tmp.path.display()
            )
        );
    }

    #[test]
    fn unsupported_schema_is_unsupported() {
        let tmp = TempDBDir::new();
        create_raw_db(&tmp.path, ["unexpected_cf"]);

        assert_eq!(inspect(&tmp.path).status, SchemaStatus::UnsupportedSchema);
    }

    #[test]
    fn non_rocksdb_file_is_unreadable() {
        let tmp = TempDBDir::new();
        let path = tmp.path.join("not-rocksdb");
        std::fs::write(&path, b"not a rocksdb directory").unwrap();

        assert!(matches!(inspect(&path).status, SchemaStatus::Unreadable(_)));
    }

    #[test]
    fn future_records_with_unknown_cfs_report_newer_version() {
        // An unknown CF set combined with more recorded steps than declared
        // means a newer binary added that CF in a later migration. The operator
        // should upgrade, not migrate, so this must be FutureMigrationRecords
        // (not UnsupportedSchema).
        let tmp = TempDBDir::new();
        create_raw_db(
            &tmp.path,
            ["unexpected_cf", MigrationRecordColumn::COLUMN_FAMILY_NAME],
        );
        let db = rocksdb::DB::open_cf(
            &rocksdb::Options::default(),
            &tmp.path,
            ["unexpected_cf", MigrationRecordColumn::COLUMN_FAMILY_NAME],
        )
        .unwrap();
        let cf = db
            .cf_handle(MigrationRecordColumn::COLUMN_FAMILY_NAME)
            .unwrap();
        for step in 0..3_u32 {
            db.put_cf(
                cf,
                step.encode().unwrap(),
                MigrationRecord::default().encode().unwrap(),
            )
            .unwrap();
        }
        drop(db);

        assert_eq!(
            inspect(&tmp.path).status,
            SchemaStatus::FutureMigrationRecords {
                declared: 2,
                recorded: 3,
            }
        );
    }

    #[test]
    fn migration_record_gap_is_reported() {
        let tmp = TempDBDir::new();
        let db = Builder::open(&tmp.path, LEGACY)
            .unwrap()
            .ensure_cfs(CURRENT)
            .unwrap()
            .finalize(CURRENT)
            .unwrap();
        db.delete::<MigrationRecordColumn>(&0).unwrap();
        drop(db);

        assert_eq!(
            inspect(&tmp.path).status,
            SchemaStatus::MigrationRecordGap(0)
        );
    }

    #[test]
    fn open_migrated_or_create_reports_inspection_failure_for_unreadable_storage() {
        let tmp = TempDBDir::new();
        let path = tmp.path.join("not-rocksdb");
        std::fs::write(&path, b"not a rocksdb directory").unwrap();

        let error = match open_migrated_or_create(&path, LEGACY, CURRENT, 2, open_current) {
            Ok(_) => panic!("unreadable storage must not be opened"),
            Err(error) => error,
        };

        // An I/O/permission failure while inspecting must not be reported as a
        // migration requirement: migration cannot fix it and it may be transient.
        let message = error.to_string();
        assert!(
            !message.contains("needs migration"),
            "inspection failure should not be reported as needing migration: {message}"
        );
        assert!(
            matches!(error, DBOpenError::StorageInspectionFailed { .. }),
            "expected StorageInspectionFailed, got {error:?}"
        );
    }

    #[test]
    fn open_migrated_or_create_reports_newer_version_for_future_records() {
        let tmp = TempDBDir::new();
        // Current data CFs plus the migration-record CF, but with more records
        // than this binary declares: the DB was written by a newer binary.
        create_raw_db(
            &tmp.path,
            [
                MetadataColumn::COLUMN_FAMILY_NAME,
                NetworkInfoColumn::COLUMN_FAMILY_NAME,
                MigrationRecordColumn::COLUMN_FAMILY_NAME,
            ],
        );
        let db = rocksdb::DB::open_cf(
            &rocksdb::Options::default(),
            &tmp.path,
            [
                MetadataColumn::COLUMN_FAMILY_NAME,
                NetworkInfoColumn::COLUMN_FAMILY_NAME,
                MigrationRecordColumn::COLUMN_FAMILY_NAME,
            ],
        )
        .unwrap();
        let cf = db
            .cf_handle(MigrationRecordColumn::COLUMN_FAMILY_NAME)
            .unwrap();
        for step in 0..3_u32 {
            db.put_cf(
                cf,
                step.encode().unwrap(),
                MigrationRecord::default().encode().unwrap(),
            )
            .unwrap();
        }
        drop(db);

        let error = match open_migrated_or_create(&tmp.path, LEGACY, CURRENT, 2, open_current) {
            Ok(_) => panic!("storage written by a newer binary must not be opened"),
            Err(error) => error,
        };

        // Telling the operator to "run explicit storage migration" is wrong
        // remediation here; the correct action is to upgrade the binary.
        let message = error.to_string();
        assert!(
            !message.contains("needs migration"),
            "future-version storage should not be reported as needing migration: {message}"
        );
        assert!(
            matches!(
                error,
                DBOpenError::StorageFromNewerVersion {
                    declared: 2,
                    recorded: 3,
                    ..
                }
            ),
            "expected StorageFromNewerVersion {{ declared: 2, recorded: 3 }}, got {error:?}"
        );
    }
}
