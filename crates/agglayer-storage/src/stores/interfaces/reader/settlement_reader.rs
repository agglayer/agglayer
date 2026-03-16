use agglayer_types::{CertificateId, SettlementJob, SettlementJobResult};
use ulid::Ulid;

use crate::error::Error;

/// Read-only access to settlement-related records stored in RocksDB.
///
/// This trait is intentionally limited to point lookups and metadata-style
/// reads. Missing records are returned as `Ok(None)`.
pub trait SettlementReader: Send + Sync {
    /// Returns the settlement job id linked to `certificate_id`, if present.
    fn get_settlement_job_id_for_certificate(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<Ulid>, Error>;

    /// Returns the settlement job linked to `certificate_id`, if present.
    fn get_settlement_job_for_certificate(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<SettlementJob>, Error>;

    /// Returns the settlement job for `settlement_job_id`, if present.
    fn get_settlement_job(&self, settlement_job_id: &Ulid) -> Result<Option<SettlementJob>, Error>;

    /// Returns the terminal result for `settlement_job_id`, if present.
    fn get_settlement_job_result(
        &self,
        settlement_job_id: &Ulid,
    ) -> Result<Option<SettlementJobResult>, Error>;
}
