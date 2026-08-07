//! Wire types and error handling for settlement administration RPC methods.
//!
//! The settlement domain types in `agglayer-types` carry no serde; their JSON
//! representation is owned here, at the RPC boundary. Attempt-result enums are
//! deliberately internally tagged (`#[serde(tag = "type")]`).

use std::time::{Duration, SystemTime};

use agglayer_settlement_service::{LiveTaskNotification, NewSettlementAttempt};
use agglayer_storage::stores::EditEvenIfCompleted;
use agglayer_types::{
    Address, CertificateId, ClientErrorType, ContractCallOutcome, Nonce, RpcErrorCode,
    SettlementAttempt, SettlementAttemptResult, SettlementJobId, SettlementJobResult,
    SettlementTxHash, B256,
};
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[cfg(test)]
mod tests;

/// Storage-derived job state: pending while no terminal result exists,
/// completed once it does.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SettlementJobStatus {
    Pending,
    Completed,
}

/// One row returned by `admin_listSettlementJobs`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementJobSummary {
    pub job_id: SettlementJobId,
    pub certificate_id: Option<CertificateId>,
    pub status: SettlementJobStatus,
    /// Whether an in-memory task currently drives the job. A pending job
    /// without a live task is wedged and needs `admin_reloadSettlementTask`.
    pub has_live_task: bool,
    pub attempt_count: u64,
    pub latest_attempt: Option<SettlementAttemptSummary>,
    /// The latest recorded attempt result rendered when it is an error.
    pub last_error: Option<String>,
}

/// Attempt identification fields shown in a job summary.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementAttemptSummary {
    pub attempt_number: u64,
    pub sender_wallet: Address,
    pub nonce: u64,
    pub tx_hash: SettlementTxHash,
}

/// Full job state returned by `admin_getSettlementJob`.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementJobDetail {
    pub job_id: SettlementJobId,
    pub certificate_id: Option<CertificateId>,
    pub status: SettlementJobStatus,
    pub has_live_task: bool,
    pub contract_address: Address,
    pub eth_value: agglayer_types::U256,
    pub gas_limit: u128,
    pub calldata: alloy::primitives::Bytes,
    pub attempts: Vec<SettlementAttemptDetail>,
    pub job_result: Option<SettlementJobResultDto>,
    pub last_error: Option<String>,
}

/// One settlement attempt and its recorded result, if any.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementAttemptDetail {
    pub attempt_number: u64,
    pub sender_wallet: Address,
    pub nonce: u64,
    pub tx_hash: SettlementTxHash,
    pub submission_time_unix_secs: u64,
    pub max_fee_per_gas: u128,
    pub max_priority_fee_per_gas: u128,
    pub result: Option<SettlementAttemptResultDto>,
}

impl SettlementAttemptDetail {
    pub fn new(
        attempt_number: u64,
        attempt: &SettlementAttempt,
        result: Option<&SettlementAttemptResult>,
    ) -> Self {
        Self {
            attempt_number,
            sender_wallet: attempt.sender_wallet,
            nonce: attempt.nonce.0,
            tx_hash: attempt.hash,
            submission_time_unix_secs: attempt
                .submission_time
                .duration_since(SystemTime::UNIX_EPOCH)
                .map(|duration| duration.as_secs())
                .unwrap_or(0),
            max_fee_per_gas: attempt.max_fee_per_gas,
            max_priority_fee_per_gas: attempt.max_priority_fee_per_gas,
            result: result.map(SettlementAttemptResultDto::from),
        }
    }
}

impl From<&SettlementAttemptDetail> for SettlementAttemptSummary {
    fn from(attempt: &SettlementAttemptDetail) -> Self {
        Self {
            attempt_number: attempt.attempt_number,
            sender_wallet: attempt.sender_wallet,
            nonce: attempt.nonce,
            tx_hash: attempt.tx_hash,
        }
    }
}

impl From<&SettlementJobDetail> for SettlementJobSummary {
    fn from(job: &SettlementJobDetail) -> Self {
        Self {
            job_id: job.job_id,
            certificate_id: job.certificate_id,
            status: job.status,
            has_live_task: job.has_live_task,
            attempt_count: job.attempts.len() as u64,
            latest_attempt: job
                .attempts
                .iter()
                .max_by_key(|attempt| attempt.attempt_number)
                .map(SettlementAttemptSummary::from),
            last_error: job.last_error.clone(),
        }
    }
}

/// Recorded outcome of one settlement attempt.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SettlementAttemptResultDto {
    #[serde(rename_all = "camelCase")]
    ClientError { kind: String, message: String },
    #[serde(rename_all = "camelCase")]
    ContractCall {
        outcome: String,
        tx_hash: SettlementTxHash,
        block_number: u64,
        block_hash: B256,
    },
}

/// Stable wire tag for a client error kind. Exhaustive on purpose: every new
/// [`ClientErrorType`] variant must choose its wire representation here.
fn client_error_kind_tag(kind: ClientErrorType) -> &'static str {
    match kind {
        ClientErrorType::Unknown => "unknown",
        ClientErrorType::NonceAlreadyUsed => "nonceAlreadyUsed",
        ClientErrorType::SettlementSucceededElsewhere => "settlementSucceededElsewhere",
        ClientErrorType::AbandonedByAdmin => "abandonedByAdmin",
    }
}

impl From<&SettlementAttemptResult> for SettlementAttemptResultDto {
    fn from(result: &SettlementAttemptResult) -> Self {
        match result {
            SettlementAttemptResult::ClientError(client_error) => Self::ClientError {
                kind: client_error_kind_tag(client_error.kind).to_string(),
                message: client_error.message.clone(),
            },
            SettlementAttemptResult::ContractCall(call) => Self::ContractCall {
                outcome: contract_call_outcome_tag(&call.outcome).to_string(),
                tx_hash: call.tx_hash,
                block_number: call.block_number,
                block_hash: call.block_hash,
            },
        }
    }
}

/// Terminal result of a completed settlement job.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementJobResultDto {
    pub wallet: Address,
    pub nonce: u64,
    pub attempt_number: u64,
    pub outcome: String,
    pub tx_hash: SettlementTxHash,
    pub block_number: u64,
}

impl From<&SettlementJobResult> for SettlementJobResultDto {
    fn from(result: &SettlementJobResult) -> Self {
        Self {
            wallet: result.wallet,
            nonce: result.nonce.0,
            attempt_number: result.attempt_number.0,
            outcome: contract_call_outcome_tag(&result.contract_call_result.outcome).to_string(),
            tx_hash: result.contract_call_result.tx_hash,
            block_number: result.contract_call_result.block_number,
        }
    }
}

fn contract_call_outcome_tag(outcome: &ContractCallOutcome) -> &'static str {
    match outcome {
        ContractCallOutcome::Success => "success",
        ContractCallOutcome::Revert => "revert",
    }
}

/// Render the latest recorded attempt result when it is an error.
pub(crate) fn render_last_error(results: &[(u64, SettlementAttemptResult)]) -> Option<String> {
    let (_, latest) = results.iter().max_by_key(|(number, _)| *number)?;
    match latest {
        SettlementAttemptResult::ClientError(client_error) => Some(format!(
            "{}: {}",
            client_error_kind_tag(client_error.kind),
            client_error.message
        )),
        SettlementAttemptResult::ContractCall(call) => match call.outcome {
            ContractCallOutcome::Revert => Some(format!(
                "Reverted on L1 in tx {} (block {})",
                call.tx_hash, call.block_number
            )),
            ContractCallOutcome::Success => None,
        },
    }
}

/// A settlement attempt as accepted by `admin_insertSettlementAttempt`.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InsertAttemptParams {
    /// Hash of the settlement transaction. The only mandatory field. When L1
    /// knows the transaction, its identity fields are authoritative; otherwise
    /// both `senderWallet` and `nonce` must be provided.
    pub tx_hash: SettlementTxHash,

    /// Wallet the settlement transaction was sent from. Resolved from L1 when
    /// omitted, and validated against L1 when explicit.
    #[serde(default)]
    pub sender_wallet: Option<Address>,

    /// L1 nonce of the settlement transaction. Resolved from L1 when omitted,
    /// and validated against L1 when explicit.
    #[serde(default)]
    pub nonce: Option<u64>,

    /// Unix timestamp (in seconds) at which the transaction was submitted to
    /// L1. Defaults to now. The settlement task uses it as the base of its
    /// retry backoff for this attempt.
    #[serde(default)]
    pub submission_time_unix_secs: Option<u64>,

    /// `max_fee_per_gas` (wei) of the transaction. A fee-bumping retry
    /// outbids this baseline. When omitted, taken from the L1 transaction if
    /// it was fetched, else 0 (which makes a retry start over from freshly
    /// estimated fees).
    #[serde(default)]
    pub max_fee_per_gas: Option<u128>,

    /// `max_priority_fee_per_gas` (wei) of the transaction. Defaulted like
    /// `maxFeePerGas`.
    #[serde(default)]
    pub max_priority_fee_per_gas: Option<u128>,
}

impl From<InsertAttemptParams> for NewSettlementAttempt {
    fn from(params: InsertAttemptParams) -> Self {
        Self {
            tx_hash: params.tx_hash,
            sender_wallet: params.sender_wallet,
            nonce: params.nonce.map(Nonce),
            submission_time: params
                .submission_time_unix_secs
                .map(|seconds| SystemTime::UNIX_EPOCH + Duration::from_secs(seconds)),
            max_fee_per_gas: params.max_fee_per_gas,
            max_priority_fee_per_gas: params.max_priority_fee_per_gas,
        }
    }
}

/// Controls whether a settlement attempt mutation may touch a job that
/// already has a terminal result.
///
/// Passed as the optional trailing `force` parameter of the attempt
/// mutations; omitting it is equivalent to `"force=false"`.
///
/// Editing a completed job's attempts is refused by default. Forcing exists
/// to prepare `admin_forceRemoveSettlementJobResult`: attempt-result
/// corrections must land while the job still has its terminal result,
/// because removing the result immediately respawns the task, which could
/// re-derive and re-record the job result from the uncorrected attempts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize)]
pub enum Force {
    /// Apply the mutation even if the job already has a terminal result.
    #[serde(rename = "force=true")]
    True,

    /// Refuse the mutation on a job that already has a terminal result.
    #[serde(rename = "force=false")]
    False,
}

pub(crate) fn edit_even_if_completed(force: Option<Force>) -> EditEvenIfCompleted {
    match force {
        Some(Force::True) => EditEvenIfCompleted::Yes,
        Some(Force::False) | None => EditEvenIfCompleted::No,
    }
}

/// Outcome of a settlement admin mutation.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MutationResponse {
    /// The attempt number the mutation landed on.
    pub attempt_number: u64,

    /// Whether a reload command was queued for the live task. `queued` is
    /// not a promptness promise: the task drains its command queue only at
    /// run-loop control checks, so a task parked in an L1 wait acts on stale
    /// state until that wait returns. The retry policy caps individual
    /// backoff sleeps, not total wait duration; settlement polling can
    /// continue until the configured settlement policy is satisfied.
    /// Anything but `queued` means not even that happened
    /// (`admin_reloadSettlementTask` is the manual escape hatch). Follow
    /// the abort → edit → reload flow when edits must be observed promptly.
    pub live_task: LiveTaskNotification,
}

/// Turns a settlement-service error report into the private admin RPC error
/// contract.
pub(crate) fn map_admin_error(report: eyre::Report) -> Error {
    match report.downcast_ref::<RpcErrorCode>() {
        Some(&code) => Error::Classified {
            code,
            message: format!("{report:?}"),
        },
        None => {
            tracing::error!(?report, "Admin operation failed with unclassified error");
            Error::internal(format!("{report:?}"))
        }
    }
}
