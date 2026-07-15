//! Wire types for the settlement admin read methods.
//!
//! The settlement domain types in `agglayer-types` carry no serde; the
//! JSON representation is owned here, at the RPC boundary (same pattern
//! as `TokenBalanceEntry` for `admin_getTokenBalance`).
//!
//! Data-carrying enums here are deliberately internally tagged
//! (`#[serde(tag = "type")]`), unlike the error family in
//! [`crate::error`], which keeps serde's default externally-tagged
//! payloads.

use std::time::SystemTime;

use agglayer_types::{
    Address, CertificateId, ClientErrorType, ContractCallOutcome, SettlementAttempt,
    SettlementAttemptResult, SettlementJobId, SettlementJobResult, SettlementTxHash, B256,
};
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Storage-derived job state: pending while no terminal result row
/// exists, completed once it does.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SettlementJobStatus {
    Pending,
    Completed,
}

/// One row of `admin_listSettlementJobs`.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementJobSummary {
    pub job_id: SettlementJobId,
    /// Certificate linked to the job. Null for jobs created without a
    /// certificate and for jobs linked before the reverse link existed.
    pub certificate_id: Option<CertificateId>,
    pub status: SettlementJobStatus,
    /// Whether an in-memory task currently drives the job. A pending
    /// job without a live task is wedged:
    /// use `admin_reloadAndRestartSettlementTask`.
    pub has_live_task: bool,
    pub attempt_count: u64,
    pub latest_attempt: Option<SettlementAttemptSummary>,
    /// Human-readable rendering of the most recent attempt result when
    /// it is a failure (client error or on-chain revert), null
    /// otherwise.
    pub last_error: Option<String>,
}

/// Attempt identification fields shown in the job list.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementAttemptSummary {
    pub attempt_number: u64,
    pub sender_wallet: Address,
    pub nonce: u64,
    pub tx_hash: SettlementTxHash,
}

impl From<(u64, &SettlementAttempt)> for SettlementAttemptSummary {
    fn from((attempt_number, attempt): (u64, &SettlementAttempt)) -> Self {
        Self {
            attempt_number,
            sender_wallet: attempt.sender_wallet,
            nonce: attempt.nonce.0,
            tx_hash: attempt.hash,
        }
    }
}

/// Full job detail returned by `admin_getSettlementJob`.
///
/// `gas_limit` is rendered as a JSON number; realistic values stay far
/// below 2^53, so this is safe for standard JSON tooling.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettlementJobDetail {
    pub job_id: SettlementJobId,
    pub certificate_id: Option<CertificateId>,
    pub status: SettlementJobStatus,
    pub has_live_task: bool,
    pub contract_address: Address,
    /// Rendered in alloy's JSON form for `U256`: a hex quantity string.
    pub eth_value: agglayer_types::U256,
    pub gas_limit: u128,
    /// Full settlement calldata, hex-encoded. Admin-only surface, so
    /// the size is acceptable.
    pub calldata: alloy::primitives::Bytes,
    pub attempts: Vec<SettlementAttemptDetail>,
    pub job_result: Option<SettlementJobResultDto>,
    pub last_error: Option<String>,
}

/// One attempt with its recorded result, as returned in the job detail.
///
/// `max_fee_per_gas` and `max_priority_fee_per_gas` are rendered as
/// JSON numbers; realistic values stay far below 2^53, so this is safe
/// for standard JSON tooling.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Recorded outcome of one attempt.
#[derive(Clone, Debug, Serialize, Deserialize)]
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

/// Stable wire tag for a client error kind. Exhaustive on purpose: a new
/// `ClientErrorType` variant must pick its wire tag here.
fn client_error_kind_tag(kind: ClientErrorType) -> &'static str {
    match kind {
        ClientErrorType::Unknown => "unknown",
        ClientErrorType::NonceAlreadyUsed => "nonceAlreadyUsed",
        ClientErrorType::SettlementSucceededElsewhere => "settlementSucceededElsewhere",
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
                outcome: match call.outcome {
                    ContractCallOutcome::Success => "success".to_string(),
                    ContractCallOutcome::Revert => "revert".to_string(),
                },
                tx_hash: call.tx_hash,
                block_number: call.block_number,
                block_hash: call.block_hash,
            },
        }
    }
}

/// Terminal result of a completed job.
#[derive(Clone, Debug, Serialize, Deserialize)]
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
            outcome: match result.contract_call_result.outcome {
                ContractCallOutcome::Success => "success".to_string(),
                ContractCallOutcome::Revert => "revert".to_string(),
            },
            tx_hash: result.contract_call_result.tx_hash,
            block_number: result.contract_call_result.block_number,
        }
    }
}

/// Render the most recent attempt result as an operator-facing error
/// string, or `None` when the latest recorded state is not a failure.
// Only unit tests call this until the settlement admin read methods land
// in the next commit.
#[allow(dead_code)]
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

/// Build one list row from its storage and service inputs.
// Used by the settlement admin read methods in the next commit.
#[allow(dead_code)]
pub(crate) fn build_job_summary(
    job_id: SettlementJobId,
    certificate_id: Option<CertificateId>,
    has_live_task: bool,
    job_result: Option<&SettlementJobResult>,
    attempts: &[(u64, SettlementAttempt)],
    attempt_results: &[(u64, SettlementAttemptResult)],
) -> SettlementJobSummary {
    let latest_attempt = attempts
        .iter()
        .max_by_key(|(number, _)| *number)
        .map(|(number, attempt)| SettlementAttemptSummary::from((*number, attempt)));
    SettlementJobSummary {
        job_id,
        certificate_id,
        status: if job_result.is_some() {
            SettlementJobStatus::Completed
        } else {
            SettlementJobStatus::Pending
        },
        has_live_task,
        attempt_count: attempts.len() as u64,
        latest_attempt,
        last_error: render_last_error(attempt_results),
    }
}
