use ulid::Ulid;

use crate::{
    error::Error,
    types::generated::agglayer::storage::v0::{SettlementAttempt, SettlementJob, TxResult},
};

/// Read-only access to settlement-related records stored in RocksDB.
///
/// This trait is intentionally limited to point lookups and metadata-style
/// reads. Missing records are returned as `Ok(None)`.
pub trait SettlementReader: Send + Sync {
    /// Returns the settlement job for `settlement_job_id`, if present.
    fn get_settlement_job(&self, settlement_job_id: &Ulid) -> Result<Option<SettlementJob>, Error>;

    /// Returns the settlement attempt identified by
    /// `(settlement_job_id, attempt_sequence_number)`, if present.
    fn get_settlement_attempt(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
    ) -> Result<Option<SettlementAttempt>, Error>;

    /// Returns the stored result for an attempt identified by
    /// `(settlement_job_id, attempt_sequence_number)`, if present.
    ///
    /// Note: If a result doesn't exist for the attempt, it can mean two things:
    /// 1. The attempt doesn't exist
    /// 2. The attempt exists but hasn't been finalized yet (i.e., its result
    ///    hasn't been stored yet).
    fn get_settlement_attempt_result(
        &self,
        settlement_job_id: &Ulid,
        attempt_sequence_number: u64,
    ) -> Result<Option<TxResult>, Error>;

    /// Returns the latest attempt sequence number for `settlement_job_id`.
    ///
    /// If no attempt exists for the job, returns `Ok(None)`.
    fn get_latest_settlement_attempt_sequence_number(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Option<u64>, Error>;
}
