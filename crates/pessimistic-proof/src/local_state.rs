use std::collections::{btree_map::Entry, BTreeMap};

use reth_primitives::{alloy_primitives::U512, ruint::UintTryFrom, B256, U256};
use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::L1_NETWORK_ID,
    imported_bridge_exit::{commit_imported_bridge_exits, Error},
    keccak::digest::Digest,
    local_balance_tree::LocalBalanceTree,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    multi_batch_header::{signature_commitment, MultiBatchHeader},
    nullifier_tree::{NullifierKey, NullifierTree},
    ProofError,
};

/// State representation of one network without the leaves, taken as input by
/// the prover.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LocalNetworkState {
    /// Commitment to the [`BridgeExit`].
    pub exit_tree: LocalExitTree<Keccak256Hasher>,
    /// Commitment to the balance for each token.
    pub balance_tree: LocalBalanceTree<Keccak256Hasher>,
    /// Commitment to the Nullifier tree for the local network, tracks claimed
    /// assets on foreign networks
    pub nullifier_tree: NullifierTree<Keccak256Hasher>,
}

/// The roots of one [`LocalNetworkState`].
#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct StateCommitment {
    pub exit_root: Digest,
    pub ler_leaf_count: u32,
    pub balance_root: Digest,
    pub nullifier_root: Digest,
}

impl StateCommitment {
    pub fn display_to_hex(&self) -> String {
        format!(
            "exit_root: {}, ler_leaf_count: {}, balance_root: {}, nullifier_root: {}",
            self.exit_root, self.ler_leaf_count, self.balance_root, self.nullifier_root,
        )
    }
}

impl LocalNetworkState {
    /// Returns the roots.
    pub fn roots(&self) -> StateCommitment {
        StateCommitment {
            exit_root: self.exit_tree.get_root(),
            ler_leaf_count: self.exit_tree.leaf_count,
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

        // Check batch_header.imported_exits_root
        let imported_exits_root = commit_imported_bridge_exits(
            multi_batch_header
                .imported_bridge_exits
                .iter()
                .map(|(exit, _)| exit),
        );

        if let Some(batch_imported_exits_root) = multi_batch_header.imported_exits_root {
            if imported_exits_root != batch_imported_exits_root {
                return Err(ProofError::InvalidImportedExitsRoot {
                    declared: batch_imported_exits_root,
                    computed: imported_exits_root,
                });
            }
        } else if !multi_batch_header.imported_bridge_exits.is_empty() {
            return Err(ProofError::MismatchImportedExitsRoot);
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
            if bridge_exit.token_info.origin_token_address.is_zero()
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

        // Verify that the signature is valid
        let combined_hash = signature_commitment(
            self.exit_tree.get_root(),
            multi_batch_header
                .imported_bridge_exits
                .iter()
                .map(|(exit, _)| exit),
        );

        // Check batch header signature
        let signer = multi_batch_header
            .signature
            .recover_signer(B256::new(combined_hash.0))
            .ok_or(ProofError::InvalidSignature)?;

        if signer != multi_batch_header.signer {
            return Err(ProofError::InvalidSigner {
                declared: multi_batch_header.signer,
                recovered: signer,
            });
        }

        Ok(self.roots())
    }
}
