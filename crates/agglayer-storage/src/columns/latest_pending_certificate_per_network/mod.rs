use agglayer_types::{CertificateId, Height, NetworkId};
use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, LATEST_PENDING_CERTIFICATE_PER_NETWORK_CF};

/// Column family for the latest pending certificate per network.
/// The key is the network_id and the value is the certificateID and
/// the height.
///
/// ## Column definition
///
/// | key         | value                       |
/// | --          | --                          |
/// | `NetworkId` | (`CertificateId`, `Height`) |
pub struct LatestPendingCertificatePerNetworkColumn;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct PendingCertificate(pub CertificateId, pub Height);

pub type Key = NetworkId;

impl Codec for PendingCertificate {}

impl ColumnSchema for LatestPendingCertificatePerNetworkColumn {
    type Key = Key;
    type Value = PendingCertificate;

    const COLUMN_FAMILY_NAME: &'static str = LATEST_PENDING_CERTIFICATE_PER_NETWORK_CF;
}
