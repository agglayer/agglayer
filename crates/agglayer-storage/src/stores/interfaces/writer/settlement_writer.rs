use agglayer_types::{
    SettlementAttempt, SettlementAttemptResult, SettlementJob, SettlementJobId, SettlementJobResult,
};

use crate::error::Error;

/// Whether an `admin_*` attempt mutation may touch a job that already has a
/// terminal result.
///
/// Editing a completed job's attempts is normally refused: the job is never
/// re-driven, so the edit could only create inconsistencies. The exception is
/// preparing the removal of a wrong terminal result: corrections to attempt
/// results must land *before* `admin_force_remove_settlement_job_result`,
/// because the removal immediately respawns the task, which could re-derive
/// and re-record the job result from the uncorrected attempts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EditEvenIfCompleted {
    Yes,
    No,
}

/// Write access to settlement-related records stored in RocksDB.
///
/// Settlement job and attempt writes are insert-only. Settlement attempt
/// results may be upgraded by `record_settlement_attempt_result` when stronger
/// final evidence supersedes a previous client-side error.
///
/// The `admin_*` methods exist for the settlement admin surface only. They
/// deliberately relax those invariants: attempt sequence numbers are assigned
/// by the store instead of the caller, and results may be overwritten or
/// removed regardless of the upgrade-only rule. The settlement task itself
/// must never call them.
///
/// Callers must orchestrate admin mutations as **abort → edit → reload**:
/// stop the live settlement task before changing stored state, then reload it
/// after all edits are complete. Conflict-avoidance behavior in the regular
/// writers is a safety net for already in-flight writes, not a substitute for
/// stopping the task. Any deviation requires an explicit, well-justified
/// reason.
pub trait SettlementWriter: Send + Sync {
    /// Inserts a settlement job under `settlement_job_id`.
    ///
    /// This is an insert-only operation and must fail if
    /// `settlement_job_id` already exists.
    fn insert_settlement_job(
        &self,
        settlement_job_id: &SettlementJobId,
        settlement_job: &SettlementJob,
    ) -> Result<(), Error>;

    /// Inserts a terminal settlement job result under `settlement_job_id`.
    ///
    /// This is an insert-only operation and must fail if
    /// `settlement_job_id` already has a stored result. The parent settlement
    /// job must already exist.
    fn insert_settlement_job_result(
        &self,
        settlement_job_id: &SettlementJobId,
        tx_result: &SettlementJobResult,
    ) -> Result<(), Error>;

    /// Inserts a settlement attempt under `(settlement_job_id,
    /// attempt_sequence_number)`.
    ///
    /// This is an insert-only operation and must fail if that composite key
    /// already exists. The parent settlement job must already exist.
    fn insert_settlement_attempt(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_sequence_number: u64,
        settlement_attempt: &SettlementAttempt,
    ) -> Result<(), Error>;

    /// Records a settlement attempt result under `(settlement_job_id,
    /// attempt_sequence_number)`.
    ///
    /// This inserts missing results, accepts idempotent re-recording, and
    /// allows a previous client error to be replaced by stronger final
    /// nonce/on-chain evidence. Other conflicting updates must fail, with one
    /// exception: a client-error write over an admin-abandoned result reports
    /// success without overwriting it, since the admin assertion outranks any
    /// client-side note (and the writing task may not have observed the
    /// override yet).
    fn record_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_sequence_number: u64,
        tx_result: &SettlementAttemptResult,
    ) -> Result<(), Error>;

    /// Appends a new settlement attempt to `settlement_job_id` under the next
    /// unused attempt sequence number, and returns that number.
    ///
    /// This never overwrites an existing attempt. It fails with
    /// [`Error::SettlementJobNotFound`] if the job does not exist, or with
    /// [`Error::SettlementJobAlreadyCompleted`] if it already has a terminal
    /// result and `edit_even_if_completed` is [`EditEvenIfCompleted::No`].
    ///
    /// Known limitation: before the live task observes its reload, it may
    /// collide with the store-assigned attempt number and panic; the job
    /// recovers on reload or restart. The per-job storage lock serializes
    /// admin writers, but not admin assignment against task-side numbering.
    /// Fully closing this race requires the out-of-scope pause mechanism.
    fn admin_insert_settlement_attempt(
        &self,
        settlement_job_id: &SettlementJobId,
        settlement_attempt: &SettlementAttempt,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> Result<u64, Error>;

    /// Force-writes the result of the settlement attempt at
    /// `(settlement_job_id, attempt_number)`, overwriting any previously
    /// recorded result regardless of the upgrade-only rule.
    ///
    /// It fails with [`Error::SettlementJobNotFound`] if the job does not
    /// exist, with [`Error::SettlementAttemptNotFound`] if the attempt does
    /// not exist, or with [`Error::SettlementJobAlreadyCompleted`] if the job
    /// already has a terminal result and `edit_even_if_completed` is
    /// [`EditEvenIfCompleted::No`].
    fn admin_override_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_number: u64,
        tx_result: &SettlementAttemptResult,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> Result<(), Error>;

    /// Removes the recorded result of the settlement attempt at
    /// `(settlement_job_id, attempt_number)`, handing the attempt back to the
    /// settlement task as pending.
    ///
    /// It fails with [`Error::SettlementJobNotFound`] if the job does not
    /// exist, with [`Error::SettlementAttemptNotFound`] if the attempt does
    /// not exist, with [`Error::SettlementAttemptResultNotRecorded`] if no
    /// result is recorded, or with [`Error::SettlementJobAlreadyCompleted`]
    /// if the job already has a terminal result and
    /// `edit_even_if_completed` is [`EditEvenIfCompleted::No`].
    fn admin_remove_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_number: u64,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> Result<(), Error>;

    /// Removes the terminal result of `settlement_job_id`, turning it back
    /// into a pending job that a settlement task will re-drive.
    ///
    /// Attempts and their results are untouched. This is a force operation:
    /// if the removed result was real, only the settlement contract's replay
    /// protection stands between the re-driven job and a double settlement.
    /// It fails with [`Error::SettlementJobNotFound`] if the job does not
    /// exist, or with [`Error::SettlementJobNotCompleted`] if it has no
    /// terminal result.
    fn admin_force_remove_settlement_job_result(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<(), Error>;
}
