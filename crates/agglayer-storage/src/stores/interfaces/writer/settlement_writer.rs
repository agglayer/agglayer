use agglayer_types::{
    SettlementAttempt, SettlementAttemptResult, SettlementJob, SettlementJobId, SettlementJobResult,
};

use crate::error::Error;

/// Write access to settlement-related records stored in RocksDB.
///
/// Settlement job and attempt writes are insert-only. Settlement attempt
/// results may be upgraded by `record_settlement_attempt_result` when stronger
/// final evidence supersedes a previous client-side error.
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
    /// nonce/on-chain evidence. Other conflicting updates must fail.
    fn record_settlement_attempt_result(
        &self,
        settlement_job_id: &SettlementJobId,
        attempt_sequence_number: u64,
        tx_result: &SettlementAttemptResult,
    ) -> Result<(), Error>;
}
