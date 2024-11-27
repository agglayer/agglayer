use agglayer_types::{Certificate, CertificateId};

use super::{ColumnSchema, DEBUG_CERTIFICATES_CF};

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
