use std::collections::{BTreeMap, BTreeSet};

use agglayer_interop_types::{ImportedBridgeExit, TokenInfo};
use agglayer_primitives::{ruint::UintTryFrom, FromBool, Hashable};
use agglayer_tries::{roots::LocalExitRoot, smt::Smt};
use pessimistic_proof::{
    core::commitment::{PessimisticRootCommitmentValues, PessimisticRootCommitmentVersion},
    local_balance_tree::{LocalBalancePath, LocalBalanceTree, LOCAL_BALANCE_TREE_DEPTH},
    local_state::StateCommitment,
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierPath, NullifierTree, NULLIFIER_TREE_DEPTH},
    LocalNetworkState,
};
use unified_bridge::LocalExitTree;

use crate::{
    aggchain_data::{CertificateAggchainDataCtx, CertificateAggchainDataWithCtx},
    Certificate, Digest, Error, U256, U512,
};

/// Local state data of one network.
/// The AggLayer tracks the [`LocalNetworkStateData`] for all networks.
#[derive(Clone, Debug, Default)]
pub struct LocalNetworkStateData {
    /// The local exit tree without leaves.
    pub exit_tree: LocalExitTree,
    /// The full local balance tree.
    pub balance_tree: Smt<LOCAL_BALANCE_TREE_DEPTH>,
    /// The full nullifier tree.
    pub nullifier_tree: Smt<NULLIFIER_TREE_DEPTH>,
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

impl From<LocalNetworkStateData> for pessimistic_proof::NetworkState {
    fn from(state: LocalNetworkStateData) -> Self {
        LocalNetworkState::from(state).into()
    }
}

/// The last pessimistic root can be either fetched from L1 or recomputed for a
/// given version.
#[derive(Debug, Clone)]
pub enum PessimisticRootInput {
    /// Computed from the given version.
    Computed(PessimisticRootCommitmentVersion),
    /// Fetched from the L1.
    Fetched(Digest),
}

/// Represents all context data fetched from the L1 for the witness generation.
#[derive(Debug, Clone)]
pub struct L1WitnessCtx {
    pub l1_info_root: Digest,
    pub prev_pessimistic_root: PessimisticRootInput,
    pub aggchain_data_ctx: CertificateAggchainDataCtx,
}

impl LocalNetworkStateData {
    /// Prune the SMTs
    pub fn prune_stale_nodes(&mut self) -> Result<(), Error> {
        self.balance_tree.traverse_and_prune()?;
        self.nullifier_tree.traverse_and_prune()?;

        Ok(())
    }

    /// Apply the [`Certificate`] on the current state and returns the
    /// [`MultiBatchHeader`] associated to the state transition.
    pub fn apply_certificate(
        &mut self,
        certificate: &Certificate,
        ctx_from_l1: L1WitnessCtx,
    ) -> Result<MultiBatchHeader, Error> {
        let L1WitnessCtx {
            prev_pessimistic_root,
            l1_info_root,
            aggchain_data_ctx,
        } = ctx_from_l1;

        let gers_are_consistent = certificate
            .imported_bridge_exits
            .iter()
            .all(|ib| ib.valid_claim());

        if !gers_are_consistent {
            return Err(Error::InconsistentGlobalExitRoot);
        }

        // Retrieve the pp root
        let prev_pessimistic_root = match prev_pessimistic_root {
            PessimisticRootInput::Fetched(settled_from_l1) => settled_from_l1,
            PessimisticRootInput::Computed(version) => PessimisticRootCommitmentValues {
                balance_root: self.balance_tree.root.into(),
                nullifier_root: self.nullifier_tree.root.into(),
                ler_leaf_count: self.exit_tree.leaf_count(),
                height: certificate.height.as_u64(),
                origin_network: certificate.network_id,
            }
            .compute_pp_root(version),
        };

        let prev_local_exit_root = self.exit_tree.get_root().into();
        if certificate.prev_local_exit_root != prev_local_exit_root {
            return Err(Error::MismatchPrevLocalExitRoot {
                computed: prev_local_exit_root,
                declared: certificate.prev_local_exit_root,
            });
        }

        for e in certificate.bridge_exits.iter() {
            self.exit_tree.add_leaf(e.hash())?;
        }

        let balances_proofs: BTreeMap<TokenInfo, (U256, LocalBalancePath)> = {
            // Consider all the imported bridge exits except for the native token
            let imported_bridge_exits = certificate.imported_bridge_exits.iter().filter(|b| {
                b.bridge_exit.amount_token_info().origin_network != certificate.network_id
            });

            // Consider all the bridge exits except for the native token
            let bridge_exits = certificate
                .bridge_exits
                .iter()
                .filter(|b| b.amount_token_info().origin_network != certificate.network_id);

            // Set of dedup tokens mutated in the transition
            let mutated_tokens: BTreeSet<TokenInfo> = {
                let imported_tokens = imported_bridge_exits
                    .clone()
                    .map(|exit| exit.bridge_exit.amount_token_info());
                let exported_tokens = bridge_exits.clone().map(|exit| exit.amount_token_info());
                imported_tokens.chain(exported_tokens).collect()
            };

            let initial_balances: BTreeMap<_, _> = mutated_tokens
                .iter()
                .map(|&token| {
                    let balance =
                        U256::from_be_bytes(*self.balance_tree.get(token).unwrap_or_default());
                    (token, balance)
                })
                .collect();

            let mut new_balances: BTreeMap<_, _> = initial_balances
                .iter()
                .map(|(&token, &balance)| (token, U512::from(balance)))
                .collect();

            for imported_bridge_exit in imported_bridge_exits {
                let token = imported_bridge_exit.bridge_exit.amount_token_info();
                new_balances.insert(
                    token,
                    new_balances[&token]
                        .checked_add(U512::from(imported_bridge_exit.bridge_exit.amount))
                        .ok_or(Error::BalanceOverflow(token))?,
                );
            }

            for bridge_exit in bridge_exits {
                let token = bridge_exit.amount_token_info();
                new_balances.insert(
                    token,
                    new_balances[&token]
                        .checked_sub(U512::from(bridge_exit.amount))
                        .ok_or(Error::BalanceUnderflow(token))?,
                );
            }

            // Get the proof against the initial balance for each token
            mutated_tokens
                .into_iter()
                .map(|token| {
                    let initial_balance = initial_balances[&token];

                    let new_balance = U256::uint_try_from(new_balances[&token])
                        .map_err(|_| Error::BalanceOverflow(token))?;

                    let balance_proof_error =
                        |source| Error::BalanceProofGenerationFailed { source, token };

                    let path = if initial_balance.is_zero() {
                        self.balance_tree
                            .get_inclusion_proof_zero(token)
                            .map_err(balance_proof_error)?
                    } else {
                        self.balance_tree
                            .get_inclusion_proof(token)
                            .map_err(balance_proof_error)?
                    };

                    self.balance_tree
                        .update(token, new_balance.to_be_bytes().into())
                        .map_err(balance_proof_error)?;

                    Ok((token, (initial_balance, path)))
                })
                .collect::<Result<BTreeMap<_, _>, Error>>()?
        };

        let imported_bridge_exits: Vec<(ImportedBridgeExit, NullifierPath)> = certificate
            .imported_bridge_exits
            .iter()
            .map(|exit| {
                let nullifier_key: NullifierKey = exit.global_index.into();
                let nullifier_error = |source| Error::NullifierPathGenerationFailed {
                    source,
                    global_index: exit.global_index,
                };
                let nullifier_path = self
                    .nullifier_tree
                    .get_non_inclusion_proof(nullifier_key)
                    .map_err(nullifier_error)?;
                self.nullifier_tree
                    .insert(nullifier_key, Digest::from_bool(true))
                    .map_err(nullifier_error)?;
                Ok((exit.clone(), nullifier_path))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        // Check that the certificate referred to the right target
        let computed = LocalExitRoot::from(self.exit_tree.get_root());
        if computed != certificate.new_local_exit_root {
            return Err(Error::MismatchNewLocalExitRoot {
                declared: certificate.new_local_exit_root,
                computed,
            });
        }

        let chain_payload = certificate
            .aggchain_data
            .clone()
            .try_into()
            .map_err(Error::InvalidChainData)?;

        let aggchain_data = CertificateAggchainDataWithCtx(chain_payload, aggchain_data_ctx)
            .try_into()
            .map_err(Error::InvalidChainData)?;

        Ok(MultiBatchHeader {
            origin_network: certificate.network_id,
            bridge_exits: certificate.bridge_exits.clone(),
            imported_bridge_exits,
            balances_proofs,
            l1_info_root,
            height: certificate.height.as_u64(),
            prev_pessimistic_root,
            aggchain_data,
            certificate_id: certificate.hash().into(),
        })
    }

    /// Generates the [`MultiBatchHeader`] from the state and a [`Certificate`].
    /// Does not mutate the current state.
    pub fn make_multi_batch_header(
        &self,
        certificate: &Certificate,
        witness_fetched_from_l1: L1WitnessCtx,
    ) -> Result<MultiBatchHeader, Error> {
        self.clone()
            .apply_certificate(certificate, witness_fetched_from_l1)
    }

    pub fn get_roots(&self) -> StateCommitment {
        StateCommitment {
            exit_root: self.exit_tree.get_root(),
            ler_leaf_count: self.exit_tree.leaf_count(),
            balance_root: self.balance_tree.root,
            nullifier_root: self.nullifier_tree.root,
        }
    }
}
