//! Report rendering for the `migrate-storage` CLI subcommand.
//!
//! This module is the operator-output side of the migration. The
//! migration runner in [`agglayer_storage::migrate`] returns a
//! [`MigrateOutcome`] of pure data; we turn that data into a markdown
//! report (printed to stdout by default) and an optional self-contained
//! HTML report (when the operator passes `--html-file`).
//!
//! Templates live under `crates/agglayer/templates/` and are compiled
//! into the binary by `askama`. Anything format-related (durations,
//! timestamps, status badges, paths) is pre-flattened into the small
//! view-model structs below so the templates stay declarative.

use std::{
    fs,
    path::Path,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use agglayer_storage::migrate::{EpochsResult, MigrateOutcome, StoreResult};
use askama::Template;

/// Render the markdown report for a migration outcome.
pub fn render_markdown(outcome: &MigrateOutcome) -> String {
    MarkdownReport::from_outcome(outcome)
        .render()
        .expect("markdown template renders")
}

/// Render the self-contained HTML report for a migration outcome.
pub fn render_html(outcome: &MigrateOutcome) -> String {
    HtmlReport::from_outcome(outcome)
        .render()
        .expect("HTML template renders")
}

/// Write `contents` to `path`, creating the parent directory if needed.
pub fn write_to_file(path: &Path, contents: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, contents)
}

// ---------------------------------------------------------------------------
// Markdown
// ---------------------------------------------------------------------------

#[derive(Template)]
#[template(path = "migration-report.md", escape = "none")]
struct MarkdownReport<'a> {
    env_label: &'a str,
    started_at: String,
    overall_duration: String,
    is_success: bool,
    status_label: &'static str,
    rows: Vec<RowVm>,
    fatals: Vec<FatalVm>,
}

struct RowVm {
    label: String,
    status: String,
    duration: String,
    notes: String,
}

struct FatalVm {
    label: String,
    error: String,
}

impl<'a> MarkdownReport<'a> {
    fn from_outcome(o: &'a MigrateOutcome) -> Self {
        let is_success = o.is_success();
        let status_label = if is_success { "OK" } else { "FAILED" };

        let rows = vec![
            store_row(o.state.as_ref(), "state"),
            store_row(o.pending.as_ref(), "pending"),
            store_row(o.debug.as_ref(), "debug"),
            epochs_row(&o.epochs),
        ];

        let mut fatals = Vec::new();
        for r in [o.state.as_ref(), o.pending.as_ref(), o.debug.as_ref()]
            .into_iter()
            .flatten()
        {
            if let Some(err) = &r.error {
                fatals.push(FatalVm {
                    label: r.label.clone(),
                    error: err.clone(),
                });
            }
        }
        for f in &o.epochs.failed {
            fatals.push(FatalVm {
                label: format!("epoch {}", f.epoch),
                error: f.error.clone(),
            });
        }

        Self {
            env_label: &o.env_label,
            started_at: format_time(o.started_at),
            overall_duration: format_duration(o.overall_duration),
            is_success,
            status_label,
            rows,
            fatals,
        }
    }
}

fn store_row(store: Option<&StoreResult>, fallback_label: &str) -> RowVm {
    let Some(r) = store else {
        return RowVm {
            label: fallback_label.into(),
            status: "—".into(),
            duration: "—".into(),
            notes: "not configured for this deployment".into(),
        };
    };
    let (status, notes) = match &r.error {
        None => ("✅ OK".to_string(), "—".to_string()),
        Some(e) => ("❌ FAILED".to_string(), e.clone()),
    };
    RowVm {
        label: r.label.clone(),
        status,
        duration: format_duration(r.duration),
        notes,
    }
}

fn epochs_row(epochs: &EpochsResult) -> RowVm {
    if epochs.epochs_dir.is_none() {
        return RowVm {
            label: "epochs".into(),
            status: "—".into(),
            duration: "—".into(),
            notes: "not configured for this deployment".into(),
        };
    }
    if let Some(reason) = epochs.skipped_reason {
        return RowVm {
            label: "epochs".into(),
            status: "—".into(),
            duration: "—".into(),
            notes: reason.to_string(),
        };
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
    RowVm {
        label: "epochs".into(),
        status: status.into(),
        duration: format_duration(epochs.duration),
        notes,
    }
}

// ---------------------------------------------------------------------------
// HTML
// ---------------------------------------------------------------------------

const STYLE: &str = include_str!("../templates/migration-report.css");

#[derive(Template)]
#[template(path = "migration-report.html")]
struct HtmlReport<'a> {
    env_label: &'a str,
    started_at: String,
    overall_duration: String,
    status_label: &'static str,
    status_class: &'static str,
    store_count: usize,
    fatal_count: usize,
    store_cards: Vec<StoreCardVm>,
    epochs: EpochsCardVm,
    path_rows: Vec<PathRowVm>,
    style: &'static str,
}

struct StoreCardVm {
    heading: &'static str,
    skipped: bool,
    label: String,
    badge_class: &'static str,
    badge_label: &'static str,
    path: String,
    duration: String,
    has_error: bool,
    error: String,
}

struct EpochsCardVm {
    kind_not_configured: bool,
    kind_skipped: bool,
    skip_reason: String,
    path: String,
    badge_class: &'static str,
    badge_label: &'static str,
    discovered: usize,
    processed: usize,
    successful: usize,
    failed_count: usize,
    failed_class: &'static str,
    duration: String,
    has_failures: bool,
    failures: Vec<EpochFailureVm>,
}

struct EpochFailureVm {
    epoch: u64,
    error: String,
}

struct PathRowVm {
    label: &'static str,
    path: String,
}

impl<'a> HtmlReport<'a> {
    fn from_outcome(o: &'a MigrateOutcome) -> Self {
        let is_success = o.is_success();
        let (status_class, status_label) = if is_success {
            ("success", "OK")
        } else {
            ("destructive", "FAILED")
        };

        let store_cards = vec![
            store_card("State DB", o.state.as_ref()),
            store_card("Pending DB", o.pending.as_ref()),
            store_card("Debug DB", o.debug.as_ref()),
        ];

        let configured_store_count = [&o.state, &o.pending, &o.debug]
            .iter()
            .filter(|s| s.is_some())
            .count();
        let store_count = configured_store_count
            + if o.epochs.epochs_dir.is_some() {
                o.epochs.processed
            } else {
                0
            };

        let mut path_rows = Vec::new();
        if let Some(s) = o.state.as_ref() {
            path_rows.push(PathRowVm {
                label: "state path",
                path: s.path.display().to_string(),
            });
        }
        if let Some(s) = o.pending.as_ref() {
            path_rows.push(PathRowVm {
                label: "pending path",
                path: s.path.display().to_string(),
            });
        }
        if let Some(s) = o.debug.as_ref() {
            path_rows.push(PathRowVm {
                label: "debug path",
                path: s.path.display().to_string(),
            });
        }
        if let Some(d) = o.epochs.epochs_dir.as_ref() {
            path_rows.push(PathRowVm {
                label: "epochs path",
                path: d.display().to_string(),
            });
        }

        Self {
            env_label: &o.env_label,
            started_at: format_time(o.started_at),
            overall_duration: format_duration(o.overall_duration),
            status_label,
            status_class,
            store_count,
            fatal_count: o.fatal_count(),
            store_cards,
            epochs: epochs_card(&o.epochs),
            path_rows,
            style: STYLE,
        }
    }
}

fn store_card(heading: &'static str, store: Option<&StoreResult>) -> StoreCardVm {
    let Some(s) = store else {
        return StoreCardVm {
            heading,
            skipped: true,
            label: String::new(),
            badge_class: "muted",
            badge_label: "SKIPPED",
            path: String::new(),
            duration: String::new(),
            has_error: false,
            error: String::new(),
        };
    };
    let (badge_class, badge_label) = match &s.error {
        None => ("success", "OK"),
        Some(_) => ("destructive", "FAILED"),
    };
    StoreCardVm {
        heading,
        skipped: false,
        label: s.label.clone(),
        badge_class,
        badge_label,
        path: s.path.display().to_string(),
        duration: format_duration(s.duration),
        has_error: s.error.is_some(),
        error: s.error.clone().unwrap_or_default(),
    }
}

fn epochs_card(epochs: &EpochsResult) -> EpochsCardVm {
    if epochs.epochs_dir.is_none() {
        return EpochsCardVm {
            kind_not_configured: true,
            kind_skipped: false,
            skip_reason: String::new(),
            path: String::new(),
            badge_class: "muted",
            badge_label: "SKIPPED",
            discovered: 0,
            processed: 0,
            successful: 0,
            failed_count: 0,
            failed_class: "success",
            duration: String::new(),
            has_failures: false,
            failures: Vec::new(),
        };
    }
    if let Some(reason) = epochs.skipped_reason {
        return EpochsCardVm {
            kind_not_configured: false,
            kind_skipped: true,
            skip_reason: reason.to_string(),
            path: epochs
                .epochs_dir
                .as_ref()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            badge_class: "muted",
            badge_label: "SKIPPED",
            discovered: 0,
            processed: 0,
            successful: 0,
            failed_count: 0,
            failed_class: "success",
            duration: String::new(),
            has_failures: false,
            failures: Vec::new(),
        };
    }
    let (badge_class, badge_label) = if epochs.failed.is_empty() {
        ("success", "OK")
    } else {
        ("destructive", "FAILED")
    };
    let failed_class = if epochs.failed.is_empty() {
        "success"
    } else {
        "destructive"
    };
    let failures = epochs
        .failed
        .iter()
        .map(|f| EpochFailureVm {
            epoch: f.epoch,
            error: f.error.clone(),
        })
        .collect::<Vec<_>>();
    EpochsCardVm {
        kind_not_configured: false,
        kind_skipped: false,
        skip_reason: String::new(),
        path: epochs
            .epochs_dir
            .as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
        badge_class,
        badge_label,
        discovered: epochs.discovered,
        processed: epochs.processed,
        successful: epochs.successful,
        failed_count: epochs.failed.len(),
        failed_class,
        duration: format_duration(epochs.duration),
        has_failures: !epochs.failed.is_empty(),
        failures,
    }
}

// ---------------------------------------------------------------------------
// Formatting helpers
// ---------------------------------------------------------------------------

/// `Duration` formatted the way the original report used it
/// (`format!("{:.2?}", d)` -- e.g. `1.23s`, `345.67ms`).
fn format_duration(d: Duration) -> String {
    format!("{d:.2?}")
}

/// Format a `SystemTime` as `YYYY-MM-DD HH:MM:SS` UTC, matching the
/// pre-templated renderer's output. Self-contained on purpose: we only
/// need this in one place and don't want to take a `chrono`/`time` dep.
fn format_time(t: SystemTime) -> String {
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
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::{
        path::PathBuf,
        time::{Duration, UNIX_EPOCH},
    };

    use agglayer_storage::migrate::{
        EpochFailure, EpochsResult, MigrateOutcome, StoreResult,
    };

    use super::*;

    /// Fixed instant so duration / timestamp formatting is deterministic
    /// across runs. Picked to land at `2024-01-15 12:34:56 UTC`.
    fn fixed_started_at() -> std::time::SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_705_321_896)
    }

    fn outcome_all_ok() -> MigrateOutcome {
        MigrateOutcome {
            started_at: fixed_started_at(),
            overall_duration: Duration::from_millis(1_234),
            env_label: "mainnet".into(),
            state: Some(StoreResult {
                label: "state".into(),
                path: PathBuf::from("/var/agglayer/state"),
                duration: Duration::from_millis(120),
                error: None,
                unparsable_rows: Vec::new(),
            }),
            pending: Some(StoreResult {
                label: "pending".into(),
                path: PathBuf::from("/var/agglayer/pending"),
                duration: Duration::from_millis(45),
                error: None,
                unparsable_rows: Vec::new(),
            }),
            debug: Some(StoreResult {
                label: "debug".into(),
                path: PathBuf::from("/var/agglayer/debug"),
                duration: Duration::from_millis(30),
                error: None,
                unparsable_rows: Vec::new(),
            }),
            epochs: EpochsResult {
                epochs_dir: Some(PathBuf::from("/var/agglayer/epochs")),
                discovered: 5,
                processed: 5,
                successful: 5,
                failed: Vec::new(),
                duration: Duration::from_millis(800),
                skipped_reason: None,
                unparsable_rows: Vec::new(),
            },
        }
    }

    fn outcome_with_failures() -> MigrateOutcome {
        MigrateOutcome {
            state: Some(StoreResult {
                label: "state".into(),
                path: PathBuf::from("/var/agglayer/state"),
                duration: Duration::from_millis(120),
                error: Some("schema mismatch".into()),
                unparsable_rows: Vec::new(),
            }),
            epochs: EpochsResult {
                epochs_dir: Some(PathBuf::from("/var/agglayer/epochs")),
                discovered: 3,
                processed: 3,
                successful: 1,
                failed: vec![
                    EpochFailure {
                        epoch: 17,
                        error: "missing CF debug_certificates".into(),
                    },
                    EpochFailure {
                        epoch: 42,
                        error: "decode error".into(),
                    },
                ],
                duration: Duration::from_millis(500),
                skipped_reason: None,
                unparsable_rows: Vec::new(),
            },
            ..outcome_all_ok()
        }
    }

    #[test]
    fn markdown_all_ok_renders_expected_shape() {
        let md = render_markdown(&outcome_all_ok());
        // High-signal substrings (avoid pinning the whole template so
        // whitespace tweaks don't churn the test).
        assert!(md.contains("## mainnet"));
        assert!(md.contains("Status: **OK**"));
        assert!(md.contains("| state | ✅ OK |"));
        assert!(md.contains("| pending | ✅ OK |"));
        assert!(md.contains("| debug | ✅ OK |"));
        assert!(md.contains("| epochs | ✅ OK |"));
        assert!(md.contains("5 processed; 5 OK; 0 failed"));
        assert!(md.contains("---"));
        assert!(!md.contains("**Fatal errors**"));
    }

    #[test]
    fn markdown_with_failures_lists_each_fatal() {
        let md = render_markdown(&outcome_with_failures());
        assert!(md.contains("Status: **FAILED**"));
        assert!(md.contains("**Fatal errors**"));
        assert!(md.contains("- state: schema mismatch"));
        assert!(md.contains("- epoch 17: missing CF debug_certificates"));
        assert!(md.contains("- epoch 42: decode error"));
    }

    #[test]
    fn html_renders_self_contained_document() {
        let html = render_html(&outcome_all_ok());
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<title>Storage migration report — mainnet</title>"));
        // CSS got inlined.
        assert!(html.contains("--background:"));
        assert!(html.contains("class=\"badge success\">OK"));
        assert!(html.contains("State DB"));
        assert!(html.contains("Pending DB"));
        assert!(html.contains("Debug DB"));
        assert!(html.contains("Epoch DBs"));
        // Failures KPI shows 0 with the success class.
        assert!(html.contains("kpi-value success\">0</div>"));
        assert!(html.trim_end().ends_with("</html>"));
    }

    #[test]
    fn html_escapes_user_supplied_strings() {
        let mut o = outcome_all_ok();
        o.env_label = "weird <env> & label".into();
        if let Some(s) = o.state.as_mut() {
            s.error = Some("oops <script>".into());
        }
        let html = render_html(&o);
        assert!(html.contains("Storage migration report — weird &#60;env&#62; &#38; label"));
        assert!(html.contains("oops &#60;script&#62;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn html_with_failures_has_destructive_styling() {
        let html = render_html(&outcome_with_failures());
        assert!(html.contains("class=\"badge destructive\">FAILED"));
        // Failure KPI flips to destructive.
        assert!(html.contains("kpi-value destructive\">3</div>"));
        // Failed epochs table is present.
        assert!(html.contains("<h3>Failed epochs (2)</h3>"));
        assert!(html.contains("<strong>17</strong>"));
        assert!(html.contains("<strong>42</strong>"));
    }

    #[test]
    fn skipped_epochs_renders_skip_reason() {
        let mut o = outcome_all_ok();
        o.epochs = EpochsResult {
            epochs_dir: Some(PathBuf::from("/var/agglayer/epochs")),
            skipped_reason: Some("skipped via --skip-epochs"),
            ..EpochsResult::default()
        };
        let md = render_markdown(&o);
        assert!(md.contains("| epochs | — | — | skipped via --skip-epochs |"));
        let html = render_html(&o);
        assert!(html.contains("class=\"badge muted\">SKIPPED"));
        assert!(html.contains("skipped via --skip-epochs"));
    }
}
