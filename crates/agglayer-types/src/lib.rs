use std::collections::{BTreeMap, BTreeSet};

use pessimistic_proof::local_balance_tree::{LocalBalanceTree, LOCAL_BALANCE_TREE_DEPTH};
pub use pessimistic_proof::local_exit_tree::hasher::Keccak256Hasher;
use pessimistic_proof::local_exit_tree::LocalExitTree;
use pessimistic_proof::local_state::StateCommitment;
use pessimistic_proof::nullifier_tree::{FromBool, NullifierTree, NULLIFIER_TREE_DEPTH};
use pessimistic_proof::utils::smt::Smt;
use pessimistic_proof::{
    bridge_exit::{BridgeExit, TokenInfo},
    imported_bridge_exit::{commit_imported_bridge_exits, ImportedBridgeExit},
    keccak::{keccak256_combine, Digest},
    local_balance_tree::LocalBalancePath,
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierPath},
};
use pessimistic_proof::{LocalNetworkState, ProofError};
use reth_primitives::{Address, Signature};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Imported bridge exits refer to multiple L1 info root")]
    MultipleL1InfoRoot,
    #[error("Computed exit root: {computed:?} differs from certificate exit root: {declared:?}")]
    MismatchNewLocalExitRoot { computed: Digest, declared: Digest },
    #[error("Overflowed imported bridge exits: {0}")]
    ImportedBridgeExitOverflow(#[from] pessimistic_proof::ProofError),
    #[error("Failed to apply the Certificate on the given state: {0}")]
    InvalidCertificateProjection(#[from] pessimistic_proof::utils::smt::SmtError),
}

pub use reth_primitives::U256;
pub type EpochNumber = u64;
pub type CertificateIndex = u64;
pub type CertificateId = [u8; 32];
pub type Hash = [u8; 32];
pub type Height = u64;
pub use pessimistic_proof::bridge_exit::NetworkId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateHeader {
    pub network_id: NetworkId,
    pub height: Height,
    pub epoch_number: Option<EpochNumber>,
    pub certificate_index: Option<CertificateIndex>,
    pub certificate_id: CertificateId,
    pub new_local_exit_root: Hash,
}

/// Proof is a wrapper around all the different types of proofs that can be
/// generated
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Proof {
    SP1(sp1_sdk::SP1ProofWithPublicValues),
}

impl Proof {
    pub fn new_for_test() -> Self {
        Proof::SP1(sp1_sdk::SP1ProofWithPublicValues {
            proof: sp1_sdk::SP1Proof::Core(Vec::new()),
            stdin: sp1_sdk::SP1Stdin::new(),
            public_values: sp1_core_machine::io::SP1PublicValues::new(),
            sp1_version: String::new(),
        })
    }
}

/// Represents the data submitted by the chains to the AggLayer.
///
/// The bridge exits plus the imported bridge exits define
/// the state transition, resp. the amount that goes out and the amount that
/// comes in.
///
/// The bridge exits refer to the [`BridgeExit`] emitted by
/// the origin network of the [`Certificate`].
///
/// The imported bridge exits refer to the [`BridgeExit`] received and imported
/// by the origin network of the [`Certificate`].
///
/// Note: be mindful to update the [`Self::hash`] method accordingly
/// upon modifying the fields of this structure.
#[derive(Default, Serialize, Deserialize, Clone, Debug)]
pub struct Certificate {
    pub network_id: NetworkId,
    pub height: Height,
    pub prev_local_exit_root: Digest,
    pub new_local_exit_root: Digest,
    pub bridge_exits: Vec<BridgeExit>,
    pub imported_bridge_exits: Vec<ImportedBridgeExit>,
    pub signature: Signature,
}

impl Certificate {
    #[cfg(any(test, feature = "testutils"))]
    pub fn new_for_test(network_id: NetworkId, height: Height) -> Self {
        Certificate {
            network_id,
            height,
            prev_local_exit_root: [0; 32],
            new_local_exit_root: [0; 32],
            bridge_exits: Vec::new(),
            imported_bridge_exits: Vec::new(),
            signature: Signature::default(),
        }
    }

    pub fn hash(&self) -> Digest {
        keccak256_combine([
            self.network_id.to_be_bytes().as_slice(),
            self.height.to_be_bytes().as_slice(),
            self.prev_local_exit_root.as_slice(),
            self.new_local_exit_root.as_slice(),
        ])
    }
}

/// Local state data of one network.
/// The AggLayer tracks the [`LocalNetworkStateData`] for all networks.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LocalNetworkStateData {
    /// The local exit tree without leaves.
    pub exit_tree: LocalExitTree<Keccak256Hasher>,
    /// The full local balance tree.
    pub balance_tree: Smt<Keccak256Hasher, LOCAL_BALANCE_TREE_DEPTH>,
    /// The full nullifier tree.
    pub nullifier_tree: Smt<Keccak256Hasher, NULLIFIER_TREE_DEPTH>,
}

impl From<LocalNetworkStateData> for LocalNetworkState {
    fn from(state: LocalNetworkStateData) -> Self {
        LocalNetworkState {
            exit_tree: state.exit_tree,
            balance_tree: LocalBalanceTree::new_with_root(state.balance_tree.root),
            nullifier_tree: NullifierTree::new_with_root(state.nullifier_tree.root),
        }
    }
}

impl LocalNetworkStateData {
    /// Apply the [`Certificate`] on the current state and returns the
    /// [`MultiBatchHeader`] associated to the state transition.
    pub fn apply_certificate(
        &mut self,
        certificate: &Certificate,
        signer: Address,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Error> {
        let prev_balance_root = self.balance_tree.root;
        let prev_nullifier_root = self.nullifier_tree.root;

        certificate
            .bridge_exits
            .iter()
            .for_each(|e| self.exit_tree.add_leaf(e.hash()));

        let balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath<Keccak256Hasher>)> = {
            // Consider all the imported bridge exits
            let imported_bridge_exits = certificate.imported_bridge_exits.iter();
            // Consider all the bridge exits except for the native token
            let bridge_exits = certificate
                .bridge_exits
                .iter()
                .filter(|b| b.token_info.origin_network != certificate.network_id);

            // Set of dedup tokens mutated in the transition
            let mutated_tokens: BTreeSet<TokenInfo> = {
                let imported_tokens = imported_bridge_exits
                    .clone()
                    .map(|exit| exit.bridge_exit.token_info);
                let exported_tokens = bridge_exits.clone().map(|exit| exit.token_info);
                imported_tokens.chain(exported_tokens).collect()
            };

            let initial_balances: BTreeMap<_, _> = mutated_tokens
                .iter()
                .map(|&token| {
                    let balance =
                        U256::from_be_bytes(self.balance_tree.get(token).unwrap_or_default());
                    (token, balance)
                })
                .collect();

            let mut new_balances = initial_balances.clone();
            for imported_bridge_exit in imported_bridge_exits {
                let token = imported_bridge_exit.bridge_exit.token_info;
                new_balances.insert(
                    token,
                    new_balances[&token]
                        .checked_add(imported_bridge_exit.bridge_exit.amount)
                        .ok_or(ProofError::BalanceOverflowInBridgeExit)?,
                );
            }

            for bridge_exit in bridge_exits {
                let token = bridge_exit.token_info;
                new_balances.insert(
                    token,
                    new_balances[&token]
                        .checked_sub(bridge_exit.amount)
                        .ok_or(ProofError::BalanceUnderflowInBridgeExit)?,
                );
            }

            // Get the proof against the initial balance for each token
            mutated_tokens
                .into_iter()
                .map(|token| {
                    let initial_balance = initial_balances[&token];

                    let path = if initial_balance.is_zero() {
                        self.balance_tree.get_inclusion_proof_zero(token)?
                    } else {
                        self.balance_tree.get_inclusion_proof(token)?
                    };

                    self.balance_tree
                        .update(token, new_balances[&token].to_be_bytes())?;

                    Ok((token, (initial_balance, path)))
                })
                .collect::<Result<BTreeMap<_, _>, Error>>()?
        };

        let l1_info_root: Digest = {
            if let Some(imported_bridge_exit) = certificate.imported_bridge_exits.first() {
                let l1_root = imported_bridge_exit.l1_root();
                if certificate
                    .imported_bridge_exits
                    .iter()
                    .all(|exit| exit.l1_root() == l1_root)
                {
                    Ok(l1_root)
                } else {
                    Err(Error::MultipleL1InfoRoot)
                }
            } else {
                Ok(Digest::default())
            }
        }?;

        let imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath<Keccak256Hasher>)> =
            certificate
                .imported_bridge_exits
                .iter()
                .map(|exit| {
                    let nullifier_key: NullifierKey = exit.global_index.into();
                    let nullifier_path =
                        self.nullifier_tree.get_non_inclusion_proof(nullifier_key)?;
                    self.nullifier_tree
                        .insert(nullifier_key, Digest::from_bool(true))?;
                    Ok((exit.clone(), nullifier_path))
                })
                .collect::<Result<Vec<_>, Error>>()?;

        let imported_hash =
            commit_imported_bridge_exits(imported_bridge_exits.iter().map(|(exit, _)| exit));

        // Check that the certificate referred to the right target
        let computed = self.exit_tree.get_root();
        if computed != certificate.new_local_exit_root {
            return Err(Error::MismatchNewLocalExitRoot {
                declared: certificate.new_local_exit_root,
                computed,
            });
        }

        Ok(MultiBatchHeader::<Keccak256Hasher> {
            origin_network: certificate.network_id,
            prev_local_exit_root: certificate.prev_local_exit_root,
            bridge_exits: certificate.bridge_exits.clone(),
            imported_bridge_exits,
            balances_proofs,
            prev_balance_root,
            prev_nullifier_root,
            signer,
            signature: certificate.signature,
            imported_exits_root: Some(imported_hash),
            target: StateCommitment {
                exit_root: certificate.new_local_exit_root,
                balance_root: self.balance_tree.root,
                nullifier_root: self.nullifier_tree.root,
            },
            l1_info_root,
        })
    }

    /// Generates the [`MultiBatchHeader`] from the state and a [`Certificate`].
    /// Does not mutate the current state.
    pub fn make_multi_batch_header(
        &self,
        certificate: &Certificate,
        signer: Address,
    ) -> Result<MultiBatchHeader<Keccak256Hasher>, Error> {
        self.clone().apply_certificate(certificate, signer)
    }
}
