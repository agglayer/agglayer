use agglayer_types::{CertificateIndex, Proof};

use crate::columns::PER_EPOCH_PROOFS_CF;

/// Column family for the proofs in an epoch.
///
/// ## Column definition
///
/// | key                | value   |
/// | --                 | --      |
/// | `CertificateIndex` | `Proof` |
pub struct ProofPerIndex;

impl crate::columns::ColumnSchema for ProofPerIndex {
    type Key = CertificateIndex;
    type Value = Proof;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_PROOFS_CF;
}
