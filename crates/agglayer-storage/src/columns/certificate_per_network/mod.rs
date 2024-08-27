use bincode::Options;
use serde::{Deserialize, Serialize};

use super::{default_bincode_options, Codec, ColumnSchema, CERTIFICATE_PER_NETWORK_CF};
use crate::error::Error;

#[cfg(test)]
mod tests;

/// Column family for the certificate per network per height.
///
/// | --- key ---------- |    | --- value ------------------------------------- |
/// | network_id height  | => | certificate_id epoch_number certificate_index   |
pub struct CertificatePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key {
    network_id: u32,
    height: u64,
}

impl Codec for Key {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(self)?)
    }

    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Value {
    certificate_id: [u8; 32],
    epoch_number: u64,
    certificate_index: u64,
}

impl Codec for Value {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(&self)?)
    }
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

impl ColumnSchema for CertificatePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = CERTIFICATE_PER_NETWORK_CF;
}
