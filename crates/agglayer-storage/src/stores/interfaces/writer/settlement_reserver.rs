use agglayer_types::{CertificateId, SettlementJobId};

use crate::error::Error;

/// Reserve a settlement job id for a certificate.
pub trait SettlementJobReserver: Send + Sync {
    /// Get-or-create: returns the existing settlement job id for
    /// `certificate_id` if one is already mapped; otherwise mints a fresh
    /// unique [`SettlementJobId`], verifies its uniqueness against the
    /// reverse index, and writes the forward and reverse mappings
    /// atomically.
    fn reserve_settlement_job(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<SettlementJobId, Error>;
}
