use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{
    keccak::Digest,
    local_balance_tree::{BalanceTree, BalanceTreeByNetwork},
    local_exit_tree::{hasher::Keccak256Hasher, LocalExitTree},
    withdrawal::NetworkId,
    Withdrawal,
};

/// Represents the required data from each CDK for the pessimistic proof.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Batch {
    /// Origin network which emitted this batch
    pub origin_network: NetworkId,
    /// Initial local exit tree
    pub prev_local_exit_tree: LocalExitTree<Keccak256Hasher>,
    /// Initial local exit root
    pub prev_local_exit_root: Digest,
    /// Initial balance tree
    pub prev_local_balance_tree: BalanceTree,
    /// Set of withdrawals
    pub withdrawals: Vec<Withdrawal>,
}

impl Batch {
    /// Creates a new [`Batch`].
    pub fn new(
        origin_network: NetworkId,
        prev_local_exit_tree: LocalExitTree<Keccak256Hasher>,
        prev_local_exit_root: Digest,
        prev_local_balance_tree: BalanceTree,
        withdrawals: Vec<Withdrawal>,
    ) -> Self {
        Self {
            origin_network,
            prev_local_exit_tree,
            prev_local_exit_root,
            prev_local_balance_tree,
            withdrawals,
        }
    }

    /// Compute the new exit root.
    pub fn compute_new_exit_root(&self) -> Digest {
        let mut new_local_exit_tree = self.prev_local_exit_tree.clone();

        for withdrawal in &self.withdrawals {
            new_local_exit_tree.add_leaf(withdrawal.hash());
        }

        new_local_exit_tree.get_root()
    }

    /// Compute the new balance tree.
    pub fn compute_new_balance_tree(&self) -> BalanceTreeByNetwork {
        let mut aggregate: BalanceTreeByNetwork = {
            let base: BTreeMap<NetworkId, BalanceTree> =
                [(self.origin_network, self.prev_local_balance_tree.clone())].into();
            base.into()
        };

        for withdrawal in &self.withdrawals {
            aggregate.insert(self.origin_network, withdrawal.clone());
        }

        aggregate
    }
}
