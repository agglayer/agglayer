use std::collections::{btree_map::Entry, BTreeMap};

use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::LeafType,
    local_balance_tree::LocalBalanceTree,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierTree},
    LeafProofOutput, ProofError,
};

/// Local state of one network.
/// The AggLayer tracks the [`LocalNetworkState`] for all networks.
/// Eventually, this state will be entirely tracked by the networks themselves.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LocalNetworkState {
    /// Commitment to the [`BridgeExit`].
    pub exit_tree: LocalExitTree<Keccak256Hasher>,
    /// Commitment to the balance for each token.
    pub balance_tree: LocalBalanceTree<Keccak256Hasher>,
    /// Commitment to the Nullifier Set for the local network, tracks claimed assets on foreign networks
    pub nullifier_set: NullifierTree<Keccak256Hasher>,
}

impl LocalNetworkState {
    pub fn roots(&self) -> LeafProofOutput {
        (self.exit_tree.get_root(), self.balance_tree.root, self.nullifier_set.root)
    }

    /// Apply the [`MultiBatchHeader`] on the current [`State`].
    /// Returns the commitment on the resulting state if successful.
    pub fn apply_batch_header(
        &mut self,
        multi_batch_header: &MultiBatchHeader<Keccak256Hasher>,
    ) -> Result<(), ProofError> {
        // Check the initial state
        let computed_root = self.exit_tree.get_root();
        if computed_root != multi_batch_header.prev_local_exit_root {
            return Err(ProofError::InvalidInitialLocalExitRoot {
                got: computed_root,
                expected: multi_batch_header.prev_local_exit_root,
            });
        }
        if self.balance_tree.root != multi_batch_header.prev_balance_root {
            return Err(ProofError::InvalidInitialBalanceRoot);
        }
        if self.nullifier_set.root != multi_batch_header.prev_nullifier_root {
            return Err(ProofError::InvalidInitialNullifierRoot);
        }

        // TODO: benchmark if BTreeMap is the best choice in terms of SP1 cycles
        let mut new_balances: BTreeMap<_, _> =
            multi_batch_header.balances_proofs.iter().map(|(k, v)| (*k, v.0)).collect();

        // TODO: check batch_header.imported_exits_root
        // Apply the imported bridge exits
        for (imported_bridge_exit, nullifier_path) in &multi_batch_header.imported_bridge_exits {
            if matches!(imported_bridge_exit.bridge_exit.leaf_type, LeafType::Message) {
                // TODO: handle bridge messages
                panic!()
            }

            if imported_bridge_exit.sending_network == multi_batch_header.origin_network {
                // We don't allow a chain to exit to itself
                return Err(ProofError::ExitToSameNetwork);
            }
            // Check the LER
            if multi_batch_header.imported_local_exit_roots[&imported_bridge_exit.sending_network]
                != imported_bridge_exit.imported_local_exit_root
            {
                return Err(ProofError::InvalidImportedBridgeExitRoot);
            }
            // Check the LER inclusion path
            if !imported_bridge_exit.verify_path() {
                return Err(ProofError::InvalidImportedBridgeExitMerklePath);
            }

            // Check the nullifier non-inclusion path and update the nullifier set
            let nullifier_key = NullifierKey {
                network_id: imported_bridge_exit.sending_network,
                let_index: imported_bridge_exit.leaf_index,
            };
            self.nullifier_set.verify_and_update(nullifier_key, nullifier_path)?;

            if multi_batch_header.origin_network
                == imported_bridge_exit.bridge_exit.token_info.origin_network
            {
                // When the token is native to the chain, we don't care about the local balance
                continue;
            }

            // Update the token balance.
            let amount = imported_bridge_exit.bridge_exit.amount;
            let entry = new_balances.entry(imported_bridge_exit.bridge_exit.token_info);
            match entry {
                Entry::Vacant(_) => return Err(ProofError::MissingTokenBalanceProof),
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = entry
                        .get()
                        .checked_add(amount)
                        .ok_or(ProofError::BalanceOverflowInBridgeExit)?;
                }
            }
        }

        // Apply the bridge exits
        for bridge_exit in &multi_batch_header.bridge_exits {
            if matches!(bridge_exit.leaf_type, LeafType::Message) {
                // TODO: handle bridge messages
                panic!()
            }

            if bridge_exit.dest_network == multi_batch_header.origin_network {
                // We don't allow a chain to exit to itself
                return Err(ProofError::ExitToSameNetwork);
            }
            self.exit_tree.add_leaf(bridge_exit.hash());
            if multi_batch_header.origin_network == bridge_exit.token_info.origin_network {
                // When the token is native to the chain, we don't care about the local balance
                continue;
            }

            // Update the token balance.
            let amount = bridge_exit.amount;
            let entry = new_balances.entry(bridge_exit.token_info);
            match entry {
                Entry::Vacant(_) => return Err(ProofError::MissingTokenBalanceProof),
                Entry::Occupied(mut entry) => {
                    *entry.get_mut() = entry
                        .get()
                        .checked_sub(amount)
                        .ok_or(ProofError::BalanceUnderflowInBridgeExit)?;
                }
            }
        }

        // Verify that the original balances were correct and update the local balance tree with the new balances.
        // TODO: implement batch `verify_and_update` for the LBT
        for (token, (old_balance, balance_path)) in &multi_batch_header.balances_proofs {
            let new_balance = new_balances[token];
            self.balance_tree
                .verify_and_update(*token, balance_path, *old_balance, new_balance)?;
        }

        if self.exit_tree.get_root() != multi_batch_header.new_local_exit_root {
            return Err(ProofError::InvalidFinalLocalExitRoot);
        }
        if self.balance_tree.root != multi_batch_header.new_balance_root {
            return Err(ProofError::InvalidFinalBalanceRoot);
        }
        if self.nullifier_set.root != multi_batch_header.new_nullifier_root {
            return Err(ProofError::InvalidFinalNullifierRoot);
        }

        Ok(())
    }
}
