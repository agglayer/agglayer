//! Operator-facing storage migration runner.
//!
//! Runs `init_db` on each storage component (state, pending, debug, every
//! epoch) **in place** so an operator can converge the on-disk schema to
//! the current binary's expectations as a separate step from starting the
//! node. This command is the explicit schema-writing entry point.
//! Node startup creates missing current-schema stores, opens current stores,
//! and rejects existing storage that needs migration.
//!
//! Designed for production maintenance windows: no staging, no row-by-row
//! equality check, just executes the migrations and reports per-store
//! outcomes.
//!
//! Report rendering lives in the CLI crate (`agglayer::migrate_report`)
//! and templates live next to it; this module owns only the migration
//! data flow and exposes [`MigrateOutcome`] for the CLI to render.

use std::{
    ffi::OsString,
    fs,
    num::NonZeroU64,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

pub use crate::diagnostics::UnparsableRow;
use crate::{
    diagnostics,
    stores::{
        debug::DebugStore, pending::PendingStore, per_epoch::PerEpochStore, state::StateStore,
    },
};

/// Inputs for a migration run. The four `*_db_path` fields mirror the
/// fields of the same name on `agglayer_config::storage::StorageConfig`,
/// so a CLI can fill this in from an `agglayer.toml`. Any field set to
/// `None` is treated as "store not present in this deployment" and is
/// skipped rather than failing.
#[derive(Debug, Clone)]
pub struct MigrateOptions {
    pub state_db_path: Option<PathBuf>,
    pub pending_db_path: Option<PathBuf>,
    pub debug_db_path: Option<PathBuf>,
    pub epochs_db_path: Option<PathBuf>,
    /// Operator-supplied label (`mainnet`, `testnet`, …) used in report
    /// filenames and headings.
    pub env_label: String,
    /// When true, the epoch sweep is bypassed entirely (state, pending,
    /// debug still run).
    pub skip_epochs: bool,
    /// Cap the epoch sweep to the N most-recent epochs (highest numeric
    /// names first). Useful for operator spot-checks where the active
    /// data is at the latest epochs and the lowest-numbered ones are
    /// typically empty.
    pub latest_epochs: Option<NonZeroU64>,
}

/// Outcome of a migration run. Aggregates per-store results and a flag
/// indicating whether any store ended in a fatal state.
#[derive(Debug)]
pub struct MigrateOutcome {
    pub started_at: SystemTime,
    pub overall_duration: Duration,
    pub env_label: String,
    pub state: Option<StoreResult>,
    pub pending: Option<StoreResult>,
    pub debug: Option<StoreResult>,
    pub epochs: EpochsResult,
}

#[derive(Debug)]
pub struct StoreResult {
    pub label: String,
    pub path: PathBuf,
    pub duration: Duration,
    /// `None` on success; `Some(message)` if `init_db` returned an error.
    pub error: Option<String>,
    /// Legacy CF rows that the post-migration diagnostics scan flagged as
    /// undecodable (skipped by the migration helper). Empty when the
    /// store has no relevant legacy CF (e.g. `state`), when `init_db`
    /// failed (we did not run the scan), when the scan found no bad rows,
    /// or when the scan itself failed (see `diagnostics_error`).
    pub unparsable_rows: Vec<UnparsableRow>,
    /// Non-fatal diagnostics scan failure for this store.
    pub diagnostics_error: Option<String>,
}

#[derive(Debug, Default)]
pub struct EpochsResult {
    pub epochs_dir: Option<PathBuf>,
    pub discovered: usize,
    pub processed: usize,
    pub successful: usize,
    /// Error raised while reading the epoch root before per-epoch discovery.
    pub discovery_error: Option<String>,
    pub failed: Vec<EpochFailure>,
    pub duration: Duration,
    pub skipped_reason: Option<&'static str>,
    /// Aggregate of unparsable rows across every epoch that was
    /// successfully migrated. Each entry's `source` field carries the
    /// epoch number (e.g. `"epoch 1234"`).
    pub unparsable_rows: Vec<UnparsableRow>,
    /// Non-fatal diagnostics scan failures for epochs that migrated
    /// successfully but could not be inspected for unparsable legacy rows.
    pub diagnostics_failures: Vec<EpochFailure>,
}

#[derive(Debug)]
pub struct EpochFailure {
    pub epoch: u64,
    pub error: String,
}

struct EpochDirEntry {
    file_name: OsString,
    path: PathBuf,
    is_dir: bool,
}

impl MigrateOutcome {
    /// Number of fatal store outcomes (any non-`None` `error`, plus
    /// individual epoch failures).
    pub fn fatal_count(&self) -> usize {
        let mut n = 0;
        for r in [&self.state, &self.pending, &self.debug] {
            if r.as_ref().and_then(|s| s.error.as_ref()).is_some() {
                n += 1;
            }
        }
        n + usize::from(self.epochs.discovery_error.is_some()) + self.epochs.failed.len()
    }

    pub fn is_success(&self) -> bool {
        self.fatal_count() == 0
    }
}

/// Run the migration in place against the paths in `opts`. Per-store
/// failures are recorded in the outcome rather than propagated, so the
/// runner always produces a complete report.
///
/// Infallible by design: the runner never aborts on a single store's
/// failure, and report rendering / file IO is the CLI's responsibility.
pub fn run(opts: MigrateOptions) -> MigrateOutcome {
    let started_clock = SystemTime::now();
    let started = Instant::now();

    // This command is the explicit schema-writing entry point. Node startup uses
    // migrated-or-create open helpers and must not call these migration-capable
    // paths for existing storage.
    let state = opts.state_db_path.as_deref().map(migrate_state);

    let pending = opts.pending_db_path.as_deref().map(migrate_pending);

    let debug = opts.debug_db_path.as_deref().map(migrate_debug);

    let epochs = match opts.epochs_db_path.as_deref() {
        None => EpochsResult::default(),
        Some(_) if opts.skip_epochs => EpochsResult {
            epochs_dir: opts.epochs_db_path.clone(),
            skipped_reason: Some(
                "skipped via skip_epochs flag; pending/debug/state were still migrated",
            ),
            ..EpochsResult::default()
        },
        Some(dir) => migrate_epochs(dir, opts.latest_epochs),
    };

    MigrateOutcome {
        started_at: started_clock,
        overall_duration: started.elapsed(),
        env_label: opts.env_label,
        state,
        pending,
        debug,
        epochs,
    }
}

fn migrate_state(path: &Path) -> StoreResult {
    let started = Instant::now();
    let error = match StateStore::init_db(path) {
        Ok(_) => None,
        Err(e) => Some(format_chain(&e)),
    };
    // State DB does not have a legacy certificate CF subject to the
    // proto migration, so there is nothing to scan here.
    StoreResult {
        label: "state".into(),
        path: path.to_path_buf(),
        duration: started.elapsed(),
        error,
        unparsable_rows: Vec::new(),
        diagnostics_error: None,
    }
}

fn migrate_pending(path: &Path) -> StoreResult {
    let started = Instant::now();
    let error = match PendingStore::init_db(path) {
        Ok(_) => None,
        Err(e) => Some(format_chain(&e)),
    };
    let (unparsable_rows, diagnostics_error) = if error.is_none() {
        scan_or_warn(diagnostics::scan_unparsable_pending_rows(path), "pending")
    } else {
        (Vec::new(), None)
    };
    StoreResult {
        label: "pending".into(),
        path: path.to_path_buf(),
        duration: started.elapsed(),
        error,
        unparsable_rows,
        diagnostics_error,
    }
}

fn migrate_debug(path: &Path) -> StoreResult {
    let started = Instant::now();
    let error = match DebugStore::init_db(path) {
        Ok(_) => None,
        Err(e) => Some(format_chain(&e)),
    };
    let (unparsable_rows, diagnostics_error) = if error.is_none() {
        scan_or_warn(diagnostics::scan_unparsable_debug_rows(path), "debug")
    } else {
        (Vec::new(), None)
    };
    StoreResult {
        label: "debug".into(),
        path: path.to_path_buf(),
        duration: started.elapsed(),
        error,
        unparsable_rows,
        diagnostics_error,
    }
}

/// Run a diagnostics scan and carry any non-fatal scan failure into the
/// migration report.
fn scan_or_warn(
    result: Result<Vec<UnparsableRow>, diagnostics::ScanError>,
    store: &'static str,
) -> (Vec<UnparsableRow>, Option<String>) {
    match result {
        Ok(rows) => (rows, None),
        Err(error) => {
            let error = error.to_string();
            tracing::warn!(
                store,
                %error,
                "diagnostics scan for unparsable legacy rows failed; report will not include them",
            );
            (Vec::new(), Some(error))
        }
    }
}

fn migrate_epochs(epochs_dir: &Path, latest_epochs: Option<NonZeroU64>) -> EpochsResult {
    let started = Instant::now();
    let mut result = EpochsResult {
        epochs_dir: Some(epochs_dir.to_path_buf()),
        ..EpochsResult::default()
    };

    let entries = match fs::read_dir(epochs_dir) {
        Ok(it) => it,
        Err(error) => {
            result.discovery_error = Some(format!(
                "failed to read epoch directory at {}: {error}",
                epochs_dir.display()
            ));
            result.duration = started.elapsed();
            return result;
        }
    };

    // Discover every numeric epoch directory.
    let mut numeric_dirs = match collect_numeric_epoch_dirs(entries.map(|entry| {
        let entry = entry.map_err(|error| {
            format!(
                "failed to read epoch directory entry under {}: {error}",
                epochs_dir.display()
            )
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|error| {
            format!(
                "failed to read file type for epoch directory entry {}: {error}",
                path.display()
            )
        })?;

        Ok(EpochDirEntry {
            file_name: entry.file_name(),
            path,
            is_dir: file_type.is_dir(),
        })
    })) {
        Ok(dirs) => dirs,
        Err(error) => {
            result.discovery_error = Some(error);
            result.duration = started.elapsed();
            return result;
        }
    };
    result.discovered = numeric_dirs.len();

    // For `latest_epochs`, pick the N highest-numbered epochs (operator
    // spot-checks usually want the active recent epochs, not the
    // typically-empty lowest-numbered ones). Then re-sort ascending so
    // the actual migration runs in chronological order, which keeps
    // progress logs monotonic and matches what an operator would expect
    // from a sequential walk.
    let to_process: Vec<_> = match latest_epochs {
        Some(cap) => {
            numeric_dirs.sort_by_key(|(n, _)| std::cmp::Reverse(*n));
            let cap = usize::try_from(cap.get()).unwrap_or(usize::MAX);
            let mut taken: Vec<_> = numeric_dirs.into_iter().take(cap).collect();
            taken.sort_by_key(|(n, _)| *n);
            taken
        }
        None => {
            numeric_dirs.sort_by_key(|(n, _)| *n);
            numeric_dirs
        }
    };

    let mut successful_epoch_dirs = Vec::new();
    for (n, path) in &to_process {
        result.processed += 1;
        match PerEpochStore::<(), ()>::init_db(path) {
            Ok(_) => {
                result.successful += 1;
                successful_epoch_dirs.push((*n, path.clone()));
            }
            Err(e) => result.failed.push(EpochFailure {
                epoch: *n,
                error: format_chain(&e),
            }),
        }
        if result.processed.is_multiple_of(200) {
            eprintln!(
                "[migrate] epochs: {} / {} processed ({:.2?} elapsed)",
                result.processed,
                result.discovered,
                started.elapsed(),
            );
        }
    }

    // Keep diagnostics bounded to the same selected epochs as migration.
    let (unparsable_rows, diagnostics_failures) = collect_epoch_diagnostics(
        &successful_epoch_dirs,
        diagnostics::scan_unparsable_epoch_dir,
    );
    result.unparsable_rows = unparsable_rows;
    result.diagnostics_failures = diagnostics_failures;

    result.duration = started.elapsed();
    result
}

fn collect_epoch_diagnostics<F>(
    epoch_dirs: &[(u64, PathBuf)],
    mut scan_epoch: F,
) -> (Vec<UnparsableRow>, Vec<EpochFailure>)
where
    F: FnMut(u64, &Path) -> Result<Vec<UnparsableRow>, diagnostics::ScanError>,
{
    let mut rows = Vec::new();
    let mut failures = Vec::new();
    for (epoch, path) in epoch_dirs {
        match scan_epoch(*epoch, path) {
            Ok(mut epoch_rows) => rows.append(&mut epoch_rows),
            Err(error) => {
                let error = format_chain(&error);
                tracing::warn!(
                    epoch,
                    %error,
                    "diagnostics scan for unparsable legacy epoch rows failed",
                );
                failures.push(EpochFailure {
                    epoch: *epoch,
                    error,
                });
            }
        }
    }
    (rows, failures)
}

fn collect_numeric_epoch_dirs<I>(entries: I) -> Result<Vec<(u64, PathBuf)>, String>
where
    I: IntoIterator<Item = Result<EpochDirEntry, String>>,
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

fn format_chain(error: &dyn std::error::Error) -> String {
    let mut out = error.to_string();
    let mut next = error.source();
    while let Some(cur) = next {
        out.push_str(" -> ");
        out.push_str(&cur.to_string());
        next = cur.source();
    }
    // Squash whitespace runs so multi-line errors fit one log line.
    let mut single_line = String::with_capacity(out.len());
    let mut prev_space = false;
    for c in out.chars() {
        if c.is_whitespace() {
            if !prev_space {
                single_line.push(' ');
                prev_space = true;
            }
        } else {
            single_line.push(c);
            prev_space = false;
        }
    }
    single_line.trim().to_string()
}

#[cfg(test)]
#[cfg(feature = "testutils")]
mod tests {
    use std::num::NonZeroU64;

    use agglayer_types::{CertificateIndex, Height, NetworkId};

    use super::*;
    use crate::{
        columns::{
            epochs::certificates::CertificatePerIndexColumn,
            pending_queue::{PendingQueueColumn, PendingQueueKey},
        },
        schema::{Codec as _, ColumnSchema as _},
        stores::{pending::cf_definitions::PENDING_DB_V0, per_epoch::cf_definitions::EPOCHS_DB_V0},
        tests::TempDBDir,
    };

    fn seed_corrupt_pending_row(path: &Path, key: PendingQueueKey, value: Vec<u8>) {
        let db = crate::storage::DB::open_cf(path, PENDING_DB_V0).unwrap();
        let cf = db
            .raw_rocksdb()
            .cf_handle(PendingQueueColumn::COLUMN_FAMILY_NAME)
            .unwrap();
        db.raw_rocksdb()
            .put_cf(&cf, key.encode().unwrap(), value)
            .unwrap();
    }

    fn seed_corrupt_epoch_row(path: &Path, key: CertificateIndex, value: Vec<u8>) {
        let db = crate::storage::DB::open_cf(path, EPOCHS_DB_V0).unwrap();
        let cf = db
            .raw_rocksdb()
            .cf_handle(CertificatePerIndexColumn::COLUMN_FAMILY_NAME)
            .unwrap();
        db.raw_rocksdb()
            .put_cf(&cf, key.encode().unwrap(), value)
            .unwrap();
    }

    #[test]
    fn missing_epochs_directory_is_a_fatal_outcome() {
        let tmp = TempDBDir::new();
        let epochs_path = tmp.path.join("missing-epochs");

        let outcome = run(MigrateOptions {
            state_db_path: None,
            pending_db_path: None,
            debug_db_path: None,
            epochs_db_path: Some(epochs_path),
            env_label: "test".into(),
            skip_epochs: false,
            latest_epochs: None,
        });

        assert_eq!(outcome.fatal_count(), 1);
        assert!(!outcome.is_success());
    }

    #[test]
    fn epoch_discovery_propagates_entry_errors() {
        let entries: [Result<EpochDirEntry, String>; 1] = [Err("entry read failed".into())];

        let error = collect_numeric_epoch_dirs(entries).unwrap_err();

        assert_eq!(error, "entry read failed");
    }

    #[test]
    fn latest_epochs_limits_epoch_diagnostics_to_processed_subset() {
        let tmp = TempDBDir::new();
        let epochs_dir = tmp.path.join("epochs");
        std::fs::create_dir_all(&epochs_dir).unwrap();

        seed_corrupt_epoch_row(
            &epochs_dir.join("42"),
            CertificateIndex::new(7),
            vec![0xff_u8; 16],
        );
        std::fs::create_dir_all(epochs_dir.join("99")).unwrap();

        let outcome = run(MigrateOptions {
            state_db_path: None,
            pending_db_path: None,
            debug_db_path: None,
            epochs_db_path: Some(epochs_dir),
            env_label: "test".into(),
            skip_epochs: false,
            latest_epochs: Some(NonZeroU64::new(1).unwrap()),
        });

        assert_eq!(outcome.epochs.discovered, 2);
        assert_eq!(outcome.epochs.processed, 1);
        assert!(
            outcome.epochs.unparsable_rows.is_empty(),
            "diagnostics should not scan epochs outside the selected latest subset"
        );
    }

    #[test]
    fn epoch_diagnostics_preserve_rows_when_one_epoch_scan_fails() {
        let good_path = PathBuf::from("42");
        let bad_path = PathBuf::from("99");
        let expected_rows = vec![UnparsableRow {
            source: "epoch 42".into(),
            cf: CertificatePerIndexColumn::COLUMN_FAMILY_NAME,
            key_hex: "0000000000000007".into(),
            error: "decode error".into(),
        }];

        let (rows, failures) =
            collect_epoch_diagnostics(&[(42, good_path), (99, bad_path.clone())], |epoch, path| {
                if epoch == 42 {
                    Ok(expected_rows.clone())
                } else {
                    Err(diagnostics::ScanError::EpochDir {
                        path: path.to_path_buf(),
                        source: std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            "scan failed",
                        ),
                    })
                }
            });

        assert_eq!(rows, expected_rows);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].epoch, 99);
        assert!(failures[0].error.contains("failed to read epoch directory"));
        assert!(failures[0].error.contains(&bad_path.display().to_string()));
    }

    #[test]
    fn outcome_surfaces_unparsable_rows_per_store() {
        let tmp = TempDBDir::new();
        let pending_path = tmp.path.join("pending");

        // Two rows in the legacy CF: one valid, one corrupt.
        let valid_v0_bytes = {
            let p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src/types/certificate/tests/encoded/v0-n15-cert_h0.hex");
            hex::decode(std::fs::read_to_string(p).unwrap().trim()).unwrap()
        };
        seed_corrupt_pending_row(
            &pending_path,
            PendingQueueKey(NetworkId::new(15), Height::ZERO),
            valid_v0_bytes,
        );
        seed_corrupt_pending_row(
            &pending_path,
            PendingQueueKey(NetworkId::new(15), Height::new(1)),
            vec![0xff_u8; 16],
        );

        let outcome = run(MigrateOptions {
            state_db_path: None,
            pending_db_path: Some(pending_path),
            debug_db_path: None,
            epochs_db_path: None,
            env_label: "test".into(),
            skip_epochs: true,
            latest_epochs: None,
        });

        let pending = outcome.pending.expect("pending store ran");
        assert!(
            pending.error.is_none(),
            "init_db should succeed despite the corrupt row, got {:?}",
            pending.error
        );
        assert_eq!(
            pending.unparsable_rows.len(),
            1,
            "exactly the corrupt row should be reported, got {:?}",
            pending.unparsable_rows
        );
        let row = &pending.unparsable_rows[0];
        assert_eq!(row.source, "pending");
        assert_eq!(row.cf, PendingQueueColumn::COLUMN_FAMILY_NAME);
        let expected_key = PendingQueueKey(NetworkId::new(15), Height::new(1))
            .encode()
            .unwrap();
        assert_eq!(row.key_hex, hex::encode(expected_key));
    }

    #[test]
    fn outcome_unparsable_rows_empty_when_legacy_cf_is_clean() {
        let tmp = TempDBDir::new();
        let pending_path = tmp.path.join("pending");

        // Initialise the pending DB with V0 schema; no rows seeded.
        let _ = crate::storage::DB::open_cf(&pending_path, PENDING_DB_V0).unwrap();

        let outcome = run(MigrateOptions {
            state_db_path: None,
            pending_db_path: Some(pending_path),
            debug_db_path: None,
            epochs_db_path: None,
            env_label: "test".into(),
            skip_epochs: true,
            latest_epochs: None,
        });

        let pending = outcome.pending.expect("pending store ran");
        assert!(pending.error.is_none());
        assert!(
            pending.unparsable_rows.is_empty(),
            "no unparsable rows expected, got {:?}",
            pending.unparsable_rows
        );
    }
}
