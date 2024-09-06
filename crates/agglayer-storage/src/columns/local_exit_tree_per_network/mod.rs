use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, LOCAL_EXIT_TREE_PER_NETWORK_CF};

/// Column family for the local exit tree per network.
///
/// ## Column definition
///
/// | key                             | value   |
/// | --                              | --      |
/// | (`NetworkId`, `Layer`, `Index`) | `Bytes` |
pub struct LocalExitTreePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key {
    network_id: u32,
    layer: u32,
    index: u64,
}

pub type Value = Vec<[u8; 32]>;

impl Codec for Key {}
impl Codec for Value {}

impl ColumnSchema for LocalExitTreePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = LOCAL_EXIT_TREE_PER_NETWORK_CF;
}
