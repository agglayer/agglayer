use agglayer_primitives::{ruint::UintTryFrom, Hashable, U256, U512};
use agglayer_tries::roots::{LocalBalanceRoot, LocalNullifierRoot};
use bytemuck::{Pod, Zeroable};
use commitment::StateCommitment;
use serde::{Deserialize, Serialize};
use unified_bridge::{Error, LocalExitTree, NetworkId, TokenInfo, L1_ETH};

use crate::{
    local_balance_tree::LocalBalanceTree,
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{NullifierKey, NullifierTree},
    ProofError,
};

pub mod commitment;

/// Zero-copy representation of NetworkState for safe transmute.
/// This struct has a stable C-compatible memory layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Pod, Zeroable)]
pub struct NetworkStateZeroCopy {
    /// Leaf count of the exit tree (u32)
    pub exit_tree_leaf_count: u32,
    /// Frontier of the exit tree (32 * 32 = 1024 bytes)
    pub exit_tree_frontier: [[u8; 32]; 32],
    /// Root of the balance tree (32 bytes)
    pub balance_tree_root: [u8; 32],
    /// Root of the nullifier tree (32 bytes)
    pub nullifier_tree_root: [u8; 32],
}

impl From<&NetworkState> for NetworkStateZeroCopy {
    fn from(state: &NetworkState) -> Self {
        Self {
            exit_tree_leaf_count: state.exit_tree.leaf_count,
            exit_tree_frontier: state.exit_tree.frontier().map(|h| *h.as_bytes()),
            balance_tree_root: *state.balance_tree.root.as_bytes(),
            nullifier_tree_root: *state.nullifier_tree.root.as_bytes(),
        }
    }
}

impl From<&NetworkStateZeroCopy> for NetworkState {
    fn from(zero_copy: &NetworkStateZeroCopy) -> Self {
        let exit_tree = LocalExitTree::from_parts(
            zero_copy.exit_tree_leaf_count,
            zero_copy
                .exit_tree_frontier
                .map(agglayer_primitives::Digest::from),
        );

        let balance_tree = LocalBalanceTree {
            root: agglayer_primitives::Digest::from(zero_copy.balance_tree_root),
        };

        let nullifier_tree = NullifierTree {
            root: agglayer_primitives::Digest::from(zero_copy.nullifier_tree_root),
        };

        NetworkState {
            exit_tree,
            balance_tree,
            nullifier_tree,
        }
    }
}

impl NetworkStateZeroCopy {
    /// Safely deserialize from bytes using bytemuck.
    pub fn from_bytes(data: &[u8]) -> Result<&Self, bytemuck::PodCastError> {
        if data.len() != std::mem::size_of::<Self>() {
            return Err(bytemuck::PodCastError::SizeMismatch);
        }
        bytemuck::try_from_bytes(data)
    }

    /// Convert this struct to a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        bytemuck::bytes_of(self)
    }

    /// Convert this struct to a owned byte vector.
    pub fn to_bytes(&self) -> Vec<u8> {
        self.as_bytes().to_vec()
    }
}

/// State representation of one network without the leaves, taken as input by
/// the prover.
#[repr(C)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkState {
    /// Commitment to the [`BridgeExit`](struct@crate::bridge_exit::BridgeExit).
    pub exit_tree: LocalExitTree,
    /// Commitment to the balance for each token.
    pub balance_tree: LocalBalanceTree,
    /// Commitment to the Nullifier tree for the local network, tracks claimed
    /// assets on foreign networks
    pub nullifier_tree: NullifierTree,
}

impl TryFrom<&[u8]> for NetworkState {
    type Error = bytemuck::PodCastError;

    fn try_from(data: &[u8]) -> Result<Self, Self::Error> {
        let zero_copy = NetworkStateZeroCopy::from_bytes(data)?;
        Ok(Self::from(zero_copy))
    }
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

    /// Serialize to zero-copy bytes.
    /// This creates a byte representation that can be safely transmuted back.
    pub fn to_bytes_zero_copy(&self) -> Vec<u8> {
        NetworkStateZeroCopy::from(self).to_bytes()
    }

    /// Apply the [`MultiBatchHeader`] on the current [`LocalNetworkState`].
    /// Checks that the transition reaches the target [`StateCommitment`].
    /// The state isn't modified on error.
    pub fn apply_batch_header(
        &mut self,
        multi_batch_header: &MultiBatchHeader,
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
        multi_batch_header: &MultiBatchHeader,
    ) -> Result<StateCommitment, ProofError> {
        // Convert balances_proofs to sorted Vec for efficient binary search
        // This is more efficient than BTreeMap for SP1 cycles
        let mut new_balances: Vec<(TokenInfo, U512)> = multi_batch_header
            .balances_proofs
            .iter()
            .map(|(k, v)| (*k, U512::from(v.0)))
            .collect();

        // Sort by TokenInfo for binary search (TokenInfo has PartialOrd)
        new_balances.sort_by(|a, b| a.0.cmp(&b.0));

        // Check for duplicates during sorting
        for i in 1..new_balances.len() {
            if new_balances[i].0 == new_balances[i - 1].0 {
                return Err(ProofError::DuplicateTokenBalanceProof(new_balances[i].0));
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

            // Update the token balance using binary search
            let amount = imported_bridge_exit.bridge_exit.amount;
            match new_balances.binary_search_by(|(k, _)| k.cmp(&token_info)) {
                Ok(index) => {
                    // Found the token, update balance
                    new_balances[index].1 = new_balances[index]
                        .1
                        .checked_add(U512::from(amount))
                        .ok_or(ProofError::BalanceOverflowInBridgeExit)?;
                }
                Err(_) => return Err(ProofError::MissingTokenBalanceProof(token_info)),
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

            // Update the token balance using binary search
            let amount = bridge_exit.amount;
            match new_balances.binary_search_by(|(k, _)| k.cmp(&token_info)) {
                Ok(index) => {
                    // Found the token, update balance (subtract for bridge exits)
                    new_balances[index].1 = new_balances[index]
                        .1
                        .checked_sub(U512::from(amount))
                        .ok_or(ProofError::BalanceUnderflowInBridgeExit)?;
                }
                Err(_) => return Err(ProofError::MissingTokenBalanceProof(token_info)),
            }
        }

        // Verify that the original balances were correct and update the local balance
        // tree with the new balances. TODO: implement batch `verify_and_update`
        // for the LBT
        for (token, (old_balance, balance_path)) in &multi_batch_header.balances_proofs {
            let new_balance = new_balances
                .binary_search_by(|(k, _)| k.cmp(token))
                .map(|index| new_balances[index].1)
                .map_err(|_| ProofError::MissingTokenBalanceProof(*token))?;
            let new_balance = U256::uint_try_from(new_balance)
                .map_err(|_| ProofError::BalanceOverflowInBridgeExit)?;
            self.balance_tree
                .verify_and_update(*token, balance_path, *old_balance, new_balance)?;
        }

        Ok(self.get_state_commitment())
    }
}

#[cfg(test)]
mod tests {
    use unified_bridge::LocalExitTree;

    use super::*;

    #[test]
    fn test_zero_copy_roundtrip() {
        let state = NetworkState {
            exit_tree: LocalExitTree::from_parts(42, [[1u8; 32].into(); 32]),
            balance_tree: LocalBalanceTree {
                root: [0xAAu8; 32].into(),
            },
            nullifier_tree: NullifierTree {
                root: [0xBBu8; 32].into(),
            },
        };

        let bytes = state.to_bytes_zero_copy();
        assert_eq!(bytes.len(), std::mem::size_of::<NetworkStateZeroCopy>());

        let deserialized = NetworkState::try_from(bytes.as_slice())
            .expect("Zero-copy deserialization should succeed");

        assert_eq!(
            state.exit_tree.leaf_count,
            deserialized.exit_tree.leaf_count
        );
        assert_eq!(state.balance_tree.root, deserialized.balance_tree.root);
        assert_eq!(state.nullifier_tree.root, deserialized.nullifier_tree.root);
    }

    #[test]
    fn test_zero_copy_invalid_input() {
        let state = NetworkState {
            exit_tree: LocalExitTree::new(),
            balance_tree: LocalBalanceTree {
                root: [1u8; 32].into(),
            },
            nullifier_tree: NullifierTree {
                root: [2u8; 32].into(),
            },
        };
        let bytes = state.to_bytes_zero_copy();

        // Test unaligned data
        let unaligned = &bytes[1..];
        assert!(NetworkState::try_from(unaligned).is_err());

        // Test wrong size (too small)
        let too_small = &bytes[..bytes.len() - 1];
        assert!(NetworkState::try_from(too_small).is_err());

        // Test wrong size (too large)
        let mut too_large = bytes.clone();
        too_large.push(0);
        assert!(NetworkState::try_from(too_large.as_slice()).is_err());

        // Test empty data
        assert!(NetworkState::try_from([].as_slice()).is_err());
    }
}
