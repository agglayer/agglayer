use crate::columns::PER_EPOCH_CERTIFICATES_CF;

/// Column family for the certificates in an epoch.
///
/// | --- key --- |    | --- value --------- |
/// | index       | => | certificate bytes   |
pub struct CertificatePerIndex;

impl crate::columns::ColumnSchema for CertificatePerIndex {
    type Key = crate::types::CertificateIndex;
    type Value = crate::types::Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_CERTIFICATES_CF;
}
