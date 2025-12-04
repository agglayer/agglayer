use agglayer_types::CertificateId;
use agglayer_tries::roots::PessimisticRoot;
use serde::{Deserialize, Serialize};

use super::{ColumnSchema, PP_ROOT_TO_CERTIFICATE_IDS_CF};

/// Column family for the pp root to certificate ID mapping.
///
/// ## Column definition
///
/// | key               | value                |
/// | ----------------- | -------------------- |
/// | `PessimisticRoot` | `Vec<CertificateId>` |
pub struct PpRootToCertificateIdsColumn;

pub type Key = PessimisticRoot;

#[derive(Debug, Serialize, Deserialize)]
pub struct Value(pub(crate) Vec<CertificateId>);

crate::columns::impl_codec_using_bincode_for!(Key);
crate::columns::impl_codec_using_bincode_for!(Value);

impl ColumnSchema for PpRootToCertificateIdsColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = PP_ROOT_TO_CERTIFICATE_IDS_CF;
}
