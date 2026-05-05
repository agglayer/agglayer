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

use std::{
    fmt::Write as _,
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
    /// When set, the markdown report is written to this file path. When
    /// unset, the caller is expected to render the markdown itself
    /// (typically by calling [`render_markdown`] and printing to stdout).
    pub markdown_file: Option<PathBuf>,
    /// When set, the HTML report is written to this file path. When
    /// unset, no HTML is produced.
    pub html_file: Option<PathBuf>,
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

#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    #[error("failed to write Markdown report: {0}")]
    ReportWrite(#[source] std::io::Error),
}

/// Run the migration in place against the paths in `opts`. Per-store
/// failures are recorded in the outcome rather than propagated, so the
/// runner always produces a complete report; only IO failures while
/// writing the report itself bubble up as `MigrateError`.
pub fn run(opts: MigrateOptions) -> Result<MigrateOutcome, MigrateError> {
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

    let outcome = MigrateOutcome {
        started_at: started_clock,
        overall_duration: started.elapsed(),
        env_label: opts.env_label.clone(),
        state,
        pending,
        debug,
        epochs,
    };

    if let Some(path) = opts.markdown_file.as_deref() {
        write_to_file(path, &render_markdown(&outcome)).map_err(MigrateError::ReportWrite)?;
    }
    if let Some(path) = opts.html_file.as_deref() {
        write_to_file(path, &render_html(&outcome)).map_err(MigrateError::ReportWrite)?;
    }

    Ok(outcome)
}

/// Write `contents` to `path`, creating the parent directory if needed.
fn write_to_file(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, contents)
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

// ---------------------------------------------------------------------------
// Markdown rendering
// ---------------------------------------------------------------------------

/// Render a compact markdown report. Same shape as the snapshot
/// validator's markdown output (h2 heading, status table, fatal-error
/// block) so multiple environments can be concatenated into a single
/// document.
pub fn render_markdown(outcome: &MigrateOutcome) -> String {
    let mut out = String::with_capacity(2 * 1024);
    let _ = writeln!(out, "## {}", outcome.env_label);
    out.push('\n');
    let _ = writeln!(
        out,
        "Migration run at {} UTC, total {:.2?}. Status: **{}**.",
        format_time(outcome.started_at),
        outcome.overall_duration,
        if outcome.is_success() { "OK" } else { "FAILED" },
    );
    out.push('\n');

    out.push_str("| Store | Status | Duration | Notes |\n");
    out.push_str("|---|---|---:|---|\n");
    push_md_row(&mut out, outcome.state.as_ref(), "state");
    push_md_row(&mut out, outcome.pending.as_ref(), "pending");
    push_md_row(&mut out, outcome.debug.as_ref(), "debug");
    push_md_epochs_row(&mut out, &outcome.epochs);
    out.push('\n');

    if !outcome.is_success() {
        out.push_str("**Fatal errors**\n\n");
        for r in [
            outcome.state.as_ref(),
            outcome.pending.as_ref(),
            outcome.debug.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(err) = &r.error {
                let _ = writeln!(out, "- {}: {}", r.label, err);
            }
        }
        for f in &outcome.epochs.failed {
            let _ = writeln!(out, "- epoch {}: {}", f.epoch, f.error);
        }
        out.push('\n');
    }

    out.push_str("---\n");
    out
}

fn push_md_row(out: &mut String, store: Option<&StoreResult>, fallback_label: &str) {
    let Some(r) = store else {
        let _ = writeln!(
            out,
            "| {fallback_label} | — | — | not configured for this deployment |"
        );
        return;
    };
    let (status, notes) = match &r.error {
        None => ("✅ OK".to_string(), "—".to_string()),
        Some(e) => ("❌ FAILED".to_string(), e.clone()),
    };
    let _ = writeln!(
        out,
        "| {} | {} | {:.2?} | {} |",
        r.label, status, r.duration, notes
    );
}

fn push_md_epochs_row(out: &mut String, epochs: &EpochsResult) {
    if epochs.epochs_dir.is_none() {
        out.push_str("| epochs | — | — | not configured for this deployment |\n");
        return;
    }
    if let Some(reason) = epochs.skipped_reason {
        let _ = writeln!(out, "| epochs | — | — | {reason} |");
        return;
    }
    let status = if epochs.failed.is_empty() {
        "✅ OK"
    } else {
        "❌ FAILED"
    };
    let notes = format!(
        "{} processed; {} OK; {} failed",
        epochs.processed,
        epochs.successful,
        epochs.failed.len(),
    );
    let _ = writeln!(
        out,
        "| epochs | {} | {:.2?} | {} |",
        status, epochs.duration, notes
    );
}

/// Make `label` filesystem-safe: keep alphanumerics, dash and underscore;
/// replace anything else with `_`. Exported so callers (e.g. the CLI) can
/// derive predictable filenames from an environment label.
pub fn sanitize_env_label(label: &str) -> String {
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

fn format_time(t: SystemTime) -> String {
    use std::time::UNIX_EPOCH;
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
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
// HTML rendering (self-contained, no external resources)
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
  --success: 142 71% 35%;
  --warning: 38 92% 45%;
  --destructive: 0 72% 50%;
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
    --success: 142 60% 45%;
    --warning: 38 90% 55%;
    --destructive: 0 70% 60%;
  }
}
* { box-sizing: border-box; }
body {
  font-family: ui-sans-serif, system-ui, -apple-system, sans-serif;
  color: hsl(var(--foreground));
  background: hsl(var(--background));
  margin: 0;
  line-height: 1.5;
  font-size: 14px;
}
.container { max-width: 1100px; margin: 0 auto; padding: 2.5rem 1.5rem 4rem; }
header.page-header { margin-bottom: 2rem; }
header.page-header h1 { font-size: 1.875rem; font-weight: 600; letter-spacing: -0.02em; margin: 0 0 0.4rem; }
header.page-header .meta { color: hsl(var(--muted-foreground)); font-size: 0.875rem; display: flex; flex-wrap: wrap; gap: 0.4rem 1.25rem; }
h2 { font-size: 1.25rem; font-weight: 600; letter-spacing: -0.01em; margin: 2.5rem 0 1rem; }
code { font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace; background: hsl(var(--muted)); padding: 0.1em 0.35em; border-radius: 4px; font-size: 0.875em; }
.mono.break { word-break: break-all; }
.kpi-grid { display: grid; gap: 1rem; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); margin-bottom: 1rem; }
.kpi { background: hsl(var(--card)); border: 1px solid hsl(var(--border)); border-radius: var(--radius); padding: 1rem 1.25rem; }
.kpi .kpi-label { color: hsl(var(--muted-foreground)); font-size: 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; font-weight: 500; }
.kpi .kpi-value { font-size: 2rem; font-weight: 600; letter-spacing: -0.02em; line-height: 1.1; margin-top: 0.4rem; }
.kpi .kpi-value.success { color: hsl(var(--success)); }
.kpi .kpi-value.warning { color: hsl(var(--warning)); }
.kpi .kpi-value.destructive { color: hsl(var(--destructive)); }
.card { background: hsl(var(--card)); border: 1px solid hsl(var(--border)); border-radius: var(--radius); padding: 1.25rem 1.5rem; box-shadow: 0 1px 2px 0 rgb(0 0 0 / 0.05); }
.card + .card { margin-top: 1rem; }
.card .card-title { font-size: 0.95rem; font-weight: 600; display: flex; align-items: center; justify-content: space-between; gap: 1rem; margin: 0 0 0.5rem; }
.badge { display: inline-flex; align-items: center; border-radius: 999px; padding: 0.15rem 0.6rem; font-size: 0.75rem; font-weight: 600; white-space: nowrap; }
.badge.success { background: hsl(var(--success) / 0.12); color: hsl(var(--success)); border: 1px solid hsl(var(--success) / 0.3); }
.badge.warning { background: hsl(var(--warning) / 0.12); color: hsl(var(--warning)); border: 1px solid hsl(var(--warning) / 0.3); }
.badge.destructive { background: hsl(var(--destructive) / 0.12); color: hsl(var(--destructive)); border: 1px solid hsl(var(--destructive) / 0.3); }
.badge.muted { background: hsl(var(--muted)); color: hsl(var(--muted-foreground)); border: 1px solid hsl(var(--border)); }
table { width: 100%; border-collapse: collapse; border: 1px solid hsl(var(--border)); border-radius: var(--radius); overflow: hidden; font-size: 0.875rem; margin: 0.85rem 0; }
thead { background: hsl(var(--muted)); }
th, td { padding: 0.55rem 0.85rem; text-align: left; vertical-align: top; }
th { font-weight: 600; color: hsl(var(--muted-foreground)); text-transform: uppercase; font-size: 0.7rem; letter-spacing: 0.06em; }
tbody tr { border-top: 1px solid hsl(var(--border)); }
td.num { text-align: right; font-variant-numeric: tabular-nums; }
.muted { color: hsl(var(--muted-foreground)); font-size: 0.875rem; }
.fatal { border-left: 3px solid hsl(var(--destructive)); background: hsl(var(--destructive) / 0.08); color: hsl(var(--destructive)); padding: 0.75rem 1rem; border-radius: 6px; margin-top: 0.75rem; font-size: 0.875rem; word-break: break-word; }
"#;

/// Render a self-contained HTML report. Mirrors the markdown's information
/// density (per-store status, durations, fatal errors, run config) but with
/// the same shadcn-inspired aesthetic as the snapshot validator's report.
pub fn render_html(outcome: &MigrateOutcome) -> String {
    let mut out = String::with_capacity(8 * 1024);
    let (status_class, status_label) = if outcome.is_success() {
        ("success", "OK")
    } else {
        ("destructive", "FAILED")
    };

    out.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    out.push_str("<meta charset=\"utf-8\">\n");
    out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
    let _ = writeln!(
        out,
        "<title>Storage migration report — {}</title>",
        html_escape(&outcome.env_label),
    );
    out.push_str("<style>");
    out.push_str(STYLE);
    out.push_str("</style>\n</head>\n<body>\n<div class=\"container\">\n");

    // Header
    let _ = write!(
        out,
        r#"<header class="page-header">
  <h1>Storage migration report</h1>
  <div class="meta">
    <span><span class="badge {sc}">{sl}</span></span>
    <span>env <strong>{env}</strong></span>
    <span>generated <strong>{ts}</strong> UTC</span>
    <span>duration <strong>{dur}</strong></span>
  </div>
</header>
"#,
        sc = status_class,
        sl = status_label,
        env = html_escape(&outcome.env_label),
        ts = format_time(outcome.started_at),
        dur = format_args!("{:.2?}", outcome.overall_duration),
    );

    // Top KPIs
    out.push_str("<section><h2>Summary</h2>\n<div class=\"kpi-grid\">\n");
    let store_count = [&outcome.state, &outcome.pending, &outcome.debug]
        .iter()
        .filter(|s| s.is_some())
        .count()
        + if outcome.epochs.epochs_dir.is_some() {
            outcome.epochs.processed
        } else {
            0
        };
    push_html_kpi(&mut out, "Status", status_label, status_class, None);
    push_html_kpi(
        &mut out,
        "Stores migrated",
        &store_count.to_string(),
        "success",
        Some("state + pending + debug + epochs"),
    );
    push_html_kpi(
        &mut out,
        "Failures",
        &outcome.fatal_count().to_string(),
        if outcome.is_success() {
            "success"
        } else {
            "destructive"
        },
        None,
    );
    push_html_kpi(
        &mut out,
        "Total time",
        &format!("{:.2?}", outcome.overall_duration),
        "success",
        None,
    );
    out.push_str("</div></section>\n");

    // Per-store cards
    push_html_store_card(&mut out, "State DB", outcome.state.as_ref());
    push_html_store_card(&mut out, "Pending DB", outcome.pending.as_ref());
    push_html_store_card(&mut out, "Debug DB", outcome.debug.as_ref());
    push_html_epochs_card(&mut out, &outcome.epochs);

    // Run configuration
    out.push_str("<h2>Run configuration</h2>\n<table><tbody>\n");
    push_html_row(
        &mut out,
        "Environment label",
        &html_escape(&outcome.env_label),
    );
    if let Some(p) = outcome.state.as_ref() {
        push_html_row(
            &mut out,
            "state path",
            &format!(
                "<code class=\"mono break\">{}</code>",
                html_escape(&p.path.display().to_string())
            ),
        );
    }
    if let Some(p) = outcome.pending.as_ref() {
        push_html_row(
            &mut out,
            "pending path",
            &format!(
                "<code class=\"mono break\">{}</code>",
                html_escape(&p.path.display().to_string())
            ),
        );
    }
    if let Some(p) = outcome.debug.as_ref() {
        push_html_row(
            &mut out,
            "debug path",
            &format!(
                "<code class=\"mono break\">{}</code>",
                html_escape(&p.path.display().to_string())
            ),
        );
    }
    if let Some(d) = outcome.epochs.epochs_dir.as_ref() {
        push_html_row(
            &mut out,
            "epochs path",
            &format!(
                "<code class=\"mono break\">{}</code>",
                html_escape(&d.display().to_string())
            ),
        );
    }
    out.push_str("</tbody></table>\n");

    out.push_str("</div>\n</body>\n</html>\n");
    out
}

fn push_html_kpi(out: &mut String, label: &str, value: &str, class: &str, hint: Option<&str>) {
    let _ = write!(
        out,
        "  <div class=\"kpi\"><div class=\"kpi-label\">{}</div><div class=\"kpi-value \
         {}\">{}</div>",
        html_escape(label),
        class,
        html_escape(value),
    );
    if let Some(h) = hint {
        let _ = write!(
            out,
            "<div class=\"muted\" style=\"margin-top:0.25rem; font-size:0.8125rem\">{}</div>",
            html_escape(h),
        );
    }
    out.push_str("</div>\n");
}

fn push_html_store_card(out: &mut String, heading: &str, store: Option<&StoreResult>) {
    let _ = writeln!(out, "<section><h2>{}</h2>", html_escape(heading));
    let Some(s) = store else {
        out.push_str(
            "<div class=\"card\"><div class=\"card-title\"><span>—</span><span class=\"badge \
             muted\">SKIPPED</span></div><p class=\"muted\">not configured for this \
             deployment</p></div></section>\n",
        );
        return;
    };
    let (badge_class, badge_label) = match &s.error {
        None => ("success", "OK"),
        Some(_) => ("destructive", "FAILED"),
    };
    let _ = write!(
        out,
        r#"<div class="card">
  <div class="card-title">
    <span>{label}</span>
    <span class="badge {bc}">{bl}</span>
  </div>
  <table><tbody>
    <tr><th style="text-align:right; width:160px;">Path</th><td><code class="mono break">{path}</code></td></tr>
    <tr><th style="text-align:right;">Duration</th><td>{dur}</td></tr>
  </tbody></table>
"#,
        label = html_escape(&s.label),
        bc = badge_class,
        bl = badge_label,
        path = html_escape(&s.path.display().to_string()),
        dur = format_args!("{:.2?}", s.duration),
    );
    if let Some(err) = &s.error {
        let _ = writeln!(out, "  <div class=\"fatal\">{}</div>", html_escape(err));
    }
    out.push_str("</div></section>\n");
}

fn push_html_epochs_card(out: &mut String, epochs: &EpochsResult) {
    out.push_str("<section><h2>Epoch DBs</h2>\n");
    let Some(dir) = &epochs.epochs_dir else {
        out.push_str(
            "<div class=\"card\"><div class=\"card-title\"><span>—</span><span class=\"badge \
             muted\">SKIPPED</span></div><p class=\"muted\">not configured for this \
             deployment</p></div></section>\n",
        );
        return;
    };
    if let Some(reason) = epochs.skipped_reason {
        let _ = write!(
            out,
            r#"<div class="card">
  <div class="card-title">
    <span>aggregate</span>
    <span class="badge muted">SKIPPED</span>
  </div>
  <p class="muted">{}</p>
</div></section>
"#,
            html_escape(reason),
        );
        return;
    }
    let (badge_class, badge_label) = if epochs.failed.is_empty() {
        ("success", "OK")
    } else {
        ("destructive", "FAILED")
    };
    let _ = write!(
        out,
        r#"<div class="card">
  <div class="card-title">
    <span>aggregate ({path})</span>
    <span class="badge {bc}">{bl}</span>
  </div>
  <div class="kpi-grid">
"#,
        path = html_escape(&dir.display().to_string()),
        bc = badge_class,
        bl = badge_label,
    );
    push_html_kpi(
        out,
        "Discovered",
        &epochs.discovered.to_string(),
        "success",
        None,
    );
    push_html_kpi(
        out,
        "Processed",
        &epochs.processed.to_string(),
        "success",
        None,
    );
    push_html_kpi(
        out,
        "Successful",
        &epochs.successful.to_string(),
        "success",
        None,
    );
    push_html_kpi(
        out,
        "Failed",
        &epochs.failed.len().to_string(),
        if epochs.failed.is_empty() {
            "success"
        } else {
            "destructive"
        },
        None,
    );
    push_html_kpi(
        out,
        "Duration",
        &format!("{:.2?}", epochs.duration),
        "success",
        None,
    );
    out.push_str("</div>\n");

    if !epochs.failed.is_empty() {
        let _ = writeln!(out, "<h3>Failed epochs ({})</h3>", epochs.failed.len());
        out.push_str("<table><thead><tr><th>Epoch</th><th>Error</th></tr></thead><tbody>\n");
        for f in &epochs.failed {
            let _ = writeln!(
                out,
                "  <tr><td class=\"num\"><strong>{}</strong></td><td>{}</td></tr>",
                f.epoch,
                html_escape(&f.error),
            );
        }
        out.push_str("</tbody></table>\n");
    }
    out.push_str("</div></section>\n");
}

fn push_html_row(out: &mut String, key: &str, value_html: &str) {
    let _ = writeln!(
        out,
        "  <tr><th style=\"text-align:right; width:200px;\">{}</th><td>{}</td></tr>",
        html_escape(key),
        value_html,
    );
}

fn html_escape(s: &str) -> String {
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
