//! Typed errors for the settlement service admin surface.
//!
//! [`SettlementAdminError`] classifies a failed admin command by the job
//! state found in storage, so operators know which recovery step applies.

use agglayer_types::SettlementJobId;

/// Errors returned by the admin surface of the settlement service.
///
/// The variants distinguish the cases an operator must react to
/// differently: a job that does not exist, a job that is already
/// completed, and a pending job whose in-memory task is dead.
#[derive(Debug, thiserror::Error)]
pub enum SettlementAdminError {
    /// No settlement job with this id exists in storage.
    #[error("No settlement job found for id {0}")]
    JobNotFound(SettlementJobId),

    /// The job already has a terminal result recorded in storage.
    #[error("Settlement job {0} is already completed")]
    JobCompleted(SettlementJobId),

    /// The job is pending in storage but no in-memory task is running.
    /// Recover with reload-and-restart.
    #[error("No live settlement task for job {0}")]
    NoLiveTask(SettlementJobId),

    /// The live task did not accept the admin command
    /// (admin channel full or closed).
    #[error("Settlement task for job {job_id} did not accept the admin command: {reason}")]
    TaskNotResponding {
        job_id: SettlementJobId,
        reason: String,
    },

    /// Reloading the task state from storage failed.
    #[error("Failed to reload settlement task for job {job_id}: {reason}")]
    ReloadFailed {
        job_id: SettlementJobId,
        reason: String,
    },

    /// A storage read failed while classifying the job state.
    #[error("Storage error while handling settlement admin command for job {job_id}")]
    Storage {
        job_id: SettlementJobId,
        #[source]
        source: agglayer_storage::error::Error,
    },
}
