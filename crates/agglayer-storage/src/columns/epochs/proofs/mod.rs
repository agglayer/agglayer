use agglayer_types::{CertificateIndex, Proof};

use crate::{
    columns::{PER_EPOCH_PROOFS_CF, PER_EPOCH_PROOFS_PROTO_CF},
    types::LegacyProof,
};

/// Legacy column family for the proofs in an epoch.
///
/// Kept readable so the proto migration can backfill existing rows. Runtime
/// reads and writes go through [`ProofPerIndexProtoColumn`].
///
/// ## Column definition
///
/// | key                | value         |
/// | --                 | --            |
/// | `CertificateIndex` | `LegacyProof` |
pub struct ProofPerIndexColumn;

impl crate::schema::ColumnSchema for ProofPerIndexColumn {
    type Key = CertificateIndex;
    type Value = LegacyProof;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_PROOFS_CF;
}

/// Proto-backed column family for the proofs in an epoch.
///
/// ## Column definition
///
/// | key                | value   |
/// | --                 | --      |
/// | `CertificateIndex` | `Proof` |
pub struct ProofPerIndexProtoColumn;

impl crate::schema::ColumnSchema for ProofPerIndexProtoColumn {
    type Key = CertificateIndex;
    type Value = Proof;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_PROOFS_PROTO_CF;
}
