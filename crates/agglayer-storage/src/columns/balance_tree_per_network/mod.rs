use super::{ColumnSchema, BALANCE_TREE_PER_NETWORK_CF};

#[cfg(test)]
mod tests;

/// Column family for the balance tree per network.
///
/// ## Column definition
///
/// | key                               | value                                   |
/// | ---                               | --                                      |
/// | (`NetworkId`, `SmtKeyType::Root`) | (`hash(root.left)`, `hash(root.right)`) |
/// | (`NetworkId`, `hash(node)`)       | (`hash(node.left)`, `hash(node.right)`) |
/// | (`NetworkId`, `hash(node)`)       | (`hash(leaf)`)                          |
pub struct BalanceTreePerNetworkColumn;

impl ColumnSchema for BalanceTreePerNetworkColumn {
    type Key = crate::types::SmtKey;
    type Value = crate::types::SmtValue;

    const COLUMN_FAMILY_NAME: &'static str = BALANCE_TREE_PER_NETWORK_CF;
}
