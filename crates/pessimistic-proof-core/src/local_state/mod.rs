// Removed BTreeMap import - now using Vec for better SP1 performance

use agglayer_primitives::{keccak::Keccak256Hasher, ruint::UintTryFrom, Hashable, U256, U512};
use agglayer_tries::roots::{LocalBalanceRoot, LocalNullifierRoot};
use commitment::StateCommitment;
use serde::{Deserialize, Serialize};
use unified_bridge::{Error, LocalExitTree, NetworkId, L1_ETH};

use crate::{
    local_balance_tree::LocalBalanceTree,
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierTree},
    ProofError,
};

pub mod commitment;

/// State representation of one network without the leaves, taken as input by
/// the prover.
#[derive(
    Clone, Debug, Serialize, Deserialize, rkyv::Archive, rkyv::Serialize, rkyv::Deserialize,
)]
#[repr(C)]
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
    pub fn get_state_commitment(&self) -> StateCommitment {
        StateCommitment {
            exit_root: self.exit_tree.get_root().into(),
            ler_leaf_count: self.exit_tree.leaf_count,
            balance_root: LocalBalanceRoot::new(self.balance_tree.root),
            nullifier_root: LocalNullifierRoot::new(self.nullifier_tree.root),
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
        // OPTIMIZATION: Use Vec instead of BTreeMap for better SP1 cycle performance
        // Since we only need insertion, lookup, and final access operations,
        // Vec with linear search is more efficient in the ZK environment
        let mut new_balances = Vec::with_capacity(multi_batch_header.balances_proofs.len());

        // Pre-populate with initial balances and check for duplicates
        for (token, (balance, _)) in &multi_batch_header.balances_proofs {
            // Check for duplicates
            if new_balances
                .iter()
                .any(|(existing_token, _)| existing_token == token)
            {
                return Err(ProofError::DuplicateTokenBalanceProof(*token));
            }
            new_balances.push((*token, U512::from(U256::from_limbs(balance.limbs))));
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

            // Update the token balance using linear search
            let amount = imported_bridge_exit.bridge_exit.amount;
            let mut found = false;
            for (token, balance) in &mut new_balances {
                if *token == token_info {
                    *balance = balance
                        .checked_add(U512::from(amount))
                        .ok_or(ProofError::BalanceOverflowInBridgeExit)?;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(ProofError::MissingTokenBalanceProof(token_info));
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
                && bridge_exit.token_info.origin_network != NetworkId::ETH_L1
            {
                return Err(ProofError::InvalidL1TokenInfo(bridge_exit.token_info));
            }

            // The amount corresponds to L1 ETH if the leaf is a message
            let token_info = bridge_exit.amount_token_info();

            if multi_batch_header.origin_network == token_info.origin_network {
                // When the token is native to the chain, we don't care about the local balance
                continue;
            }

            // Update the token balance using linear search
            let amount = bridge_exit.amount;
            let mut found = false;
            for (token, balance) in &mut new_balances {
                if *token == token_info {
                    *balance = balance
                        .checked_sub(U512::from(amount))
                        .ok_or(ProofError::BalanceUnderflowInBridgeExit)?;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(ProofError::MissingTokenBalanceProof(token_info));
            }
        }

        // Verify that the original balances were correct and update the local balance
        // tree with the new balances
        for (token, (old_balance, balance_path)) in &multi_batch_header.balances_proofs {
            // Find the new balance using linear search
            let new_balance = new_balances
                .iter()
                .find(|(t, _)| t == token)
                .map(|(_, balance)| *balance)
                .unwrap_or(U512::from(U256::from_limbs(old_balance.limbs))); // Fallback to old balance if not found

            let new_balance = U256::uint_try_from(new_balance)
                .map_err(|_| ProofError::BalanceOverflowInBridgeExit)?;
            self.balance_tree.verify_and_update(
                *token,
                balance_path,
                U256::from_limbs(old_balance.limbs),
                new_balance,
            )?;
        }

        Ok(self.get_state_commitment())
    }
}
