use agglayer_primitives::Digest;
use agglayer_tries::roots::{LocalBalanceRoot, LocalExitRoot, LocalNullifierRoot};
use serde::{Deserialize, Serialize};
use unified_bridge::LocalExitTree;

use crate::{local_balance_tree::LocalBalanceTree, nullifier_tree::NullifierTree};

/// State representation of one network without the leaves, taken as input by
/// the prover.
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct LocalNetworkState {
    /// Commitment to the [`BridgeExit`](struct@crate::bridge_exit::BridgeExit).
    pub exit_tree: LocalExitTree,
    /// Commitment to the balance for each token.
    pub balance_tree: LocalBalanceTree,
    /// Commitment to the Nullifier tree for the local network, tracks claimed
    /// assets on foreign networks
    pub nullifier_tree: NullifierTree,
}

impl From<LocalNetworkState> for pessimistic_proof_core::NetworkState {
    fn from(state: LocalNetworkState) -> Self {
        pessimistic_proof_core::NetworkState {
            exit_tree: state.exit_tree,
            balance_tree: state.balance_tree.into(),
            nullifier_tree: state.nullifier_tree.into(),
        }
    }
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

impl From<StateCommitment> for pessimistic_proof_core::local_state::commitment::StateCommitment {
    fn from(commitment: StateCommitment) -> Self {
        Self {
            exit_root: LocalExitRoot::new(commitment.exit_root),
            ler_leaf_count: commitment.ler_leaf_count,
            balance_root: LocalBalanceRoot::new(commitment.balance_root),
            nullifier_root: LocalNullifierRoot::new(commitment.nullifier_root),
        }
    }
}
