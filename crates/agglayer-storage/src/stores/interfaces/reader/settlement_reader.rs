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

    /// Returns the terminal result for `settlement_job_id`, if present.
    fn get_settlement_job_result(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Option<TxResult>, Error>;

    /// Returns all settlement attempts stored for `settlement_job_id`.
    ///
    /// Returned tuples are `(attempt_sequence_number, settlement_attempt)`.
    fn list_settlement_attempts(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Vec<(u64, SettlementAttempt)>, Error>;

    /// Returns all stored attempt results for `settlement_job_id`.
    ///
    /// Returned tuples are `(attempt_sequence_number, tx_result)`.
    fn list_settlement_attempt_results(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Vec<(u64, TxResult)>, Error>;
}
