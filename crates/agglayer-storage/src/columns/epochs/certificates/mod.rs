use crate::columns::PER_EPOCH_CERTIFICATES_CF;

/// Column family for the certificates in an epoch.
///
/// ## Column definition
///
/// | key                | value         |
/// | --                 | --            |
/// | `CertificateIndex` | `Certificate` |
pub struct CertificatePerIndex;

impl crate::columns::ColumnSchema for CertificatePerIndex {
    type Key = crate::types::CertificateIndex;
    type Value = crate::types::Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_CERTIFICATES_CF;
}
