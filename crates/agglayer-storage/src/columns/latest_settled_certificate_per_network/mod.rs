use agglayer_types::{
    CertificateId, CertificateIndex, EpochNumber, Height, NetworkId, SettlementBlockNumber,
};
use serde::{Deserialize, Serialize};

use super::{ColumnSchema, LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF};

#[cfg(test)]
mod tests;

/// Column family for the latest settled certificate per network.
/// The key is the network_id and the value is the certificateID,
/// the height and the epoch_number.
///
/// ## Column definition
///
/// | key         | value                                                          |
/// | --          | --                                                             |
/// | `NetworkId` | (`CertificateId`, `Height`, `EpochNumber`, `CertificateIndex`) |
pub struct LatestSettledCertificatePerNetworkColumn;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct SettledCertificate(
    pub CertificateId,
    pub Height,
    pub EpochNumber,
    pub CertificateIndex,
    pub SettlementBlockNumber,
);

pub type Key = NetworkId;

crate::columns::impl_codec_using_bincode_for!(SettledCertificate);

impl ColumnSchema for LatestSettledCertificatePerNetworkColumn {
    type Key = Key;
    type Value = SettledCertificate;

    const COLUMN_FAMILY_NAME: &'static str = LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF;
}
