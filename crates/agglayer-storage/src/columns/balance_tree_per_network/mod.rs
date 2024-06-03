use bincode::Options;
use serde::{Deserialize, Serialize};

use super::{default_bincode_options, Codec, ColumnSchema, BALANCE_TREE_PER_NETWORK_CF};
use crate::{error::Error, types::Hash};

#[cfg(test)]
mod tests;

/// Column family for the balance tree per network.
///
/// | ------------ key ------------ |    | ------------- value -------------- |
/// | NetworkId "root"              | => | hash(node.left) hash(node.right)   |
/// | NetworkId hash(node)          | => | hash(node.left) hash(node.right)   |
/// | NetworkId hash(node)          | => | hash(leaf)                         |
/// | NetworkId "leaves" hash(leaf) | => | leaf bytes array                   |
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

impl Codec for Key {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(&self)?)
    }

    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Value {
    Node(Hash, Hash),
    Leaf(Hash),
    LeafData(Vec<u8>),
}

impl Codec for Value {
    fn encode(&self) -> Result<Vec<u8>, Error> {
        Ok(default_bincode_options().serialize(&self)?)
    }
    fn decode(buf: &[u8]) -> Result<Self, Error> {
        Ok(default_bincode_options().deserialize(buf)?)
    }
}

impl ColumnSchema for BalanceTreePerNetworkColumn {
    type Key = Key;
    type Value = Value;

    const COLUMN_FAMILY_NAME: &'static str = BALANCE_TREE_PER_NETWORK_CF;
}
