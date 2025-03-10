use std::collections::{btree_map::Entry, BTreeMap};

use agglayer_primitives::{ruint::UintTryFrom, Signature, B256, U256, U512};
use commitment::StateCommitment;
use serde::{Deserialize, Serialize};
#[cfg(not(target_os = "zkvm"))]
use tracing::warn;

use self::commitment::{PPRootVersion, PessimisticRoot, SignatureCommitment};
#[cfg(target_os = "zkvm")]
use crate::aggchain_proof::AggchainProofPublicValues;
use crate::{
    aggchain_proof::AggchainData,
    bridge_exit::{L1_ETH, L1_NETWORK_ID},
    imported_bridge_exit::{commit_imported_bridge_exits, Error},
    keccak::digest::Digest,
    local_balance_tree::LocalBalanceTree,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierTree},
    ProofError,
};

pub mod commitment;

/// State representation of one network without the leaves, taken as input by
/// the prover.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkState {
    /// Commitment to the [`BridgeExit`](struct@crate::bridge_exit::BridgeExit).
    pub exit_tree: LocalExitTree<Keccak256Hasher>,
    /// Commitment to the balance for each token.
    pub balance_tree: LocalBalanceTree<Keccak256Hasher>,
    /// Commitment to the Nullifier tree for the local network, tracks claimed
    /// assets on foreign networks
    pub nullifier_tree: NullifierTree<Keccak256Hasher>,
}

impl NetworkState {
    /// Returns the roots.
    pub fn roots(&self) -> StateCommitment {
        StateCommitment {
            exit_root: self.exit_tree.get_root(),
            balance_root: self.balance_tree.root,
            nullifier_root: self.nullifier_tree.root,
        }
    }

    /// Apply the [`MultiBatchHeader`] on the current [`LocalNetworkState`].
    /// Checks that the transition reaches the target [`StateCommitment`].
    /// The state isn't modified on error.
    pub fn apply_batch_header(
        &mut self,
        multi_batch_header: &MultiBatchHeader<Keccak256Hasher>,
    ) -> Result<StateCommitment, ProofError> {
        let mut clone = self.clone();
        let roots = clone.apply_batch_header_helper(multi_batch_header)?;
        *self = clone;

        Ok(roots)
    }

    /// Apply the [`MultiBatchHeader`] on the current [`LocalNetworkState`].
    /// Returns the resulting [`StateCommitment`] upon success.
    /// The state can be modified on error.
    fn apply_batch_header_helper(
        &mut self,
        multi_batch_header: &MultiBatchHeader<Keccak256Hasher>,
    ) -> Result<StateCommitment, ProofError> {
        // Check the initial state
        let computed_root = self.exit_tree.get_root();
        if computed_root != multi_batch_header.prev_local_exit_root {
            return Err(ProofError::InvalidPreviousLocalExitRoot {
                computed: computed_root,
                declared: multi_batch_header.prev_local_exit_root,
            });
        }
        if self.balance_tree.root != multi_batch_header.prev_balance_root {
            return Err(ProofError::InvalidPreviousBalanceRoot {
                computed: self.balance_tree.root,
                declared: multi_batch_header.prev_balance_root,
            });
        }

        if self.nullifier_tree.root != multi_batch_header.prev_nullifier_root {
            return Err(ProofError::InvalidPreviousNullifierRoot {
                computed: self.nullifier_tree.root,
                declared: multi_batch_header.prev_nullifier_root,
            });
        }

        // TODO: benchmark if BTreeMap is the best choice in terms of SP1 cycles
        let mut new_balances = BTreeMap::new();
        for (k, v) in &multi_batch_header.balances_proofs {
            if new_balances.insert(*k, U512::from(v.0)).is_some() {
                return Err(ProofError::DuplicateTokenBalanceProof(*k));
            }
        }

        // Apply the imported bridge exits
        for (imported_bridge_exit, nullifier_path) in &multi_batch_header.imported_bridge_exits {
            if imported_bridge_exit.global_index.network_id() == multi_batch_header.origin_network {
                // We don't allow a chain to exit to itself
                return Err(ProofError::CannotExitToSameNetwork);
            }
            // Check that the destination network of the bridge exit matches the current
            // network
            if imported_bridge_exit.bridge_exit.dest_network != multi_batch_header.origin_network {
                return Err(ProofError::InvalidImportedBridgeExit {
                    source: Error::InvalidExitNetwork,
                    global_index: imported_bridge_exit.global_index,
                });
            }

            // Check the inclusion proof
            imported_bridge_exit
                .verify_path(multi_batch_header.l1_info_root)
                .map_err(|source| ProofError::InvalidImportedBridgeExit {
                    source,
                    global_index: imported_bridge_exit.global_index,
                })?;

            // Check the nullifier non-inclusion path and update the nullifier tree
            let nullifier_key: NullifierKey = imported_bridge_exit.global_index.into();
            self.nullifier_tree
                .verify_and_update(nullifier_key, nullifier_path)?;

            // The amount corresponds to L1 ETH if the leaf is a message
            let token_info = imported_bridge_exit.bridge_exit.amount_token_info();

            if multi_batch_header.origin_network == token_info.origin_network {
                // When the token is native to the chain, we don't care about the local balance
                continue;
            }

            // Update the token balance.
            let amount = imported_bridge_exit.bridge_exit.amount;
            let entry = new_balances.entry(token_info);
            match entry {
                Entry::Vacant(_) => return Err(ProofError::MissingTokenBalanceProof(token_info)),
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = entry
                        .get()
                        .checked_add(U512::from(amount))
                        .ok_or(ProofError::BalanceOverflowInBridgeExit)?;
                }
            }
        }

        // Apply the bridge exits
        for bridge_exit in &multi_batch_header.bridge_exits {
            if bridge_exit.dest_network == multi_batch_header.origin_network {
                // We don't allow a chain to exit to itself
                return Err(ProofError::CannotExitToSameNetwork);
            }
            self.exit_tree.add_leaf(bridge_exit.hash())?;

            // For message exits, the origin network in token info should be the origin
            // network of the batch header.
            if bridge_exit.is_message()
                && bridge_exit.token_info.origin_network != multi_batch_header.origin_network
            {
                return Err(ProofError::InvalidMessageOriginNetwork);
            }

            // For ETH transfers, we need to check that the origin network is the L1 network
            if bridge_exit.token_info.origin_token_address == L1_ETH.origin_token_address
                && bridge_exit.token_info.origin_network != L1_NETWORK_ID
            {
                return Err(ProofError::InvalidL1TokenInfo(bridge_exit.token_info));
            }

            // The amount corresponds to L1 ETH if the leaf is a message
            let token_info = bridge_exit.amount_token_info();

            if multi_batch_header.origin_network == token_info.origin_network {
                // When the token is native to the chain, we don't care about the local balance
                continue;
            }

            // Update the token balance.
            let amount = bridge_exit.amount;
            let entry = new_balances.entry(token_info);
            match entry {
                Entry::Vacant(_) => return Err(ProofError::MissingTokenBalanceProof(token_info)),
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = entry
                        .get()
                        .checked_sub(U512::from(amount))
                        .ok_or(ProofError::BalanceUnderflowInBridgeExit)?;
                }
            }
        }

        // Verify that the original balances were correct and update the local balance
        // tree with the new balances. TODO: implement batch `verify_and_update`
        // for the LBT
        for (token, (old_balance, balance_path)) in &multi_batch_header.balances_proofs {
            let new_balance = new_balances[token];
            let new_balance = U256::uint_try_from(new_balance)
                .map_err(|_| ProofError::BalanceOverflowInBridgeExit)?;
            self.balance_tree
                .verify_and_update(*token, balance_path, *old_balance, new_balance)?;
        }

        Ok(self.roots())
    }

    /// Verify the signature or aggchain proof
    /// Returns the pp root version on success.
    pub fn verify_consensus(
        &self,
        multi_batch_header: &MultiBatchHeader<Keccak256Hasher>,
    ) -> Result<PPRootVersion, ProofError> {
        let base_pp_root_version = PessimisticRoot {
            balance_root: self.balance_tree.root,
            nullifier_root: self.nullifier_tree.root,
            ler_leaf_count: self.exit_tree.leaf_count,
            height: multi_batch_header.height,
            origin_network: multi_batch_header.origin_network,
        }
        .infer_settled_pp_root_version(multi_batch_header.prev_pessimistic_root)?;

        let imported_hash = commit_imported_bridge_exits(
            multi_batch_header
                .imported_bridge_exits
                .iter()
                .map(|(exit, _)| exit.global_index),
        );

        // Verify the aggchain proof which can be either one signature or one sp1 proof.
        // NOTE: The STARK is verified exclusively within the SP1 VM.
        let target_pp_root_version = match &multi_batch_header.aggchain_proof {
            AggchainData::ECDSA { signer, signature } => {
                let verify_signature = |digest: Digest, signature: &Signature| {
                    signature
                        .recover_address_from_prehash(&B256::new(digest.0))
                        .map_err(|_| ProofError::InvalidSignature)
                };

                let signature_commitment = SignatureCommitment {
                    new_local_exit_root: multi_batch_header.target.exit_root,
                    commit_imported_bridge_exits: imported_hash,
                    height: multi_batch_header.height,
                };

                let target_pp_root_version = {
                    if *signer
                        == verify_signature(
                            signature_commitment.commitment(PPRootVersion::V3),
                            signature,
                        )?
                    {
                        PPRootVersion::V3
                    } else if *signer
                        == verify_signature(
                            signature_commitment.commitment(PPRootVersion::V2),
                            signature,
                        )?
                    {
                        PPRootVersion::V2
                    } else {
                        return Err(ProofError::InvalidSignature);
                    }
                };

                match (base_pp_root_version, target_pp_root_version) {
                    // From V2 to V2: OK
                    (PPRootVersion::V2, PPRootVersion::V2) => {}
                    // From V3 to V3: OK
                    (PPRootVersion::V3, PPRootVersion::V3) => {}
                    // From V2 to V3: OK (migration)
                    (PPRootVersion::V2, PPRootVersion::V3) => {}
                    // Inconsistent signed payload.
                    _ => return Err(ProofError::InconsistentSignedPayload),
                }

                target_pp_root_version
            }
            #[cfg(not(target_os = "zkvm"))]
            AggchainData::Generic { .. } => {
                // NOTE: No stark verification in the native rust code due to
                // the sp1_zkvm::lib::verify::verify_sp1_proof syscall
                warn!("verify_sp1_proof is not callable outside of SP1");
                PPRootVersion::V3
            }
            #[cfg(target_os = "zkvm")]
            AggchainData::Generic {
                aggchain_vkey,
                aggchain_params,
            } => {
                let aggchain_proof_public_values = AggchainProofPublicValues {
                    prev_local_exit_root: multi_batch_header.prev_local_exit_root,
                    new_local_exit_root: multi_batch_header.target.exit_root,
                    l1_info_root: multi_batch_header.l1_info_root,
                    origin_network: multi_batch_header.origin_network,
                    aggchain_params: *aggchain_params,
                    commit_imported_bridge_exits: imported_hash,
                };

                sp1_zkvm::lib::verify::verify_sp1_proof(
                    aggchain_vkey,
                    &aggchain_proof_public_values.hash().into(),
                );

                PPRootVersion::V3
            }
        };

        Ok(target_pp_root_version)
    }
}
