use agglayer_types::NetworkId;
use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF};

#[cfg(test)]
mod tests;

/// Column family for the latest settled certificate per network.
/// The key is the network_id and the value is the certificateID,
/// the height and the epoch_number.
///
/// ## Column definition
///
/// | key         | value                                |
/// | --          | --                                   |
/// | `NetworkId` | (`Proof`, `CertificateId`, `Height`) |
pub struct LatestSettledCertificatePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenCertificate(pub [u8; 32], pub u64, pub u64);

pub type Key = NetworkId;

impl Codec for ProvenCertificate {}

impl ColumnSchema for LatestSettledCertificatePerNetworkColumn {
    type Key = Key;
    type Value = ProvenCertificate;

    const COLUMN_FAMILY_NAME: &'static str = LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF;
}
