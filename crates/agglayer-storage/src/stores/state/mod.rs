use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::Path,
    sync::Arc,
};

use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateIndex, CertificateStatus,
    EpochNumber, Height, LocalNetworkStateData, NetworkId,
};
use pessimistic_proof::{
    keccak::digest::Digest,
    local_balance_tree::LOCAL_BALANCE_TREE_DEPTH,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    nullifier_tree::NULLIFIER_TREE_DEPTH,
    utils::smt::{Node, Smt},
};
use rocksdb::{Direction, ReadOptions, WriteBatch};
use tracing::{info, warn};

use self::LET::LocalExitTreePerNetworkColumn;
use super::{MetadataReader, MetadataWriter, StateReader, StateWriter};
use crate::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{self, CertificatePerNetworkColumn},
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
        local_exit_tree_per_network as LET,
        metadata::MetadataColumn,
        nullifier_tree_per_network::NullifierTreePerNetworkColumn,
        ColumnSchema,
    },
    error::Error,
    storage::DB,
    types::{MetadataKey, MetadataValue, SmtKey, SmtKeyType, SmtValue},
};

#[cfg(test)]
mod tests;

/// A logical store for the state.
pub struct StateStore {
    db: Arc<DB>,
}

impl StateStore {
    pub fn new(db: Arc<DB>) -> Self {
        Self { db }
    }

    pub fn new_with_path(path: &Path) -> Result<Self, Error> {
        let db = Arc::new(DB::open_cf(
            path,
            crate::storage::state_db_cf_definitions(),
        )?);

        Ok(Self { db })
    }
}

impl StateWriter for StateStore {
    fn update_settlement_tx_hash(
        &self,
        certificate_id: &CertificateId,
        tx_hash: Digest,
    ) -> Result<(), Error> {
        // TODO: make lockguard for certificate_id
        let certificate_header = self.db.get::<CertificateHeaderColumn>(certificate_id)?;

        if let Some(mut certificate_header) = certificate_header {
            if certificate_header.settlement_tx_hash.is_some() {
                return Err(Error::UnprocessedAction(
                    "Tried to update settlement tx hash for a certificate that already has a \
                     settlement tx hash"
                        .to_string(),
                ));
            }

            if certificate_header.status == CertificateStatus::Settled {
                return Err(Error::UnprocessedAction(
                    "Tried to update settlement tx hash for a certificate that is already settled"
                        .to_string(),
                ));
            }

            certificate_header.settlement_tx_hash = Some(tx_hash);
            certificate_header.status = CertificateStatus::Candidate;

            self.db
                .put::<CertificateHeaderColumn>(certificate_id, &certificate_header)?;
        } else {
            info!(
                hash = %certificate_id,
                "Certificate header not found for certificate_id: {}",
                certificate_id
            )
        }

        Ok(())
    }

    fn assign_certificate_to_epoch(
        &self,
        certificate_id: &CertificateId,
        epoch_number: &EpochNumber,
        certificate_index: &CertificateIndex,
    ) -> Result<(), Error> {
        // TODO: make lockguard for certificate_id
        let certificate_header = self.db.get::<CertificateHeaderColumn>(certificate_id)?;

        if let Some(mut certificate_header) = certificate_header {
            if certificate_header.epoch_number.is_some()
                || certificate_header.certificate_index.is_some()
            {
                return Err(Error::UnprocessedAction(
                    "Tried to assign a certificate to an epoch that is already assigned"
                        .to_string(),
                ));
            }

            certificate_header.status = CertificateStatus::Settled;
            certificate_header.epoch_number = Some(*epoch_number);
            certificate_header.certificate_index = Some(*certificate_index);

            self.db
                .put::<CertificateHeaderColumn>(certificate_id, &certificate_header)?;
        }

        Ok(())
    }

    fn insert_certificate_header(
        &self,
        certificate: &Certificate,
        status: CertificateStatus,
    ) -> Result<(), Error> {
        // TODO: make it a batch write
        self.db.put::<CertificateHeaderColumn>(
            &certificate.hash(),
            &CertificateHeader {
                certificate_id: certificate.hash(),
                network_id: certificate.network_id,
                height: certificate.height,
                epoch_number: None,
                certificate_index: None,
                prev_local_exit_root: certificate.prev_local_exit_root,
                new_local_exit_root: certificate.new_local_exit_root,
                status: status.clone(),
                metadata: certificate.metadata,
                settlement_tx_hash: None,
            },
        )?;

        if let CertificateStatus::Settled = status {
            // TODO: Check certificate conflict during insert (if conflict it's too late)
            self.db.put::<CertificatePerNetworkColumn>(
                &certificate_per_network::Key {
                    network_id: *certificate.network_id,
                    height: certificate.height,
                },
                &certificate.hash(),
            )?;
        }

        Ok(())
    }

    fn update_certificate_header_status(
        &self,
        certificate_id: &CertificateId,
        status: &CertificateStatus,
    ) -> Result<(), Error> {
        // TODO: make lockguard for certificate_id
        let certificate_header = self.db.get::<CertificateHeaderColumn>(certificate_id)?;

        if let Some(mut certificate_header) = certificate_header {
            certificate_header.status = status.clone();
            self.db
                .put::<CertificateHeaderColumn>(certificate_id, &certificate_header)?;

            if let CertificateStatus::Settled = status {
                self.db.put::<CertificatePerNetworkColumn>(
                    &certificate_per_network::Key {
                        network_id: *certificate_header.network_id,
                        height: certificate_header.height,
                    },
                    &certificate_header.certificate_id,
                )?;
            }
        }

        Ok(())
    }

    fn set_latest_settled_certificate_for_network(
        &self,
        network_id: &NetworkId,
        height: &Height,
        certificate_id: &CertificateId,
        epoch_number: &EpochNumber,
        certificate_index: &CertificateIndex,
    ) -> Result<(), Error> {
        Ok(self.db.put::<LatestSettledCertificatePerNetworkColumn>(
            network_id,
            &SettledCertificate(*certificate_id, *height, *epoch_number, *certificate_index),
        )?)
    }

    fn write_local_network_state(
        &self,
        network_id: &NetworkId,
        new_state: &LocalNetworkStateData,
        new_leaves: &[Digest],
    ) -> Result<(), Error> {
        let network_id: u32 = (*network_id).into();

        let mut atomic_batch = WriteBatch::default();
        // Store the LET
        {
            let new_leaf_count = new_state.exit_tree.leaf_count();
            let start_leaf_count = new_leaf_count - new_leaves.len() as u32;

            if let Some(stored_exit_tree) = self.read_local_exit_tree(network_id.into())? {
                if stored_exit_tree.leaf_count() != start_leaf_count {
                    return Err(Error::InconsistentState {
                        network_id: network_id.into(),
                    });
                }
            }

            // Collect the exit tree writes
            let exit_tree_writes = {
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
                (0..32).for_each(|layer| {
                    writes.insert(
                        LET::Key {
                            network_id,
                            key_type: LET::KeyType::Frontier(layer),
                        },
                        LET::Value::Frontier(*new_state.exit_tree.frontier()[layer as usize]),
                    );
                });

                writes
            };

            self.db
                .multi_insert_batch::<LocalExitTreePerNetworkColumn>(
                    exit_tree_writes.iter(),
                    &mut atomic_batch,
                )?;
        }

        // Collect the balance tree writes
        self.write_smt::<BalanceTreePerNetworkColumn, LOCAL_BALANCE_TREE_DEPTH>(
            network_id,
            &new_state.balance_tree,
            &mut atomic_batch,
        )?;

        // Collect nullifier tree writes
        self.write_smt::<NullifierTreePerNetworkColumn, NULLIFIER_TREE_DEPTH>(
            network_id,
            &new_state.nullifier_tree,
            &mut atomic_batch,
        )?;

        // Atomic write across the 3 cfs
        self.db.write_batch(atomic_batch)?;

        Ok(())
    }
}

impl StateStore {
    fn write_smt<C, const DEPTH: usize>(
        &self,
        network_id: u32,
        smt: &Smt<Keccak256Hasher, DEPTH>,
        batch: &mut WriteBatch,
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
                        SmtKeyType::Node(node_hash)
                    },
                },
                SmtValue::Node(node.left, node.right),
            );

            // Write the children as leaves if they are
            [node.left, node.right]
                .iter()
                .filter(|&maybe_leaf| !smt.tree.contains_key(maybe_leaf))
                .for_each(|&leaf| {
                    kv.insert(
                        SmtKey {
                            network_id,
                            key_type: SmtKeyType::Node(leaf),
                        },
                        SmtValue::Leaf(leaf),
                    );
                });
        });

        self.db.multi_insert_batch::<C>(&kv, batch)?;

        Ok(())
    }

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
            .multi_get::<LocalExitTreePerNetworkColumn>((0..32).map(|layer| LET::Key {
                network_id: network_id.into(),
                key_type: LET::KeyType::Frontier(layer),
            }))?
            .iter()
            .map(|v| match v {
                Some(LET::Value::Frontier(hash)) => Ok(*hash),
                _ => Err(Error::InconsistentFrontier),
            })
            .collect::<Result<_, _>>()?;

        let mut frontier = [[0u8; 32].into(); 32];
        for (i, l) in retrieved_frontier.iter().enumerate() {
            frontier[i] = Digest(*l);
        }

        Ok(Some(LocalExitTree::<Keccak256Hasher>::from_parts(
            leaf_count, frontier,
        )))
    }

    fn read_smt<C, const DEPTH: usize>(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<Smt<Keccak256Hasher, DEPTH>>, Error>
    where
        C: ColumnSchema<Key = SmtKey, Value = SmtValue>,
    {
        let root_node: Node<Keccak256Hasher> = if let Some(root_node_value) =
            self.db.get::<C>(&SmtKey {
                network_id: network_id.into(),
                key_type: SmtKeyType::Root,
            })? {
            match root_node_value {
                SmtValue::Node(left, right) => Node {
                    left: Digest(*left.as_bytes()),
                    right: Digest(*right.as_bytes()),
                },
                _ => return Err(Error::WrongValueType),
            }
        } else {
            return Ok(None);
        };

        let mut keys = VecDeque::new();
        keys.push_back(SmtKeyType::Node(root_node.left));
        keys.push_back(SmtKeyType::Node(root_node.right));

        let mut nodes: Vec<Node<Keccak256Hasher>> = Vec::new();
        nodes.push(root_node);

        let mut queued = BTreeSet::new();
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
                        left: Digest(*left.as_bytes()),
                        right: Digest(*right.as_bytes()),
                    });
                    if queued.insert(left) {
                        keys.push_back(SmtKeyType::Node(left));
                    }
                    if queued.insert(right) {
                        keys.push_back(SmtKeyType::Node(right));
                    }
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

impl StateReader for StateStore {
    /// Get the active networks.
    /// Meaning, the networks that have at least one submitted certificate.
    ///
    /// Performance: O(n) where n is the number of networks.
    /// This is because we need to scan all the keys in the
    /// `last_certificate_per_network` column family.
    /// This is not a problem because the number of networks is expected to be
    /// small. This function is only called once when the node starts.
    /// Benchmark: `last_certificate_bench.rs`
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, Error> {
        Ok(self
            .db
            .keys::<LatestSettledCertificatePerNetworkColumn>()?
            .filter_map(|v| v.ok())
            .collect())
    }

    fn get_certificate_header(
        &self,
        certificate_id: &CertificateId,
    ) -> Result<Option<CertificateHeader>, Error> {
        Ok(self.db.get::<CertificateHeaderColumn>(certificate_id)?)
    }

    fn get_certificate_header_by_cursor(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<Option<CertificateHeader>, Error> {
        self.db
            .get::<CertificatePerNetworkColumn>(&certificate_per_network::Key {
                network_id: *network_id,
                height,
            })?
            .map_or(Ok(None), |certificate_id| {
                let result = self.get_certificate_header(&certificate_id);

                if let Ok(None) = result {
                    warn!(
                        "Certificate header not found for certificate_id: {} while having a \
                         reference in the CertificatePerNetworkColumn",
                        certificate_id
                    );
                }

                result
            })
    }

    fn get_current_settled_height(&self) -> Result<Vec<(NetworkId, SettledCertificate)>, Error> {
        Ok(self
            .db
            .iter_with_direction::<LatestSettledCertificatePerNetworkColumn>(
                ReadOptions::default(),
                Direction::Forward,
            )?
            .filter_map(|v| v.ok())
            .collect())
    }

    fn get_latest_settled_certificate_per_network(
        &self,
        network_id: &NetworkId,
    ) -> Result<Option<(NetworkId, SettledCertificate)>, Error> {
        Ok(self
            .db
            .get::<LatestSettledCertificatePerNetworkColumn>(network_id)
            .map(|v| v.map(|v| (*network_id, v)))?)
    }

    fn read_local_network_state(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<LocalNetworkStateData>, Error> {
        let local_exit_tree = self.read_local_exit_tree(network_id)?;
        let balance_tree =
            self.read_smt::<BalanceTreePerNetworkColumn, LOCAL_BALANCE_TREE_DEPTH>(network_id)?;
        let nullifier_tree =
            self.read_smt::<NullifierTreePerNetworkColumn, NULLIFIER_TREE_DEPTH>(network_id)?;

        match (local_exit_tree, balance_tree, nullifier_tree) {
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

impl MetadataWriter for StateStore {
    fn set_latest_settled_epoch(&self, value: u64) -> Result<(), Error> {
        if let Some(current_latest_settled_epoch) = self.get_latest_settled_epoch()? {
            if current_latest_settled_epoch >= value {
                return Err(Error::UnprocessedAction(
                    "Tried to set a lower value for latest settled epoch".to_string(),
                ));
            }
        }

        Ok(self.db.put::<MetadataColumn>(
            &MetadataKey::LatestSettledEpoch,
            &MetadataValue::LatestSettledEpoch(value),
        )?)
    }
}

impl MetadataReader for StateStore {
    fn get_latest_settled_epoch(&self) -> Result<Option<u64>, Error> {
        self.db
            .get::<MetadataColumn>(&MetadataKey::LatestSettledEpoch)
            .map_err(Into::into)
            .and_then(|v| {
                v.map_or(Ok(None), |v| match v {
                    MetadataValue::LatestSettledEpoch(value) => Ok(Some(value)),
                    _ => Err(Error::Unexpected(
                        "Wrong value type decoded, was expecting LastSettledEpoch, decoded \
                         another type"
                            .to_string(),
                    )),
                })
            })
    }
}
