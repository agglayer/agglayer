use agglayer_types::{Certificate, CertificateId};

use super::{ColumnSchema, DEBUG_CERTIFICATES_CF, DEBUG_CERTIFICATES_PROTO_CF};
use crate::types::LegacyCertificate;

/// Legacy column family for debug certificates.
///
/// Kept readable so the proto migration can backfill existing rows. The CF
/// historically received both bincode rows and (later) proto rows, so its
/// `Value` codec is `LegacyCertificate`, which
/// accepts both. Runtime reads and writes go through
/// [`DebugCertificatesProtoColumn`].
///
/// **Transitional:** this CF will be dropped in a follow-up ticket once the
/// proto migration has been validated in production.
///
/// ## Column definition
///
/// | key             | value               |
/// | --              | --                  |
/// | `CertificateId` | `LegacyCertificate` |
pub(crate) struct DebugCertificatesColumn;

impl ColumnSchema for DebugCertificatesColumn {
    type Key = CertificateId;
    type Value = LegacyCertificate;

    const COLUMN_FAMILY_NAME: &'static str = DEBUG_CERTIFICATES_CF;
}

/// Proto-backed column family containing debug certificates.
///
/// ## Column definition
///
/// | key             | value         |
/// | --              | --            |
/// | `CertificateId` | `Certificate` |
pub(crate) struct DebugCertificatesProtoColumn;

impl ColumnSchema for DebugCertificatesProtoColumn {
    type Key = CertificateId;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = DEBUG_CERTIFICATES_PROTO_CF;
}
