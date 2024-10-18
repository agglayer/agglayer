use agglayer_types::{CertificateId, Height, NetworkId};
use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, LATEST_PROVEN_CERTIFICATE_PER_NETWORK_CF};

/// Column family for the latest proven certificate per network.
/// The key is the network_id and the value is the certificateID and
/// the height.
///
/// ## Column definition
///
/// | key         | value                       |
/// | --          | --                          |
/// | `NetworkId` | (`CertificateId`, `Height`) |
pub struct LatestProvenCertificatePerNetworkColumn;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenCertificate(pub CertificateId, pub NetworkId, pub Height);

pub type Key = NetworkId;

impl Codec for ProvenCertificate {}

impl ColumnSchema for LatestProvenCertificatePerNetworkColumn {
    type Key = Key;
    type Value = ProvenCertificate;

    const COLUMN_FAMILY_NAME: &'static str = LATEST_PROVEN_CERTIFICATE_PER_NETWORK_CF;
}
