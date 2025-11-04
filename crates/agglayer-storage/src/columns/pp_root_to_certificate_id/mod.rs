use agglayer_types::{CertificateId, Digest};

use super::{ColumnSchema, PP_ROOT_TO_CERTIFICATE_ID_CF};

/// Column family for the pp root to certificate ID mapping.
///
/// ## Column definition
///
/// | key        | value           |
/// | --         | --              |
/// | `Digest`   | `CertificateId` |
pub struct PpRootToCertificateIdColumn;

pub type Key = Digest;
pub type Value = CertificateId;

impl ColumnSchema for PpRootToCertificateIdColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = PP_ROOT_TO_CERTIFICATE_ID_CF;
}
