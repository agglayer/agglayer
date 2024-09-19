use agglayer_types::{CertificateId, CertificateIndex, EpochNumber};
use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, CERTIFICATE_PER_NETWORK_CF};

#[cfg(test)]
mod tests;

/// Column family for the certificate per network per height.
///
/// ## Column definition
///
/// | key                     | value                                                |
/// | --                      | --                                                   |
/// | (`NetworkId`, `Height`) | (`CertificateId`, `EpochNumber`, `CertificateIndex`) |
pub struct CertificatePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key {
    network_id: u32,
    height: u64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Value {
    certificate_id: CertificateId,
    epoch_number: EpochNumber,
    certificate_index: CertificateIndex,
}

impl Codec for Key {}
impl Codec for Value {}

impl ColumnSchema for CertificatePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = CERTIFICATE_PER_NETWORK_CF;
}
