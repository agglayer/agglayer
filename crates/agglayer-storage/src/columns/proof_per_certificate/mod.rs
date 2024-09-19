use agglayer_types::{CertificateId, Proof};

use super::{ColumnSchema, PROOF_PER_CERTIFICATE_CF};

#[cfg(test)]
mod tests;

/// Column family that returns the generated proof for one certificate.
///
/// ## Column definition
///
/// | key             | value   |
/// | --              | --      |
/// | `CertificateId` | `Proof` |
pub struct ProofPerCertificateColumn;

impl ColumnSchema for ProofPerCertificateColumn {
    type Key = CertificateId;
    type Value = Proof;

    const COLUMN_FAMILY_NAME: &'static str = PROOF_PER_CERTIFICATE_CF;
}
