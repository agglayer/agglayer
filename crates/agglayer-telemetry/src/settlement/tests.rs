use super::*;
use crate::testutils::MetricsHarness;

/// Extracts the value of the sample line for `name` carrying every
/// label pair in `labels`.
fn sample_value(metrics: &str, name: &str, labels: &[(&str, &str)]) -> Option<f64> {
    let labels: Vec<String> = labels
        .iter()
        .map(|(key, value)| format!("{key}=\"{value}\""))
        .collect();
    metrics
        .lines()
        .find(|line| {
            line.starts_with(&format!("{name}{{"))
                && labels.iter().all(|label| line.contains(label))
        })
        .and_then(|line| line.rsplit(' ').next())
        .map(|value| value.parse().unwrap())
}

#[test]
fn settlement_metrics_export_the_issue_1676_series() {
    // This test deliberately owns the process-global meter provider:
    // nextest runs one process per test, so nothing else can race it.
    let harness = MetricsHarness::install();

    record_settlement_attempt(SettlementAttemptKind::Submission);
    record_settlement_attempt(SettlementAttemptKind::Submission);
    record_settlement_attempt(SettlementAttemptKind::GasBump);
    record_settlement_attempt(SettlementAttemptKind::Replacement);

    record_settlement_attempt_error(SettlementAttemptErrorKind::NonceTooLow, "wallet-0");
    record_settlement_attempt_error(SettlementAttemptErrorKind::Underpriced, "wallet-0");
    record_settlement_attempt_error(SettlementAttemptErrorKind::Rpc, "wallet-1");
    record_settlement_attempt_error(SettlementAttemptErrorKind::Other, "wallet-1");

    record_settlement_job_duration(SettlementJobOutcome::Success, "wallet-0", 42.0);
    record_settlement_job_duration(SettlementJobOutcome::Revert, "wallet-1", 3.0);

    let metrics = harness.gather();

    // The prometheus exporter appends `_total` when rendering counters,
    // so the exported series carry a suffix the instrument names do
    // not. `sample_value` matches `name{` exactly: a doubled or missing
    // suffix fails these lookups.
    let attempts_series = format!("{SETTLEMENT_ATTEMPTS}_total");
    let attempt_errors_series = format!("{SETTLEMENT_ATTEMPT_ERRORS}_total");
    assert_eq!(
        sample_value(&metrics, &attempts_series, &[("kind", "submission")]),
        Some(2.0),
        "attempts counter, got:\n{metrics}"
    );
    assert_eq!(
        sample_value(&metrics, &attempts_series, &[("kind", "gas_bump")]),
        Some(1.0),
    );
    assert_eq!(
        sample_value(&metrics, &attempts_series, &[("kind", "replacement")]),
        Some(1.0),
    );

    assert_eq!(
        sample_value(
            &metrics,
            &attempt_errors_series,
            &[("kind", "nonce_too_low"), ("wallet", "wallet-0")],
        ),
        Some(1.0),
        "attempt errors counter, got:\n{metrics}"
    );
    assert_eq!(
        sample_value(
            &metrics,
            &attempt_errors_series,
            &[("kind", "underpriced"), ("wallet", "wallet-0")],
        ),
        Some(1.0),
    );
    assert_eq!(
        sample_value(
            &metrics,
            &attempt_errors_series,
            &[("kind", "rpc"), ("wallet", "wallet-1")],
        ),
        Some(1.0),
    );
    assert_eq!(
        sample_value(
            &metrics,
            &attempt_errors_series,
            &[("kind", "other"), ("wallet", "wallet-1")],
        ),
        Some(1.0),
    );

    let count_series = format!("{SETTLEMENT_JOB_DURATION_SECONDS}_count");
    let sum_series = format!("{SETTLEMENT_JOB_DURATION_SECONDS}_sum");
    let bucket_series = format!("{SETTLEMENT_JOB_DURATION_SECONDS}_bucket");
    assert_eq!(
        sample_value(
            &metrics,
            &count_series,
            &[("outcome", "success"), ("wallet", "wallet-0")],
        ),
        Some(1.0),
        "job duration histogram, got:\n{metrics}"
    );
    assert_eq!(
        sample_value(
            &metrics,
            &sum_series,
            &[("outcome", "success"), ("wallet", "wallet-0")],
        ),
        Some(42.0),
    );
    assert_eq!(
        sample_value(
            &metrics,
            &count_series,
            &[("outcome", "revert"), ("wallet", "wallet-1")],
        ),
        Some(1.0),
    );
    assert_eq!(
        sample_value(
            &metrics,
            &sum_series,
            &[("outcome", "revert"), ("wallet", "wallet-1")],
        ),
        Some(3.0),
    );
    // The 60s boundary is not one of the OTel defaults, so this bucket
    // only exists when the shared duration buckets were applied.
    assert_eq!(
        sample_value(
            &metrics,
            &bucket_series,
            &[("outcome", "success"), ("wallet", "wallet-0"), ("le", "60"),],
        ),
        Some(1.0),
    );
}

#[test]
fn recovery_skipped_jobs_counter_exports() {
    let harness = MetricsHarness::install();

    record_settlement_recovery_skipped_jobs(0);
    record_settlement_recovery_skipped_jobs(2);

    let metrics = harness.gather();

    // The counter carries no labels, so the sample line may or may not have
    // a `{...}` label set depending on exporter attributes; match on the
    // series name prefix only.
    let skipped_series = format!("{SETTLEMENT_RECOVERY_SKIPPED_JOBS}_total");
    let skipped_value = metrics
        .lines()
        .find(|line| line.starts_with(&skipped_series))
        .and_then(|line| line.rsplit(' ').next())
        .and_then(|value| value.parse::<f64>().ok());
    assert_eq!(
        skipped_value,
        Some(2.0),
        "recovery skipped counter, got:\n{metrics}"
    );
}
