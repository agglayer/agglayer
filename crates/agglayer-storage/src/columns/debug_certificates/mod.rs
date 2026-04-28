use agglayer_types::{Certificate, CertificateId};

use super::{ColumnSchema, DEBUG_CERTIFICATES_CF, DEBUG_CERTIFICATES_PROTO_CF};

/// Column family containing the certificates received.
///
/// ## Column definition
///
/// | key             | value           |
/// | --              | --              |
/// | `CertificateId` | `Certificate`   |
pub(crate) struct DebugCertificatesColumn;

impl ColumnSchema for DebugCertificatesColumn {
    type Key = CertificateId;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = DEBUG_CERTIFICATES_CF;
}

/// Proto-backed column family containing debug certificates.
pub(crate) struct DebugCertificatesProtoColumn;

impl ColumnSchema for DebugCertificatesProtoColumn {
    type Key = CertificateId;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = DEBUG_CERTIFICATES_PROTO_CF;
}
