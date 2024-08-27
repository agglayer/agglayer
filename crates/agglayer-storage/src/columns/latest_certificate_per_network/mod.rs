use bincode::Options;
use serde::{Deserialize, Serialize};

use super::{
    default_bincode_options, Codec, ColumnSchema, LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF,
};
use crate::{error::Error, types::NetworkId};

#[cfg(test)]
mod tests;

/// Column family for the latest settled certificate per network.
/// The key is the network_id and the value is the certificateID,
/// the height and the epoch_number.
///
/// | --- key ---- |    | ---------- value ----------  |
/// | NetworkID    | => | Proof CertificateID Height   |
pub struct LatestSettledCertificatePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenCertificate(pub [u8; 32], pub u64, pub u64);

pub type Key = NetworkId;

impl Codec for ProvenCertificate {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(self)?)
    }
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

impl ColumnSchema for LatestSettledCertificatePerNetworkColumn {
    type Key = Key;
    type Value = ProvenCertificate;

    const COLUMN_FAMILY_NAME: &'static str = LATEST_SETTLED_CERTIFICATE_PER_NETWORK_CF;
}
