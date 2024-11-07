use std::{
    collections::{BTreeMap, VecDeque},
    path::Path,
    sync::Arc,
};

use agglayer_types::{Hash, Keccak256Hasher, LocalNetworkStateData, NetworkId};
use pessimistic_proof::{
    local_balance_tree::LOCAL_BALANCE_TREE_DEPTH,
    local_exit_tree::LocalExitTree,
    nullifier_tree::NULLIFIER_TREE_DEPTH,
    utils::smt::{Node, Smt},
};

use super::{
    interfaces::reader::LocalNetworkStateReader, interfaces::writer::LocalNetworkStateWriter,
};
use crate::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn, local_exit_tree_per_network as LET,
        local_exit_tree_per_network::LocalExitTreePerNetworkColumn,
        nullifier_tree_per_network::NullifierTreePerNetworkColumn, ColumnSchema,
    },
    error::Error,
    storage::DB,
    types::{SmtKey, SmtKeyType, SmtValue},
};

#[cfg(test)]
mod tests;

/// A logical store for the local network states.
pub struct LocalNetworkStateStore {
    db: Arc<DB>,
}

impl LocalNetworkStateStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    pub fn new_with_path(path: &Path) -> Result<Self, Error> {
        let db = Arc::new(DB::open_cf(
            path,
            crate::storage::local_network_state_db_cf_definitions(),
        )?);

        Ok(Self { db })
    }
}

impl LocalNetworkStateWriter for LocalNetworkStateStore {
    fn write_local_network_state(
        &self,
        network_id: &NetworkId,
        new_state: &LocalNetworkStateData,
        new_leaves: &[Hash],
    ) -> Result<(), Error> {
        let network_id: u32 = (*network_id).into();

        // Store the LET
        {
            let new_leaf_count = new_state.exit_tree.leaf_count;
            let start_leaf_count = new_leaf_count - new_leaves.len() as u32;

            if let Some(stored_exit_tree) = self.read_local_exit_tree(network_id.into())? {
                if stored_exit_tree.leaf_count != start_leaf_count {
                    // inconsistent state
                }
            } else {
                // state from scratch
            }

            // TODO: check how to make atomic the whole state update across the cfs
            let atomic_batch_write = {
                let mut writes = BTreeMap::new();

                // Write new leaf count
                writes.insert(
                    LET::Key {
                        network_id,
                        key_type: LET::KeyType::LeafCount,
                    },
                    LET::Value::LeafCount(new_leaf_count),
                );

                // Write new leaves
                (start_leaf_count..new_leaf_count)
                    .zip(new_leaves.iter())
                    .for_each(|(index, leaf)| {
                        writes.insert(
                            LET::Key {
                                network_id,
                                key_type: LET::KeyType::Leaf(index),
                            },
                            LET::Value::Leaf(*leaf.as_bytes()),
                        );
                    });

                // Write frontier
                (1..32).for_each(|layer| {
                    writes.insert(
                        LET::Key {
                            network_id,
                            key_type: LET::KeyType::Frontier(layer),
                        },
                        LET::Value::Frontier(new_state.exit_tree.frontier[layer as usize]),
                    );
                });

                writes
            };

            self.db
                .multi_insert::<LocalExitTreePerNetworkColumn>(atomic_batch_write.iter())?;
        }

        // Store the balance tree
        self.write_smt::<BalanceTreePerNetworkColumn, LOCAL_BALANCE_TREE_DEPTH>(
            network_id,
            &new_state.balance_tree,
        )?;

        // Store the nullifier tree
        self.write_smt::<NullifierTreePerNetworkColumn, NULLIFIER_TREE_DEPTH>(
            network_id,
            &new_state.nullifier_tree,
        )?;

        Ok(())
    }
}

impl LocalNetworkStateStore {
    fn write_smt<C, const DEPTH: usize>(
        &self,
        network_id: u32,
        smt: &Smt<Keccak256Hasher, DEPTH>,
    ) -> Result<(), Error>
    where
        C: ColumnSchema<Key = SmtKey, Value = SmtValue>,
    {
        let mut kv = BTreeMap::new();
        smt.tree.iter().for_each(|(&node_hash, node)| {
            // Write the node
            kv.insert(
                SmtKey {
                    network_id,
                    key_type: if node_hash == smt.root {
                        SmtKeyType::Root
                    } else {
                        SmtKeyType::Node(Hash(node_hash))
                    },
                },
                SmtValue::Node(Hash(node.left), Hash(node.right)),
            );

            // Write the children as leaves if they are
            [node.left, node.right]
                .iter()
                .filter(|&maybe_leaf| !smt.tree.contains_key(maybe_leaf))
                .for_each(|&leaf| {
                    kv.insert(
                        SmtKey {
                            network_id,
                            key_type: SmtKeyType::Node(Hash(leaf)),
                        },
                        SmtValue::Leaf(Hash(leaf)),
                    );
                });
        });

        self.db.multi_insert::<C>(&kv)?;

        Ok(())
    }
}

impl LocalNetworkStateStore {
    fn read_local_exit_tree(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<LocalExitTree<Keccak256Hasher>>, Error> {
        let leaf_count = if let Some(leaf_count_value) =
            self.db.get::<LocalExitTreePerNetworkColumn>(&LET::Key {
                network_id: network_id.into(),
                key_type: LET::KeyType::LeafCount,
            })? {
            match leaf_count_value {
                LET::Value::LeafCount(leaf_count) => leaf_count,
                _ => return Err(Error::InconsistentFrontier),
            }
        } else {
            return Ok(None);
        };

        let retrieved_frontier: Vec<_> = self
            .db
            .multi_get::<LocalExitTreePerNetworkColumn>((1..32).map(|layer| LET::Key {
                network_id: network_id.into(),
                key_type: LET::KeyType::Frontier(layer),
            }))?
            .iter()
            .map(|v| match v {
                Some(LET::Value::Frontier(hash)) => Ok(*hash),
                _ => Err(Error::InconsistentFrontier),
            })
            .collect::<Result<_, _>>()?;

        let mut frontier = [[0u8; 32]; 32];
        for (i, l) in retrieved_frontier.iter().enumerate() {
            frontier[i] = *l;
        }

        Ok(Some(LocalExitTree::<Keccak256Hasher> {
            frontier,
            leaf_count,
        }))
    }

    fn read_smt<C, const DEPTH: usize>(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Smt<Keccak256Hasher, DEPTH>>, Error>
    where
        C: ColumnSchema<Key = SmtKey, Value = SmtValue>,
    {
        let root_node = if let Some(root_node_value) = self.db.get::<C>(&SmtKey {
            network_id: network_id.into(),
            key_type: SmtKeyType::Root,
        })? {
            match root_node_value {
                SmtValue::Node(left, right) => Node {
                    left: *left.as_bytes(),
                    right: *right.as_bytes(),
                },
                _ => return Err(Error::WrongValueType),
            }
        } else {
            return Ok(None);
        };

        let mut keys = VecDeque::new();
        keys.push_back(SmtKeyType::Node(Hash(root_node.left)));
        keys.push_back(SmtKeyType::Node(Hash(root_node.right)));

        let mut nodes: Vec<Node<Keccak256Hasher>> = Vec::new();
        nodes.push(root_node);

        while let Some(key) = keys.pop_front() {
            let value = self
                .db
                .get::<C>(&SmtKey {
                    network_id: network_id.into(),
                    key_type: key.clone(),
                })?
                .ok_or(Error::SmtNodeNotFound)?;

            match value {
                SmtValue::Node(left, right) => {
                    nodes.push(Node {
                        left: *left.as_bytes(),
                        right: *right.as_bytes(),
                    });

                    keys.push_back(SmtKeyType::Node(left));
                    keys.push_back(SmtKeyType::Node(right));
                }
                SmtValue::Leaf(_) => {} // nothing to do
            }
        }

        Ok(Some(Smt::<Keccak256Hasher, DEPTH>::new_with_nodes(
            root_node.hash(),
            nodes.as_slice(),
        )))
    }
}

impl LocalNetworkStateReader for LocalNetworkStateStore {
    fn read_local_network_state(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<LocalNetworkStateData>, Error> {
        match (
            self.read_local_exit_tree(network_id)?,
            self.read_smt::<BalanceTreePerNetworkColumn, LOCAL_BALANCE_TREE_DEPTH>(network_id)?,
            self.read_smt::<NullifierTreePerNetworkColumn, NULLIFIER_TREE_DEPTH>(network_id)?,
        ) {
            (None, None, None) => Ok(None), // consistent empty state
            (Some(exit_tree), Some(balance_tree), Some(nullifier_tree)) => {
                Ok(Some(LocalNetworkStateData {
                    exit_tree,
                    balance_tree,
                    nullifier_tree,
                }))
            }
            _ => Err(Error::InconsistentState { network_id }),
        }
    }
}
