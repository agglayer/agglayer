use super::{ColumnSchema, NULLIFIER_TREE_PER_NETWORK_CF};

/// Column family for the nullifier tree per network.
///
/// ## Column definition
///
/// | key                               | value                                   |
/// | ---                               | --                                      |
/// | (`NetworkId`, `SmtKeyType::Root`) | (`hash(root.left)`, `hash(root.right)`) |
/// | (`NetworkId`, `hash(node)`)       | (`hash(node.left)`, `hash(node.right)`) |
/// | (`NetworkId`, `hash(node)`)       | (`hash(leaf)`)                          |
pub struct NullifierTreePerNetworkColumn;

impl ColumnSchema for NullifierTreePerNetworkColumn {
    type Key = crate::types::SmtKey;
    type Value = crate::types::SmtValue;

    const COLUMN_FAMILY_NAME: &'static str = NULLIFIER_TREE_PER_NETWORK_CF;
}
