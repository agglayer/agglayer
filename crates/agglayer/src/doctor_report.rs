//! Report rendering for the `storage-doctor` CLI subcommand.
//!
//! Distinct from [`crate::migrate_report`] because the doctor's output
//! is focused on a single concern -- the operator-readable enumeration
//! of legacy CF rows that the migration helper had to skip -- with no
//! per-store migration timing or status table. Templates live next to
//! the migration-report templates and share the same CSS.

use std::{
    fs,
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use agglayer_storage::diagnostics::UnparsableRow;
use askama::Template;

/// Render the markdown doctor report.
pub fn render_markdown(env_label: &str, generated_at: SystemTime, rows: &[UnparsableRow]) -> String {
    MarkdownDoctor::new(env_label, generated_at, rows)
        .render()
        .expect("markdown doctor template renders")
}

/// Render the self-contained HTML doctor report.
pub fn render_html(env_label: &str, generated_at: SystemTime, rows: &[UnparsableRow]) -> String {
    HtmlDoctor::new(env_label, generated_at, rows)
        .render()
        .expect("HTML doctor template renders")
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

const STYLE: &str = include_str!("../templates/migration-report.css");

#[derive(Template)]
#[template(path = "doctor-report.md", escape = "none")]
struct MarkdownDoctor<'a> {
    env_label: &'a str,
    generated_at: String,
    total: usize,
    has_rows: bool,
    rows: Vec<RowVm>,
}

#[derive(Template)]
#[template(path = "doctor-report.html")]
struct HtmlDoctor<'a> {
    env_label: &'a str,
    generated_at: String,
    total: usize,
    has_rows: bool,
    rows: Vec<RowVm>,
    status_class: &'static str,
    status_label: &'static str,
    style: &'static str,
}

struct RowVm {
    source: String,
    cf: String,
    key_hex: String,
    error: String,
}

impl<'a> MarkdownDoctor<'a> {
    fn new(env_label: &'a str, generated_at: SystemTime, rows: &[UnparsableRow]) -> Self {
        let row_vms = rows.iter().map(RowVm::from).collect::<Vec<_>>();
        Self {
            env_label,
            generated_at: format_time(generated_at),
            total: row_vms.len(),
            has_rows: !row_vms.is_empty(),
            rows: row_vms,
        }
    }
}

impl<'a> HtmlDoctor<'a> {
    fn new(env_label: &'a str, generated_at: SystemTime, rows: &[UnparsableRow]) -> Self {
        let row_vms = rows.iter().map(RowVm::from).collect::<Vec<_>>();
        let total = row_vms.len();
        let (status_class, status_label) = if total == 0 {
            ("success", "OK")
        } else {
            ("warning", "ATTENTION")
        };
        Self {
            env_label,
            generated_at: format_time(generated_at),
            total,
            has_rows: total > 0,
            rows: row_vms,
            status_class,
            status_label,
            style: STYLE,
        }
    }
}

impl From<&UnparsableRow> for RowVm {
    fn from(u: &UnparsableRow) -> Self {
        Self {
            source: u.source.clone(),
            cf: u.cf.to_string(),
            key_hex: u.key_hex.clone(),
            error: u.error.clone(),
        }
    }
}

/// Same `YYYY-MM-DD HH:MM:SS` format used by `migrate_report`.
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

#[cfg(test)]
mod tests {
    use std::time::{Duration, UNIX_EPOCH};

    use super::*;

    fn fixed_time() -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(1_705_321_896)
    }

    fn sample_rows() -> Vec<UnparsableRow> {
        vec![
            UnparsableRow {
                source: "pending".into(),
                cf: "pending_queue",
                key_hex: "00000000000000010000000000000005".into(),
                error: "invalid varint".into(),
            },
            UnparsableRow {
                source: "epoch 17".into(),
                cf: "epoch_certificate_per_index",
                key_hex: "0000000000000003".into(),
                error: "BadCertificateVersion { version: 7 }".into(),
            },
        ]
    }

    #[test]
    fn markdown_clean_renders_no_unparsable_message() {
        let md = render_markdown("mainnet", fixed_time(), &[]);
        assert!(md.contains("## storage-doctor: mainnet"));
        assert!(md.contains("No unparsable rows found."));
        assert!(!md.contains("**Unparsable rows"));
    }

    #[test]
    fn markdown_with_rows_lists_each_one() {
        let md = render_markdown("mainnet", fixed_time(), &sample_rows());
        assert!(md.contains("**Unparsable rows (2)**"), "missing header in:\n{md}");
        assert!(md.contains(
            "`pending_queue` (pending) at key `00000000000000010000000000000005`: invalid varint"
        ));
        assert!(md.contains(
            "`epoch_certificate_per_index` (epoch 17) at key `0000000000000003`: \
             BadCertificateVersion { version: 7 }"
        ));
        assert!(!md.contains("No unparsable rows"));
    }

    #[test]
    fn html_clean_renders_self_contained_document() {
        let html = render_html("mainnet", fixed_time(), &[]);
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<title>storage-doctor — mainnet</title>"));
        assert!(html.contains("--background:")); // CSS inlined.
        assert!(html.contains("class=\"badge success\">OK"));
        assert!(html.contains("kpi-value success\">0</div>"));
        assert!(html.contains("No unparsable rows found."));
    }

    #[test]
    fn html_with_rows_renders_table_and_warning_status() {
        let html = render_html("mainnet", fixed_time(), &sample_rows());
        assert!(html.contains("class=\"badge warning\">ATTENTION"));
        assert!(html.contains("kpi-value warning\">2</div>"));
        assert!(html.contains("<code>pending_queue</code>"));
        assert!(html.contains("00000000000000010000000000000005"));
        assert!(html.contains("invalid varint"));
        assert!(html.contains("epoch 17"));
    }

    #[test]
    fn html_escapes_user_supplied_strings() {
        let mut rows = sample_rows();
        rows[0].error = "boom <script>".into();
        let html = render_html("weird <env>", fixed_time(), &rows);
        assert!(html.contains("storage-doctor — weird &#60;env&#62;"));
        assert!(html.contains("boom &#60;script&#62;"));
        assert!(!html.contains("<script>"));
    }
}
