use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, NULLIFIER_TREE_PER_NETWORK_CF};
use crate::types::Hash;

#[cfg(test)]
mod tests;

/// Column family for the nullifier tree per network.
///
/// ## Column definition
/// ```
/// |-key--------------------------------------|    |-value-------------------------------|
/// | (NetworkId, KeyType::Root)                 =>   (hash(root.left), hash(root.right)) |
/// | (NetworkId, hash(node))                    =>   (hash(node.left), hash(node.right)) |
/// | (NetworkId, hash(node))                    =>   hash(leat)                          |
/// | (NetworkId, KeyType::Leaves, hash(left))   =>   hash(leat)                          |
/// ```
pub struct NullifierTreePerNetworkColumn;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Key {
    network_id: u32,
    key_type: KeyType,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum KeyType {
    Root,
    Node(Hash),
    Leaf(Hash),
    Leaves,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Node(Hash, Hash),
    Leaf(Hash),
    LeafData(bool),
}

impl Codec for Key {}
impl Codec for Value {}

impl ColumnSchema for NullifierTreePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = NULLIFIER_TREE_PER_NETWORK_CF;
}
