use std::collections::{btree_map::Entry, BTreeMap};

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

/// Zero-copy representation of NetworkState for safe transmute.
/// This struct has a stable C-compatible memory layout.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NetworkStateZeroCopy {
    /// Leaf count of the exit tree (u32)
    pub exit_tree_leaf_count: u32,
    /// Frontier of the exit tree (32 * 32 = 1024 bytes)
    pub exit_tree_frontier: [[u8; 32]; 32],
    /// Root of the balance tree (32 bytes)
    pub balance_tree_root: [u8; 32],
    /// Root of the nullifier tree (32 bytes)
    pub nullifier_tree_root: [u8; 32],
    /// Empty hash array for nullifier tree (64 * 32 = 2048 bytes)
    pub nullifier_empty_hash_at_height: [[u8; 32]; 64],
}

impl NetworkStateZeroCopy {
    /// Create a zero-copy representation from a regular NetworkState.
    pub fn from_network_state(state: &NetworkState) -> Self {
        Self {
            exit_tree_leaf_count: state.exit_tree.leaf_count,
            exit_tree_frontier: state.exit_tree.frontier().map(|h| *h.as_bytes()),
            balance_tree_root: *state.balance_tree.root.as_bytes(),
            nullifier_tree_root: *state.nullifier_tree.root.as_bytes(),
            nullifier_empty_hash_at_height: state
                .nullifier_tree
                .empty_hash_at_height
                .map(|h| *h.as_bytes()),
        }
    }

    /// Convert back to a regular NetworkState.
    pub fn to_network_state(&self) -> NetworkState {
        // Reconstruct the exit tree with frontier
        let exit_tree = LocalExitTree::from_parts(
            self.exit_tree_leaf_count,
            self.exit_tree_frontier
                .map(|h| agglayer_primitives::Digest::from(h)),
        );

        // Reconstruct the balance tree
        let balance_tree = LocalBalanceTree::<Keccak256Hasher> {
            root: agglayer_primitives::Digest::from(self.balance_tree_root),
        };

        // Reconstruct the nullifier tree
        let nullifier_tree = NullifierTree::<Keccak256Hasher> {
            root: agglayer_primitives::Digest::from(self.nullifier_tree_root),
            empty_hash_at_height: self
                .nullifier_empty_hash_at_height
                .map(|h| agglayer_primitives::Digest::from(h)),
        };

        NetworkState {
            exit_tree,
            balance_tree,
            nullifier_tree,
        }
    }

    /// Get the size of this struct in bytes.
    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }

    /// Check if the given byte slice has the correct size for this struct.
    pub fn check_size(data: &[u8]) -> bool {
        data.len() == Self::size()
    }

    /// Safely transmute a byte slice to this struct.
    /// This is only safe if the data was originally a NetworkStateZeroCopy.
    pub unsafe fn from_bytes(data: &[u8]) -> Option<&Self> {
        let align = std::mem::align_of::<Self>();
        let ptr = data.as_ptr() as usize;
        if Self::check_size(data) && (ptr % align == 0) {
            // SAFETY: We've checked size and alignment, and assume the caller ensures
            // the data was originally a NetworkStateZeroCopy
            Some(&*(data.as_ptr() as *const Self))
        } else {
            None
        }
    }

    /// Convert this struct to a byte slice.
    pub fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self as *const Self as *const u8, Self::size()) }
    }
}

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
    pub fn get_state_commitment(&self) -> StateCommitment {
        StateCommitment {
            exit_root: self.exit_tree.get_root().into(),
            ler_leaf_count: self.exit_tree.leaf_count,
            balance_root: LocalBalanceRoot::new(self.balance_tree.root),
            nullifier_root: LocalNullifierRoot::new(self.nullifier_tree.root),
        }
    }

    /// Convert to zero-copy representation for safe transmute.
    pub fn to_zero_copy(&self) -> NetworkStateZeroCopy {
        NetworkStateZeroCopy::from_network_state(self)
    }

    /// Create from zero-copy representation.
    pub fn from_zero_copy(zero_copy: &NetworkStateZeroCopy) -> Self {
        zero_copy.to_network_state()
    }

    /// Serialize the NetworkState to a vector of bytes for zero-copy reading.
    /// This is compatible with `sp1_zkvm::io::read_vec`.
    ///
    /// # Example
    /// ```rust
    /// use agglayer_primitives::keccak::Keccak256Hasher;
    /// use pessimistic_proof_core::{
    ///     local_balance_tree::LocalBalanceTree, local_state::NetworkState,
    ///     nullifier_tree::NullifierTree,
    /// };
    /// use unified_bridge::LocalExitTree;
    ///
    /// let network_state = NetworkState {
    ///     exit_tree: LocalExitTree::new(),
    ///     balance_tree: LocalBalanceTree::<Keccak256Hasher> {
    ///         root: [0u8; 32].into(),
    ///     },
    ///     nullifier_tree: NullifierTree::<Keccak256Hasher> {
    ///         root: [0u8; 32].into(),
    ///         empty_hash_at_height: [[0u8; 32].into(); 64],
    ///     },
    /// };
    ///
    /// let serialized = network_state.to_vec().expect("Serialization failed");
    /// // Use with sp1_zkvm::io::read_vec in your SP1 program
    /// ```
    pub fn to_vec(&self) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        use agglayer_bincode as bincode;
        bincode::contracts().serialize(self).map_err(|e| e.into())
    }

    /// Deserialize the NetworkState from a vector of bytes.
    /// This is compatible with `sp1_zkvm::io::read_vec`.
    ///
    /// # Example
    /// ```rust
    /// #![no_main]
    /// use pessimistic_proof_core::local_state::NetworkState;
    /// use sp1_zkvm::entrypoint;
    ///
    /// sp1_zkvm::entrypoint!(main);
    ///
    /// pub fn main() {
    ///     // Read the serialized NetworkState from SP1 input
    ///     let serialized_data = sp1_zkvm::io::read_vec();
    ///
    ///     // Deserialize the NetworkState
    ///     let network_state =
    ///         NetworkState::from_vec(&serialized_data).expect("Deserialization failed");
    ///
    ///     // Use the network_state in your proof generation
    /// }
    /// ```
    pub fn from_vec(data: &[u8]) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        use agglayer_bincode as bincode;
        bincode::contracts().deserialize(data).map_err(|e| e.into())
    }

    /// Zero-copy deserialization from bytes.
    /// This is the true zero-copy approach using transmute.
    ///
    /// # Safety
    /// This function is unsafe because it assumes the input data was originally
    /// a `NetworkStateZeroCopy` struct. The caller must ensure this invariant.
    ///
    /// # Example
    /// ```rust
    /// #![no_main]
    /// use pessimistic_proof_core::local_state::NetworkState;
    /// use sp1_zkvm::entrypoint;
    ///
    /// sp1_zkvm::entrypoint!(main);
    ///
    /// pub fn main() {
    ///     // Read the raw bytes from SP1 input
    ///     let raw_data = sp1_zkvm::io::read_vec();
    ///
    ///     // Zero-copy deserialization (unsafe but efficient)
    ///     let network_state =
    ///         unsafe { NetworkState::from_bytes_zero_copy(&raw_data) }.expect("Invalid data size");
    ///
    ///     // Use the network_state in your proof generation
    /// }
    /// ```
    pub unsafe fn from_bytes_zero_copy(data: &[u8]) -> Option<Self> {
        NetworkStateZeroCopy::from_bytes(data).map(|zc| Self::from_zero_copy(zc))
    }

    /// Serialize to zero-copy bytes.
    /// This creates a byte representation that can be safely transmuted back.
    pub fn to_bytes_zero_copy(&self) -> Vec<u8> {
        let zero_copy = self.to_zero_copy();
        zero_copy.as_bytes().to_vec()
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

        Ok(self.get_state_commitment())
    }
}

#[cfg(test)]
mod tests {
    use agglayer_primitives::keccak::Keccak256Hasher;
    use unified_bridge::LocalExitTree;

    use super::*;

    #[test]
    fn test_zero_copy_roundtrip() {
        let state = NetworkState {
            exit_tree: LocalExitTree::from_parts(42, [[1u8; 32].into(); 32]),
            balance_tree: LocalBalanceTree::<Keccak256Hasher> {
                root: [0xAAu8; 32].into(),
            },
            nullifier_tree: NullifierTree::<Keccak256Hasher> {
                root: [0xBBu8; 32].into(),
                empty_hash_at_height: [[0xCCu8; 32].into(); 64],
            },
        };

        let bytes = state.to_bytes_zero_copy();
        assert_eq!(bytes.len(), NetworkStateZeroCopy::size());

        let deserialized = unsafe { NetworkState::from_bytes_zero_copy(&bytes) }
            .expect("Zero-copy deserialization should succeed");

        assert_eq!(state.exit_tree.leaf_count, deserialized.exit_tree.leaf_count);
        assert_eq!(state.balance_tree.root, deserialized.balance_tree.root);
        assert_eq!(state.nullifier_tree.root, deserialized.nullifier_tree.root);
    }

    #[test]
    fn test_zero_copy_invalid_input() {
        let state = NetworkState {
            exit_tree: LocalExitTree::new(),
            balance_tree: LocalBalanceTree::<Keccak256Hasher> {
                root: [1u8; 32].into(),
            },
            nullifier_tree: NullifierTree::<Keccak256Hasher> {
                root: [2u8; 32].into(),
                empty_hash_at_height: [[3u8; 32].into(); 64],
            },
        };
        let bytes = state.to_bytes_zero_copy();

        // Test unaligned data
        let unaligned = &bytes[1..];
        assert!(unsafe { NetworkState::from_bytes_zero_copy(unaligned) }.is_none());

        // Test wrong size
        let too_small = &bytes[..bytes.len() - 1];
        assert!(unsafe { NetworkState::from_bytes_zero_copy(too_small) }.is_none());

        // Test empty data
        assert!(unsafe { NetworkState::from_bytes_zero_copy(&[]) }.is_none());
    }
}
