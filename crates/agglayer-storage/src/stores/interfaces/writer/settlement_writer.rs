use ulid::Ulid;

use crate::{
    error::Error,
    types::generated::agglayer::storage::v0::{SettlementAttempt, SettlementJob, TxResult},
};

/// Write access to settlement-related records stored in RocksDB.
///
/// All write operations in this trait are insert-only: implementations must
/// reject attempts to overwrite an existing key.
pub trait SettlementWriter: Send + Sync {
    /// Inserts a settlement job under `settlement_job_id`.
    ///
    /// This is an insert-only operation and must fail if
    /// `settlement_job_id` already exists.
    fn insert_settlement_job(
        &self,
        settlement_job_id: &Ulid,
        settlement_job: &SettlementJob,
    ) -> Result<(), Error>;

    /// Inserts a settlement attempt under `(settlement_job_id,
    /// attempt_sequence_number)`.
    ///
    /// This is an insert-only operation and must fail if that composite key
    /// already exists.
    fn insert_settlement_attempt(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
        settlement_attempt: &SettlementAttempt,
    ) -> Result<(), Error>;

    /// Inserts a settlement attempt result under
    /// `(settlement_job_id, attempt_sequence_number)`.
    ///
    /// This is an insert-only operation and must fail if that composite key
    /// already exists.
    fn insert_settlement_attempt_result(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
        tx_result: &TxResult,
    ) -> Result<(), Error>;
}
