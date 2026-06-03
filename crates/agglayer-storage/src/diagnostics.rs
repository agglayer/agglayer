//! Operator-facing diagnostics for the legacy → proto certificate
//! migration.
//!
//! After the migration runs, the legacy column families may still contain
//! rows whose value bytes fail to decode through
//! [`crate::types::LegacyCertificate`] — typically pre-existing corruption
//! in the on-disk data that the migration helper logged-and-skipped. The
//! scan functions in this module enumerate those rows so migration reports
//! can show operators what to inspect, purge, or restore from backup.
//!
//! The scan opens each store in read-only mode and re-iterates the legacy
//! certificate CFs, attempting to decode each value. It is purely
//! diagnostic: nothing is written. Re-run as often as needed.
//!
//! These functions only inspect the legacy certificate CFs, not the
//! proto CFs. After a clean migration the legacy CFs are still on disk
//! (for rollback) but unused at runtime; their contents are exactly what
//! the scan reports on.

use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::{
    columns::{
        debug_certificates::DebugCertificatesColumn,
        epochs::certificates::CertificatePerIndexColumn, pending_queue::PendingQueueColumn,
    },
    schema::{Codec as _, ColumnSchema as _},
    storage::DB,
    stores::{
        debug::cf_definitions::DEBUG_DB_V0, pending::cf_definitions::PENDING_DB_V0,
        per_epoch::cf_definitions::EPOCHS_DB_V0,
    },
    types::LegacyCertificate,
};

/// A single legacy-CF row that could not be decoded as a
/// [`crate::types::LegacyCertificate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnparsableRow {
    /// Human-readable origin (`"pending"`, `"debug"`, `"epoch <n>"`).
    pub source: String,
    /// Raw column family name (e.g. `pending_queue`,
    /// `debug_certificates`).
    pub cf: &'static str,
    /// Hex of the raw key bytes as stored on disk.
    pub key_hex: String,
    /// `LegacyCertificate::decode` error message.
    pub error: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ScanError {
    #[error("failed to open database at {path}: {source}")]
    OpenError {
        path: PathBuf,
        #[source]
        source: crate::storage::DBError,
    },

    #[error("failed to read epoch directory at {path}: {source}")]
    EpochDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to iterate {cf} for {source}: {error}")]
    LegacyCfIterator {
        source: String,
        cf: &'static str,
        #[source]
        error: rocksdb::Error,
    },
}

struct EpochDirEntry {
    file_name: OsString,
    path: PathBuf,
    is_dir: bool,
}

/// Scan the legacy `pending_queue` CF in the pending DB and return rows
/// whose value bytes fail to decode as a `LegacyCertificate`.
pub fn scan_unparsable_pending_rows(db_path: &Path) -> Result<Vec<UnparsableRow>, ScanError> {
    let db = open_readonly_v0(db_path, PENDING_DB_V0)?;
    scan_legacy_cf(
        &db,
        PendingQueueColumn::COLUMN_FAMILY_NAME,
        "pending".into(),
    )
}

/// Scan the legacy `debug_certificates` CF in the debug DB.
pub fn scan_unparsable_debug_rows(db_path: &Path) -> Result<Vec<UnparsableRow>, ScanError> {
    let db = open_readonly_v0(db_path, DEBUG_DB_V0)?;
    scan_legacy_cf(
        &db,
        DebugCertificatesColumn::COLUMN_FAMILY_NAME,
        "debug".into(),
    )
}

/// Scan the legacy `epoch_certificate_per_index` CF in every numeric
/// epoch subdirectory under `epochs_db_path`. Non-numeric subdirectories
/// (`lost+found`, etc.) are ignored.
pub fn scan_unparsable_epoch_rows(epochs_db_path: &Path) -> Result<Vec<UnparsableRow>, ScanError> {
    let entries = std::fs::read_dir(epochs_db_path).map_err(|source| ScanError::EpochDir {
        path: epochs_db_path.to_path_buf(),
        source,
    })?;

    let mut numeric_dirs = collect_numeric_epoch_dirs(entries.map(|entry| {
        let entry = entry.map_err(|source| ScanError::EpochDir {
            path: epochs_db_path.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| ScanError::EpochDir {
            path: path.clone(),
            source,
        })?;

        Ok(EpochDirEntry {
            file_name: entry.file_name(),
            path,
            is_dir: file_type.is_dir(),
        })
    }))?;
    numeric_dirs.sort_by_key(|(n, _)| *n);

    scan_unparsable_epoch_dirs(&numeric_dirs)
}

fn collect_numeric_epoch_dirs<I>(entries: I) -> Result<Vec<(u64, PathBuf)>, ScanError>
where
    I: IntoIterator<Item = Result<EpochDirEntry, ScanError>>,
{
    let mut numeric_dirs = Vec::new();
    for entry in entries {
        let entry = entry?;
        if !entry.is_dir {
            continue;
        }
        let Some(n) = entry.file_name.to_str().and_then(|s| s.parse::<u64>().ok()) else {
            continue;
        };
        numeric_dirs.push((n, entry.path));
    }
    Ok(numeric_dirs)
}

pub(crate) fn scan_unparsable_epoch_dirs(
    numeric_dirs: &[(u64, PathBuf)],
) -> Result<Vec<UnparsableRow>, ScanError> {
    let mut out = Vec::new();
    for (n, path) in numeric_dirs {
        out.extend(scan_unparsable_epoch_dir(*n, path)?);
    }
    Ok(out)
}

pub(crate) fn scan_unparsable_epoch_dir(
    epoch: u64,
    db_path: &Path,
) -> Result<Vec<UnparsableRow>, ScanError> {
    let db = open_readonly_v0(db_path, EPOCHS_DB_V0)?;
    scan_legacy_cf(
        &db,
        CertificatePerIndexColumn::COLUMN_FAMILY_NAME,
        format!("epoch {epoch}"),
    )
}

fn open_readonly_v0(
    path: &Path,
    cfs_v0: &[crate::schema::ColumnDescriptor],
) -> Result<DB, ScanError> {
    DB::open_cf_readonly(path, cfs_v0).map_err(|source| ScanError::OpenError {
        path: path.to_path_buf(),
        source,
    })
}

/// Iterate every row in `cf_name` via the raw rocksdb cursor (no key/
/// value decoding upfront), then attempt `LegacyCertificate::decode` on
/// each value. Decode failures land in the returned vector with the raw
/// key bytes as hex; success rows are silently dropped.
fn scan_legacy_cf(
    db: &DB,
    cf_name: &'static str,
    source: String,
) -> Result<Vec<UnparsableRow>, ScanError> {
    let Some(cf) = db.raw_rocksdb().cf_handle(cf_name) else {
        // CF missing on disk: nothing to scan. Shouldn't happen when
        // we've opened with the V0 schema (which lists the legacy CFs),
        // but treat defensively.
        return Ok(Vec::new());
    };

    let mut iter = db.raw_rocksdb().raw_iterator_cf(&cf);
    iter.seek_to_first();

    let mut out = Vec::new();
    while iter.valid() {
        if let (Some(key), Some(value)) = (iter.key(), iter.value()) {
            if let Err(error) = LegacyCertificate::decode(value) {
                out.push(UnparsableRow {
                    source: source.clone(),
                    cf: cf_name,
                    key_hex: hex::encode(key),
                    error: error.to_string(),
                });
            }
        }
        iter.next();
    }

    iter.status().map_err(|error| ScanError::LegacyCfIterator {
        source,
        cf: cf_name,
        error,
    })?;

    Ok(out)
}

#[cfg(test)]
#[cfg(feature = "testutils")]
mod tests {
    use std::path::{Path, PathBuf};

    use agglayer_types::{CertificateIndex, Height, NetworkId};

    use super::*;
    use crate::{
        columns::pending_queue::PendingQueueKey,
        stores::{
            debug::{cf_definitions::DEBUG_DB, DebugStore},
            pending::{cf_definitions::PENDING_DB, PendingStore},
            per_epoch::{cf_definitions::EPOCHS_DB, PerEpochStore},
        },
        tests::TempDBDir,
    };

    fn load_v0_certificate_bytes(name: &str) -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src/types/certificate/tests/encoded")
            .join(name);
        let hex = std::fs::read_to_string(path).unwrap();
        hex::decode(hex.trim()).unwrap()
    }

    fn write_raw_pending(
        path: &Path,
        cfs: &[crate::schema::ColumnDescriptor],
        key: PendingQueueKey,
        value: Vec<u8>,
    ) {
        let db = open_raw_cf(path, cfs);
        let cf = db
            .cf_handle(PendingQueueColumn::COLUMN_FAMILY_NAME)
            .unwrap();
        db.put_cf(&cf, key.encode().unwrap(), value).unwrap();
    }

    fn write_raw_debug(
        path: &Path,
        cfs: &[crate::schema::ColumnDescriptor],
        id: agglayer_types::CertificateId,
        value: Vec<u8>,
    ) {
        let db = open_raw_cf(path, cfs);
        let cf = db
            .cf_handle(DebugCertificatesColumn::COLUMN_FAMILY_NAME)
            .unwrap();
        db.put_cf(&cf, id.encode().unwrap(), value).unwrap();
    }

    fn write_raw_epoch(
        path: &Path,
        cfs: &[crate::schema::ColumnDescriptor],
        idx: CertificateIndex,
        value: Vec<u8>,
    ) {
        let db = open_raw_cf(path, cfs);
        let cf = db
            .cf_handle(CertificatePerIndexColumn::COLUMN_FAMILY_NAME)
            .unwrap();
        db.put_cf(&cf, idx.encode().unwrap(), value).unwrap();
    }

    fn open_raw_cf(path: &Path, cfs: &[crate::schema::ColumnDescriptor]) -> rocksdb::DB {
        let mut options = rocksdb::Options::default();
        options.create_if_missing(true);
        options.create_missing_column_families(true);

        let names = rocksdb::DB::list_cf(&options, path).unwrap_or_else(|_| {
            cfs.iter()
                .map(|cf| cf.name().to_string())
                .collect::<Vec<_>>()
        });

        rocksdb::DB::open_cf(&options, path, names).unwrap()
    }

    /// Bytes that are not valid bincode v0/v1 (first byte != 0/1) and
    /// not valid proto (continuation bits never terminate).
    fn corrupt_bytes() -> Vec<u8> {
        vec![0xff_u8; 16]
    }

    #[test]
    fn pending_scan_returns_only_unparsable_rows() {
        let tmp = TempDBDir::new();
        let path = tmp.path.join("pending");

        write_raw_pending(
            &path,
            PENDING_DB_V0,
            PendingQueueKey(NetworkId::new(15), Height::ZERO),
            load_v0_certificate_bytes("v0-n15-cert_h0.hex"),
        );
        write_raw_pending(
            &path,
            PENDING_DB_V0,
            PendingQueueKey(NetworkId::new(15), Height::new(1)),
            corrupt_bytes(),
        );

        let result = scan_unparsable_pending_rows(&path).unwrap();
        assert_eq!(
            result.len(),
            1,
            "exactly the corrupt row should be reported, got {result:?}"
        );
        let row = &result[0];
        assert_eq!(row.source, "pending");
        assert_eq!(row.cf, PendingQueueColumn::COLUMN_FAMILY_NAME);
        let expected_key = PendingQueueKey(NetworkId::new(15), Height::new(1))
            .encode()
            .unwrap();
        assert_eq!(row.key_hex, hex::encode(&expected_key));
        assert!(!row.error.is_empty(), "error message should not be empty");
    }

    #[test]
    fn pending_scan_returns_empty_for_clean_legacy_cf() {
        let tmp = TempDBDir::new();
        let path = tmp.path.join("pending");

        write_raw_pending(
            &path,
            PENDING_DB_V0,
            PendingQueueKey(NetworkId::new(15), Height::ZERO),
            load_v0_certificate_bytes("v0-n15-cert_h0.hex"),
        );

        let result = scan_unparsable_pending_rows(&path).unwrap();
        assert!(
            result.is_empty(),
            "no unparsable rows expected, got {result:?}"
        );
    }

    #[test]
    fn pending_scan_works_after_proto_cf_migration() {
        let tmp = TempDBDir::new();
        let pending_path = tmp.path.join("pending");

        let pending = PendingStore::new_with_path(&pending_path).unwrap();
        drop(pending);

        write_raw_pending(
            &pending_path,
            PENDING_DB,
            PendingQueueKey(NetworkId::new(15), Height::new(1)),
            corrupt_bytes(),
        );

        let result = scan_unparsable_pending_rows(&pending_path).unwrap();
        assert_eq!(
            result.len(),
            1,
            "expected corrupt legacy row after migration"
        );
    }

    #[test]
    fn debug_scan_returns_only_unparsable_rows() {
        let tmp = TempDBDir::new();
        let path = tmp.path.join("debug");
        let id = agglayer_types::CertificateId::new(agglayer_types::Digest([1u8; 32]));

        write_raw_debug(&path, DEBUG_DB_V0, id, corrupt_bytes());

        let result = scan_unparsable_debug_rows(&path).unwrap();
        assert_eq!(result.len(), 1);
        let row = &result[0];
        assert_eq!(row.source, "debug");
        assert_eq!(row.cf, DebugCertificatesColumn::COLUMN_FAMILY_NAME);
        assert_eq!(row.key_hex, hex::encode(id.encode().unwrap()));
    }

    #[test]
    fn debug_scan_works_after_proto_cf_migration() {
        let tmp = TempDBDir::new();
        let path = tmp.path.join("debug");
        let id = agglayer_types::CertificateId::new(agglayer_types::Digest([1u8; 32]));

        let debug = DebugStore::new_with_path(&path).unwrap();
        drop(debug);

        write_raw_debug(&path, DEBUG_DB, id, corrupt_bytes());

        let result = scan_unparsable_debug_rows(&path).unwrap();
        assert_eq!(
            result.len(),
            1,
            "expected corrupt legacy row after migration"
        );
    }

    #[test]
    fn epoch_scan_walks_subdirectories_and_tags_with_epoch_number() {
        let tmp = TempDBDir::new();
        let epochs_dir = tmp.path.join("epochs");
        std::fs::create_dir_all(&epochs_dir).unwrap();

        let epoch_42 = epochs_dir.join("42");
        let epoch_99 = epochs_dir.join("99");

        // Epoch 42: one corrupt row.
        write_raw_epoch(
            &epoch_42,
            EPOCHS_DB_V0,
            CertificateIndex::new(7),
            corrupt_bytes(),
        );
        // Epoch 99: clean (one valid v0 row).
        write_raw_epoch(
            &epoch_99,
            EPOCHS_DB_V0,
            CertificateIndex::new(0),
            load_v0_certificate_bytes("v0-n15-cert_h0.hex"),
        );

        let result = scan_unparsable_epoch_rows(&epochs_dir).unwrap();
        assert_eq!(
            result.len(),
            1,
            "exactly one corrupt row across the two epochs, got {result:?}"
        );
        let row = &result[0];
        assert_eq!(row.source, "epoch 42");
        assert_eq!(row.cf, CertificatePerIndexColumn::COLUMN_FAMILY_NAME);
        assert_eq!(
            row.key_hex,
            hex::encode(CertificateIndex::new(7).encode().unwrap())
        );
    }

    #[test]
    fn epoch_scan_ignores_non_numeric_subdirectories() {
        let tmp = TempDBDir::new();
        let epochs_dir = tmp.path.join("epochs");
        std::fs::create_dir_all(&epochs_dir).unwrap();
        std::fs::create_dir_all(epochs_dir.join("lost+found")).unwrap();
        std::fs::create_dir_all(epochs_dir.join("not-an-epoch")).unwrap();

        let result = scan_unparsable_epoch_rows(&epochs_dir).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn epoch_scan_discovery_propagates_entry_errors() {
        let path = PathBuf::from("epochs");
        let entries: [Result<EpochDirEntry, ScanError>; 1] = [Err(ScanError::EpochDir {
            path: path.clone(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "entry failed"),
        })];

        let error = collect_numeric_epoch_dirs(entries).unwrap_err();

        assert!(matches!(error, ScanError::EpochDir { path: p, .. } if p == path));
    }

    #[test]
    fn epoch_scan_works_after_proto_cf_migration() {
        let tmp = TempDBDir::new();
        let epochs_dir = tmp.path.join("epochs");
        let epoch_path = epochs_dir.join("42");
        std::fs::create_dir_all(&epochs_dir).unwrap();

        let epoch_db = PerEpochStore::<(), ()>::init_db(&epoch_path).unwrap();
        drop(epoch_db);

        write_raw_epoch(
            &epoch_path,
            EPOCHS_DB,
            CertificateIndex::new(7),
            corrupt_bytes(),
        );

        let result = scan_unparsable_epoch_rows(&epochs_dir).unwrap();
        assert_eq!(
            result.len(),
            1,
            "expected corrupt legacy row after migration"
        );
    }
}
