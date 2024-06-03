use bincode::Options;
use serde::{Deserialize, Serialize};

use super::{default_bincode_options, Codec, ColumnSchema, LOCAL_EXIT_TREE_PER_NETWORK_CF};
use crate::error::Error;

/// Column family for the local exit tree per network.
///
/// | --- key ------------- |    | --- value ---------- |
/// | NetworkId layer index | => | array of bytes array |
pub struct LocalExitTreePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key {
    network_id: u32,
    layer: u32,
    index: u64,
}

impl Codec for Key {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(&self)?)
    }

    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

pub type Value = Vec<[u8; 32]>;

impl Codec for Value {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(&self)?)
    }
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

impl ColumnSchema for LocalExitTreePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = LOCAL_EXIT_TREE_PER_NETWORK_CF;
}
