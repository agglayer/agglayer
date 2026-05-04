//! Manual validation of the proto-backed CF migration against a real DB
//! snapshot.
//!
//! Tests in this module are `#[ignore]` by default. They are intended to be
//! run on demand by an operator who points `AGGLAYER_MIGRATION_SNAPSHOT_*`
//! environment variables at copies of real production databases. No data is
//! committed to the repository: each test reads from the env var and stages
//! into a per-test tempdir, so the operator's source snapshot is never
//! mutated.
//!
//! ## Usage
//!
//! `--run-ignored` requires an explicit value (`only`, `all`, or `default`).
//! Use `only` to skip the regular test suite and run just the snapshot
//! validators:
//!
//! ```sh
//! AGGLAYER_MIGRATION_SNAPSHOT_PENDING=/path/to/pending-db \
//! AGGLAYER_MIGRATION_SNAPSHOT_EPOCH=/path/to/single-epoch-db \
//! AGGLAYER_MIGRATION_SNAPSHOT_DEBUG=/path/to/debug-db \
//! cargo nextest run -p agglayer-storage --run-ignored only \
//!     --no-capture stores::migration_snapshot
//! ```
//!
//! `--no-capture` is recommended because the validators emit progress logs
//! on stderr that nextest swallows by default.
//!
//! ## HTML report
//!
//! Each test writes a self-contained HTML report at the end (success or
//! failure). The output directory defaults to the system temp dir; override
//! with:
//!
//! ```sh
//! AGGLAYER_MIGRATION_SNAPSHOT_REPORT_DIR=/path/to/reports
//! ```
//!
//! Filenames are `agglayer-migration-snapshot-report-<label>.html`. Each
//! report shows: per-store status, source/staging paths, CF lists before
//! and after migration, copy/migration/validation timings, validated and
//! skipped counts, and a table of every skipped row with its key and
//! decode error. The report is written via a `Drop` guard, so it is
//! produced even when the validator panics on a fatal error.
//!
//! ## Expected snapshot layout
//!
//! Each env var must point at the **rocksdb directory itself**, not at the
//! parent agglayer data directory. A typical agglayer node layout is:
//!
//! ```text
//! <data-dir>/storage/
//!     pending/      <-- AGGLAYER_MIGRATION_SNAPSHOT_PENDING
//!     epochs/
//!         0/        <-- AGGLAYER_MIGRATION_SNAPSHOT_EPOCH (single epoch)
//!         1/
//!         ...
//!     debug/        <-- AGGLAYER_MIGRATION_SNAPSHOT_DEBUG
//!     state/        (not migrated by this PR)
//!     metadata/     (not migrated by this PR)
//! ```
//!
//! ## What is validated
//!
//! 1. The snapshot is staged into a fresh tempdir so the source is never
//!    mutated.
//! 2. The store's `init_db` is invoked, which runs the proto migration. If the
//!    migration aborts on a non-codec error this fails fast with the underlying
//!    `DBOpenError`. (Codec errors no longer abort migration; they are logged
//!    and the row is skipped.)
//! 3. The legacy CF (still intact post-migration) is iterated; for every legacy
//!    key the proto CF is checked for an equivalent row whose decoded
//!    `Certificate` matches `Certificate::from(legacy)`. Rows whose bytes
//!    cannot be decoded mirror the migration's skip behavior and are recorded
//!    in the report.

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use agglayer_types::Certificate;

use crate::{
    columns::{
        debug_certificates::{DebugCertificatesColumn, DebugCertificatesProtoColumn},
        epochs::certificates::{CertificatePerIndexColumn, CertificatePerIndexProtoColumn},
        pending_queue::{PendingQueueColumn, PendingQueueProtoColumn},
    },
    schema::ColumnSchema,
    storage::{DBError, DBOpenError, DB},
    stores::{debug::DebugStore, pending::PendingStore, per_epoch::PerEpochStore},
    tests::TempDBDir,
    types::LegacyCertificate,
};

const PENDING_ENV: &str = "AGGLAYER_MIGRATION_SNAPSHOT_PENDING";
const EPOCH_ENV: &str = "AGGLAYER_MIGRATION_SNAPSHOT_EPOCH";
const DEBUG_ENV: &str = "AGGLAYER_MIGRATION_SNAPSHOT_DEBUG";

pub(super) const REPORT_DIR_ENV: &str = "AGGLAYER_MIGRATION_SNAPSHOT_REPORT_DIR";

mod full_sweep;

/// How many validated rows between progress lines.
const PROGRESS_EVERY: usize = 1_000;

// ---------------------------------------------------------------------------
// Validation report
// ---------------------------------------------------------------------------

/// Aggregated outcome of one store's snapshot validation.
struct ValidationReport {
    label: String,
    env_var: String,
    started_at: SystemTime,
    source_path: Option<PathBuf>,
    source_listing: Vec<String>,
    staging_path: Option<PathBuf>,
    pre_migration_cfs: Vec<String>,
    post_migration_cfs: Vec<String>,
    copy_duration: Option<Duration>,
    migration_duration: Option<Duration>,
    validation_duration: Option<Duration>,
    validated_count: usize,
    skipped_rows: Vec<SkippedRow>,
    /// Set when the test fails before validation completes (env-var missing,
    /// snapshot not a rocksdb directory, missing legacy CF, init_db error,
    /// etc).
    fatal_error: Option<String>,
}

pub(super) struct SkippedRow {
    pub(super) key: String,
    pub(super) error: String,
    /// Optional cross-reference enrichment populated by the full-sweep
    /// validator: where possible, the analyzer looks up the corresponding
    /// `CertificateHeader` in the state DB so the report can show whether
    /// the unparsable row corresponds to a known certificate and what its
    /// status is.
    pub(super) analysis: Option<SkipAnalysis>,
}

/// Cross-reference info for an unparsable row, derived by looking up the
/// state DB at sweep time. `None` fields mean the lookup failed at that
/// step.
#[derive(Default)]
pub(super) struct SkipAnalysis {
    /// Source of the lookup: which state CF was consulted.
    pub(super) source: &'static str,
    /// CertificateId resolved from the unparsable key (hex). For debug
    /// rows this is the row's key directly; for pending rows it's the
    /// result of looking up `(network_id, height)` in
    /// `certificate_per_network_cf`.
    pub(super) certificate_id: Option<String>,
    /// `Some(CertificateStatus)` formatted as a string when the header
    /// was found; `None` when the certificate header is absent from the
    /// state DB (e.g. the row references an id never settled).
    pub(super) header_status: Option<String>,
    /// Network ID extracted from the header (debug rows) or the lookup
    /// key (pending rows), useful for cross-referencing.
    pub(super) network_id: Option<u32>,
    /// Height extracted from the header (debug rows) or the lookup key
    /// (pending rows).
    pub(super) height: Option<u64>,
    /// Free-form note describing why the analysis is incomplete (e.g.
    /// state DB was not opened, or no `certificate_per_network` row).
    pub(super) note: Option<String>,
}

/// Pessimistic default for `fatal_error`: if the test panics before
/// reaching the `mark_complete` call at the end of `run_store_validation`,
/// the report reflects a failure. Specific failure sites overwrite this
/// with a more informative message before panicking.
const INCOMPLETE_RUN_MESSAGE: &str = "validation did not complete; the test panicked before \
                                      finishing. See the captured stderr for the panic message.";

impl ValidationReport {
    fn new(label: &str, env_var: &str) -> Self {
        Self {
            label: label.to_owned(),
            env_var: env_var.to_owned(),
            started_at: SystemTime::now(),
            source_path: None,
            source_listing: Vec::new(),
            staging_path: None,
            pre_migration_cfs: Vec::new(),
            post_migration_cfs: Vec::new(),
            copy_duration: None,
            migration_duration: None,
            validation_duration: None,
            validated_count: 0,
            skipped_rows: Vec::new(),
            fatal_error: Some(INCOMPLETE_RUN_MESSAGE.to_string()),
        }
    }

    /// Mark a run as having completed successfully end-to-end. Cleared at
    /// the very end of `run_store_validation`; if the run panics before that
    /// point, the pessimistic default in `new` keeps the report's status as
    /// failed.
    fn mark_complete(&mut self) {
        self.fatal_error = None;
    }

    fn is_success(&self) -> bool {
        self.fatal_error.is_none()
    }

    fn has_skips(&self) -> bool {
        !self.skipped_rows.is_empty()
    }
}

/// `Drop` guard that writes the HTML report when the test scope ends. Using
/// `Drop` here means the report is produced even if the test panics inside
/// `assert!` or unwraps a fatal error.
struct ReportGuard {
    report: ValidationReport,
}

impl Drop for ReportGuard {
    fn drop(&mut self) {
        match write_html_report(&self.report) {
            Ok(path) => eprintln!("[{}] HTML report: {}", self.report.label, path.display()),
            Err(e) => eprintln!("[{}] failed to write HTML report: {e}", self.report.label),
        }
    }
}

// ---------------------------------------------------------------------------
// HTML rendering
// ---------------------------------------------------------------------------

fn write_html_report(report: &ValidationReport) -> std::io::Result<PathBuf> {
    let dir = std::env::var_os(REPORT_DIR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    fs::create_dir_all(&dir)?;
    let path = dir.join(format!(
        "agglayer-migration-snapshot-report-{}.html",
        report.label
    ));
    fs::write(&path, render_html(report))?;
    Ok(path)
}

fn render_html(report: &ValidationReport) -> String {
    let (status_label, status_class) = if !report.is_success() {
        ("FAILED", "err")
    } else if report.has_skips() {
        ("OK with skipped rows", "warn")
    } else {
        ("OK", "ok")
    };

    let mut out = String::new();
    let _ = write!(
        out,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Migration snapshot report — {label}</title>
<style>
  body {{ font-family: system-ui, -apple-system, sans-serif; max-width: 1100px; margin: 2em auto; padding: 0 1em; color: #1a1a1a; line-height: 1.5; }}
  h1, h2 {{ border-bottom: 1px solid #d0d7de; padding-bottom: 0.3em; }}
  code {{ background: #f6f8fa; padding: 0.1em 0.3em; border-radius: 3px; }}
  pre {{ background: #f6f8fa; padding: 1em; border-radius: 6px; overflow-x: auto; font-size: 0.9em; }}
  table {{ border-collapse: collapse; width: 100%; margin: 1em 0; }}
  th, td {{ border: 1px solid #d0d7de; padding: 0.4em 0.8em; text-align: left; vertical-align: top; }}
  th {{ background: #f6f8fa; font-weight: 600; }}
  tbody tr:nth-child(odd) {{ background: #f9f9fb; }}
  td.mono, code.mono {{ font-family: ui-monospace, monospace; word-break: break-all; }}
  .badge {{ display: inline-block; padding: 0.2em 0.7em; border-radius: 12px; font-size: 0.85em; font-weight: 600; }}
  .badge.ok {{ background: #dafbe1; color: #1a7f37; }}
  .badge.warn {{ background: #fff8c5; color: #9a6700; }}
  .badge.err {{ background: #ffebe9; color: #cf222e; }}
  .summary-row {{ display: flex; gap: 1em; margin: 1em 0; flex-wrap: wrap; }}
  .summary-card {{ flex: 1 1 200px; padding: 1em; border: 1px solid #d0d7de; border-radius: 6px; background: #f6f8fa; }}
  .summary-card h3 {{ margin: 0 0 0.3em 0; font-size: 0.9em; text-transform: uppercase; color: #57606a; letter-spacing: 0.04em; }}
  .summary-card .number {{ font-size: 2.4em; font-weight: 700; line-height: 1; }}
  .summary-card .number.ok {{ color: #1a7f37; }}
  .summary-card .number.warn {{ color: #9a6700; }}
  .summary-card .number.err {{ color: #cf222e; }}
  .meta {{ color: #57606a; font-size: 0.9em; }}
  details summary {{ cursor: pointer; }}
</style>
</head>
<body>
  <h1>Migration snapshot report — {label}</h1>
  <p>
    <span class="badge {status_class}">{status_label}</span>
    <span class="meta">generated {generated} (UTC)</span>
  </p>

  <h2>Source</h2>
  <table>
    <tbody>
      <tr><th>Env var</th><td><code>{env_var}</code></td></tr>
      <tr><th>Source path</th><td><code class="mono">{source_path}</code></td></tr>
      <tr><th>Staging path</th><td><code class="mono">{staging_path}</code></td></tr>
    </tbody>
  </table>
  <details>
    <summary>Source directory listing ({src_count} entries)</summary>
    <pre>{source_listing}</pre>
  </details>

  <h2>Column families</h2>
  <table>
    <thead><tr><th>Phase</th><th>Column families</th></tr></thead>
    <tbody>
      <tr><th>Before migration</th><td><code class="mono">{pre_cfs}</code></td></tr>
      <tr><th>After migration</th><td><code class="mono">{post_cfs}</code></td></tr>
    </tbody>
  </table>

  <h2>Counts</h2>
  <div class="summary-row">
    <div class="summary-card"><h3>Validated</h3><div class="number ok">{validated}</div></div>
    <div class="summary-card"><h3>Skipped</h3><div class="number {skipped_class}">{skipped}</div></div>
  </div>

  <h2>Timings</h2>
  <table>
    <tbody>
      <tr><th>Snapshot copy</th><td>{copy_dur}</td></tr>
      <tr><th>Migration (init_db)</th><td>{mig_dur}</td></tr>
      <tr><th>Validation</th><td>{val_dur}</td></tr>
    </tbody>
  </table>
"#,
        label = html_escape(&report.label),
        status_label = status_label,
        status_class = status_class,
        generated = format_system_time(report.started_at),
        env_var = html_escape(&report.env_var),
        source_path = html_escape(&path_or_dash(&report.source_path)),
        staging_path = html_escape(&path_or_dash(&report.staging_path)),
        src_count = report.source_listing.len(),
        source_listing = html_escape(&report.source_listing.join("\n")),
        pre_cfs = html_escape(&format_cfs(&report.pre_migration_cfs)),
        post_cfs = html_escape(&format_cfs(&report.post_migration_cfs)),
        validated = report.validated_count,
        skipped = report.skipped_rows.len(),
        skipped_class = if report.has_skips() { "warn" } else { "ok" },
        copy_dur = format_duration(report.copy_duration),
        mig_dur = format_duration(report.migration_duration),
        val_dur = format_duration(report.validation_duration),
    );

    if !report.skipped_rows.is_empty() {
        let _ = writeln!(out, "<h2>Skipped rows ({})</h2>", report.skipped_rows.len());
        out.push_str(
            "<p>These legacy rows could not be decoded as either bincode or proto and were \
             skipped by the migration. Investigate the source rows in the legacy CF before the \
             follow-up cleanup drops them.</p>\n",
        );
        out.push_str(
            "<table>\n<thead><tr><th>#</th><th>Key</th><th>Error</th></tr></thead>\n<tbody>\n",
        );
        for (i, row) in report.skipped_rows.iter().enumerate() {
            let _ = writeln!(
                out,
                "  <tr><td>{}</td><td class=\"mono\">{}</td><td>{}</td></tr>",
                i + 1,
                html_escape(&row.key),
                html_escape(&row.error),
            );
        }
        out.push_str("</tbody>\n</table>\n");
    }

    if let Some(err) = &report.fatal_error {
        let _ = writeln!(out, "<h2>Fatal error</h2>\n<pre>{}</pre>", html_escape(err));
    }

    out.push_str("</body>\n</html>\n");
    out
}

pub(super) fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

fn path_or_dash(path: &Option<PathBuf>) -> String {
    match path {
        Some(p) => p.display().to_string(),
        None => "—".to_string(),
    }
}

fn format_cfs(cfs: &[String]) -> String {
    if cfs.is_empty() {
        "—".to_string()
    } else {
        cfs.join(", ")
    }
}

pub(super) fn format_duration(d: Option<Duration>) -> String {
    match d {
        Some(d) => format!("{:.2?}", d),
        None => "—".to_string(),
    }
}

pub(super) fn format_system_time(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    // Compact ISO-8601-ish formatter without pulling in chrono.
    let (y, m, d, hh, mm, ss) = unix_to_ymd_hms(secs);
    format!("{y:04}-{m:02}-{d:02} {hh:02}:{mm:02}:{ss:02}")
}

fn unix_to_ymd_hms(mut secs: i64) -> (i32, u32, u32, u32, u32, u32) {
    const SECS_PER_DAY: i64 = 86_400;
    let days = secs.div_euclid(SECS_PER_DAY);
    secs = secs.rem_euclid(SECS_PER_DAY);
    let hh = (secs / 3600) as u32;
    let mm = ((secs % 3600) / 60) as u32;
    let ss = (secs % 60) as u32;

    // Howard Hinnant's civil_from_days, returns (year, month, day).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = (yoe + era * 400) as i32;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = (if mp < 10 { mp + 3 } else { mp - 9 }) as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d, hh, mm, ss)
}

// ---------------------------------------------------------------------------
// Snapshot helpers
// ---------------------------------------------------------------------------

fn require_snapshot(env_var: &str, label: &str, report: &mut ValidationReport) -> PathBuf {
    let raw = match std::env::var_os(env_var) {
        Some(raw) => raw,
        None => {
            let msg = format!(
                "{env_var} is not set; this test only runs against a manually provided DB \
                 snapshot. See module docs for usage."
            );
            report.fatal_error = Some(msg.clone());
            panic!("[{label}] {msg}");
        }
    };
    let path = PathBuf::from(raw);
    if !path.is_dir() {
        let msg = format!(
            "{env_var} must point to an existing directory ({path:?} is not a directory). \
             Expected a rocksdb directory with files like CURRENT/MANIFEST-*/*.sst."
        );
        report.fatal_error = Some(msg.clone());
        panic!("[{label}] {msg}");
    }
    path
}

/// Print top-level files/directories and return the listing for the report.
fn collect_dir_listing(path: &Path, label: &str) -> Vec<String> {
    eprintln!("[{label}] source listing of {path:?}:");
    match fs::read_dir(path) {
        Ok(entries) => {
            let mut names: Vec<_> = entries
                .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().into_owned()))
                .collect();
            names.sort();
            if names.is_empty() {
                eprintln!("[{label}]   (empty directory)");
            } else {
                for name in &names {
                    eprintln!("[{label}]   {name}");
                }
            }
            names
        }
        Err(e) => {
            eprintln!("[{label}]   <read_dir failed: {e}>");
            Vec::new()
        }
    }
}

fn list_existing_cfs(path: &Path, label: &str) -> Option<Vec<String>> {
    match rocksdb::DB::list_cf(&rocksdb::Options::default(), path) {
        Ok(mut cfs) => {
            cfs.sort();
            eprintln!(
                "[{label}] existing column families ({}): {cfs:?}",
                cfs.len()
            );
            Some(cfs)
        }
        Err(e) => {
            eprintln!(
                "[{label}] rocksdb::DB::list_cf failed for {path:?}: {e}\n[{label}]   (this \
                 usually means the directory is not a rocksdb DB, or the rocksdb files are \
                 missing/corrupt)"
            );
            None
        }
    }
}

/// Recursive directory copy. Skips symlinks; production rocksdb databases do
/// not contain them.
pub(super) fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if entry_type.is_dir() {
            copy_dir_recursive(&entry.path(), &dst_path)?;
        } else if entry_type.is_file() {
            fs::copy(entry.path(), &dst_path)?;
        }
    }
    Ok(())
}

fn stage_snapshot(src: &Path, label: &str) -> (TempDBDir, PathBuf) {
    let staging = TempDBDir::new();
    eprintln!("[{label}] staging snapshot to {:?}", staging.path);
    copy_dir_recursive(src, &staging.path).expect("copy snapshot to staging tempdir");
    let path = staging.path.clone();
    (staging, path)
}

pub(super) fn format_error_chain(error: &dyn std::error::Error) -> String {
    let mut out = format!("{error}");
    let mut next = error.source();
    while let Some(cur) = next {
        out.push_str(" -> ");
        out.push_str(&cur.to_string());
        next = cur.source();
    }
    out
}

/// Iterate the legacy CF and validate every row against the proto CF,
/// recording outcomes into `report`.
fn validate_into_report<L, P>(db: &DB, label: &str, report: &mut ValidationReport)
where
    L: ColumnSchema<Value = LegacyCertificate>,
    P: ColumnSchema<Value = Certificate, Key = L::Key>,
    L::Key: std::fmt::Debug,
{
    eprintln!("[{label}] iterating legacy CF and validating each proto row");
    let started = Instant::now();
    for key in db.keys::<L>().expect("iterate legacy keys") {
        let key = key.expect("decode legacy key");

        let legacy = match db.get::<L>(&key) {
            Ok(Some(legacy)) => legacy,
            Ok(None) => {
                eprintln!("[{label}]   key {key:?} vanished between iterate and read; skipping");
                continue;
            }
            Err(DBError::CodecError(e)) => {
                eprintln!("[{label}]   skipping unparsable legacy row at key {key:?}: {e}");
                report.skipped_rows.push(SkippedRow {
                    key: format!("{key:?}"),
                    error: e.to_string(),
                    analysis: None,
                });
                continue;
            }
            Err(other) => panic!("[{label}] read legacy row at key {key:?}: {other}"),
        };

        let proto = db.get::<P>(&key).expect("read proto row");
        match proto {
            Some(proto) => assert_eq!(
                proto,
                Certificate::from(legacy),
                "{label}: decoded certificates differ for key {key:?}"
            ),
            None => panic!(
                "{label}: proto CF is missing a row for key {key:?} that exists in the legacy CF"
            ),
        }
        report.validated_count += 1;
        if report.validated_count.is_multiple_of(PROGRESS_EVERY) {
            eprintln!(
                "[{label}]   progress: {validated} rows validated ({:.2?} elapsed)",
                started.elapsed(),
                validated = report.validated_count,
            );
        }
    }
    let elapsed = started.elapsed();
    report.validation_duration = Some(elapsed);
    eprintln!(
        "[{label}] DONE: validated {} certificates, skipped {} unparsable rows in {:.2?}",
        report.validated_count,
        report.skipped_rows.len(),
        elapsed
    );
}

// ---------------------------------------------------------------------------
// Per-store driver
// ---------------------------------------------------------------------------

fn run_store_validation<L, P>(
    label: &'static str,
    env_var: &'static str,
    expected_legacy_cf: &str,
    init_db: impl FnOnce(&Path) -> Result<DB, DBOpenError>,
    report: &mut ValidationReport,
) where
    L: ColumnSchema<Value = LegacyCertificate>,
    P: ColumnSchema<Value = Certificate, Key = L::Key>,
    L::Key: std::fmt::Debug,
{
    let src = require_snapshot(env_var, label, report);
    eprintln!("[{label}] env {env_var} = {src:?}");
    report.source_path = Some(src.clone());

    report.source_listing = collect_dir_listing(&src, label);

    let cfs = match list_existing_cfs(&src, label) {
        Some(cfs) => {
            report.pre_migration_cfs = cfs.clone();
            cfs
        }
        None => {
            let err = format!("not a rocksdb directory: {src:?}");
            report.fatal_error = Some(err.clone());
            panic!("[{label}] {err}");
        }
    };
    if !cfs.iter().any(|cf| cf == expected_legacy_cf) {
        let err = format!(
            "{src:?} is a rocksdb directory but does not contain the expected legacy CF \
             {expected_legacy_cf:?}; found {cfs:?}"
        );
        report.fatal_error = Some(err.clone());
        panic!("[{label}] {err}");
    }

    let copy_started = Instant::now();
    let (_staging, path) = stage_snapshot(&src, label);
    report.copy_duration = Some(copy_started.elapsed());
    report.staging_path = Some(path.clone());
    eprintln!(
        "[{label}] copy complete in {:.2?}",
        report.copy_duration.unwrap()
    );

    eprintln!("[{label}] running init_db (this triggers the proto migration)");
    let mig_started = Instant::now();
    let db = match init_db(&path) {
        Ok(db) => {
            report.migration_duration = Some(mig_started.elapsed());
            eprintln!(
                "[{label}] init_db succeeded in {:.2?}",
                report.migration_duration.unwrap()
            );
            db
        }
        Err(e) => {
            report.migration_duration = Some(mig_started.elapsed());
            let chain = format_error_chain(&e);
            report.fatal_error = Some(format!("init_db failed: {chain}"));
            panic!("[{label}] init_db failed: {chain}");
        }
    };

    if let Some(post) = list_existing_cfs(&path, label) {
        report.post_migration_cfs = post;
    }

    validate_into_report::<L, P>(&db, label, report);

    // Reaching this line means everything completed without panicking;
    // clear the pessimistic default fatal error.
    report.mark_complete();
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[test]
#[ignore = "manual: requires AGGLAYER_MIGRATION_SNAPSHOT_PENDING pointing to a DB snapshot"]
fn pending_snapshot_migrates_and_proto_matches_legacy() {
    let mut guard = ReportGuard {
        report: ValidationReport::new("pending", PENDING_ENV),
    };
    run_store_validation::<PendingQueueColumn, PendingQueueProtoColumn>(
        "pending",
        PENDING_ENV,
        PendingQueueColumn::COLUMN_FAMILY_NAME,
        PendingStore::init_db,
        &mut guard.report,
    );
}

#[test]
#[ignore = "manual: requires AGGLAYER_MIGRATION_SNAPSHOT_EPOCH pointing to a single-epoch DB \
            snapshot"]
fn epoch_snapshot_migrates_and_proto_matches_legacy() {
    let mut guard = ReportGuard {
        report: ValidationReport::new("epoch", EPOCH_ENV),
    };
    run_store_validation::<CertificatePerIndexColumn, CertificatePerIndexProtoColumn>(
        "epoch",
        EPOCH_ENV,
        CertificatePerIndexColumn::COLUMN_FAMILY_NAME,
        // `PerEpochStore::init_db` does not depend on its type parameters;
        // supply unit types just to disambiguate the impl block.
        PerEpochStore::<(), ()>::init_db,
        &mut guard.report,
    );
}

#[test]
#[ignore = "manual: requires AGGLAYER_MIGRATION_SNAPSHOT_DEBUG pointing to a DB snapshot"]
fn debug_snapshot_migrates_and_proto_matches_legacy() {
    let mut guard = ReportGuard {
        report: ValidationReport::new("debug", DEBUG_ENV),
    };
    run_store_validation::<DebugCertificatesColumn, DebugCertificatesProtoColumn>(
        "debug",
        DEBUG_ENV,
        DebugCertificatesColumn::COLUMN_FAMILY_NAME,
        DebugStore::init_db,
        &mut guard.report,
    );
}
