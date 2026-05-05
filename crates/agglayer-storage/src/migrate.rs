//! Operator-facing storage migration runner.
//!
//! Runs `init_db` on each storage component (state, pending, debug, every
//! epoch) **in place** so an operator can converge the on-disk schema to
//! the current binary's expectations as a separate step from starting the
//! node. The same code path also runs implicitly when the live binary
//! starts (each store's `init_db` is invoked during normal startup), so
//! this command is an opt-in optimisation: pre-paying the migration cost
//! during a maintenance window instead of hitting it at first read.
//!
//! Designed for production maintenance windows: no staging, no row-by-row
//! equality check, just executes the migrations and reports per-store
//! outcomes.
//!
//! Report rendering lives in the CLI crate (`agglayer::migrate_report`)
//! and templates live next to it; this module owns only the migration
//! data flow and exposes [`MigrateOutcome`] for the CLI to render.

use std::{
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

use crate::stores::{
    debug::DebugStore, pending::PendingStore, per_epoch::PerEpochStore, state::StateStore,
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
    pub latest_epochs: Option<u64>,
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
}

#[derive(Debug, Default)]
pub struct EpochsResult {
    pub epochs_dir: Option<PathBuf>,
    pub discovered: usize,
    pub processed: usize,
    pub successful: usize,
    pub failed: Vec<EpochFailure>,
    pub duration: Duration,
    pub skipped_reason: Option<&'static str>,
}

#[derive(Debug)]
pub struct EpochFailure {
    pub epoch: u64,
    pub error: String,
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
        n + self.epochs.failed.len()
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
    StoreResult {
        label: "state".into(),
        path: path.to_path_buf(),
        duration: started.elapsed(),
        error,
    }
}

fn migrate_pending(path: &Path) -> StoreResult {
    let started = Instant::now();
    let error = match PendingStore::init_db(path) {
        Ok(_) => None,
        Err(e) => Some(format_chain(&e)),
    };
    StoreResult {
        label: "pending".into(),
        path: path.to_path_buf(),
        duration: started.elapsed(),
        error,
    }
}

fn migrate_debug(path: &Path) -> StoreResult {
    let started = Instant::now();
    let error = match DebugStore::init_db(path) {
        Ok(_) => None,
        Err(e) => Some(format_chain(&e)),
    };
    StoreResult {
        label: "debug".into(),
        path: path.to_path_buf(),
        duration: started.elapsed(),
        error,
    }
}

fn migrate_epochs(epochs_dir: &Path, latest_epochs: Option<u64>) -> EpochsResult {
    let started = Instant::now();
    let mut result = EpochsResult {
        epochs_dir: Some(epochs_dir.to_path_buf()),
        ..EpochsResult::default()
    };

    let entries = match fs::read_dir(epochs_dir) {
        Ok(it) => it,
        Err(_) => {
            result.duration = started.elapsed();
            return result;
        }
    };

    // Discover every numeric epoch directory.
    let mut numeric_dirs: Vec<(u64, PathBuf)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().map(|t| t.is_dir()).unwrap_or(false))
        .filter_map(|e| {
            let name = e.file_name();
            let n = name.to_str().and_then(|s| s.parse::<u64>().ok())?;
            Some((n, e.path()))
        })
        .collect();
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
            let mut taken: Vec<_> = numeric_dirs.into_iter().take(cap as usize).collect();
            taken.sort_by_key(|(n, _)| *n);
            taken
        }
        None => {
            numeric_dirs.sort_by_key(|(n, _)| *n);
            numeric_dirs
        }
    };

    for (n, path) in to_process {
        result.processed += 1;
        match PerEpochStore::<(), ()>::init_db(&path) {
            Ok(_) => result.successful += 1,
            Err(e) => result.failed.push(EpochFailure {
                epoch: n,
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

    result.duration = started.elapsed();
    result
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
