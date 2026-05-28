use agglayer_types::{CertificateId, Proof};

use super::{ColumnSchema, PROOF_PER_CERTIFICATE_CF, PROOF_PER_CERTIFICATE_PROTO_CF};
use crate::types::LegacyProof;

#[cfg(test)]
mod tests;

/// Legacy column family that returns the generated proof for one certificate.
///
/// Kept readable so the proto migration can backfill existing rows. Runtime
/// reads and writes go through [`ProofPerCertificateProtoColumn`].
///
/// ## Column definition
///
/// | key             | value         |
/// | --              | --            |
/// | `CertificateId` | `LegacyProof` |
pub struct ProofPerCertificateColumn;

impl ColumnSchema for ProofPerCertificateColumn {
    type Key = CertificateId;
    type Value = LegacyProof;

    const COLUMN_FAMILY_NAME: &'static str = PROOF_PER_CERTIFICATE_CF;
}

/// Proto-backed column family that returns the generated proof for one
/// certificate.
///
/// ## Column definition
///
/// | key             | value   |
/// | --              | --      |
/// | `CertificateId` | `Proof` |
pub struct ProofPerCertificateProtoColumn;

impl ColumnSchema for ProofPerCertificateProtoColumn {
    type Key = CertificateId;
    type Value = Proof;

    const COLUMN_FAMILY_NAME: &'static str = PROOF_PER_CERTIFICATE_PROTO_CF;
}
