//! Read-only extraction and validation of settled local tree state.

#[cfg(test)]
mod tests;

use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    fmt, fs, io,
    path::{Path, PathBuf},
};

use agglayer_tries::{error::SmtError, node::Node, smt::Smt};
use agglayer_types::{
    primitives::Hashable, Certificate, CertificateHeader, CertificateId, CertificateIndex,
    CertificateStatus, Digest, EpochNumber, Height, NetworkId, SettlementTxHash, U256,
};
use pessimistic_proof::{
    local_balance_tree::{BalanceTree, LOCAL_BALANCE_TREE_DEPTH},
    unified_bridge::{BridgeExit, LocalExitTree, LocalExitTreeError, TokenInfo},
};
use rocksdb::{Direction, ReadOptions};

use super::{
    debug::{
        open_db_readonly as open_debug_db_readonly,
        read_certificate_from_db as read_debug_certificate_from_db,
    },
    per_epoch::{open_db_readonly, read_certificate_from_db},
    state::StateStore,
};
use crate::{
    columns::{
        balance_tree_per_network::BalanceTreePerNetworkColumn,
        certificate_header::CertificateHeaderColumn,
        certificate_per_network::{self, CertificatePerNetworkColumn},
        latest_settled_certificate_per_network::{
            LatestSettledCertificatePerNetworkColumn, SettledCertificate,
        },
        local_exit_tree_per_network::{self as let_column, LocalExitTreePerNetworkColumn},
    },
    error::Error,
    storage::{DBError, DB},
    types::{SmtKey, SmtKeyType, SmtValue},
};

/// One historical local-exit-tree leaf and the settled certificate that added
/// it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SettledBridgeExit {
    pub leaf_index: u32,
    pub leaf_hash: Digest,
    pub bridge_exit: BridgeExit,
    pub certificate_id: CertificateId,
    pub certificate_height: Height,
    pub epoch_number: EpochNumber,
    pub certificate_index: CertificateIndex,
    pub settlement_tx_hash: SettlementTxHash,
}

/// One settled certificate and the position at which its bridge exits begin
/// in the network's local exit tree.
///
/// The certificate is yielded only after its state header, persisted epoch or
/// debug record, and complete local-exit-tree transition have been validated.
#[derive(Clone, Debug)]
pub struct SettledCertificateSnapshot {
    pub certificate: Certificate,
    pub certificate_id: CertificateId,
    pub epoch_number: EpochNumber,
    pub certificate_index: CertificateIndex,
    pub settlement_tx_hash: SettlementTxHash,
    pub first_local_exit_index: u32,
}

/// One non-zero entry in the current local balance tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenBalance {
    pub token: TokenInfo,
    pub amount: U256,
}

/// Why a settled certificate had to be recovered from the debug database.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum DebugCertificateFallbackReason {
    EpochDatabaseUnavailable,
    EpochCertificateMissing,
}

impl fmt::Display for DebugCertificateFallbackReason {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EpochDatabaseUnavailable => formatter.write_str("epoch database unavailable"),
            Self::EpochCertificateMissing => formatter.write_str("epoch certificate missing"),
        }
    }
}

/// A recoverable condition encountered while reading a tree snapshot.
///
/// Callers should make these warnings visible to operators because the copied
/// database is incomplete or contains a corrupt payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum TreeSnapshotWarning {
    CertificateReadFromDebug {
        network_id: NetworkId,
        height: Height,
        certificate_id: CertificateId,
        epoch_number: EpochNumber,
        certificate_index: CertificateIndex,
        reason: DebugCertificateFallbackReason,
    },
    BalanceNodeHashMismatch {
        network_id: NetworkId,
        expected: Digest,
        actual: Digest,
    },
    BalanceLeafHashMismatch {
        network_id: NetworkId,
        expected: Digest,
        actual: Digest,
    },
}

impl fmt::Display for TreeSnapshotWarning {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CertificateReadFromDebug {
                network_id,
                height,
                certificate_id,
                epoch_number,
                certificate_index,
                reason,
            } => write!(
                formatter,
                "settled certificate {certificate_id} for network {network_id} at height {height} \
                 (epoch {epoch_number}, index {certificate_index}) was read from the debug \
                 database (fallback reason: {reason})"
            ),
            Self::BalanceNodeHashMismatch {
                network_id,
                expected,
                actual,
            } => write!(
                formatter,
                "balance-tree node for network {network_id} is stored under {expected} but hashes \
                 to {actual}"
            ),
            Self::BalanceLeafHashMismatch {
                network_id,
                expected,
                actual,
            } => write!(
                formatter,
                "balance-tree leaf for network {network_id} is stored under {expected} but \
                 contains {actual}"
            ),
        }
    }
}

/// Failure to read a complete and internally consistent tree snapshot.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TreeSnapshotError {
    #[error("unable to open state database {path}: {source}")]
    OpenState { path: PathBuf, source: DBError },

    #[error("unable to open epoch {epoch_number} database {path}: {source}")]
    OpenEpoch {
        epoch_number: EpochNumber,
        path: PathBuf,
        source: DBError,
    },

    #[error("unable to open debug database {path}: {source}")]
    OpenDebug { path: PathBuf, source: DBError },

    #[error("unable to inspect epoch {epoch_number} database marker {path}: {source}")]
    InspectEpoch {
        epoch_number: EpochNumber,
        path: PathBuf,
        source: io::Error,
    },

    #[error("epoch {epoch_number} database {path} is unavailable because CURRENT is missing")]
    EpochUnavailable {
        epoch_number: EpochNumber,
        path: PathBuf,
    },

    #[error("epoch {epoch_number} database marker {path} is not a regular file")]
    InvalidEpochMarker {
        epoch_number: EpochNumber,
        path: PathBuf,
    },

    #[error(transparent)]
    Storage(#[from] Error),

    #[error(transparent)]
    Database(#[from] DBError),

    #[error("unable to reconstruct the local exit tree: {0}")]
    LocalExitTree(#[from] LocalExitTreeError),

    #[error("unable to read the balance tree for network {network_id}: {source}")]
    BalanceTree {
        network_id: NetworkId,
        source: SmtError,
    },

    #[error("copied storage is inconsistent: {0}")]
    Inconsistent(String),
}

#[derive(Default)]
struct CursorRows {
    count: u64,
    highest: Option<Height>,
}

#[derive(Default)]
struct ExitTreeRows {
    leaf_count: Option<u32>,
    leaf_rows: u64,
    highest_leaf: Option<u32>,
}

#[derive(Default)]
struct NetworkInventory {
    cursors: CursorRows,
    latest_settled: Option<SettledCertificate>,
    exit_tree: ExitTreeRows,
}

enum EpochCertificateFailure {
    Unavailable(TreeSnapshotError),
    Missing(TreeSnapshotError),
    Fatal(TreeSnapshotError),
}

impl EpochCertificateFailure {
    fn reason(&self) -> DebugCertificateFallbackReason {
        match self {
            Self::Unavailable(_) => DebugCertificateFallbackReason::EpochDatabaseUnavailable,
            Self::Missing(_) => DebugCertificateFallbackReason::EpochCertificateMissing,
            Self::Fatal(_) => unreachable!("fatal epoch read failures cannot use debug fallback"),
        }
    }

    fn into_error(self) -> TreeSnapshotError {
        match self {
            Self::Unavailable(error) | Self::Missing(error) | Self::Fatal(error) => error,
        }
    }
}

/// A semantic reader for a copied Agglayer storage directory.
///
/// All RocksDB instances are opened read-only. The reader expects `state/` and
/// `epochs/<epoch-number>/` below `storage_root`; an optional `debug/` database
/// can recover an exact settled certificate from the latest epoch referenced
/// by state when its epoch database is absent or lacks the indexed row. Earlier
/// epoch gaps remain fatal. Opening scans only enough metadata to inventory
/// each network; certificate and exit histories are fetched one row at a time
/// while a network is visited.
pub struct TreeSnapshotReader {
    state_store: StateStore,
    epochs_path: PathBuf,
    debug_path: PathBuf,
    latest_referenced_epoch: Option<EpochNumber>,
    inventory: BTreeMap<NetworkId, NetworkInventory>,
}

impl TreeSnapshotReader {
    pub fn open(storage_root: impl AsRef<Path>) -> Result<Self, TreeSnapshotError> {
        let storage_root = storage_root.as_ref();
        let state_path = storage_root.join("state");
        let state_store = StateStore::new_readonly_with_path(&state_path).map_err(|source| {
            TreeSnapshotError::OpenState {
                path: state_path,
                source,
            }
        })?;
        let inventory = Self::read_inventory(&state_store)?;
        let latest_referenced_epoch = inventory
            .values()
            .filter_map(|network| network.latest_settled.as_ref().map(|latest| latest.2))
            .max();

        Ok(Self {
            state_store,
            epochs_path: storage_root.join("epochs"),
            debug_path: storage_root.join("debug"),
            latest_referenced_epoch,
            inventory,
        })
    }

    /// Returns all networks represented in settled cursors, latest-settled
    /// pointers, or local exit-tree state, including disabled networks.
    pub fn network_ids(&self) -> Vec<NetworkId> {
        self.inventory.keys().copied().collect()
    }

    /// Reads the distinct settlement transaction hashes for one network
    /// without loading certificates or exit leaves.
    pub fn network_settlement_tx_hashes(
        &self,
        network_id: NetworkId,
    ) -> Result<Vec<SettlementTxHash>, TreeSnapshotError> {
        let mut seen = HashSet::new();
        let mut hashes = Vec::new();

        let Some(latest) = self.validate_history_shape(network_id)? else {
            return Ok(hashes);
        };

        for raw_height in 0..=latest.1.as_u64() {
            let height = Height::new(raw_height);
            let (certificate_id, header) = self.read_settled_header(network_id, height)?;
            let (epoch_number, certificate_index, settlement_tx_hash) =
                self.header_settlement_fields(certificate_id, &header)?;
            self.validate_latest_header(
                network_id,
                height,
                certificate_id,
                epoch_number,
                certificate_index,
                latest,
            )?;

            if seen.insert(settlement_tx_hash) {
                hashes.push(settlement_tx_hash);
            }
        }

        Ok(hashes)
    }

    /// Validates and visits every settled certificate for one network without
    /// retaining its history in memory.
    ///
    /// The visitor receives a certificate only after its header and persisted
    /// epoch or debug record have been matched and its complete local-exit-tree
    /// transition has been reconstructed against the state database. Imported
    /// bridge exits are returned as certificate data committed by the
    /// certificate hash; current proof-verification rules are deliberately not
    /// replayed against historical settled certificates.
    ///
    /// `E: From<TreeSnapshotError>` lets callers stop immediately on their own
    /// output error while preserving storage-validation failures.
    ///
    /// Debug-database recoveries are logged through `tracing`. Call
    /// [`Self::try_visit_network_certificates_with_warnings`] to receive them
    /// as structured values instead.
    pub fn try_visit_network_certificates<E, F>(
        &self,
        network_id: NetworkId,
        visitor: F,
    ) -> Result<(), E>
    where
        E: From<TreeSnapshotError>,
        F: FnMut(SettledCertificateSnapshot) -> Result<(), E>,
    {
        self.try_visit_network_certificates_with_warnings(
            network_id,
            |warning| tracing::warn!(%warning, "tree snapshot recovery warning"),
            visitor,
        )
    }

    /// Like [`Self::try_visit_network_certificates`], while reporting any
    /// validated certificate recovered from the optional debug database.
    pub fn try_visit_network_certificates_with_warnings<E, W, F>(
        &self,
        network_id: NetworkId,
        mut on_warning: W,
        mut visitor: F,
    ) -> Result<(), E>
    where
        E: From<TreeSnapshotError>,
        W: FnMut(TreeSnapshotWarning),
        F: FnMut(SettledCertificateSnapshot) -> Result<(), E>,
    {
        let latest = self.validate_history_shape(network_id).map_err(E::from)?;
        let stored_leaf_count = self.validate_exit_tree_shape(network_id).map_err(E::from)?;
        let local_exit_tree = self
            .state_store
            .read_local_exit_tree(network_id)
            .map_err(TreeSnapshotError::from)
            .map_err(E::from)?
            .ok_or_else(|| {
                E::from(TreeSnapshotError::Inconsistent(format!(
                    "network {network_id} is referenced by settled state but has no local exit \
                     tree"
                )))
            })?;

        if local_exit_tree.leaf_count() != stored_leaf_count {
            return Err(E::from(TreeSnapshotError::Inconsistent(format!(
                "local state for network {network_id} has {} leaves but the leaf records contain \
                 {stored_leaf_count}",
                local_exit_tree.leaf_count()
            ))));
        }

        let mut reconstructed_tree = LocalExitTree::<32>::new();
        let mut epoch_database = None;
        let mut debug_database = None;

        if let Some(latest) = latest {
            for raw_height in 0..=latest.1.as_u64() {
                let height = Height::new(raw_height);
                let (certificate_id, header) = self
                    .read_settled_header(network_id, height)
                    .map_err(E::from)?;
                let (epoch_number, certificate_index, settlement_tx_hash) = self
                    .header_settlement_fields(certificate_id, &header)
                    .map_err(E::from)?;
                self.validate_latest_header(
                    network_id,
                    height,
                    certificate_id,
                    epoch_number,
                    certificate_index,
                    latest,
                )
                .map_err(E::from)?;

                let (certificate, fallback_reason) = self
                    .read_certificate(
                        certificate_id,
                        epoch_number,
                        certificate_index,
                        &mut epoch_database,
                        &mut debug_database,
                    )
                    .map_err(E::from)?;
                self.validate_certificate(&certificate, certificate_id, &header)
                    .map_err(E::from)?;

                let expected_prev_root = reconstructed_tree.get_root().into();
                if certificate.prev_local_exit_root != expected_prev_root {
                    return Err(E::from(TreeSnapshotError::Inconsistent(format!(
                        "certificate {certificate_id} previous local exit root does not match the \
                         reconstructed tree"
                    ))));
                }

                let first_local_exit_index = reconstructed_tree.leaf_count();
                let expected_new_root = certificate.new_local_exit_root;
                for bridge_exit in &certificate.bridge_exits {
                    let leaf_hash = bridge_exit.hash();
                    let leaf_index = reconstructed_tree.leaf_count();
                    let stored_hash = self
                        .read_stored_exit_leaf(network_id, leaf_index)
                        .map_err(E::from)?;
                    if stored_hash != leaf_hash {
                        return Err(E::from(TreeSnapshotError::Inconsistent(format!(
                            "local exit leaf {leaf_index} for network {network_id} does not match \
                             certificate {certificate_id}"
                        ))));
                    }

                    reconstructed_tree
                        .add_leaf(leaf_hash)
                        .map_err(TreeSnapshotError::from)
                        .map_err(E::from)?;
                }

                if expected_new_root != reconstructed_tree.get_root().into() {
                    return Err(E::from(TreeSnapshotError::Inconsistent(format!(
                        "certificate {certificate_id} new local exit root does not match its \
                         bridge exits"
                    ))));
                }

                if let Some(reason) = fallback_reason {
                    on_warning(TreeSnapshotWarning::CertificateReadFromDebug {
                        network_id,
                        height,
                        certificate_id,
                        epoch_number,
                        certificate_index,
                        reason,
                    });
                }

                visitor(SettledCertificateSnapshot {
                    certificate,
                    certificate_id,
                    epoch_number,
                    certificate_index,
                    settlement_tx_hash,
                    first_local_exit_index,
                })?;
            }
        }

        if reconstructed_tree.leaf_count() != stored_leaf_count {
            return Err(E::from(TreeSnapshotError::Inconsistent(format!(
                "settled certificates for network {network_id} reconstruct {} leaves but the \
                 state database contains {stored_leaf_count}",
                reconstructed_tree.leaf_count()
            ))));
        }
        if reconstructed_tree.get_root() != local_exit_tree.get_root() {
            return Err(E::from(TreeSnapshotError::Inconsistent(format!(
                "reconstructed local exit root for network {network_id} does not match the \
                 current state"
            ))));
        }

        Ok(())
    }

    /// Validates and visits every historical local-exit-tree leaf for one
    /// network without retaining the network's history in memory.
    ///
    /// `E: From<TreeSnapshotError>` lets callers stop immediately on their own
    /// output error while preserving storage-validation failures.
    pub fn try_visit_network_exits<E, F>(
        &self,
        network_id: NetworkId,
        mut visitor: F,
    ) -> Result<(), E>
    where
        E: From<TreeSnapshotError>,
        F: FnMut(SettledBridgeExit) -> Result<(), E>,
    {
        self.try_visit_network_certificates(network_id, |snapshot| {
            let SettledCertificateSnapshot {
                certificate,
                certificate_id,
                epoch_number,
                certificate_index,
                settlement_tx_hash,
                first_local_exit_index,
            } = snapshot;
            let certificate_height = certificate.height;
            let mut leaf_index = first_local_exit_index;

            for bridge_exit in certificate.bridge_exits {
                let leaf_hash = bridge_exit.hash();
                visitor(SettledBridgeExit {
                    leaf_index,
                    leaf_hash,
                    bridge_exit,
                    certificate_id,
                    certificate_height,
                    epoch_number,
                    certificate_index,
                    settlement_tx_hash,
                })?;
                leaf_index = leaf_index.checked_add(1).ok_or_else(|| {
                    E::from(TreeSnapshotError::Inconsistent(format!(
                        "local exit index overflow for certificate {certificate_id}"
                    )))
                })?;
            }

            Ok(())
        })
    }

    /// Reads current non-zero balances for one network without loading its
    /// nullifier tree. Content-address mismatches are reported through
    /// `on_warning`, but traversal continues using the parent-committed hash.
    pub fn read_network_balances<F>(
        &self,
        network_id: NetworkId,
        mut on_warning: F,
    ) -> Result<Vec<TokenBalance>, TreeSnapshotError>
    where
        F: FnMut(TreeSnapshotWarning),
    {
        if !self.inventory.contains_key(&network_id) {
            return Err(TreeSnapshotError::Inconsistent(format!(
                "network {network_id} is not present in copied storage"
            )));
        }

        let tree = self
            .read_balance_tree(network_id, &mut on_warning)?
            .ok_or_else(|| {
                TreeSnapshotError::Inconsistent(format!(
                    "network {network_id} has no local balance tree"
                ))
            })?;
        let mut balances = BalanceTree(tree)
            .get_all_balances()
            .map_err(|source| TreeSnapshotError::BalanceTree { network_id, source })?
            .map(|(path, value)| TokenBalance {
                token: TokenInfo::from_bits(&path.as_bits()),
                amount: U256::from_be_bytes(*value.as_bytes()),
            })
            .collect::<Vec<_>>();
        balances.sort_unstable_by_key(|entry| entry.token);
        Ok(balances)
    }

    fn read_inventory(
        state_store: &StateStore,
    ) -> Result<BTreeMap<NetworkId, NetworkInventory>, TreeSnapshotError> {
        let mut inventory = BTreeMap::<NetworkId, NetworkInventory>::new();

        for row in state_store
            .database()
            .iter_with_direction::<CertificatePerNetworkColumn>(
                ReadOptions::default(),
                Direction::Forward,
            )?
        {
            let (key, _) = row?;
            let network_id = NetworkId::from(key.network_id);
            let cursors = &mut inventory.entry(network_id).or_default().cursors;
            cursors.count = cursors.count.checked_add(1).ok_or_else(|| {
                TreeSnapshotError::Inconsistent(format!(
                    "settled certificate cursor count overflow for network {network_id}"
                ))
            })?;
            cursors.highest = Some(
                cursors
                    .highest
                    .map_or(key.height, |highest| highest.max(key.height)),
            );
        }

        for row in state_store
            .database()
            .iter_with_direction::<LatestSettledCertificatePerNetworkColumn>(
                ReadOptions::default(),
                Direction::Forward,
            )?
        {
            let (network_id, latest) = row?;
            inventory.entry(network_id).or_default().latest_settled = Some(latest);
        }

        for row in state_store
            .database()
            .iter_with_direction::<LocalExitTreePerNetworkColumn>(
                ReadOptions::default(),
                Direction::Forward,
            )?
        {
            let (key, value) = row?;
            let network_id = NetworkId::from(key.network_id);
            let rows = &mut inventory.entry(network_id).or_default().exit_tree;

            match (key.key_type, value) {
                (let_column::KeyType::LeafCount, let_column::Value::LeafCount(count)) => {
                    rows.leaf_count = Some(count);
                }
                (let_column::KeyType::Leaf(index), let_column::Value::Leaf(_)) => {
                    rows.leaf_rows = rows.leaf_rows.checked_add(1).ok_or_else(|| {
                        TreeSnapshotError::Inconsistent(format!(
                            "local exit leaf row count overflow for network {network_id}"
                        ))
                    })?;
                    rows.highest_leaf = Some(
                        rows.highest_leaf
                            .map_or(index, |highest| highest.max(index)),
                    );
                }
                (let_column::KeyType::Frontier(layer), let_column::Value::Frontier(_))
                    if layer < 32 => {}
                (key_type, _) => {
                    return Err(TreeSnapshotError::Inconsistent(format!(
                        "local exit tree key/value mismatch for network {network_id}: {key_type:?}"
                    )));
                }
            }
        }

        Ok(inventory)
    }

    fn validate_history_shape(
        &self,
        network_id: NetworkId,
    ) -> Result<Option<&SettledCertificate>, TreeSnapshotError> {
        let rows = self.inventory.get(&network_id).ok_or_else(|| {
            TreeSnapshotError::Inconsistent(format!(
                "network {network_id} is not present in copied storage"
            ))
        })?;

        match (&rows.cursors.highest, &rows.latest_settled) {
            (None, None) if rows.cursors.count == 0 => Ok(None),
            (None, Some(_)) => Err(TreeSnapshotError::Inconsistent(format!(
                "network {network_id} has a latest-settled pointer but no settled certificate \
                 cursors"
            ))),
            (Some(_), None) => Err(TreeSnapshotError::Inconsistent(format!(
                "network {network_id} has settled certificate cursors but no latest-settled \
                 pointer"
            ))),
            (Some(highest), Some(latest)) => {
                let expected_count = highest.as_u64().checked_add(1).ok_or_else(|| {
                    TreeSnapshotError::Inconsistent(format!(
                        "settled certificate height overflows for network {network_id}"
                    ))
                })?;
                if latest.1 != *highest || rows.cursors.count != expected_count {
                    return Err(TreeSnapshotError::Inconsistent(format!(
                        "settled certificate cursors for network {network_id} are not contiguous \
                         through the latest-settled height"
                    )));
                }

                let latest_cursor = self
                    .state_store
                    .database()
                    .get::<CertificatePerNetworkColumn>(&certificate_per_network::Key {
                        network_id: network_id.to_u32(),
                        height: *highest,
                    })?
                    .ok_or_else(|| {
                        TreeSnapshotError::Inconsistent(format!(
                            "settled certificate history for network {network_id} is missing \
                             height {highest}"
                        ))
                    })?;
                if latest_cursor != latest.0 {
                    return Err(TreeSnapshotError::Inconsistent(format!(
                        "latest-settled pointer for network {network_id} does not match its \
                         highest settled cursor"
                    )));
                }
                Ok(Some(latest))
            }
            (None, None) => Err(TreeSnapshotError::Inconsistent(format!(
                "settled certificate cursor count for network {network_id} is inconsistent"
            ))),
        }
    }

    fn validate_exit_tree_shape(&self, network_id: NetworkId) -> Result<u32, TreeSnapshotError> {
        let rows = &self
            .inventory
            .get(&network_id)
            .ok_or_else(|| {
                TreeSnapshotError::Inconsistent(format!(
                    "network {network_id} is not present in copied storage"
                ))
            })?
            .exit_tree;
        let leaf_count = rows.leaf_count.ok_or_else(|| {
            TreeSnapshotError::Inconsistent(format!(
                "local exit tree for network {network_id} has no leaf count"
            ))
        })?;

        if rows.leaf_rows != u64::from(leaf_count)
            || match leaf_count {
                0 => rows.highest_leaf.is_some(),
                count => rows.highest_leaf != Some(count - 1),
            }
        {
            return Err(TreeSnapshotError::Inconsistent(format!(
                "local exit tree for network {network_id} declares {leaf_count} leaves but its \
                 indexed leaf rows are not contiguous"
            )));
        }
        Ok(leaf_count)
    }

    fn read_settled_header(
        &self,
        network_id: NetworkId,
        height: Height,
    ) -> Result<(CertificateId, CertificateHeader), TreeSnapshotError> {
        let certificate_id = self
            .state_store
            .database()
            .get::<CertificatePerNetworkColumn>(&certificate_per_network::Key {
                network_id: network_id.to_u32(),
                height,
            })?
            .ok_or_else(|| {
                TreeSnapshotError::Inconsistent(format!(
                    "settled certificate history for network {network_id} is missing height \
                     {height}"
                ))
            })?;
        let header = self
            .state_store
            .database()
            .get::<CertificateHeaderColumn>(&certificate_id)?
            .ok_or_else(|| {
                TreeSnapshotError::Inconsistent(format!(
                    "certificate {certificate_id} for network {network_id} at height {height} has \
                     no header"
                ))
            })?;

        if header.certificate_id != certificate_id
            || header.network_id != network_id
            || header.height != height
            || header.status != CertificateStatus::Settled
        {
            return Err(TreeSnapshotError::Inconsistent(format!(
                "header for certificate {certificate_id} does not match its settled \
                 network/height cursor"
            )));
        }

        Ok((certificate_id, header))
    }

    fn header_settlement_fields(
        &self,
        certificate_id: CertificateId,
        header: &CertificateHeader,
    ) -> Result<(EpochNumber, CertificateIndex, SettlementTxHash), TreeSnapshotError> {
        let epoch_number = header.epoch_number.ok_or_else(|| {
            TreeSnapshotError::Inconsistent(format!(
                "settled certificate {certificate_id} has no epoch number"
            ))
        })?;
        let certificate_index = header.certificate_index.ok_or_else(|| {
            TreeSnapshotError::Inconsistent(format!(
                "settled certificate {certificate_id} has no epoch index"
            ))
        })?;
        let settlement_tx_hash = header.settlement_tx_hash.ok_or_else(|| {
            TreeSnapshotError::Inconsistent(format!(
                "settled certificate {certificate_id} has no settlement transaction hash"
            ))
        })?;
        Ok((epoch_number, certificate_index, settlement_tx_hash))
    }

    #[allow(clippy::too_many_arguments)]
    fn validate_latest_header(
        &self,
        network_id: NetworkId,
        height: Height,
        certificate_id: CertificateId,
        epoch_number: EpochNumber,
        certificate_index: CertificateIndex,
        latest: &SettledCertificate,
    ) -> Result<(), TreeSnapshotError> {
        if height == latest.1
            && (certificate_id != latest.0
                || epoch_number != latest.2
                || certificate_index != latest.3)
        {
            return Err(TreeSnapshotError::Inconsistent(format!(
                "latest-settled certificate metadata for network {network_id} does not match \
                 certificate {certificate_id}"
            )));
        }
        Ok(())
    }

    fn read_stored_exit_leaf(
        &self,
        network_id: NetworkId,
        leaf_index: u32,
    ) -> Result<Digest, TreeSnapshotError> {
        match self
            .state_store
            .database()
            .get::<LocalExitTreePerNetworkColumn>(&let_column::Key {
                network_id: network_id.to_u32(),
                key_type: let_column::KeyType::Leaf(leaf_index),
            })? {
            Some(let_column::Value::Leaf(hash)) => Ok(Digest::from(hash)),
            Some(_) => Err(TreeSnapshotError::Inconsistent(format!(
                "local exit leaf {leaf_index} for network {network_id} has the wrong value type"
            ))),
            None => Err(TreeSnapshotError::Inconsistent(format!(
                "local exit tree for network {network_id} is missing leaf {leaf_index}"
            ))),
        }
    }

    fn read_balance_tree<F>(
        &self,
        network_id: NetworkId,
        on_warning: &mut F,
    ) -> Result<Option<Smt<LOCAL_BALANCE_TREE_DEPTH>>, TreeSnapshotError>
    where
        F: FnMut(TreeSnapshotWarning),
    {
        let root_node = match self
            .state_store
            .database()
            .get::<BalanceTreePerNetworkColumn>(&SmtKey {
                network_id: network_id.to_u32(),
                key_type: SmtKeyType::Root,
            })? {
            Some(SmtValue::Node(left, right)) => Node { left, right },
            Some(_) => return Err(Error::WrongValueType.into()),
            None => return Ok(None),
        };

        // Preserve the hashes committed by each parent as the map keys. If a
        // stored node payload hashes differently, re-keying it by its computed
        // hash would silently drop that entire branch during enumeration.
        let root_hash = root_node.hash();
        let mut tree = HashMap::from([(root_hash, root_node)]);
        let mut queued = HashSet::from([root_hash]);
        let mut keys = VecDeque::new();
        for child in [root_node.left, root_node.right] {
            if queued.insert(child) {
                keys.push_back(child);
            }
        }

        while let Some(expected) = keys.pop_front() {
            let value = self
                .state_store
                .database()
                .get::<BalanceTreePerNetworkColumn>(&SmtKey {
                    network_id: network_id.to_u32(),
                    key_type: SmtKeyType::Node(expected),
                })?
                .ok_or(Error::SmtNodeNotFound)?;

            match value {
                SmtValue::Node(left, right) => {
                    let node = Node { left, right };
                    let actual = node.hash();
                    if actual != expected {
                        on_warning(TreeSnapshotWarning::BalanceNodeHashMismatch {
                            network_id,
                            expected,
                            actual,
                        });
                    }
                    tree.insert(expected, node);
                    for child in [left, right] {
                        if queued.insert(child) {
                            keys.push_back(child);
                        }
                    }
                }
                SmtValue::Leaf(actual) => {
                    if actual != expected {
                        on_warning(TreeSnapshotWarning::BalanceLeafHashMismatch {
                            network_id,
                            expected,
                            actual,
                        });
                    }
                }
            }
        }

        Ok(Some(Smt {
            root: root_hash,
            tree,
        }))
    }

    fn read_certificate(
        &self,
        certificate_id: CertificateId,
        epoch_number: EpochNumber,
        certificate_index: CertificateIndex,
        epoch_database: &mut Option<(EpochNumber, DB)>,
        debug_database: &mut Option<DB>,
    ) -> Result<(Certificate, Option<DebugCertificateFallbackReason>), TreeSnapshotError> {
        let epoch_result =
            self.read_epoch_certificate(epoch_number, certificate_index, epoch_database);
        match epoch_result {
            Ok(certificate) => Ok((certificate, None)),
            Err(EpochCertificateFailure::Fatal(error)) => Err(error),
            Err(epoch_failure)
                if self.latest_referenced_epoch != Some(epoch_number)
                    || !self.debug_path.exists() =>
            {
                Err(epoch_failure.into_error())
            }
            Err(epoch_failure) => {
                let fallback_reason = epoch_failure.reason();
                if debug_database.is_none() {
                    *debug_database =
                        Some(open_debug_db_readonly(&self.debug_path).map_err(|source| {
                            TreeSnapshotError::OpenDebug {
                                path: self.debug_path.clone(),
                                source,
                            }
                        })?);
                }

                let certificate = read_debug_certificate_from_db(
                    debug_database
                        .as_ref()
                        .expect("debug database was inserted above"),
                    &certificate_id,
                )?
                .ok_or_else(|| epoch_failure.into_error())?;

                Ok((certificate, Some(fallback_reason)))
            }
        }
    }

    fn read_epoch_certificate(
        &self,
        epoch_number: EpochNumber,
        certificate_index: CertificateIndex,
        epoch_database: &mut Option<(EpochNumber, DB)>,
    ) -> Result<Certificate, EpochCertificateFailure> {
        if epoch_database
            .as_ref()
            .is_none_or(|(open_epoch, _)| *open_epoch != epoch_number)
        {
            let path = self.epochs_path.join(epoch_number.to_string());
            let current_path = path.join("CURRENT");
            match fs::metadata(&current_path) {
                Ok(metadata) if metadata.is_file() => {}
                Ok(_) => {
                    return Err(EpochCertificateFailure::Fatal(
                        TreeSnapshotError::InvalidEpochMarker {
                            epoch_number,
                            path: current_path,
                        },
                    ));
                }
                Err(source) if source.kind() == io::ErrorKind::NotFound => {
                    return Err(EpochCertificateFailure::Unavailable(
                        TreeSnapshotError::EpochUnavailable { epoch_number, path },
                    ));
                }
                Err(source) => {
                    return Err(EpochCertificateFailure::Fatal(
                        TreeSnapshotError::InspectEpoch {
                            epoch_number,
                            path: current_path,
                            source,
                        },
                    ));
                }
            }

            let db = open_db_readonly(&path).map_err(|source| {
                EpochCertificateFailure::Fatal(TreeSnapshotError::OpenEpoch {
                    epoch_number,
                    path,
                    source,
                })
            })?;
            *epoch_database = Some((epoch_number, db));
        }

        match read_certificate_from_db(
            &epoch_database
                .as_ref()
                .expect("epoch database was inserted above")
                .1,
            epoch_number,
            certificate_index,
        ) {
            Ok(Some(certificate)) => Ok(certificate),
            Ok(None) => Err(EpochCertificateFailure::Missing(
                TreeSnapshotError::Inconsistent(format!(
                    "epoch {epoch_number} has no certificate at index {certificate_index}"
                )),
            )),
            Err(source) => Err(EpochCertificateFailure::Fatal(source.into())),
        }
    }

    fn validate_certificate(
        &self,
        certificate: &Certificate,
        expected_id: CertificateId,
        header: &CertificateHeader,
    ) -> Result<(), TreeSnapshotError> {
        if certificate.hash() != expected_id
            || certificate.network_id != header.network_id
            || certificate.height != header.height
            || certificate.prev_local_exit_root != header.prev_local_exit_root
            || certificate.new_local_exit_root != header.new_local_exit_root
            || certificate.metadata != header.metadata
        {
            return Err(TreeSnapshotError::Inconsistent(format!(
                "certificate {expected_id} does not match its state header"
            )));
        }

        Ok(())
    }
}
