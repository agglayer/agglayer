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
    /// exception: a client-error write over an admin-abandoned result
    /// reports success without overwriting it, since the admin assertion
    /// outranks any client-side note (and the writing task may not have
    /// observed the override yet).
    fn record_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_sequence_number: u64,
        tx_result: &SettlementAttemptResult,
    ) -> Result<(), Error>;

    /// Appends a new settlement attempt to `settlement_job_id` under the next
    /// unused attempt sequence number, and returns that number.
    ///
    /// This never overwrites an existing attempt. It fails if the job does
    /// not exist, or if it already has a terminal result and
    /// `edit_even_if_completed` is [`EditEvenIfCompleted::No`].
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
    /// It fails if the attempt does not exist, or if the job already has a
    /// terminal result and `edit_even_if_completed` is
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
    /// It fails if the attempt does not exist, no result is recorded, or if
    /// the job already has a terminal result and `edit_even_if_completed` is
    /// [`EditEvenIfCompleted::No`].
    fn admin_remove_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_number: u64,
        edit_even_if_completed: EditEvenIfCompleted,
    ) -> Result<(), Error>;

    /// Removes the terminal result of `settlement_job_id`, turning the job
    /// back into a pending one that a settlement task will re-drive.
    ///
    /// This un-completes a job: if the removed result was real, only the
    /// settlement contract's replay protection stands between the re-driven
    /// job and a double settlement. It fails if the job does not exist or has
    /// no terminal result.
    fn admin_force_remove_settlement_job_result(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<(), Error>;
}
