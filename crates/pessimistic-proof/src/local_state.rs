use serde::{Deserialize, Serialize};

use crate::{
    batch_header::BatchHeader,
    local_balance_tree::BalanceTree,
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    nullifier_set::{NullifierSet, NetworkNullifierSet},
    proof::{BalanceRoot, ExitRoot},
    ProofError,
};

/// Local state of one network.
/// The AggLayer tracks the [`LocalNetworkState`] for all networks.
/// Eventually, this state will be entirely tracked by the networks themselves.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocalNetworkState {
    /// Commitment to the [`BridgeExit`].
    pub exit_tree: LocalExitTree<Keccak256Hasher>,
    /// Commitment to the balance for each token.
    pub balance_tree: BalanceTree,
    /// Commitment to the Nullifier Set for the local network, tracks claimed assets on foreign networks
    pub nullifier_set: NullifierSet,
}

impl LocalNetworkState {
    /// Apply the [`BatchHeader`] on the current [`State`].
    /// Returns the commitment on the resulting state if successful.
    pub fn apply_batch_header(
        &mut self,
        batch_header: &BatchHeader,
    ) -> Result<(ExitRoot, BalanceRoot), ProofError> {
        // Check the initial state
        let computed_root = self.exit_tree.get_root();
        if computed_root != batch_header.prev_local_exit_root {
            return Err(ProofError::InvalidLocalExitRoot {
                got: computed_root,
                expected: batch_header.prev_local_exit_root,
            });
        }

        // Apply the bridge exits
        batch_header.bridge_exits.iter().for_each(|bridge_exit| {
            self.exit_tree.add_leaf(bridge_exit.hash());
            self.balance_tree.withdraw(bridge_exit.token_info, bridge_exit.amount);
        });

        // Apply the imported bridge exits
        // TODO: omitting three required checks:
        // 1: that each imported bridge exit is valid according to the imported local exit roots,
        // 2: each imported bridge exit has not been claimed in the nullifier set,
        // 3: each imported_local_exit_root in an imported_bridge_exit is contained in the imported_lers set in the batch_header
        if let Some(imported_bridge_exits) = &batch_header.imported_bridge_exits {
            imported_bridge_exits.iter().for_each(|imported_bridge_exit| {
                // TODO: check provided inclusion path in the imported_bridge_exit
                // TODO: check that the LER for the imported bridge exit is contained in the batch header
                // TODO: update nullifier set
                self.balance_tree.deposit(imported_bridge_exit.bridge_exit.token_info, imported_bridge_exit.bridge_exit.amount);
            })
        }

        // Check whether the origin network has some debt
        if self.balance_tree.has_debt() {
            return Err(ProofError::HasDebt {
                network: batch_header.origin_network,
            });
        }

        Ok((self.exit_tree.get_root(), self.balance_tree.hash()))
    }
}
