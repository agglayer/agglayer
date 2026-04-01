use agglayer_types::{
    SettlementAttempt, SettlementAttemptResult, SettlementJob, SettlementJobResult,
};
use ulid::Ulid;

use crate::error::Error;

/// Read-only access to settlement-related records stored in RocksDB.
///
/// Point lookups return `Ok(None)` when records are missing. Prefix-scoped
/// list reads return an empty vector when no records are found.
pub trait SettlementReader: Send + Sync {
    /// Returns the settlement job for `settlement_job_id`, if present.
    fn get_settlement_job(&self, settlement_job_id: &Ulid) -> Result<Option<SettlementJob>, Error>;

    /// Returns the terminal result for `settlement_job_id`, if present.
    fn get_settlement_job_result(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Option<SettlementJobResult>, Error>;

    /// Returns all settlement attempts recorded for `settlement_job_id`.
    fn list_settlement_attempts(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Vec<(u64, SettlementAttempt)>, Error>;

    /// Returns all settlement attempt results recorded for `settlement_job_id`.
    fn list_settlement_attempt_results(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Vec<(u64, SettlementAttemptResult)>, Error>;
}
