use agglayer_types::{Certificate, CertificateIndex};

use crate::{
    columns::{PER_EPOCH_CERTIFICATES_CF, PER_EPOCH_CERTIFICATES_PROTO_CF},
    types::LegacyCertificate,
};

/// Legacy column family for the certificates in an epoch.
///
/// Kept readable so the proto migration can backfill existing rows. The CF
/// historically received both bincode rows and (later) proto rows, so its
/// `Value` codec is `LegacyCertificate`, which
/// accepts both. Runtime reads and writes go through
/// [`CertificatePerIndexProtoColumn`].
///
/// **Transitional:** this CF will be dropped in a follow-up ticket once the
/// proto migration has been validated in production.
///
/// ## Column definition
///
/// | key                | value               |
/// | --                 | --                  |
/// | `CertificateIndex` | `LegacyCertificate` |
pub struct CertificatePerIndexColumn;

impl crate::schema::ColumnSchema for CertificatePerIndexColumn {
    type Key = CertificateIndex;
    type Value = LegacyCertificate;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_CERTIFICATES_CF;
}

/// Proto-backed column family for certificates in an epoch.
///
/// ## Column definition
///
/// | key                | value         |
/// | --                 | --            |
/// | `CertificateIndex` | `Certificate` |
pub struct CertificatePerIndexProtoColumn;

impl crate::schema::ColumnSchema for CertificatePerIndexProtoColumn {
    type Key = CertificateIndex;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_CERTIFICATES_PROTO_CF;
}
