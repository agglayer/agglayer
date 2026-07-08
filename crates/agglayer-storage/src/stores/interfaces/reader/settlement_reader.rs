use agglayer_types::{
    Address, Nonce, SettlementAttempt, SettlementAttemptResult, SettlementJob, SettlementJobId,
    SettlementJobResult,
};

use crate::error::Error;

/// Read-only access to settlement-related records stored in RocksDB.
///
/// Point lookups return `Ok(None)` when records are missing. Prefix-scoped
/// list reads return an empty vector when no records are found.
pub trait SettlementReader: Send + Sync {
    /// Returns every known settlement job id.
    fn list_settlement_job_ids(&self) -> Result<Vec<SettlementJobId>, Error>;

    /// Returns the settlement job for `settlement_job_id`, if present.
    fn get_settlement_job(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Option<SettlementJob>, Error>;

    /// Returns the terminal result for `settlement_job_id`, if present.
    fn get_settlement_job_result(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Option<SettlementJobResult>, Error>;

    /// Returns all settlement attempts recorded for `settlement_job_id`.
    fn list_settlement_attempts(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Vec<(u64, SettlementAttempt)>, Error>;

    /// Returns all settlement attempt results recorded for `settlement_job_id`.
    fn list_settlement_attempt_results(
        &self,
        settlement_job_id: &SettlementJobId,
    ) -> Result<Vec<(u64, SettlementAttemptResult)>, Error>;

    /// Returns the highest settlement attempt nonce recorded for `wallet`.
    fn max_settlement_nonce_for_wallet(&self, wallet: Address) -> Result<Option<Nonce>, Error>;
}
