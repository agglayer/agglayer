use std::collections::{btree_map::Entry, BTreeMap};

use reth_primitives::B256;
use serde::{Deserialize, Serialize};

use crate::{
    bridge_exit::{LeafType, L1_ETH, L1_NETWORK_ID},
    imported_bridge_exit::commit_imported_bridge_exits,
    keccak::keccak256_combine,
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
    /// The state isn't modified on error.
    pub fn apply_batch_header(
        &mut self,
        multi_batch_header: &MultiBatchHeader<Keccak256Hasher>,
    ) -> Result<(), ProofError> {
        let mut clone = self.clone();
        clone.apply_batch_header_helper(multi_batch_header)?;
        *self = clone;

        Ok(())
    }

    /// Apply the [`MultiBatchHeader`] on the current [`State`].
    /// The state can be modified on error.
    fn apply_batch_header_helper(
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

        // Check batch_header.imported_exits_root
        let imported_exits_root = commit_imported_bridge_exits(
            multi_batch_header.imported_bridge_exits.iter().map(|(exit, _)| exit),
        );
        if let Some(batch_imported_exits_root) = multi_batch_header.imported_exits_root {
            if imported_exits_root != batch_imported_exits_root {
                return Err(ProofError::InvalidImportedExitsRoot);
            }
        } else if !multi_batch_header.imported_bridge_exits.is_empty() {
            return Err(ProofError::InvalidImportedExitsRoot);
        }

        let combined_hash = keccak256_combine([
            multi_batch_header.new_local_exit_root.as_slice(),
            imported_exits_root.as_slice(),
        ]);

        // Check batch header signature
        let signer = multi_batch_header
            .signature
            .recover_signer(B256::new(combined_hash))
            .ok_or(ProofError::InvalidSignature)?;
        if signer != multi_batch_header.signer {
            return Err(ProofError::InvalidSignature);
        }

        // Apply the imported bridge exits
        for (imported_bridge_exit, nullifier_path) in &multi_batch_header.imported_bridge_exits {
            if imported_bridge_exit.global_index.network_id() == multi_batch_header.origin_network {
                // We don't allow a chain to exit to itself
                return Err(ProofError::ExitToSameNetwork);
            }
            // Check that the destination network of the bridge exit matches the current network
            if imported_bridge_exit.bridge_exit.dest_network != multi_batch_header.origin_network {
                return Err(ProofError::InvalidImportedBridgeExitNetwork);
            }

            // Check the inclusion proof
            imported_bridge_exit.verify_path(
                multi_batch_header.imported_mainnet_exit_root,
                multi_batch_header.imported_rollup_exit_root,
            )?;

            // Check the nullifier non-inclusion path and update the nullifier set
            let nullifier_key = NullifierKey {
                network_id: imported_bridge_exit.global_index.network_id(),
                let_index: imported_bridge_exit.global_index.leaf_index,
            };
            self.nullifier_set.verify_and_update(nullifier_key, nullifier_path)?;

            // The amount corresponds to L1 ETH if the leaf is a message
            let token_info = match imported_bridge_exit.bridge_exit.leaf_type {
                LeafType::Message => L1_ETH,
                _ => imported_bridge_exit.bridge_exit.token_info,
            };

            if multi_batch_header.origin_network == token_info.origin_network {
                // When the token is native to the chain, we don't care about the local balance
                continue;
            }

            // Update the token balance.
            let amount = imported_bridge_exit.bridge_exit.amount;
            let entry = new_balances.entry(token_info);
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
            if bridge_exit.dest_network == multi_batch_header.origin_network {
                // We don't allow a chain to exit to itself
                return Err(ProofError::ExitToSameNetwork);
            }
            self.exit_tree.add_leaf(bridge_exit.hash());

            // For message exits, the origin network in token info should be the origin network
            // of the batch header.
            if bridge_exit.is_message()
                && bridge_exit.token_info.origin_network != multi_batch_header.origin_network
            {
                return Err(ProofError::InvalidMessageOriginNetwork);
            }

            // For ETH transfers, we need to check that the origin network is the L1 network
            if bridge_exit.token_info.origin_token_address.is_zero()
                && bridge_exit.token_info.origin_network != L1_NETWORK_ID
            {
                return Err(ProofError::InvalidEthNetwork);
            }

            // The amount corresponds to L1 ETH if the leaf is a message
            let token_info = match bridge_exit.leaf_type {
                LeafType::Message => L1_ETH,
                _ => bridge_exit.token_info,
            };

            if multi_batch_header.origin_network == token_info.origin_network {
                // When the token is native to the chain, we don't care about the local balance
                continue;
            }

            // Update the token balance.
            let amount = bridge_exit.amount;
            let entry = new_balances.entry(token_info);
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
