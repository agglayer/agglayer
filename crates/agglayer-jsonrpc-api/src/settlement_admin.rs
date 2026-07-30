//! Shared types and error handling for settlement administration RPC methods.

use agglayer_settlement_service::LiveTaskNotification;
use agglayer_storage::stores::EditEvenIfCompleted;
use agglayer_types::RpcErrorCode;
use serde::{Deserialize, Serialize};

use crate::error::Error;

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
    /// state until that wait returns. The retry cap bounds nonce-inclusion
    /// waits only; a settlement wait runs until the configured settlement
    /// policy is satisfied. Anything but `queued` means not even that happened
    /// (`admin_reloadSettlementTask` is the manual escape hatch). Follow the
    /// abort → edit → reload flow when edits must be observed promptly.
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
