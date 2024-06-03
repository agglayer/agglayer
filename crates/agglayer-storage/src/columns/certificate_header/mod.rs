use super::{ColumnSchema, CERTIFICATE_HEADER_CF};
use crate::types::{CertificateHeader, CertificateId};

#[cfg(test)]
mod tests;

/// Column family for the certificate headers.
///
/// | --- key ------- |    | --- value --------- |
/// | Certificate id  | => | Certificate header  |
pub struct CertificateHeaderColumn;

pub type Key = CertificateId;
pub type Value = CertificateHeader;

impl ColumnSchema for CertificateHeaderColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = CERTIFICATE_HEADER_CF;
}
