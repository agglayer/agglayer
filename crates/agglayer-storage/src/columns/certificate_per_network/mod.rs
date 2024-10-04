use agglayer_types::CertificateId;
use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, CERTIFICATE_PER_NETWORK_CF};

#[cfg(test)]
mod tests;

/// Column family for the certificate per network per height.
///
/// ## Column definition
///
/// | key                     | value           |
/// | --                      | --              |
/// | (`NetworkId`, `Height`) | `CertificateId` |
pub struct CertificatePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key {
    pub(crate) network_id: u32,
    pub(crate) height: u64,
}

impl Codec for Key {}

impl ColumnSchema for CertificatePerNetworkColumn {
    type Key = Key;
    type Value = CertificateId;

    const COLUMN_FAMILY_NAME: &'static str = CERTIFICATE_PER_NETWORK_CF;
}
