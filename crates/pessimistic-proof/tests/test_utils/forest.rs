use std::collections::{BTreeMap, BTreeSet};

use pessimistic_proof::{
    bridge_exit::{BridgeExit, LeafType, NetworkId, TokenInfo},
    imported_bridge_exit::ImportedBridgeExit,
    keccak::Digest,
    local_balance_tree::{LocalBalancePath, LocalBalanceTree, LOCAL_BALANCE_TREE_DEPTH},
    local_exit_tree::{data::LocalExitTreeData, hasher::Keccak256Hasher, LocalExitTree},
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{FromBool, NullifierKey, NullifierPath, NullifierTree, NULLIFIER_TREE_DEPTH},
    utils::smt::Smt,
    LocalNetworkState,
};
use rand::random;
use reth_primitives::U256;

use super::sample_data::{NETWORK_A, NETWORK_B};

/// Trees for the network B, as well as the LET for network A.
pub struct Forest {
    pub local_exit_tree_data_a: LocalExitTreeData<Keccak256Hasher>,
    pub local_exit_tree: LocalExitTree<Keccak256Hasher>,
    pub local_balance_tree: Smt<Keccak256Hasher, LOCAL_BALANCE_TREE_DEPTH>,
    pub nullifier_set: Smt<Keccak256Hasher, NULLIFIER_TREE_DEPTH>,
}

impl Forest {
    /// Create a new forest based on given initial balances.
    pub fn new(initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>) -> Self {
        Self::new_with_local_exit_tree(initial_balances, LocalExitTree::new())
    }

    /// Override the local exit tree for network B
    pub fn new_with_local_exit_tree(
        initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>,
        local_exit_tree: LocalExitTree<Keccak256Hasher>,
    ) -> Self {
        let mut local_balance_tree = Smt::new();
        for (token, balance) in initial_balances {
            local_balance_tree.insert(token, balance.to_be_bytes()).unwrap();
        }

        Self {
            local_exit_tree_data_a: LocalExitTreeData::new(),
            local_exit_tree,
            local_balance_tree,
            nullifier_set: Smt::new(),
        }
    }

    /// Imported bridge exits from network A to network B.
    pub fn imported_bridge_exits(
        &mut self,
        events: &[(TokenInfo, U256)],
    ) -> Vec<(ImportedBridgeExit, NullifierPath<Keccak256Hasher>)> {
        let mut res = Vec::new();
        for (token, amount) in events {
            let exit = exit_to_b(*token, *amount);
            let index = self.local_exit_tree_data_a.add_leaf(exit.hash());
            let proof = self.local_exit_tree_data_a.get_proof(index);
            let imported_exit = ImportedBridgeExit {
                bridge_exit: exit,
                sending_network: *NETWORK_A,
                imported_local_exit_root: self.local_exit_tree_data_a.get_root(),
                leaf_index: index,
                inclusion_proof: proof,
            };
            let null_key = NullifierKey {
                network_id: *NETWORK_A,
                let_index: index,
            };
            let nullifier_path = self.nullifier_set.get_non_inclusion_proof(null_key).unwrap();
            self.nullifier_set.insert(null_key, Digest::from_bool(true)).unwrap();
            res.push((imported_exit, nullifier_path));
        }

        // We need to update the LER/LEP to the final versions
        for (exit, _) in res.iter_mut() {
            exit.imported_local_exit_root = self.local_exit_tree_data_a.get_root();
            exit.inclusion_proof = self.local_exit_tree_data_a.get_proof(exit.leaf_index);
        }

        res
    }

    /// Bridge exits from network B to network A.
    pub fn bridge_exits(&mut self, events: &[(TokenInfo, U256)]) -> Vec<BridgeExit> {
        let mut res = Vec::new();
        for (token, amount) in events {
            let exit = exit_to_a(*token, *amount);
            self.local_exit_tree.add_leaf(exit.hash());
            res.push(exit);
        }

        res
    }

    /// Collect balance proofs for given set of bridge events.
    pub fn balances_proofs(
        &mut self,
        imported_bridge_events: &[(TokenInfo, U256)],
        bridge_events: &[(TokenInfo, U256)],
    ) -> BTreeMap<TokenInfo, (U256, LocalBalancePath<Keccak256Hasher>)> {
        let mut res = BTreeMap::new();
        let tokens: BTreeSet<_> = imported_bridge_events
            .iter()
            .chain(bridge_events)
            .map(|(token, _)| *token)
            .collect();
        let initial_balances: BTreeMap<_, _> = tokens
            .iter()
            .map(|&token| {
                (
                    token,
                    U256::from_be_bytes(self.local_balance_tree.get(token).unwrap_or_default()),
                )
            })
            .collect();
        let mut new_balances = initial_balances.clone();
        for &(token, amount) in imported_bridge_events {
            new_balances.insert(token, new_balances[&token].checked_add(amount).unwrap());
        }
        for &(token, amount) in bridge_events {
            new_balances.insert(token, new_balances[&token].checked_sub(amount).unwrap());
        }
        for token in tokens {
            let balance = initial_balances[&token];
            let path = if balance.is_zero() {
                self.local_balance_tree.get_inclusion_proof_zero(token).unwrap()
            } else {
                self.local_balance_tree.get_inclusion_proof(token).unwrap()
            };
            res.insert(token, (balance, path));
            self.local_balance_tree
                .update(token, new_balances[&token].to_be_bytes())
                .unwrap();
        }
        res
    }

    /// Local state associated with this forest.
    pub fn local_state(&self) -> LocalNetworkState {
        LocalNetworkState {
            exit_tree: self.local_exit_tree.clone(),
            balance_tree: LocalBalanceTree::new_with_root(self.local_balance_tree.root),
            nullifier_set: NullifierTree::new_with_root(self.nullifier_set.root),
        }
    }

    /// Apply a sequence of events and return the corresponding batch header.
    pub fn apply_events(
        &mut self,
        imported_bridge_events: &[(TokenInfo, U256)],
        bridge_events: &[(TokenInfo, U256)],
    ) -> MultiBatchHeader<Keccak256Hasher> {
        let prev_local_exit_root = self.local_exit_tree.get_root();
        let prev_balance_root = self.local_balance_tree.root;
        let prev_nullifier_root = self.nullifier_set.root;
        let balances_proofs = self.balances_proofs(imported_bridge_events, bridge_events);
        let imported_bridge_exits = self.imported_bridge_exits(imported_bridge_events);
        let bridge_exits = self.bridge_exits(bridge_events);
        MultiBatchHeader {
            origin_network: *NETWORK_B,
            prev_local_exit_root,
            new_local_exit_root: self.local_exit_tree.get_root(),
            bridge_exits,
            imported_bridge_exits,
            imported_exits_root: None,
            imported_local_exit_roots: [(*NETWORK_A, self.local_exit_tree_data_a.get_root())]
                .into(),
            balances_proofs,
            prev_balance_root,
            new_balance_root: self.local_balance_tree.root,
            prev_nullifier_root,
            new_nullifier_root: self.nullifier_set.root,
        }
    }
}

fn exit(token_info: TokenInfo, dest_network: NetworkId, amount: U256) -> BridgeExit {
    BridgeExit {
        leaf_type: LeafType::Transfer,
        token_info,
        dest_network,
        dest_address: random(),
        amount,
        metadata: vec![],
    }
}

fn exit_to_a(token_info: TokenInfo, amount: U256) -> BridgeExit {
    exit(token_info, *NETWORK_A, amount)
}

fn exit_to_b(token_info: TokenInfo, amount: U256) -> BridgeExit {
    exit(token_info, *NETWORK_B, amount)
}
