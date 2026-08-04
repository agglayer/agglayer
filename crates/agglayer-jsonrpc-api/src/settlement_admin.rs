//! Shared types and error handling for settlement administration RPC methods.

use std::time::{Duration, SystemTime};

use agglayer_settlement_service::{LiveTaskNotification, NewSettlementAttempt};
use agglayer_storage::stores::EditEvenIfCompleted;
use agglayer_types::{Address, Nonce, RpcErrorCode, SettlementTxHash};
use serde::{Deserialize, Serialize};

use crate::error::Error;

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
