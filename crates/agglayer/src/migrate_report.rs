//! Report rendering for the `migrate-storage` CLI subcommand.
//!
//! This module is the operator-output side of the migration. The
//! migration runner in [`agglayer_storage::migrate`] returns a
//! [`MigrateOutcome`] of pure data; we turn that data into a markdown
//! report (printed to stdout by default) and an optional self-contained
//! HTML report (when the operator passes `--html-file`).
//!
//! Templates live under `crates/agglayer/src/templates/` and are compiled
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
use chrono::{DateTime, Utc};

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
    diagnostics_warnings: Vec<DiagnosticWarningVm>,
    unparsable: Vec<UnparsableRowVm>,
    unparsable_count: usize,
    has_unparsable: bool,
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

struct DiagnosticWarningVm {
    label: String,
    error: String,
}

#[derive(Clone)]
struct UnparsableRowVm {
    source: String,
    cf: String,
    key_hex: String,
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
        if let Some(err) = &o.epochs.discovery_error {
            fatals.push(FatalVm {
                label: "epochs".into(),
                error: err.clone(),
            });
        }
        for f in &o.epochs.failed {
            fatals.push(FatalVm {
                label: format!("epoch {}", f.epoch),
                error: f.error.clone(),
            });
        }
        let mut diagnostics_warnings = [&o.state, &o.pending, &o.debug]
            .iter()
            .filter_map(|s| s.as_ref())
            .filter_map(|s| {
                s.diagnostics_error
                    .as_ref()
                    .map(|error| DiagnosticWarningVm {
                        label: s.label.clone(),
                        error: error.clone(),
                    })
            })
            .collect::<Vec<_>>();
        diagnostics_warnings.extend(o.epochs.diagnostics_failures.iter().map(|f| {
            DiagnosticWarningVm {
                label: format!("epoch {}", f.epoch),
                error: f.error.clone(),
            }
        }));

        let unparsable = collect_unparsable(o);
        let unparsable_count = unparsable.len();

        Self {
            env_label: &o.env_label,
            started_at: format_time(o.started_at),
            overall_duration: format_duration(o.overall_duration),
            is_success,
            status_label,
            rows,
            fatals,
            diagnostics_warnings,
            has_unparsable: !unparsable.is_empty(),
            unparsable_count,
            unparsable,
        }
    }
}

/// Flatten every store's unparsable rows into a single, deterministically-
/// ordered list (state, pending, debug, epochs) for the markdown summary
/// section. The HTML report keeps the per-store buckets separate.
fn collect_unparsable(o: &MigrateOutcome) -> Vec<UnparsableRowVm> {
    let mut out = Vec::new();
    for store in [&o.state, &o.pending, &o.debug]
        .iter()
        .filter_map(|s| s.as_ref())
    {
        for u in &store.unparsable_rows {
            out.push(UnparsableRowVm {
                source: u.source.clone(),
                cf: u.cf.to_string(),
                key_hex: u.key_hex.clone(),
                error: u.error.clone(),
            });
        }
    }
    for u in &o.epochs.unparsable_rows {
        out.push(UnparsableRowVm {
            source: u.source.clone(),
            cf: u.cf.to_string(),
            key_hex: u.key_hex.clone(),
            error: u.error.clone(),
        });
    }
    out
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
    let unparsable_count = r.unparsable_rows.len();
    let (status, notes) = match &r.error {
        None => {
            let notes = match (&r.diagnostics_error, unparsable_count) {
                (Some(error), 0) => format!("diagnostics failed: {error}"),
                (Some(error), _) => {
                    format!("{unparsable_count} unparsable rows; diagnostics failed: {error}")
                }
                (None, count) if count > 0 => format!("{count} unparsable rows"),
                (None, _) => "—".to_string(),
            };
            let status = if r.diagnostics_error.is_some() {
                "⚠️ INCOMPLETE"
            } else {
                "✅ OK"
            };
            (status.to_string(), notes)
        }
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
    if let Some(error) = &epochs.discovery_error {
        return RowVm {
            label: "epochs".into(),
            status: "❌ FAILED".into(),
            duration: format_duration(epochs.duration),
            notes: error.clone(),
        };
    }
    let status = if epochs.failed.is_empty() {
        "✅ OK"
    } else {
        "❌ FAILED"
    };
    let mut notes = format!(
        "{} processed; {} OK; {} failed",
        epochs.processed,
        epochs.successful,
        epochs.failed.len(),
    );
    let unparsable_count = epochs.unparsable_rows.len();
    if unparsable_count > 0 {
        notes.push_str(&format!("; {unparsable_count} unparsable"));
    }
    let diagnostics_failure_count = epochs.diagnostics_failures.len();
    if diagnostics_failure_count > 0 {
        notes.push_str(&format!("; {diagnostics_failure_count} diagnostics failed"));
    }
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

const STYLE: &str = include_str!("templates/migration-report.css");

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
    unparsable_total: usize,
    unparsable_class: &'static str,
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
    has_diagnostics_error: bool,
    diagnostics_error: String,
    has_unparsable: bool,
    unparsable_count: usize,
    unparsable: Vec<UnparsableRowVm>,
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
    has_discovery_error: bool,
    discovery_error: String,
    has_failures: bool,
    failures: Vec<EpochFailureVm>,
    has_diagnostics_failures: bool,
    diagnostics_failure_count: usize,
    diagnostics_failures: Vec<EpochFailureVm>,
    has_unparsable: bool,
    unparsable_count: usize,
    unparsable: Vec<UnparsableRowVm>,
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

        let unparsable_total: usize = [&o.state, &o.pending, &o.debug]
            .iter()
            .filter_map(|s| s.as_ref())
            .map(|s| s.unparsable_rows.len())
            .sum::<usize>()
            + o.epochs.unparsable_rows.len();

        Self {
            env_label: &o.env_label,
            started_at: format_time(o.started_at),
            overall_duration: format_duration(o.overall_duration),
            status_label,
            status_class,
            store_count,
            fatal_count: o.fatal_count(),
            unparsable_total,
            unparsable_class: if unparsable_total == 0 {
                "success"
            } else {
                "warning"
            },
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
            has_diagnostics_error: false,
            diagnostics_error: String::new(),
            has_unparsable: false,
            unparsable_count: 0,
            unparsable: Vec::new(),
        };
    };
    let (badge_class, badge_label) = match (&s.error, &s.diagnostics_error) {
        (None, None) => ("success", "OK"),
        (None, Some(_)) => ("warning", "INCOMPLETE"),
        (Some(_), _) => ("destructive", "FAILED"),
    };
    let unparsable: Vec<UnparsableRowVm> = s
        .unparsable_rows
        .iter()
        .map(|u| UnparsableRowVm {
            source: u.source.clone(),
            cf: u.cf.to_string(),
            key_hex: u.key_hex.clone(),
            error: u.error.clone(),
        })
        .collect();
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
        has_diagnostics_error: s.diagnostics_error.is_some(),
        diagnostics_error: s.diagnostics_error.clone().unwrap_or_default(),
        has_unparsable: !unparsable.is_empty(),
        unparsable_count: unparsable.len(),
        unparsable,
    }
}

fn epochs_card(epochs: &EpochsResult) -> EpochsCardVm {
    let unparsable: Vec<UnparsableRowVm> = epochs
        .unparsable_rows
        .iter()
        .map(|u| UnparsableRowVm {
            source: u.source.clone(),
            cf: u.cf.to_string(),
            key_hex: u.key_hex.clone(),
            error: u.error.clone(),
        })
        .collect();
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
            has_discovery_error: false,
            discovery_error: String::new(),
            has_failures: false,
            failures: Vec::new(),
            has_diagnostics_failures: false,
            diagnostics_failure_count: 0,
            diagnostics_failures: Vec::new(),
            has_unparsable: false,
            unparsable_count: 0,
            unparsable: Vec::new(),
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
            has_discovery_error: false,
            discovery_error: String::new(),
            has_failures: false,
            failures: Vec::new(),
            has_diagnostics_failures: false,
            diagnostics_failure_count: 0,
            diagnostics_failures: Vec::new(),
            has_unparsable: !unparsable.is_empty(),
            unparsable_count: unparsable.len(),
            unparsable,
        };
    }
    let has_discovery_error = epochs.discovery_error.is_some();
    let (badge_class, badge_label) = if !has_discovery_error && epochs.failed.is_empty() {
        ("success", "OK")
    } else {
        ("destructive", "FAILED")
    };
    let failed_count = epochs.failed.len() + usize::from(has_discovery_error);
    let failed_class = if failed_count == 0 {
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
    let diagnostics_failures = epochs
        .diagnostics_failures
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
        failed_count,
        failed_class,
        duration: format_duration(epochs.duration),
        has_discovery_error,
        discovery_error: epochs.discovery_error.clone().unwrap_or_default(),
        has_failures: !epochs.failed.is_empty(),
        failures,
        has_diagnostics_failures: !diagnostics_failures.is_empty(),
        diagnostics_failure_count: diagnostics_failures.len(),
        diagnostics_failures,
        has_unparsable: !unparsable.is_empty(),
        unparsable_count: unparsable.len(),
        unparsable,
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

/// Format a `SystemTime` as `YYYY-MM-DD HH:MM:SS` UTC.
fn format_time(t: SystemTime) -> String {
    let secs = t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64;
    DateTime::<Utc>::from_timestamp(secs, 0)
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
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
        EpochFailure, EpochsResult, MigrateOutcome, StoreResult, UnparsableRow,
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
                diagnostics_error: None,
            }),
            pending: Some(StoreResult {
                label: "pending".into(),
                path: PathBuf::from("/var/agglayer/pending"),
                duration: Duration::from_millis(45),
                error: None,
                unparsable_rows: Vec::new(),
                diagnostics_error: None,
            }),
            debug: Some(StoreResult {
                label: "debug".into(),
                path: PathBuf::from("/var/agglayer/debug"),
                duration: Duration::from_millis(30),
                error: None,
                unparsable_rows: Vec::new(),
                diagnostics_error: None,
            }),
            epochs: EpochsResult {
                epochs_dir: Some(PathBuf::from("/var/agglayer/epochs")),
                discovered: 5,
                processed: 5,
                successful: 5,
                discovery_error: None,
                failed: Vec::new(),
                duration: Duration::from_millis(800),
                skipped_reason: None,
                unparsable_rows: Vec::new(),
                diagnostics_failures: Vec::new(),
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
                diagnostics_error: None,
            }),
            epochs: EpochsResult {
                epochs_dir: Some(PathBuf::from("/var/agglayer/epochs")),
                discovered: 3,
                processed: 3,
                successful: 1,
                discovery_error: None,
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
                diagnostics_failures: Vec::new(),
            },
            ..outcome_all_ok()
        }
    }

    #[test]
    fn markdown_all_ok_renders_expected_shape() {
        insta::assert_snapshot!(
            "migration_report_markdown_all_ok",
            render_markdown(&outcome_all_ok())
        );
    }

    #[test]
    fn markdown_with_failures_lists_each_fatal() {
        insta::assert_snapshot!(
            "migration_report_markdown_with_failures",
            render_markdown(&outcome_with_failures())
        );
    }

    #[test]
    fn epoch_discovery_errors_render_as_fatal() {
        let mut o = outcome_all_ok();
        o.epochs = EpochsResult {
            epochs_dir: Some(PathBuf::from("/var/agglayer/epochs")),
            discovery_error: Some("failed to read epoch directory".into()),
            duration: Duration::from_millis(10),
            ..EpochsResult::default()
        };

        insta::assert_snapshot!(
            "migration_report_markdown_epoch_discovery_error",
            render_markdown(&o)
        );

        insta::assert_snapshot!(
            "migration_report_html_epoch_discovery_error",
            render_html(&o)
        );
    }

    #[test]
    fn html_renders_self_contained_document() {
        insta::assert_snapshot!(
            "migration_report_html_self_contained_document",
            render_html(&outcome_all_ok())
        );
    }

    #[test]
    fn html_escapes_user_supplied_strings() {
        let mut o = outcome_all_ok();
        o.env_label = "weird <env> & label".into();
        if let Some(s) = o.state.as_mut() {
            s.error = Some("oops <script>".into());
        }
        let html = render_html(&o);
        insta::assert_snapshot!("migration_report_html_escapes_user_strings", html);
        assert_eq!(html.find("<script>"), None);
    }

    #[test]
    fn html_with_failures_has_destructive_styling() {
        insta::assert_snapshot!(
            "migration_report_html_with_failures",
            render_html(&outcome_with_failures())
        );
    }

    #[test]
    fn diagnostics_failures_render_as_warnings_without_failing_run() {
        let mut o = outcome_all_ok();
        o.epochs.diagnostics_failures = vec![EpochFailure {
            epoch: 99,
            error: "read-only diagnostics open failed".into(),
        }];

        assert!(o.is_success());

        insta::assert_snapshot!(
            "migration_report_markdown_epoch_diagnostics_warnings",
            render_markdown(&o)
        );

        insta::assert_snapshot!(
            "migration_report_html_epoch_diagnostics_warnings",
            render_html(&o)
        );
    }

    #[test]
    fn per_store_diagnostics_failures_render_as_incomplete_warnings() {
        let mut o = outcome_all_ok();
        if let Some(pending) = o.pending.as_mut() {
            pending.diagnostics_error = Some("pending scan failed".into());
        }

        assert!(o.is_success());

        insta::assert_snapshot!(
            "migration_report_markdown_per_store_diagnostics_warning",
            render_markdown(&o)
        );

        insta::assert_snapshot!(
            "migration_report_html_per_store_diagnostics_warning",
            render_html(&o)
        );
    }

    #[test]
    fn skipped_epochs_renders_skip_reason() {
        let mut o = outcome_all_ok();
        o.epochs = EpochsResult {
            epochs_dir: Some(PathBuf::from("/var/agglayer/epochs")),
            skipped_reason: Some("skipped via --skip-epochs"),
            ..EpochsResult::default()
        };
        insta::assert_snapshot!(
            "migration_report_markdown_skipped_epochs",
            render_markdown(&o)
        );
        insta::assert_snapshot!("migration_report_html_skipped_epochs", render_html(&o));
    }

    fn outcome_with_unparsable_rows() -> MigrateOutcome {
        let mut o = outcome_all_ok();
        if let Some(p) = o.pending.as_mut() {
            p.unparsable_rows = vec![
                UnparsableRow {
                    source: "pending".into(),
                    cf: "pending_queue",
                    key_hex: "00000000000000010000000000000005".into(),
                    error: "invalid varint".into(),
                },
                UnparsableRow {
                    source: "pending".into(),
                    cf: "pending_queue",
                    key_hex: "0000000000000002000000000000000a".into(),
                    error: "BadCertificateVersion { version: 7 }".into(),
                },
            ];
        }
        o.epochs.unparsable_rows = vec![UnparsableRow {
            source: "epoch 17".into(),
            cf: "epoch_certificate_per_index",
            key_hex: "0000000000000003".into(),
            error: "decode error".into(),
        }];
        o
    }

    #[test]
    fn markdown_lists_unparsable_rows_per_store() {
        insta::assert_snapshot!(
            "migration_report_markdown_with_unparsable_rows",
            render_markdown(&outcome_with_unparsable_rows())
        );
    }

    #[test]
    fn html_with_unparsable_rows_renders_section_per_store() {
        insta::assert_snapshot!(
            "migration_report_html_with_unparsable_rows",
            render_html(&outcome_with_unparsable_rows())
        );
    }
}
