use agglayer_types::{Certificate, CertificateIndex};

use crate::columns::{PER_EPOCH_CERTIFICATES_CF, PER_EPOCH_CERTIFICATES_PROTO_CF};

/// Column family for the certificates in an epoch.
///
/// ## Column definition
///
/// | key                | value         |
/// | --                 | --            |
/// | `CertificateIndex` | `Certificate` |
pub struct CertificatePerIndexColumn;

impl crate::schema::ColumnSchema for CertificatePerIndexColumn {
    type Key = CertificateIndex;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_CERTIFICATES_CF;
}

/// Proto-backed column family for certificates in an epoch.
pub struct CertificatePerIndexProtoColumn;

impl crate::schema::ColumnSchema for CertificatePerIndexProtoColumn {
    type Key = CertificateIndex;
    type Value = Certificate;

    const COLUMN_FAMILY_NAME: &'static str = PER_EPOCH_CERTIFICATES_PROTO_CF;
}
