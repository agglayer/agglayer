//! Settlement transaction metrics: attempt counts, attempt errors, and job
//! durations, labeled by wallet where the wallet dimension matters.
//!
//! This module only declares the instruments and their record helpers. The
//! settlement service is responsible for calling them; that wiring, and the
//! jobs-by-state gauge it requires, land in follow-ups to issue #1676.

use lazy_static::lazy_static;
use opentelemetry::{global, metrics::*, KeyValue};

use crate::certificate::DURATION_BUCKETS_SECONDS;

const AGGLAYER_NODE_SETTLEMENT_OTEL_SCOPE_NAME: &str = "agglayer_node_settlement";

/// Name of the label carrying an attempt or attempt-error kind.
const KIND_LABEL_NAME: &str = "kind";

/// Name of the label carrying the settlement wallet.
const WALLET_LABEL_NAME: &str = "wallet";

/// Name of the label carrying the job outcome.
const OUTCOME_LABEL_NAME: &str = "outcome";

/// Counter instrument name: settlement transaction attempts by `kind`.
///
/// Instrument names carry no `_total` suffix; the prometheus exporter adds
/// it when rendering counters, so this one exports as
/// `agglayer_node_settlement_attempts_total`, the name issue #1676 uses.
pub const SETTLEMENT_ATTEMPTS: &str = "agglayer_node_settlement_attempts";

/// Counter instrument name: failed settlement transaction attempts by
/// `kind` and `wallet`.
///
/// Exported as `agglayer_node_settlement_attempt_errors_total`; see
/// [`SETTLEMENT_ATTEMPTS`] for the suffix convention.
pub const SETTLEMENT_ATTEMPT_ERRORS: &str = "agglayer_node_settlement_attempt_errors";

/// Histogram name: settlement job duration in seconds, from job creation to
/// terminal state, by `outcome` and `wallet`.
pub const SETTLEMENT_JOB_DURATION_SECONDS: &str = "agglayer_node_settlement_job_duration_seconds";

/// Counter instrument name: settlement jobs skipped during startup recovery
/// because they could not be loaded from storage.
///
/// Exported as `agglayer_node_settlement_recovery_skipped_jobs_total`; see
/// [`SETTLEMENT_ATTEMPTS`] for the suffix convention.
pub const SETTLEMENT_RECOVERY_SKIPPED_JOBS: &str = "agglayer_node_settlement_recovery_skipped_jobs";

/// A kind of settlement transaction attempt, rendered as the `kind` label
/// value on [`SETTLEMENT_ATTEMPTS`].
///
/// The variants follow the transaction churn taxonomy of issue #1676:
/// initial submissions, gas bumps, and replacements. Which service code
/// paths map onto which variant is decided where emission is wired in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum SettlementAttemptKind {
    Submission,
    GasBump,
    Replacement,
}

/// A kind of settlement attempt failure, rendered as the `kind` label value
/// on [`SETTLEMENT_ATTEMPT_ERRORS`].
///
/// The variants follow the error taxonomy of issue #1676, with nonce and
/// gas-price errors kept separate from generic RPC failures so nonce
/// contention is visible on its own. The mapping from concrete errors to
/// variants is decided where emission is wired in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum SettlementAttemptErrorKind {
    NonceTooLow,
    Underpriced,
    Rpc,
    Other,
}

/// A terminal settlement job outcome, rendered as the `outcome` label value
/// on [`SETTLEMENT_JOB_DURATION_SECONDS`].
///
/// The variants follow the outcome taxonomy of issue #1676. Which terminal
/// job states map onto which variant is decided where emission is wired in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, strum_macros::Display)]
#[strum(serialize_all = "snake_case")]
pub enum SettlementJobOutcome {
    Success,
    Revert,
}

lazy_static! {
    static ref SETTLEMENT_ATTEMPTS_COUNTER: Counter<u64> =
        global::meter(AGGLAYER_NODE_SETTLEMENT_OTEL_SCOPE_NAME)
            .u64_counter(SETTLEMENT_ATTEMPTS)
            .with_description("Number of settlement transaction attempts, by kind")
            .build();
    static ref SETTLEMENT_ATTEMPT_ERRORS_COUNTER: Counter<u64> =
        global::meter(AGGLAYER_NODE_SETTLEMENT_OTEL_SCOPE_NAME)
            .u64_counter(SETTLEMENT_ATTEMPT_ERRORS)
            .with_description(
                "Number of failed settlement transaction attempts, by kind and wallet"
            )
            .build();
    static ref SETTLEMENT_JOB_DURATION: Histogram<f64> =
        global::meter(AGGLAYER_NODE_SETTLEMENT_OTEL_SCOPE_NAME)
            .f64_histogram(SETTLEMENT_JOB_DURATION_SECONDS)
            .with_description("Settlement job time from creation to terminal state, in seconds")
            .with_boundaries(DURATION_BUCKETS_SECONDS.to_vec())
            .build();
    static ref SETTLEMENT_RECOVERY_SKIPPED_JOBS_COUNTER: Counter<u64> =
        global::meter(AGGLAYER_NODE_SETTLEMENT_OTEL_SCOPE_NAME)
            .u64_counter(SETTLEMENT_RECOVERY_SKIPPED_JOBS)
            .with_description(
                "Number of settlement jobs skipped during startup recovery because they could not \
                 be loaded"
            )
            .build();
}

/// Records one settlement transaction attempt.
#[inline]
pub fn record_settlement_attempt(kind: SettlementAttemptKind) {
    SETTLEMENT_ATTEMPTS_COUNTER.add(1, &[KeyValue::new(KIND_LABEL_NAME, kind.to_string())]);
}

/// Records one failed settlement transaction attempt on `wallet`.
#[inline]
pub fn record_settlement_attempt_error(kind: SettlementAttemptErrorKind, wallet: &str) {
    SETTLEMENT_ATTEMPT_ERRORS_COUNTER.add(
        1,
        &[
            KeyValue::new(KIND_LABEL_NAME, kind.to_string()),
            KeyValue::new(WALLET_LABEL_NAME, wallet.to_string()),
        ],
    );
}

/// Records the duration of one settlement job that reached a terminal
/// state on `wallet`.
#[inline]
pub fn record_settlement_job_duration(outcome: SettlementJobOutcome, wallet: &str, seconds: f64) {
    SETTLEMENT_JOB_DURATION.record(
        seconds,
        &[
            KeyValue::new(OUTCOME_LABEL_NAME, outcome.to_string()),
            KeyValue::new(WALLET_LABEL_NAME, wallet.to_string()),
        ],
    );
}

/// Records how many settlement jobs the startup recovery scan skipped
/// because they could not be loaded. Called once at node startup; a zero
/// count still exports the series.
#[inline]
pub fn record_settlement_recovery_skipped_jobs(count: u64) {
    SETTLEMENT_RECOVERY_SKIPPED_JOBS_COUNTER.add(count, &[]);
}

#[cfg(test)]
mod tests;
