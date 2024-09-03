use agglayer_types::{Hash, U256};
use serde::{Deserialize, Serialize};

use super::{Codec, ColumnSchema, BALANCE_TREE_PER_NETWORK_CF};

#[cfg(test)]
mod tests;

/// Column family for the balance tree per network.
///
/// ## Column definition
///
/// | key                            | value                                   |
/// | ---                            | --                                      |
/// | (`NetworkId`, `KeyType::Root`) | (`hash(root.left)`, `hash(root.right)`) |
/// | (`NetworkId`, `hash(node)`)    | (`hash(node.left)`, `hash(node.right)`) |
/// | (`NetworkId`, `hash(node)`)    | (`hash(leaf)`)                          |
/// | (`NetworkId`, `KeyType::Leaf`) | [`U256`](type@reth_primitives::U256)    |
pub struct BalanceTreePerNetworkColumn;

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
    LeafData(U256),
}

impl Codec for Key {}
impl Codec for Value {}

impl ColumnSchema for BalanceTreePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = BALANCE_TREE_PER_NETWORK_CF;
}
