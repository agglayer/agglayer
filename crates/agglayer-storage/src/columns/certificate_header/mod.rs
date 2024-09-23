use agglayer_types::CertificateHeader;

use super::{ColumnSchema, CERTIFICATE_HEADER_CF};

#[cfg(test)]
mod tests;

/// Column family for the certificate headers.
///
/// ## Column definition
///
/// | key               | value                |
/// | --                | --                   |
/// | `CertificateId`   |  `CertificateHeader` |
pub struct CertificateHeaderColumn;

pub type Key = agglayer_types::CertificateId;
pub type Value = CertificateHeader;

impl ColumnSchema for CertificateHeaderColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = CERTIFICATE_HEADER_CF;
}
