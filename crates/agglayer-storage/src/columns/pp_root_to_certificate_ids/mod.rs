use crate::types::pp_root_to_certificate_ids::{Key, Value};

use super::{ColumnSchema, PP_ROOT_TO_CERTIFICATE_IDS_CF};

/// Column family for the pp root to certificate ID mapping.
///
/// ## Column definition
///
/// | key                           | value                       |
/// | ----------------------------- | --------------------------- |
/// | `SettledPessimisticProofRoot` | `Vec<SettledCertificateId>` |
pub struct PpRootToCertificateIdsColumn;

impl ColumnSchema for PpRootToCertificateIdsColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = PP_ROOT_TO_CERTIFICATE_IDS_CF;
}
