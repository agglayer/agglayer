use agglayer_types::Digest;
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

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.network_id, self.key_type)
    }
}

impl Key {
    pub fn new(network_id: u32, key_type: KeyType) -> Self {
        Self {
            network_id,
            key_type,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyType {
    LeafCount,
    Leaf(u32),
    Frontier(u32),
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::LeafCount => write!(f, "LeafCount"),
            KeyType::Leaf(index) => write!(f, "Leaf({})", index),
            KeyType::Frontier(layer) => write!(f, "Frontier({})", layer),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    LeafCount(u32),
    Leaf([u8; 32]),
    Frontier([u8; 32]),
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::LeafCount(count) => write!(f, "LeafCount({})", count),
            Value::Leaf(hash) => write!(f, "Leaf({})", Digest(*hash)),
            Value::Frontier(hash) => write!(f, "Frontier({})", Digest(*hash)),
        }
    }
}

impl Codec for Key {}
impl Codec for Value {}

impl ColumnSchema for LocalExitTreePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = LOCAL_EXIT_TREE_PER_NETWORK_CF;
}
