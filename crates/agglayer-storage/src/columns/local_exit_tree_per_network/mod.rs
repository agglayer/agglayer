use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, LOCAL_EXIT_TREE_PER_NETWORK_CF};

/// Column family for the local exit tree per network.
///
/// ## Column definition
///
/// | key                                       | value    |
/// | --                                        | --       |
/// | (`NetworkId`, `KeyType::LeafCount`)       | (`u32`)  |
/// | (`NetworkId`, `KeyType::Leaf(index)`)     | (`Hash`) |
/// | (`NetworkId`, `KeyType::Frontier(layer)`) | (`Hash`) |
pub struct LocalExitTreePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key {
    pub(crate) network_id: u32,
    pub(crate) key_type: KeyType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyType {
    LeafCount,
    Leaf(u32),
    Frontier(u32),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    LeafCount(u32),
    Leaf([u8; 32]),
    Frontier([u8; 32]),
}

impl Codec for Key {}
impl Codec for Value {}

impl ColumnSchema for LocalExitTreePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = LOCAL_EXIT_TREE_PER_NETWORK_CF;
}
