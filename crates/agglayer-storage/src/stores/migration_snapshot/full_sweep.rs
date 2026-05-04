//! Full data-directory sweep: pending + debug + every epoch, in one
//! invocation, producing a single comprehensive HTML report.
//!
//! Point `AGGLAYER_MIGRATION_SNAPSHOT_DATA_DIR` at the parent storage
//! directory (the one that contains `pending/`, `debug/`, `epochs/`). The
//! test discovers each sub-DB, runs the proto migration on a staged copy,
//! validates every legacy row against the proto CF, and aggregates the
//! results into one HTML file.
//!
//! Optional knobs:
//!
//! * `AGGLAYER_MIGRATION_SNAPSHOT_REPORT_DIR=/path` – where to write the HTML
//!   (default: system temp dir). Filename is
//!   `agglayer-migration-snapshot-report-full.html`.
//! * `AGGLAYER_MIGRATION_SNAPSHOT_EPOCH_LIMIT=N` – validate only the first N
//!   epochs (numerically). Useful for spot-checks against very large data dirs
//!   without paying the full sweep cost.
//!
//! ## Run
//!
//! ```sh
//! AGGLAYER_MIGRATION_SNAPSHOT_DATA_DIR=$HOME/Downloads/data \
//! AGGLAYER_MIGRATION_SNAPSHOT_REPORT_DIR=$HOME/Downloads/migration-reports \
//! cargo nextest run -p agglayer-storage --run-ignored only --no-capture \
//!     stores::migration_snapshot::full_sweep
//! ```
//!
//! Expected layout:
//!
//! ```text
//! <data_dir>/
//!     pending/
//!     debug/
//!     epochs/
//!         0/
//!         1/
//!         ...
//! ```
//!
//! Validation per sub-DB is **non-panicking**: a single bad epoch records
//! a `fatal_error` in the report and the sweep continues. The test only
//! panics at the end if any sub-DB ended in a fatal state, after the HTML
//! report has been written.

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime},
};

use agglayer_types::{Certificate, CertificateHeader, CertificateId, CertificateStatus};

use crate::{
    columns::{
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{CertificatePerNetworkColumn, Key as CertPerNetworkKey},
        debug_certificates::{DebugCertificatesColumn, DebugCertificatesProtoColumn},
        epochs::certificates::{CertificatePerIndexColumn, CertificatePerIndexProtoColumn},
        pending_queue::{PendingQueueColumn, PendingQueueKey, PendingQueueProtoColumn},
    },
    schema::{ColumnDescriptor, ColumnSchema},
    storage::{DBError, DBOpenError, DB},
    stores::{
        debug::DebugStore,
        migration_snapshot::{
            copy_dir_recursive, format_duration, format_error_chain, format_system_time,
            html_escape, SkipAnalysis, SkippedRow, REPORT_DIR_ENV,
        },
        pending::PendingStore,
        per_epoch::PerEpochStore,
        state::StateStore,
    },
    tests::TempDBDir,
    types::LegacyCertificate,
};

const DATA_DIR_ENV: &str = "AGGLAYER_MIGRATION_SNAPSHOT_DATA_DIR";
const EPOCH_LIMIT_ENV: &str = "AGGLAYER_MIGRATION_SNAPSHOT_EPOCH_LIMIT";
const SKIP_EPOCHS_ENV: &str = "AGGLAYER_MIGRATION_SNAPSHOT_SKIP_EPOCHS";
const ENV_LABEL_ENV: &str = "AGGLAYER_MIGRATION_SNAPSHOT_ENV_LABEL";

/// Parse a boolean-ish env-var value: empty/unset → false, anything in
/// `{1, true, yes, on}` (case-insensitive) → true, anything else → false.
fn env_flag(name: &str) -> bool {
    match std::env::var(name) {
        Ok(v) => matches!(
            v.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ),
        Err(_) => false,
    }
}

/// How many skipped rows to embed in the HTML per store (the rest are
/// summarized as "... and N more"). Keeps reports readable on databases
/// with many corrupt rows.
const MAX_SKIPPED_ROWS_PER_STORE: usize = 200;

/// How many problem epochs to embed in the HTML (epochs with skips or
/// fatal errors). Clean epochs are aggregated into a single counter.
const MAX_PROBLEM_EPOCHS: usize = 500;

// ---------------------------------------------------------------------------
// Report types
// ---------------------------------------------------------------------------

#[derive(Default)]
struct StoreOutcome {
    label: String,
    source_path: PathBuf,
    cfs_before: Vec<String>,
    cfs_after: Vec<String>,
    validated: usize,
    skipped: Vec<SkippedRow>,
    duration: Duration,
    fatal_error: Option<String>,
}

impl StoreOutcome {
    fn new(label: impl Into<String>, source: &Path) -> Self {
        Self {
            label: label.into(),
            source_path: source.to_path_buf(),
            ..Self::default()
        }
    }

    fn classify(&self) -> Status {
        if self.fatal_error.is_some() {
            Status::Failed
        } else if !self.skipped.is_empty() {
            Status::Skips
        } else {
            Status::Ok
        }
    }
}

#[derive(Default)]
struct EpochsOutcome {
    epochs_dir: Option<PathBuf>,
    discovered: usize,
    processed: usize,
    skipped_due_to_limit: usize,
    successful: usize,
    with_skips: usize,
    failed: usize,
    total_validated: usize,
    total_skipped: usize,
    /// Per-epoch outcomes, but only those with skips or fatal errors get
    /// kept verbatim; clean epochs are aggregated into `successful`.
    problem_epochs: Vec<(u64, StoreOutcome)>,
    extra_problem_epochs: usize,
    duration: Duration,
}

impl EpochsOutcome {
    fn classify(&self) -> Status {
        if self.failed > 0 {
            Status::Failed
        } else if self.with_skips > 0 {
            Status::Skips
        } else {
            Status::Ok
        }
    }
}

struct FullSweepReport {
    started_at: SystemTime,
    data_dir: PathBuf,
    overall_duration: Duration,
    pending: Option<StoreOutcome>,
    debug: Option<StoreOutcome>,
    epochs: EpochsOutcome,
    state: Option<StateDbInfo>,
    epoch_limit: Option<u64>,
    /// Operator-supplied environment label (`mainnet`, `testnet`,
    /// `devnet`, …) used to disambiguate concurrent reports. Defaults to
    /// the data directory's basename when the env var is unset.
    env_label: String,
    /// `Some(reason)` when the epoch sweep was deliberately skipped by
    /// configuration (e.g. `AGGLAYER_MIGRATION_SNAPSHOT_SKIP_EPOCHS=1`).
    /// The HTML report shows this in the Epoch DBs section instead of the
    /// "missing subdir" or aggregate-stats cards.
    epochs_skipped_reason: Option<&'static str>,
}

/// State DB outcome captured during a sweep. Two independent checks:
///
/// * Read-only open with the analyzer's CF subset (needed for skip enrichment
///   via `certificate_per_network_cf` / `certificate_header_cf`).
/// * Full `StateStore::init_db` against a staged copy: confirms the migration
///   framework can bring the snapshot up to the current schema. This is the
///   check that catches latent upgrade bugs like missing `disabled_networks_cf`
///   on legacy state DBs.
#[derive(Default)]
struct StateDbInfo {
    source_path: PathBuf,
    cfs: Vec<String>,
    /// Total entries in `certificate_header_cf`. Populated only when the
    /// read-only open succeeds.
    certificate_header_count: Option<usize>,
    duration: Duration,
    fatal_error: Option<String>,
    /// Outcome of `StateStore::init_db` against a staged copy. `None`
    /// when the migration check was not attempted (e.g. read-only open
    /// already failed).
    migration_outcome: Option<StateMigrationOutcome>,
}

#[derive(Default)]
struct StateMigrationOutcome {
    cfs_after: Vec<String>,
    duration: Duration,
    error: Option<String>,
}

impl FullSweepReport {
    fn overall_status(&self) -> Status {
        let mut status = Status::Ok;
        for s in self.store_statuses() {
            status = status.worse(s);
        }
        status
    }

    fn store_statuses(&self) -> Vec<Status> {
        let mut out = Vec::new();
        if let Some(p) = &self.pending {
            out.push(p.classify());
        }
        if let Some(d) = &self.debug {
            out.push(d.classify());
        }
        // Only count the epoch sweep's status if it actually ran.
        if self.epochs_skipped_reason.is_none() && self.epochs.epochs_dir.is_some() {
            out.push(self.epochs.classify());
        }
        out
    }

    fn total_validated(&self) -> usize {
        self.pending.as_ref().map(|s| s.validated).unwrap_or(0)
            + self.debug.as_ref().map(|s| s.validated).unwrap_or(0)
            + self.epochs.total_validated
    }

    fn total_skipped(&self) -> usize {
        self.pending.as_ref().map(|s| s.skipped.len()).unwrap_or(0)
            + self.debug.as_ref().map(|s| s.skipped.len()).unwrap_or(0)
            + self.epochs.total_skipped
    }

    fn store_count(&self) -> usize {
        let mut n = 0;
        if self.pending.is_some() {
            n += 1;
        }
        if self.debug.is_some() {
            n += 1;
        }
        if self.state.is_some() {
            n += 1;
        }
        n + self.epochs.processed
    }
}

#[derive(Clone, Copy)]
enum Status {
    Ok,
    Skips,
    Failed,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Ok => "OK",
            Status::Skips => "OK with skips",
            Status::Failed => "FAILED",
        }
    }

    fn css_class(self) -> &'static str {
        match self {
            Status::Ok => "success",
            Status::Skips => "warning",
            Status::Failed => "destructive",
        }
    }

    fn worse(self, other: Status) -> Status {
        match (self, other) {
            (Status::Failed, _) | (_, Status::Failed) => Status::Failed,
            (Status::Skips, _) | (_, Status::Skips) => Status::Skips,
            _ => Status::Ok,
        }
    }
}

// ---------------------------------------------------------------------------
// Non-panicking validator
// ---------------------------------------------------------------------------

/// Stage `src` to a tempdir, run `init_db` (which triggers the proto
/// migration), then iterate the legacy CF and validate every row against
/// the proto CF. Records validated/skipped counts and any fatal error into
/// `outcome` instead of panicking, so a single bad sub-DB does not abort
/// the whole sweep.
///
/// `analyze_skip` is called for each unparsable legacy row with the
/// typed key. The returned `SkipAnalysis` is attached to the recorded
/// `SkippedRow`. Pass [`no_skip_analysis`] when no enrichment is wanted.
fn validate_one_store<L, P>(
    src: &Path,
    label: &str,
    init_db: impl FnOnce(&Path) -> Result<DB, DBOpenError>,
    expected_legacy_cf: &str,
    analyze_skip: impl Fn(&L::Key) -> Option<SkipAnalysis>,
) -> StoreOutcome
where
    L: ColumnSchema<Value = LegacyCertificate>,
    P: ColumnSchema<Value = Certificate, Key = L::Key>,
    L::Key: std::fmt::Debug,
{
    let started = Instant::now();
    let mut outcome = StoreOutcome::new(label, src);

    // Phase 1: schema check on the source.
    let cfs = match rocksdb::DB::list_cf(&rocksdb::Options::default(), src) {
        Ok(mut cfs) => {
            cfs.sort();
            cfs
        }
        Err(e) => {
            outcome.fatal_error = Some(format!("not a rocksdb directory: {e}"));
            outcome.duration = started.elapsed();
            return outcome;
        }
    };
    outcome.cfs_before = cfs.clone();

    if !cfs.iter().any(|cf| cf == expected_legacy_cf) {
        outcome.fatal_error = Some(format!(
            "expected legacy CF {expected_legacy_cf:?} not found; CFs in source are {cfs:?}"
        ));
        outcome.duration = started.elapsed();
        return outcome;
    }

    // Phase 2: stage to tempdir.
    let staging = TempDBDir::new();
    if let Err(e) = copy_dir_recursive(src, &staging.path) {
        outcome.fatal_error = Some(format!("copy snapshot to tempdir failed: {e}"));
        outcome.duration = started.elapsed();
        return outcome;
    }

    // Phase 3: init_db (runs migration).
    let db = match init_db(&staging.path) {
        Ok(db) => db,
        Err(e) => {
            outcome.fatal_error = Some(format!("init_db: {}", format_error_chain(&e)));
            outcome.duration = started.elapsed();
            return outcome;
        }
    };

    if let Ok(mut post) = rocksdb::DB::list_cf(&rocksdb::Options::default(), &staging.path) {
        post.sort();
        outcome.cfs_after = post;
    }

    // Phase 4: iterate + validate.
    let keys_iter = match db.keys::<L>() {
        Ok(it) => it,
        Err(e) => {
            outcome.fatal_error = Some(format!("open keys iterator on legacy CF: {e}"));
            outcome.duration = started.elapsed();
            return outcome;
        }
    };

    for key_result in keys_iter {
        let key = match key_result {
            Ok(k) => k,
            Err(e) => {
                outcome.skipped.push(SkippedRow {
                    key: "<undecodable key>".to_string(),
                    error: format!("legacy key decode: {e}"),
                    analysis: None,
                });
                continue;
            }
        };

        let legacy = match db.get::<L>(&key) {
            Ok(Some(legacy)) => legacy,
            Ok(None) => continue,
            Err(DBError::CodecError(e)) => {
                let analysis = analyze_skip(&key);
                let detail = match analysis.as_ref() {
                    Some(a) => format_analysis_for_log(a),
                    None => String::new(),
                };
                // CodecError's `Display` includes newlines/indent for the
                // "report this" tail; collapse to a single line so the per-row
                // SKIP record stays one log entry that grep'able.
                let one_line_error = squash_whitespace(&e.to_string());
                eprintln!("[{label}] SKIP key={key:?}{detail} error={one_line_error}");
                outcome.skipped.push(SkippedRow {
                    key: format!("{key:?}"),
                    error: e.to_string(),
                    analysis,
                });
                continue;
            }
            Err(other) => {
                outcome.fatal_error = Some(format!("read legacy at {key:?}: {other}"));
                outcome.duration = started.elapsed();
                return outcome;
            }
        };

        match db.get::<P>(&key) {
            Ok(Some(proto)) => {
                let cert = Certificate::from(legacy);
                if proto != cert {
                    outcome.fatal_error =
                        Some(format!("decoded certificates differ for key {key:?}"));
                    outcome.duration = started.elapsed();
                    return outcome;
                }
                outcome.validated += 1;
            }
            Ok(None) => {
                outcome.fatal_error = Some(format!(
                    "proto CF is missing a row for key {key:?} that exists in the legacy CF"
                ));
                outcome.duration = started.elapsed();
                return outcome;
            }
            Err(e) => {
                outcome.fatal_error = Some(format!("read proto at {key:?}: {e}"));
                outcome.duration = started.elapsed();
                return outcome;
            }
        }
    }

    outcome.duration = started.elapsed();
    outcome
}

// ---------------------------------------------------------------------------
// State DB & skip analyzers
// ---------------------------------------------------------------------------

/// Used as the `analyze_skip` argument when no state-DB cross-reference is
/// wanted (e.g. epoch validation, where the key is just a `CertificateIndex`
/// and there is no direct way to map it to a `CertificateHeader`).
fn no_skip_analysis<K>(_: &K) -> Option<SkipAnalysis> {
    None
}

/// CFs required by the skip analyzers. We do NOT use the full `STATE_DB`
/// definition here because older agglayer state DBs may not contain every
/// CF the current schema declares (e.g. settlement-related families added
/// later). Opening read-only with a CF that does not exist on disk fails
/// hard, so the analyzer narrows to just what it reads.
const ANALYZER_STATE_CFS: &[ColumnDescriptor] = &[
    ColumnDescriptor::new::<CertificateHeaderColumn>(),
    ColumnDescriptor::new::<CertificatePerNetworkColumn>(),
];

/// Open the state DB at `state_dir` read-only with the minimal CF set needed
/// by the skip analyzers. Returns `(info, Some(db))` on success and
/// `(info, None)` on failure (with `info.fatal_error` populated). Read-only
/// opens do not take the LOCK file, so this is safe even if a node is
/// concurrently writing the source DB (we still get a consistent snapshot
/// of whatever has been flushed).
fn open_state_db_readonly(state_dir: &Path) -> (StateDbInfo, Option<DB>) {
    let started = Instant::now();
    let mut info = StateDbInfo {
        source_path: state_dir.to_path_buf(),
        ..StateDbInfo::default()
    };

    let cfs_present = match rocksdb::DB::list_cf(&rocksdb::Options::default(), state_dir) {
        Ok(mut cfs) => {
            cfs.sort();
            info.cfs = cfs.clone();
            cfs
        }
        Err(e) => {
            info.fatal_error = Some(format!("not a rocksdb directory: {e}"));
            info.duration = started.elapsed();
            return (info, None);
        }
    };

    // Verify the analyzer's required CFs exist before attempting to open;
    // otherwise the open will fail with `Column family not found` and the
    // operator gets a clearer diagnostic from us.
    for required in ANALYZER_STATE_CFS {
        if !cfs_present.iter().any(|cf| cf == required.name()) {
            info.fatal_error = Some(format!(
                "state DB is missing the {:?} CF required for skip analysis; found {cfs_present:?}",
                required.name()
            ));
            info.duration = started.elapsed();
            return (info, None);
        }
    }

    let db = match DB::open_cf_readonly(state_dir, ANALYZER_STATE_CFS) {
        Ok(db) => db,
        Err(e) => {
            info.fatal_error = Some(format!("open state DB read-only failed: {e}"));
            info.duration = started.elapsed();
            return (info, None);
        }
    };

    // Best-effort count of certificate headers; if iteration fails for any
    // reason we leave it as `None` and continue.
    let count = db
        .keys::<CertificateHeaderColumn>()
        .ok()
        .map(|iter| iter.filter_map(Result::ok).count());
    info.certificate_header_count = count;

    // Independently from the read-only analyzer open, verify
    // `StateStore::init_db` can bring the snapshot up to the current
    // schema. This catches latent upgrade bugs (legacy schemas missing
    // CFs added later, etc.) without affecting the source.
    info.migration_outcome = Some(check_state_migration(state_dir));

    info.duration = started.elapsed();
    (info, Some(db))
}

/// Stage the state DB into a fresh tempdir and run `StateStore::init_db`
/// to verify the current binary can open and migrate it. Errors are
/// captured in the returned outcome rather than propagated, so the rest
/// of the sweep keeps running.
fn check_state_migration(state_dir: &Path) -> StateMigrationOutcome {
    let started = Instant::now();
    let mut outcome = StateMigrationOutcome::default();

    let staging = TempDBDir::new();
    if let Err(e) = copy_dir_recursive(state_dir, &staging.path) {
        outcome.error = Some(format!("copy state to staging tempdir failed: {e}"));
        outcome.duration = started.elapsed();
        return outcome;
    }

    match StateStore::init_db(&staging.path) {
        Ok(_db) => {
            if let Ok(mut post) = rocksdb::DB::list_cf(&rocksdb::Options::default(), &staging.path)
            {
                post.sort();
                outcome.cfs_after = post;
            }
        }
        Err(e) => {
            outcome.error = Some(format!("StateStore::init_db: {}", format_error_chain(&e)));
        }
    }
    outcome.duration = started.elapsed();
    outcome
}

/// Render `CertificateId` as a hex string, matching the runtime
/// `Display`/`Debug` representation used elsewhere in the codebase.
fn format_certificate_id(id: &CertificateId) -> String {
    format!("{id:?}")
}

/// Resolve the `CertificateHeader` for a debug skipped row. The key is a
/// `CertificateId`, which is also the key in `certificate_header_cf`.
fn analyze_debug_key(state_db: &DB, key: &CertificateId) -> Option<SkipAnalysis> {
    let mut analysis = SkipAnalysis {
        source: "state_db: certificate_header_cf",
        certificate_id: Some(format_certificate_id(key)),
        ..SkipAnalysis::default()
    };

    match state_db.get::<CertificateHeaderColumn>(key) {
        Ok(Some(header)) => populate_from_header(&mut analysis, &header),
        Ok(None) => {
            analysis.note = Some(
                "no entry in certificate_header_cf for this id; certificate may have been dropped \
                 or the row is genuinely orphaned"
                    .to_string(),
            );
        }
        Err(e) => {
            analysis.note = Some(format!("state DB read failed: {e}"));
        }
    }
    Some(analysis)
}

/// Resolve the `CertificateHeader` for a pending skipped row. The key is
/// `(NetworkId, Height)`. `certificate_per_network_cf` maps that to a
/// `CertificateId`; we then look up `certificate_header_cf`.
fn analyze_pending_key(state_db: &DB, key: &PendingQueueKey) -> Option<SkipAnalysis> {
    let network_id = key.0;
    let height = key.1;
    let mut analysis = SkipAnalysis {
        source: "state_db: certificate_per_network_cf -> certificate_header_cf",
        network_id: Some(network_id.to_u32()),
        height: Some(height.as_u64()),
        ..SkipAnalysis::default()
    };

    let lookup_key = CertPerNetworkKey {
        network_id: network_id.to_u32(),
        height,
    };
    let cert_id = match state_db.get::<CertificatePerNetworkColumn>(&lookup_key) {
        Ok(Some(id)) => id,
        Ok(None) => {
            analysis.note = Some(format!(
                "no certificate_per_network_cf entry for (network_id={}, height={}); the \
                 unparsable row appears orphaned",
                network_id.to_u32(),
                height.as_u64(),
            ));
            return Some(analysis);
        }
        Err(e) => {
            analysis.note = Some(format!("state DB read on certificate_per_network_cf: {e}"));
            return Some(analysis);
        }
    };
    analysis.certificate_id = Some(format_certificate_id(&cert_id));

    match state_db.get::<CertificateHeaderColumn>(&cert_id) {
        Ok(Some(header)) => populate_from_header(&mut analysis, &header),
        Ok(None) => {
            analysis.note = Some(format!(
                "certificate_per_network_cf points at id {cert_id:?} but certificate_header_cf \
                 has no entry for it",
            ));
        }
        Err(e) => {
            analysis.note = Some(format!("state DB read on certificate_header_cf: {e}"));
        }
    }
    Some(analysis)
}

fn populate_from_header(analysis: &mut SkipAnalysis, header: &CertificateHeader) {
    analysis.header_status = Some(format_status(&header.status));
    analysis.network_id = Some(header.network_id.to_u32());
    analysis.height = Some(header.height.as_u64());
}

/// Collapse whitespace runs (including newlines) into single spaces so a
/// multi-line error message renders as one log line.
fn squash_whitespace(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            if !prev_space {
                out.push(' ');
                prev_space = true;
            }
        } else {
            out.push(c);
            prev_space = false;
        }
    }
    out.trim().to_string()
}

/// Render the per-row analysis as a single concise stderr line for the
/// operator to follow during the run.
fn format_analysis_for_log(a: &SkipAnalysis) -> String {
    let mut out = String::new();
    if let Some(id) = &a.certificate_id {
        out.push_str(" cert_id=");
        out.push_str(id);
    }
    if let Some(s) = &a.header_status {
        out.push_str(" status=");
        out.push_str(s);
    }
    match (a.network_id, a.height) {
        (Some(n), Some(h)) => {
            out.push_str(&format!(" network={n} height={h}"));
        }
        (Some(n), None) => out.push_str(&format!(" network={n}")),
        (None, Some(h)) => out.push_str(&format!(" height={h}")),
        _ => {}
    }
    if let Some(note) = &a.note {
        out.push_str(" note=\"");
        out.push_str(note);
        out.push('"');
    }
    out
}

fn format_status(status: &CertificateStatus) -> String {
    // The default `Display` for `InError` includes the full chain;
    // collapse to a short tag plus the cause for table compactness.
    match status {
        CertificateStatus::InError { error } => format!("InError ({error})"),
        other => other.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Discovery & orchestration
// ---------------------------------------------------------------------------

/// Discover epoch sub-directories: numeric names under `epochs_dir`, sorted
/// numerically. Filters out non-directories and non-numeric entries
/// (e.g. `lost+found`).
fn discover_epochs(epochs_dir: &Path) -> Vec<(u64, PathBuf)> {
    let entries = match fs::read_dir(epochs_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut epochs = Vec::new();
    for entry in entries.flatten() {
        if !entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            continue;
        }
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else {
            continue;
        };
        let Ok(n) = name_str.parse::<u64>() else {
            continue;
        };
        epochs.push((n, entry.path()));
    }
    epochs.sort_by_key(|(n, _)| *n);
    epochs
}

fn validate_data_dir(
    data_dir: &Path,
    epoch_limit: Option<u64>,
    skip_epochs: bool,
    env_label: String,
) -> FullSweepReport {
    let started = Instant::now();
    let mut report = FullSweepReport {
        started_at: SystemTime::now(),
        data_dir: data_dir.to_path_buf(),
        overall_duration: Duration::ZERO,
        pending: None,
        debug: None,
        epochs: EpochsOutcome::default(),
        state: None,
        epoch_limit,
        env_label,
        epochs_skipped_reason: None,
    };

    // State DB (read-only): opened first so the pending and debug
    // validators can use it to enrich skipped rows with the
    // CertificateHeader / status from `state_db`.
    let state_dir = data_dir.join("state");
    let state_db_handle: Option<DB> = if state_dir.is_dir() {
        eprintln!("[full] opening state DB read-only at {state_dir:?}");
        let (info, db) = open_state_db_readonly(&state_dir);
        if let Some(err) = &info.fatal_error {
            eprintln!("[full] state DB unavailable for skip analysis: {err}");
        } else if let Some(n) = info.certificate_header_count {
            eprintln!("[full] state DB opened: {n} certificate_header_cf entries");
        }
        report.state = Some(info);
        db
    } else {
        eprintln!(
            "[full] no state/ subdirectory under {data_dir:?}; skipped-row analysis will be \
             unavailable"
        );
        None
    };

    // Pending
    let pending_dir = data_dir.join("pending");
    if pending_dir.is_dir() {
        eprintln!("[full] validating pending at {pending_dir:?}");
        let analyzer = |key: &PendingQueueKey| -> Option<SkipAnalysis> {
            state_db_handle
                .as_ref()
                .and_then(|db| analyze_pending_key(db, key))
        };
        report.pending = Some(validate_one_store::<
            PendingQueueColumn,
            PendingQueueProtoColumn,
        >(
            &pending_dir,
            "pending",
            PendingStore::init_db,
            PendingQueueColumn::COLUMN_FAMILY_NAME,
            analyzer,
        ));
        let p = report.pending.as_ref().unwrap();
        eprintln!(
            "[full] pending: {} validated, {} skipped, {:.2?}{}",
            p.validated,
            p.skipped.len(),
            p.duration,
            p.fatal_error
                .as_deref()
                .map(|e| format!(", FATAL: {e}"))
                .unwrap_or_default(),
        );
    } else {
        eprintln!("[full] no pending/ subdirectory under {data_dir:?}; skipping");
    }

    // Debug
    let debug_dir = data_dir.join("debug");
    if debug_dir.is_dir() {
        eprintln!("[full] validating debug at {debug_dir:?}");
        let analyzer = |key: &CertificateId| -> Option<SkipAnalysis> {
            state_db_handle
                .as_ref()
                .and_then(|db| analyze_debug_key(db, key))
        };
        report.debug = Some(validate_one_store::<
            DebugCertificatesColumn,
            DebugCertificatesProtoColumn,
        >(
            &debug_dir,
            "debug",
            DebugStore::init_db,
            DebugCertificatesColumn::COLUMN_FAMILY_NAME,
            analyzer,
        ));
        let d = report.debug.as_ref().unwrap();
        eprintln!(
            "[full] debug: {} validated, {} skipped, {:.2?}{}",
            d.validated,
            d.skipped.len(),
            d.duration,
            d.fatal_error
                .as_deref()
                .map(|e| format!(", FATAL: {e}"))
                .unwrap_or_default(),
        );
    } else {
        eprintln!("[full] no debug/ subdirectory under {data_dir:?}; skipping");
    }

    // Epochs
    if skip_epochs {
        eprintln!(
            "[full] skipping epoch sweep (AGGLAYER_MIGRATION_SNAPSHOT_SKIP_EPOCHS=1); use this \
             flag to validate pending+debug+state quickly without paying the full epoch cost"
        );
        report.epochs_skipped_reason = Some(
            "skipped via AGGLAYER_MIGRATION_SNAPSHOT_SKIP_EPOCHS=1; pending/debug/state were \
             still validated",
        );
        report.overall_duration = started.elapsed();
        return report;
    }
    let epochs_dir = data_dir.join("epochs");
    if epochs_dir.is_dir() {
        report.epochs.epochs_dir = Some(epochs_dir.clone());
        let all_epochs = discover_epochs(&epochs_dir);
        report.epochs.discovered = all_epochs.len();
        eprintln!(
            "[full] discovered {} epoch directories under {epochs_dir:?}",
            all_epochs.len()
        );

        let to_process: Vec<_> = match epoch_limit {
            Some(limit) => {
                let n = (limit as usize).min(all_epochs.len());
                report.epochs.skipped_due_to_limit = all_epochs.len().saturating_sub(n);
                all_epochs.into_iter().take(n).collect()
            }
            None => all_epochs,
        };

        let epoch_started = Instant::now();
        for (epoch_num, epoch_path) in to_process {
            let outcome =
                validate_one_store::<CertificatePerIndexColumn, CertificatePerIndexProtoColumn>(
                    &epoch_path,
                    &format!("epoch-{epoch_num}"),
                    PerEpochStore::<(), ()>::init_db,
                    CertificatePerIndexColumn::COLUMN_FAMILY_NAME,
                    no_skip_analysis,
                );

            report.epochs.processed += 1;
            report.epochs.total_validated += outcome.validated;
            report.epochs.total_skipped += outcome.skipped.len();

            match outcome.classify() {
                Status::Ok => report.epochs.successful += 1,
                Status::Skips => report.epochs.with_skips += 1,
                Status::Failed => report.epochs.failed += 1,
            }

            // Keep verbatim only the problem epochs (skips or failures).
            if !matches!(outcome.classify(), Status::Ok) {
                if report.epochs.problem_epochs.len() < MAX_PROBLEM_EPOCHS {
                    report.epochs.problem_epochs.push((epoch_num, outcome));
                } else {
                    report.epochs.extra_problem_epochs += 1;
                }
            }

            if report.epochs.processed.is_multiple_of(200) {
                eprintln!(
                    "[full] processed {} / {} epochs ({:.2?} elapsed; {} ok, {} skips, {} fail)",
                    report.epochs.processed,
                    report.epochs.discovered,
                    epoch_started.elapsed(),
                    report.epochs.successful,
                    report.epochs.with_skips,
                    report.epochs.failed,
                );
            }
        }
        report.epochs.duration = epoch_started.elapsed();
        eprintln!(
            "[full] epochs DONE: {} ok, {} with skips, {} failed in {:.2?}",
            report.epochs.successful,
            report.epochs.with_skips,
            report.epochs.failed,
            report.epochs.duration,
        );
    } else {
        eprintln!("[full] no epochs/ subdirectory under {data_dir:?}; skipping");
    }

    report.overall_duration = started.elapsed();
    report
}

// ---------------------------------------------------------------------------
// HTML rendering (shadcn-inspired, self-contained, no external resources)
// ---------------------------------------------------------------------------

const STYLE: &str = r#"
:root {
  --background: 0 0% 100%;
  --foreground: 240 10% 3.9%;
  --card: 0 0% 100%;
  --card-foreground: 240 10% 3.9%;
  --muted: 240 4.8% 95.9%;
  --muted-foreground: 240 3.8% 46.1%;
  --border: 240 5.9% 90%;
  --primary: 240 5.9% 10%;
  --primary-foreground: 0 0% 98%;
  --success: 142 71% 35%;
  --success-foreground: 142 71% 95%;
  --warning: 38 92% 45%;
  --warning-foreground: 38 92% 95%;
  --destructive: 0 72% 50%;
  --destructive-foreground: 0 0% 98%;
  --radius: 0.625rem;
}

@media (prefers-color-scheme: dark) {
  :root {
    --background: 240 10% 3.9%;
    --foreground: 0 0% 98%;
    --card: 240 10% 5.9%;
    --card-foreground: 0 0% 98%;
    --muted: 240 3.7% 15.9%;
    --muted-foreground: 240 5% 64.9%;
    --border: 240 3.7% 15.9%;
    --primary: 0 0% 98%;
    --primary-foreground: 240 5.9% 10%;
    --success: 142 60% 45%;
    --success-foreground: 142 60% 12%;
    --warning: 38 90% 55%;
    --warning-foreground: 38 90% 12%;
    --destructive: 0 70% 60%;
    --destructive-foreground: 0 0% 98%;
  }
}

* { box-sizing: border-box; }

body {
  font-family: ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont,
               "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  color: hsl(var(--foreground));
  background: hsl(var(--background));
  margin: 0;
  line-height: 1.5;
  font-size: 14px;
}

.container {
  max-width: 1280px;
  margin: 0 auto;
  padding: 2.5rem 1.5rem 4rem;
}

header.page-header { margin-bottom: 2rem; }
header.page-header h1 {
  font-size: 1.875rem; font-weight: 600;
  letter-spacing: -0.02em; margin: 0 0 0.4rem;
}
header.page-header .meta {
  color: hsl(var(--muted-foreground));
  font-size: 0.875rem;
  display: flex; flex-wrap: wrap; gap: 0.4rem 1.25rem;
}
header.page-header .meta code { word-break: break-all; }

h2 {
  font-size: 1.25rem; font-weight: 600;
  letter-spacing: -0.01em; margin: 2.5rem 0 1rem;
}
h3 {
  font-size: 1rem; font-weight: 600;
  margin: 1.5rem 0 0.75rem;
}

code, .mono { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; }
code {
  background: hsl(var(--muted));
  padding: 0.1em 0.35em;
  border-radius: 4px;
  font-size: 0.875em;
}
.mono.break { word-break: break-all; }

.card {
  background: hsl(var(--card));
  color: hsl(var(--card-foreground));
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  padding: 1.25rem 1.5rem;
  box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05);
}
.card + .card { margin-top: 1rem; }
.card .card-title {
  font-size: 0.95rem; font-weight: 600;
  display: flex; align-items: center; justify-content: space-between;
  gap: 1rem; margin: 0 0 0.5rem;
}
.card .card-meta {
  color: hsl(var(--muted-foreground));
  font-size: 0.8125rem;
  margin-top: 0.25rem;
}

.kpi-grid {
  display: grid; gap: 1rem;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  margin-bottom: 1rem;
}
.kpi {
  background: hsl(var(--card));
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  padding: 1rem 1.25rem;
}
.kpi .kpi-label {
  color: hsl(var(--muted-foreground));
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 500;
}
.kpi .kpi-value {
  font-size: 2rem;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1.1;
  margin-top: 0.4rem;
}
.kpi .kpi-value.success { color: hsl(var(--success)); }
.kpi .kpi-value.warning { color: hsl(var(--warning)); }
.kpi .kpi-value.destructive { color: hsl(var(--destructive)); }
.kpi .kpi-hint {
  color: hsl(var(--muted-foreground));
  font-size: 0.8125rem;
  margin-top: 0.25rem;
}

.badge {
  display: inline-flex;
  align-items: center;
  border-radius: 999px;
  padding: 0.15rem 0.6rem;
  font-size: 0.75rem;
  font-weight: 600;
  letter-spacing: 0.01em;
  white-space: nowrap;
}
.badge.success {
  background: hsl(var(--success) / 0.12);
  color: hsl(var(--success));
  border: 1px solid hsl(var(--success) / 0.3);
}
.badge.warning {
  background: hsl(var(--warning) / 0.12);
  color: hsl(var(--warning));
  border: 1px solid hsl(var(--warning) / 0.3);
}
.badge.destructive {
  background: hsl(var(--destructive) / 0.12);
  color: hsl(var(--destructive));
  border: 1px solid hsl(var(--destructive) / 0.3);
}
.badge.muted {
  background: hsl(var(--muted));
  color: hsl(var(--muted-foreground));
  border: 1px solid hsl(var(--border));
}

table {
  width: 100%;
  border-collapse: collapse;
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  overflow: hidden;
  font-size: 0.875rem;
}
thead { background: hsl(var(--muted)); }
th, td {
  padding: 0.55rem 0.85rem;
  text-align: left;
  vertical-align: top;
}
th {
  font-weight: 600;
  color: hsl(var(--muted-foreground));
  text-transform: uppercase;
  font-size: 0.7rem;
  letter-spacing: 0.06em;
}
tbody tr { border-top: 1px solid hsl(var(--border)); }
tbody tr:hover { background: hsl(var(--muted) / 0.4); }
td.num { text-align: right; font-variant-numeric: tabular-nums; }
td.mono, code.mono { font-family: ui-monospace, monospace; }

details {
  margin-top: 0.75rem;
  border: 1px solid hsl(var(--border));
  border-radius: var(--radius);
  background: hsl(var(--card));
}
details > summary {
  list-style: none;
  cursor: pointer;
  padding: 0.65rem 0.85rem;
  font-weight: 500;
  color: hsl(var(--foreground));
}
details > summary::-webkit-details-marker { display: none; }
details[open] > summary { border-bottom: 1px solid hsl(var(--border)); }
details > .details-body { padding: 0.85rem; }

pre {
  background: hsl(var(--muted));
  padding: 0.85rem;
  border-radius: 6px;
  overflow-x: auto;
  font-size: 0.8125rem;
  margin: 0;
}

.section-grid { display: grid; gap: 1rem; }
.muted { color: hsl(var(--muted-foreground)); font-size: 0.875rem; }
.fatal {
  border-left: 3px solid hsl(var(--destructive));
  background: hsl(var(--destructive) / 0.08);
  color: hsl(var(--destructive));
  padding: 0.75rem 1rem;
  border-radius: 6px;
  margin-top: 0.75rem;
  font-size: 0.875rem;
}
"#;

fn render_full_html_report(report: &FullSweepReport) -> String {
    let mut out = String::with_capacity(64 * 1024);
    let status = report.overall_status();

    out.push_str("<!DOCTYPE html>\n");
    out.push_str("<html lang=\"en\">\n<head>\n");
    out.push_str("<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    out.push_str("<title>Migration snapshot report — full sweep</title>\n");
    out.push_str("<style>");
    out.push_str(STYLE);
    out.push_str("</style>\n</head>\n<body>\n");
    out.push_str("<div class=\"container\">\n");

    // ---- Header ----
    let _ = write!(
        out,
        r#"<header class="page-header">
  <h1>Migration snapshot report</h1>
  <div class="meta">
    <span><span class="badge {sc}">{slabel}</span></span>
    <span>Full data-directory sweep</span>
    <span>Generated <strong>{generated}</strong> UTC</span>
    <span>Duration <strong>{dur}</strong></span>
    <span>Data dir <code class="mono break">{data_dir}</code></span>
  </div>
</header>
"#,
        sc = status.css_class(),
        slabel = status.label(),
        generated = format_system_time(report.started_at),
        dur = format_duration(Some(report.overall_duration)),
        data_dir = html_escape(&report.data_dir.display().to_string()),
    );

    // ---- Top KPIs ----
    out.push_str("<section><h2>Summary</h2>\n<div class=\"kpi-grid\">\n");
    push_kpi(
        &mut out,
        "Validated rows",
        &report.total_validated().to_string(),
        Status::Ok,
        Some("Across all sub-DBs"),
    );
    push_kpi(
        &mut out,
        "Skipped rows",
        &report.total_skipped().to_string(),
        if report.total_skipped() == 0 {
            Status::Ok
        } else {
            Status::Skips
        },
        Some("Unparsable; mirrored from migration"),
    );
    push_kpi(
        &mut out,
        "Stores checked",
        &report.store_count().to_string(),
        Status::Ok,
        Some("Pending + Debug + Epochs"),
    );
    push_kpi(
        &mut out,
        "Total time",
        &format_duration(Some(report.overall_duration)),
        Status::Ok,
        None,
    );
    out.push_str("</div>\n</section>\n");

    // ---- State DB (read-only, used for analysis) ----
    if let Some(s) = &report.state {
        push_state_card(&mut out, s);
    } else {
        push_missing_section(
            &mut out,
            "State DB",
            "no state/ subdirectory under data dir; skipped-row analysis was unavailable",
        );
    }

    // ---- Pending ----
    if let Some(p) = &report.pending {
        push_store_card(&mut out, "Pending DB", p);
    } else {
        push_missing_section(
            &mut out,
            "Pending DB",
            "no pending/ subdirectory under data dir",
        );
    }

    // ---- Debug ----
    if let Some(d) = &report.debug {
        push_store_card(&mut out, "Debug DB", d);
    } else {
        push_missing_section(
            &mut out,
            "Debug DB",
            "no debug/ subdirectory under data dir",
        );
    }

    // ---- Epochs ----
    if let Some(reason) = report.epochs_skipped_reason {
        push_missing_section(&mut out, "Epoch DBs", reason);
    } else if report.epochs.epochs_dir.is_some() {
        push_epochs_section(&mut out, &report.epochs, report.epoch_limit);
    } else {
        push_missing_section(
            &mut out,
            "Epoch DBs",
            "no epochs/ subdirectory under data dir",
        );
    }

    // ---- Footer / config ----
    out.push_str("<h2>Run configuration</h2>\n");
    out.push_str("<table>\n<tbody>\n");
    push_row(
        &mut out,
        "Data directory",
        &format!(
            "<code class=\"mono break\">{}</code>",
            html_escape(&report.data_dir.display().to_string())
        ),
    );
    push_row(
        &mut out,
        "Epoch limit",
        match report.epoch_limit {
            Some(n) => format!("first <strong>{n}</strong> epochs"),
            None => "(no limit; all epochs scanned)".to_string(),
        }
        .as_str(),
    );
    push_row(
        &mut out,
        "Skip epochs flag",
        match report.epochs_skipped_reason {
            Some(_) => format!(
                "<strong>true</strong> (set via <code>{SKIP_EPOCHS_ENV}</code>); epoch sweep was \
                 bypassed entirely"
            ),
            None => format!("false (set <code>{SKIP_EPOCHS_ENV}=1</code> to skip the epoch sweep)"),
        }
        .as_str(),
    );
    push_row(
        &mut out,
        "Reports written under",
        &format!("<code class=\"mono break\">${{{REPORT_DIR_ENV}:-$TMPDIR}}</code>"),
    );
    out.push_str("</tbody>\n</table>\n");

    out.push_str("</div>\n</body>\n</html>\n");
    out
}

fn push_kpi(out: &mut String, label: &str, value: &str, status: Status, hint: Option<&str>) {
    let _ = write!(
        out,
        r#"  <div class="kpi">
    <div class="kpi-label">{label}</div>
    <div class="kpi-value {sc}">{value}</div>
"#,
        label = html_escape(label),
        sc = status.css_class(),
        value = html_escape(value),
    );
    if let Some(h) = hint {
        let _ = writeln!(out, "    <div class=\"kpi-hint\">{}</div>", html_escape(h));
    }
    out.push_str("  </div>\n");
}

fn push_store_card(out: &mut String, heading: &str, store: &StoreOutcome) {
    let status = store.classify();
    let _ = write!(
        out,
        r#"<section><h2>{heading}</h2>
<div class="card">
  <div class="card-title">
    <span>{label}</span>
    <span class="badge {sc}">{slabel}</span>
  </div>
  <div class="kpi-grid">
"#,
        heading = html_escape(heading),
        label = html_escape(&store.label),
        sc = status.css_class(),
        slabel = status.label(),
    );
    push_kpi(
        out,
        "Validated",
        &store.validated.to_string(),
        Status::Ok,
        None,
    );
    push_kpi(
        out,
        "Skipped",
        &store.skipped.len().to_string(),
        if store.skipped.is_empty() {
            Status::Ok
        } else {
            Status::Skips
        },
        None,
    );
    push_kpi(
        out,
        "Duration",
        &format_duration(Some(store.duration)),
        Status::Ok,
        None,
    );
    out.push_str("  </div>\n");

    let _ = write!(out, "  <table style=\"margin-top: 0.85rem\">\n  <tbody>\n");
    push_row(
        out,
        "Source path",
        &format!(
            "<code class=\"mono break\">{}</code>",
            html_escape(&store.source_path.display().to_string())
        ),
    );
    push_row(
        out,
        "CFs before migration",
        &format_cf_list(&store.cfs_before),
    );
    push_row(
        out,
        "CFs after migration",
        &format_cf_list(&store.cfs_after),
    );
    out.push_str("  </tbody>\n  </table>\n");

    if let Some(err) = &store.fatal_error {
        let _ = writeln!(out, "  <div class=\"fatal\">{}</div>", html_escape(err));
    }

    if !store.skipped.is_empty() {
        push_skipped_table(out, &store.skipped);
    }

    out.push_str("</div>\n</section>\n");
}

fn push_epochs_section(out: &mut String, epochs: &EpochsOutcome, limit: Option<u64>) {
    let status = epochs.classify();
    out.push_str("<section><h2>Epoch DBs</h2>\n");
    let _ = write!(
        out,
        r#"<div class="card">
  <div class="card-title">
    <span>Aggregate across all epochs</span>
    <span class="badge {sc}">{slabel}</span>
  </div>
  <div class="kpi-grid">
"#,
        sc = status.css_class(),
        slabel = status.label(),
    );

    let limit_hint = if epochs.skipped_due_to_limit > 0 {
        Some(format!(
            "{} skipped due to AGGLAYER_MIGRATION_SNAPSHOT_EPOCH_LIMIT",
            epochs.skipped_due_to_limit
        ))
    } else {
        None
    };
    push_kpi(
        out,
        "Discovered",
        &epochs.discovered.to_string(),
        Status::Ok,
        limit_hint.as_deref(),
    );
    push_kpi(
        out,
        "Processed",
        &epochs.processed.to_string(),
        Status::Ok,
        None,
    );
    push_kpi(
        out,
        "Clean",
        &epochs.successful.to_string(),
        Status::Ok,
        None,
    );
    push_kpi(
        out,
        "With skips",
        &epochs.with_skips.to_string(),
        if epochs.with_skips == 0 {
            Status::Ok
        } else {
            Status::Skips
        },
        None,
    );
    push_kpi(
        out,
        "Failed",
        &epochs.failed.to_string(),
        if epochs.failed == 0 {
            Status::Ok
        } else {
            Status::Failed
        },
        None,
    );
    push_kpi(
        out,
        "Validated rows",
        &epochs.total_validated.to_string(),
        Status::Ok,
        Some("Sum across all processed epochs"),
    );
    push_kpi(
        out,
        "Skipped rows",
        &epochs.total_skipped.to_string(),
        if epochs.total_skipped == 0 {
            Status::Ok
        } else {
            Status::Skips
        },
        None,
    );
    push_kpi(
        out,
        "Duration",
        &format_duration(Some(epochs.duration)),
        Status::Ok,
        None,
    );
    out.push_str("  </div>\n");

    if let Some(n) = limit {
        let _ = writeln!(
            out,
            "<p class=\"muted\">Run was capped at the first <strong>{n}</strong> epochs by \
             AGGLAYER_MIGRATION_SNAPSHOT_EPOCH_LIMIT.</p>"
        );
    }

    if !epochs.problem_epochs.is_empty() {
        let _ = writeln!(
            out,
            "<h3>Epochs needing attention ({} listed)</h3>",
            epochs.problem_epochs.len()
        );
        out.push_str("<table>\n<thead><tr>");
        out.push_str(
            "<th>Epoch</th><th>Status</th><th>Validated</th><th>Skipped</th><th>Duration</\
             th><th>Detail</th>",
        );
        out.push_str("</tr></thead>\n<tbody>\n");
        for (n, outcome) in &epochs.problem_epochs {
            let st = outcome.classify();
            let detail = match (&outcome.fatal_error, outcome.skipped.len()) {
                (Some(err), _) => format!("FATAL: {}", html_escape(err)),
                (None, k) if k > 0 => format!("{k} unparsable rows"),
                _ => "—".to_string(),
            };
            let _ = writeln!(
                out,
                "<tr><td class=\"num\"><strong>{n}</strong></td><td><span class=\"badge \
                 {sc}\">{sl}</span></td><td class=\"num\">{val}</td><td \
                 class=\"num\">{skp}</td><td class=\"num\">{dur}</td><td>{det}</td></tr>",
                sc = st.css_class(),
                sl = st.label(),
                val = outcome.validated,
                skp = outcome.skipped.len(),
                dur = format_duration(Some(outcome.duration)),
                det = detail,
            );
        }
        out.push_str("</tbody>\n</table>\n");

        if epochs.extra_problem_epochs > 0 {
            let _ = writeln!(
                out,
                "<p class=\"muted\">… and <strong>{}</strong> more problem epochs not listed \
                 (cap: {} per report).</p>",
                epochs.extra_problem_epochs, MAX_PROBLEM_EPOCHS,
            );
        }

        // Per-epoch skip details, collapsed.
        out.push_str(
            "<details><summary>Per-epoch skipped rows</summary><div class=\"details-body\">\n",
        );
        for (n, outcome) in &epochs.problem_epochs {
            if outcome.skipped.is_empty() {
                continue;
            }
            let _ = writeln!(
                out,
                "<h3>epoch {n} — {} skipped row(s)</h3>",
                outcome.skipped.len()
            );
            push_skipped_table(out, &outcome.skipped);
        }
        out.push_str("</div></details>\n");
    } else {
        out.push_str(
            "<p class=\"muted\">No epochs needed attention. Every processed epoch validated \
             cleanly.</p>\n",
        );
    }

    out.push_str("</div>\n</section>\n");
}

fn push_skipped_table(out: &mut String, skipped: &[SkippedRow]) {
    let has_analysis = skipped.iter().any(|r| r.analysis.is_some());

    // Status-breakdown summary (only when at least some rows have a
    // resolved status). Gives the operator an at-a-glance answer to "what
    // are these unparsable rows?" without scrolling the full table.
    if has_analysis {
        let breakdown = build_status_breakdown(skipped);
        if !breakdown.is_empty() {
            let _ = writeln!(out, "<h3>Status distribution ({} rows)</h3>", skipped.len());
            out.push_str("<div class=\"kpi-grid\">\n");
            for (status, count) in &breakdown {
                let badge = status_badge_class(status);
                let _ = write!(
                    out,
                    "  <div class=\"kpi\">\n    <div class=\"kpi-label\"><span class=\"badge \
                     {bc}\">{s}</span></div>\n    <div class=\"kpi-value\">{c}</div>\n  </div>\n",
                    bc = badge,
                    s = html_escape(status),
                    c = count,
                );
            }
            out.push_str("</div>\n");
        }
    }

    // Open by default so the per-row analysis is immediately visible.
    let _ = writeln!(
        out,
        "<details open><summary>Per-row detail ({} rows)</summary><div class=\"details-body\">",
        skipped.len()
    );
    out.push_str("<table>\n<thead><tr><th>#</th><th>Key</th><th>Error</th>");
    if has_analysis {
        out.push_str(
            "<th>Cert ID (state)</th><th>Status</th><th>Network / Height</th><th>Notes</th>",
        );
    }
    out.push_str("</tr></thead>\n<tbody>\n");

    let shown = skipped.len().min(MAX_SKIPPED_ROWS_PER_STORE);
    for (i, row) in skipped.iter().take(shown).enumerate() {
        let _ = write!(
            out,
            "  <tr><td class=\"num\">{}</td><td class=\"mono break\">{}</td><td>{}</td>",
            i + 1,
            html_escape(&row.key),
            html_escape(&row.error),
        );
        if has_analysis {
            push_analysis_cells(out, row.analysis.as_ref());
        }
        out.push_str("</tr>\n");
    }
    out.push_str("</tbody>\n</table>\n");
    if shown < skipped.len() {
        let _ = writeln!(
            out,
            "<p class=\"muted\">… and <strong>{}</strong> more rows not listed (cap: {} per \
             store).</p>",
            skipped.len() - shown,
            MAX_SKIPPED_ROWS_PER_STORE,
        );
    }
    out.push_str("</div></details>\n");
}

/// Aggregate the analysis statuses across skipped rows into
/// `(status_label, count)` pairs, sorted by descending count. Rows whose
/// analysis is missing or has no status are bucketed under
/// `"no header"` / `"no analysis"`.
fn build_status_breakdown(skipped: &[SkippedRow]) -> Vec<(String, usize)> {
    use std::collections::BTreeMap;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    for row in skipped {
        let label = match &row.analysis {
            None => "no analysis".to_string(),
            Some(a) => a
                .header_status
                .clone()
                .unwrap_or_else(|| "no header".to_string()),
        };
        *counts.entry(label).or_insert(0) += 1;
    }
    let mut sorted: Vec<_> = counts.into_iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    sorted
}

/// Render the four analysis columns for one row. Empty / em-dash cells when
/// the analysis is missing (e.g. state DB unavailable).
fn push_analysis_cells(out: &mut String, analysis: Option<&SkipAnalysis>) {
    let Some(a) = analysis else {
        out.push_str("<td>—</td><td>—</td><td>—</td><td class=\"muted\">no state DB</td>");
        return;
    };
    // Cert ID
    let _ = write!(
        out,
        "<td class=\"mono break\">{}</td>",
        html_escape(a.certificate_id.as_deref().unwrap_or("—"))
    );
    // Status badge
    out.push_str("<td>");
    if let Some(status) = &a.header_status {
        let badge_class = status_badge_class(status);
        let _ = write!(
            out,
            "<span class=\"badge {bc}\">{s}</span>",
            bc = badge_class,
            s = html_escape(status),
        );
    } else {
        out.push_str("<span class=\"badge muted\">no header</span>");
    }
    out.push_str("</td>");
    // Network / height
    let net_h = match (a.network_id, a.height) {
        (Some(n), Some(h)) => format!("network <code>{n}</code> · height <code>{h}</code>"),
        (Some(n), None) => format!("network <code>{n}</code>"),
        (None, Some(h)) => format!("height <code>{h}</code>"),
        (None, None) => "—".to_string(),
    };
    let _ = write!(out, "<td>{net_h}</td>");
    // Notes
    out.push_str("<td>");
    if let Some(note) = &a.note {
        let _ = write!(out, "{}", html_escape(note));
    } else {
        let _ = write!(
            out,
            "<span class=\"muted\">via {}</span>",
            html_escape(a.source)
        );
    }
    out.push_str("</td>");
}

/// Map a `CertificateStatus` display string to one of the badge variants
/// so the report colours statuses consistently.
fn status_badge_class(status: &str) -> &'static str {
    if status.starts_with("InError") {
        "destructive"
    } else if status == "Settled" || status == "Proven" {
        "success"
    } else if status == "Pending" || status == "Candidate" {
        "warning"
    } else {
        "muted"
    }
}

fn push_state_card(out: &mut String, info: &StateDbInfo) {
    // Aggregate badge: read-only failure or migration failure both downgrade
    // the badge; otherwise green.
    let migration_ok = info
        .migration_outcome
        .as_ref()
        .map(|o| o.error.is_none())
        .unwrap_or(true);
    let badge = if info.fatal_error.is_some() || !migration_ok {
        ("destructive", "FAILED")
    } else {
        ("success", "AVAILABLE")
    };

    let _ = write!(
        out,
        r#"<section><h2>State DB</h2>
<div class="card">
  <div class="card-title">
    <span>state</span>
    <span class="badge {bc}">{bl}</span>
  </div>
  <p class="muted">Two checks are run: (1) read-only open with the analyzer's CFs
   (<code>certificate_per_network_cf</code>, <code>certificate_header_cf</code>) for
   skipped-row enrichment, and (2) <code>StateStore::init_db</code> on a staged copy to
   confirm the snapshot can be opened and migrated by the current binary.</p>
  <div class="kpi-grid">
"#,
        bc = badge.0,
        bl = badge.1,
    );

    let header_count_value = info
        .certificate_header_count
        .map(|n| n.to_string())
        .unwrap_or_else(|| "—".to_string());
    push_kpi(
        out,
        "certificate_header_cf entries",
        &header_count_value,
        Status::Ok,
        Some("Total CertificateHeader rows discovered"),
    );
    push_kpi(
        out,
        "Read-only open",
        if info.fatal_error.is_some() {
            "FAIL"
        } else {
            "OK"
        },
        if info.fatal_error.is_some() {
            Status::Failed
        } else {
            Status::Ok
        },
        Some(&format_duration(Some(info.duration))),
    );

    let (mig_label, mig_status, mig_hint) = match &info.migration_outcome {
        None => (
            "—".to_string(),
            Status::Ok,
            "skipped (read-only open failed)".to_string(),
        ),
        Some(m) => match &m.error {
            Some(_) => (
                "FAIL".to_string(),
                Status::Failed,
                format_duration(Some(m.duration)),
            ),
            None => (
                "OK".to_string(),
                Status::Ok,
                format_duration(Some(m.duration)),
            ),
        },
    };
    push_kpi(
        out,
        "StateStore::init_db",
        &mig_label,
        mig_status,
        Some(&mig_hint),
    );
    out.push_str("  </div>\n");

    out.push_str("  <table style=\"margin-top: 0.85rem\"><tbody>\n");
    push_row(
        out,
        "Source path",
        &format!(
            "<code class=\"mono break\">{}</code>",
            html_escape(&info.source_path.display().to_string())
        ),
    );
    push_row(
        out,
        "CFs on disk (before migration)",
        &format_cf_list(&info.cfs),
    );
    if let Some(m) = &info.migration_outcome {
        if !m.cfs_after.is_empty() && m.cfs_after != info.cfs {
            push_row(
                out,
                "CFs after StateStore::init_db",
                &format_cf_list(&m.cfs_after),
            );
        }
    }
    out.push_str("  </tbody></table>\n");

    if let Some(err) = &info.fatal_error {
        let _ = writeln!(
            out,
            "  <div class=\"fatal\"><strong>Read-only open:</strong> {}</div>",
            html_escape(err)
        );
    }
    if let Some(m) = &info.migration_outcome {
        if let Some(err) = &m.error {
            let _ = writeln!(
                out,
                "  <div class=\"fatal\"><strong>StateStore::init_db:</strong> {}</div>",
                html_escape(err)
            );
        }
    }
    out.push_str("</div></section>\n");
}

fn push_missing_section(out: &mut String, heading: &str, reason: &str) {
    let _ = write!(
        out,
        r#"<section><h2>{heading}</h2>
<div class="card">
  <div class="card-title">
    <span>{heading}</span>
    <span class="badge muted">SKIPPED</span>
  </div>
  <p class="muted">{reason}</p>
</div>
</section>
"#,
        heading = html_escape(heading),
        reason = html_escape(reason),
    );
}

fn push_row(out: &mut String, key: &str, value_html: &str) {
    let _ = writeln!(
        out,
        "    <tr><th style=\"text-align: right; width: 220px;\">{}</th><td>{}</td></tr>",
        html_escape(key),
        value_html,
    );
}

fn format_cf_list(cfs: &[String]) -> String {
    if cfs.is_empty() {
        "—".to_string()
    } else {
        cfs.iter()
            .map(|cf| format!("<code class=\"mono\">{}</code>", html_escape(cf)))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

// ---------------------------------------------------------------------------
// Test
// ---------------------------------------------------------------------------

#[test]
#[ignore = "manual: requires AGGLAYER_MIGRATION_SNAPSHOT_DATA_DIR pointing to an agglayer storage \
            data directory (the parent of pending/, debug/, epochs/)"]
fn full_data_dir_snapshot_validates_all_stores() {
    let raw = std::env::var_os(DATA_DIR_ENV).unwrap_or_else(|| {
        panic!(
            "{DATA_DIR_ENV} is not set; this test only runs against a manually provided data \
             directory. See module docs for usage."
        )
    });
    let data_dir = PathBuf::from(raw);
    if !data_dir.is_dir() {
        panic!("{DATA_DIR_ENV} must be an existing directory ({data_dir:?} is not)");
    }

    let epoch_limit = std::env::var(EPOCH_LIMIT_ENV)
        .ok()
        .and_then(|s| s.parse::<u64>().ok());
    let skip_epochs = env_flag(SKIP_EPOCHS_ENV);
    let env_label = std::env::var(ENV_LABEL_ENV)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| {
            data_dir
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("snapshot")
                .to_string()
        });

    eprintln!("[full] data dir: {data_dir:?}");
    eprintln!("[full] env label: {env_label}");
    if skip_epochs {
        eprintln!("[full] skip-epochs flag set via {SKIP_EPOCHS_ENV}");
    } else if let Some(n) = epoch_limit {
        eprintln!("[full] epoch limit (via {EPOCH_LIMIT_ENV}): first {n} epochs");
    }

    let report = validate_data_dir(&data_dir, epoch_limit, skip_epochs, env_label.clone());

    let html_path = match write_html(&report) {
        Ok(path) => path,
        Err(e) => panic!("failed to write HTML report: {e}"),
    };
    eprintln!("[full] HTML report: {}", html_path.display());

    let md_path = match write_markdown(&report) {
        Ok(path) => path,
        Err(e) => panic!("failed to write Markdown report: {e}"),
    };
    eprintln!("[full] Markdown report: {}", md_path.display());

    // Surface per-store fatals as a final test failure so CI/dev sees a
    // failed test alongside the report. Skip-only outcomes do NOT fail
    // the test (matching the migration's skip-with-log semantics).
    let mut fatals: Vec<String> = Vec::new();
    if let Some(p) = &report.pending {
        if let Some(e) = &p.fatal_error {
            fatals.push(format!("pending: {e}"));
        }
    }
    if let Some(d) = &report.debug {
        if let Some(e) = &d.fatal_error {
            fatals.push(format!("debug: {e}"));
        }
    }
    if report.epochs.failed > 0 {
        fatals.push(format!(
            "{} epoch(s) failed; see HTML report and table above",
            report.epochs.failed
        ));
    }
    if !fatals.is_empty() {
        panic!(
            "[full] {} fatal store outcome(s):\n  - {}\n(see HTML report at {})",
            fatals.len(),
            fatals.join("\n  - "),
            html_path.display(),
        );
    }
}

fn write_html(report: &FullSweepReport) -> std::io::Result<PathBuf> {
    let dir = report_dir()?;
    let path = dir.join(format!(
        "agglayer-migration-snapshot-report-{}.html",
        sanitize_label(&report.env_label)
    ));
    fs::write(&path, render_full_html_report(report))?;
    Ok(path)
}

fn write_markdown(report: &FullSweepReport) -> std::io::Result<PathBuf> {
    let dir = report_dir()?;
    let path = dir.join(format!(
        "agglayer-migration-snapshot-report-{}.md",
        sanitize_label(&report.env_label)
    ));
    fs::write(&path, render_markdown_report(report))?;
    Ok(path)
}

fn report_dir() -> std::io::Result<PathBuf> {
    let dir = std::env::var_os(REPORT_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

/// Make `label` filesystem-safe: keep alphanumerics, dash and underscore;
/// replace anything else with `_`. Stops the env-label from creating
/// directory traversal or weird filenames.
fn sanitize_label(label: &str) -> String {
    label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Markdown rendering (compact, concatenable across environments)
// ---------------------------------------------------------------------------

/// Render a small markdown summary of the sweep, intended to be
/// concatenated across multiple environments (`mainnet`, `testnet`,
/// `devnet`, …) into one document. Every fragment uses `##` for its
/// section title so callers can wrap with their own `#` heading. Fragments
/// end with a `---` horizontal rule for visual separation.
fn render_markdown_report(report: &FullSweepReport) -> String {
    let mut out = String::with_capacity(2 * 1024);
    let status = report.overall_status();

    let _ = writeln!(out, "## {}", report.env_label);
    out.push('\n');
    let _ = writeln!(
        out,
        "Run at {} UTC, total {}. Status: **{}**.",
        format_system_time(report.started_at),
        format_duration(Some(report.overall_duration)),
        status.label(),
    );
    out.push('\n');

    // Per-store one-line summary.
    out.push_str("| Store | Status | Validated | Skipped | Time | Notes |\n");
    out.push_str("|---|---|---:|---:|---:|---|\n");
    push_md_store_row(&mut out, "pending", report.pending.as_ref());
    push_md_store_row(&mut out, "debug", report.debug.as_ref());
    push_md_state_row(&mut out, report.state.as_ref());
    push_md_epochs_row(
        &mut out,
        &report.epochs,
        report.epoch_limit,
        report.epochs_skipped_reason,
    );
    out.push('\n');

    // Skip-status breakdown across pending + debug. Empty when nothing
    // was skipped.
    let mut breakdown_lines = Vec::new();
    if let Some(p) = report.pending.as_ref() {
        if let Some(line) = md_breakdown_line("pending", &p.skipped) {
            breakdown_lines.push(line);
        }
    }
    if let Some(d) = report.debug.as_ref() {
        if let Some(line) = md_breakdown_line("debug", &d.skipped) {
            breakdown_lines.push(line);
        }
    }
    if !breakdown_lines.is_empty() {
        out.push_str("**Skip analysis**\n\n");
        for line in breakdown_lines {
            out.push_str(&line);
            out.push('\n');
        }
        out.push('\n');
    }

    // Fatal errors, one-liner each. Multi-line CodecError messages are
    // squashed so the markdown stays compact.
    let mut fatals = Vec::new();
    for (label, store) in [
        ("pending", report.pending.as_ref()),
        ("debug", report.debug.as_ref()),
    ] {
        if let Some(s) = store {
            if let Some(err) = &s.fatal_error {
                fatals.push(format!("- {label}: {}", squash_whitespace(err)));
            }
        }
    }
    if let Some(s) = report.state.as_ref() {
        if let Some(err) = &s.fatal_error {
            fatals.push(format!("- state (read-only): {}", squash_whitespace(err)));
        }
        if let Some(m) = &s.migration_outcome {
            if let Some(err) = &m.error {
                fatals.push(format!(
                    "- state (StateStore::init_db): {}",
                    squash_whitespace(err)
                ));
            }
        }
    }
    if report.epochs.failed > 0 {
        fatals.push(format!("- epochs: {} failed", report.epochs.failed));
    }
    if !fatals.is_empty() {
        out.push_str("**Fatal errors**\n\n");
        for f in fatals {
            out.push_str(&f);
            out.push('\n');
        }
        out.push('\n');
    }

    out.push_str("---\n");
    out
}

fn push_md_store_row(out: &mut String, label: &str, store: Option<&StoreOutcome>) {
    match store {
        None => {
            let _ = writeln!(out, "| {label} | — | — | — | — | not present in data dir |");
        }
        Some(s) => {
            let status = s.classify();
            let notes = if let Some(err) = &s.fatal_error {
                format!("FAILED: {}", squash_whitespace(err))
            } else if !s.skipped.is_empty() {
                let breakdown = build_status_breakdown(&s.skipped);
                breakdown
                    .iter()
                    .map(|(k, v)| format!("{v}× {k}"))
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                "—".to_string()
            };
            let _ = writeln!(
                out,
                "| {label} | {emoji} {st} | {val} | {skp} | {dur} | {notes} |",
                emoji = status_emoji(status),
                st = status.label(),
                val = s.validated,
                skp = s.skipped.len(),
                dur = format_duration(Some(s.duration)),
            );
        }
    }
}

fn push_md_state_row(out: &mut String, state: Option<&StateDbInfo>) {
    match state {
        None => {
            out.push_str("| state | — | — | — | — | not present in data dir |\n");
        }
        Some(info) => {
            let migration_ok = info
                .migration_outcome
                .as_ref()
                .map(|o| o.error.is_none())
                .unwrap_or(true);
            let read_ok = info.fatal_error.is_none();
            let status = if read_ok && migration_ok {
                Status::Ok
            } else {
                Status::Failed
            };
            let header_count = info
                .certificate_header_count
                .map(|n| n.to_string())
                .unwrap_or_else(|| "—".to_string());
            let mut notes = vec![format!("{header_count} headers")];
            if read_ok {
                notes.push("read-only OK".to_string());
            }
            if let Some(m) = &info.migration_outcome {
                if m.error.is_none() {
                    notes.push(format!(
                        "init_db OK ({})",
                        format_duration(Some(m.duration))
                    ));
                } else {
                    notes.push("init_db FAIL".to_string());
                }
            }
            let _ = writeln!(
                out,
                "| state | {emoji} {st} | — | — | {dur} | {notes} |",
                emoji = status_emoji(status),
                st = status.label(),
                dur = format_duration(Some(info.duration)),
                notes = notes.join("; "),
            );
        }
    }
}

fn push_md_epochs_row(
    out: &mut String,
    epochs: &EpochsOutcome,
    limit: Option<u64>,
    skipped_reason: Option<&'static str>,
) {
    if let Some(reason) = skipped_reason {
        let _ = writeln!(
            out,
            "| epochs | — | — | — | — | {} |",
            squash_whitespace(reason)
        );
        return;
    }
    if epochs.epochs_dir.is_none() {
        out.push_str("| epochs | — | — | — | — | not present in data dir |\n");
        return;
    }
    let status = epochs.classify();
    let notes = match limit {
        Some(n) if epochs.skipped_due_to_limit > 0 => format!(
            "{} processed (cap {n}); {} clean / {} skips / {} fail",
            epochs.processed, epochs.successful, epochs.with_skips, epochs.failed
        ),
        _ => format!(
            "{} processed; {} clean / {} skips / {} fail",
            epochs.processed, epochs.successful, epochs.with_skips, epochs.failed
        ),
    };
    let _ = writeln!(
        out,
        "| epochs | {emoji} {st} | {val} | {skp} | {dur} | {notes} |",
        emoji = status_emoji(status),
        st = status.label(),
        val = epochs.total_validated,
        skp = epochs.total_skipped,
        dur = format_duration(Some(epochs.duration)),
    );
}

fn md_breakdown_line(label: &str, skipped: &[SkippedRow]) -> Option<String> {
    if skipped.is_empty() {
        return None;
    }
    let breakdown = build_status_breakdown(skipped);
    if breakdown.is_empty() {
        return None;
    }
    let summary = breakdown
        .iter()
        .map(|(k, v)| format!("{v}× `{k}`"))
        .collect::<Vec<_>>()
        .join(", ");
    Some(format!("- {label} ({} skipped): {summary}", skipped.len()))
}

fn status_emoji(status: Status) -> &'static str {
    match status {
        Status::Ok => "✅",
        Status::Skips => "⚠️",
        Status::Failed => "❌",
    }
}
