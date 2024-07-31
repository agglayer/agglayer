use std::collections::{BTreeMap, BTreeSet};

use lazy_static::lazy_static;
use pessimistic_proof::{
    bridge_exit::{BridgeExit, LeafType, NetworkId, TokenInfo},
    imported_bridge_exit::ImportedBridgeExit,
    keccak::Digest,
    local_balance_tree::{LocalBalancePath, LocalBalanceTree, LOCAL_BALANCE_TREE_DEPTH},
    local_exit_tree::{data::LocalExitTreeData, hasher::Keccak256Hasher},
    multi_batch_header::MultiBatchHeader,
    nullifier_tree::{FromBool, NullifierKey, NullifierPath, NullifierTree, NULLIFIER_TREE_DEPTH},
    utils::smt::Smt,
    LocalNetworkState,
};
use rand::random;
use reth_primitives::{address, U256};
use sp1_sdk::{utils, ProverClient, SP1Stdin};

/// The ELF we want to execute inside the zkVM.
const ELF: &[u8] =
    include_bytes!("../../pessimistic-proof-program/elf/riscv32im-succinct-zkvm-elf");

lazy_static! {
    pub static ref NETWORK_A: NetworkId = 0.into();
    pub static ref NETWORK_B: NetworkId = 1.into();
    pub static ref USDC: TokenInfo = TokenInfo {
        origin_network: *NETWORK_A,
        origin_token_address: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
    };
    pub static ref ETH: TokenInfo = TokenInfo {
        origin_network: *NETWORK_A,
        origin_token_address: address!("0000000000000000000000000000000000000000"),
    };
}

fn u(x: u64) -> U256 {
    x.try_into().unwrap()
}

// Trees for the network B, as well as the LET for network A.
struct Forest {
    local_exit_tree_data_a: LocalExitTreeData<Keccak256Hasher>,
    local_exit_tree_data: LocalExitTreeData<Keccak256Hasher>,
    local_balance_tree: Smt<Keccak256Hasher, LOCAL_BALANCE_TREE_DEPTH>,
    nullifier_set: Smt<Keccak256Hasher, NULLIFIER_TREE_DEPTH>,
}

impl Forest {
    fn new(initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>) -> Self {
        let mut local_balance_tree = Smt::new();
        for (token, balance) in initial_balances {
            local_balance_tree.insert(token, balance.to_be_bytes()).unwrap();
        }

        Self {
            local_exit_tree_data_a: LocalExitTreeData::new(),
            local_exit_tree_data: LocalExitTreeData::new(),
            local_balance_tree,
            nullifier_set: Smt::new(),
        }
    }

    // Imported bridge exits from network A to network B
    fn imported_bridge_exits(
        &mut self,
        events: impl IntoIterator<Item = (TokenInfo, U256)>,
    ) -> Vec<(ImportedBridgeExit, NullifierPath<Keccak256Hasher>)> {
        let mut res = Vec::new();
        for (token, amount) in events {
            let exit = exit_to_b(token, amount);
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

    // Bridge exits from network B to network A
    fn bridge_exits(
        &mut self,
        events: impl IntoIterator<Item = (TokenInfo, U256)>,
    ) -> Vec<BridgeExit> {
        let mut res = Vec::new();
        for (token, amount) in events {
            let exit = exit_to_a(token, amount);
            self.local_exit_tree_data.add_leaf(exit.hash());
            res.push(exit);
        }

        res
    }
    fn balances_proofs(
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

fn e2e_local_pp_simple_helper(
    initial_balances: impl IntoIterator<Item = (TokenInfo, U256)>,
    imported_events: impl IntoIterator<Item = (TokenInfo, U256)>,
    events: impl IntoIterator<Item = (TokenInfo, U256)>,
) {
    let mut forest = Forest::new(initial_balances);
    let prev_local_exit_root = forest.local_exit_tree_data.get_root();
    let prev_balance_root = forest.local_balance_tree.root;
    let prev_nullifier_root = forest.nullifier_set.root;
    let mut local_state = LocalNetworkState {
        exit_tree: (&forest.local_exit_tree_data).into(),
        balance_tree: LocalBalanceTree::new_with_root(prev_balance_root),
        nullifier_set: NullifierTree::new_with_root(prev_nullifier_root),
    };
    let imported_bridge_events = imported_events.into_iter().collect::<Vec<_>>();
    let bridge_events = events.into_iter().collect::<Vec<_>>();
    let balances_proofs = forest.balances_proofs(&imported_bridge_events, &bridge_events);
    let imported_bridge_exits = forest.imported_bridge_exits(imported_bridge_events);
    let bridge_exits = forest.bridge_exits(bridge_events);
    let batch_header = MultiBatchHeader {
        origin_network: *NETWORK_B,
        prev_local_exit_root,
        new_local_exit_root: forest.local_exit_tree_data.get_root(),
        bridge_exits,
        imported_bridge_exits,
        imported_exits_root: None,
        imported_local_exit_roots: [(*NETWORK_A, forest.local_exit_tree_data_a.get_root())].into(),
        balances_proofs,
        prev_balance_root,
        new_balance_root: forest.local_balance_tree.root,
        prev_nullifier_root,
        new_nullifier_root: forest.nullifier_set.root,
    };

    local_state.apply_batch_header(&batch_header).unwrap()
}
#[test]
fn e2e_local_pp_simple() {
    e2e_local_pp_simple_helper(
        vec![(*USDC, u(100)), (*ETH, u(200))],
        vec![(*USDC, u(50)), (*ETH, u(100)), (*USDC, u(10))],
        vec![(*USDC, u(20)), (*ETH, u(50)), (*USDC, u(130))],
    )
}

#[test]
fn e2e_local_pp_simple_zero_initial_balances() {
    e2e_local_pp_simple_helper(
        [],
        vec![(*USDC, u(50)), (*ETH, u(100)), (*USDC, u(10))],
        vec![(*USDC, u(20)), (*ETH, u(50)), (*USDC, u(30))],
    )
}

#[test]
fn e2e_local_pp_random() {
    let target = u(u64::MAX);
    let upper = u64::MAX / 10;
    let mut forest = Forest::new(vec![(*USDC, target), (*ETH, target)]);
    let prev_local_exit_root = forest.local_exit_tree_data.get_root();
    let prev_balance_root = forest.local_balance_tree.root;
    let prev_nullifier_root = forest.nullifier_set.root;
    let mut local_state = LocalNetworkState {
        exit_tree: (&forest.local_exit_tree_data).into(),
        balance_tree: LocalBalanceTree::new_with_root(prev_balance_root),
        nullifier_set: NullifierTree::new_with_root(prev_nullifier_root),
    };
    // Generate random bridge events such that the sum of the USDC and ETH amounts is less than `target`
    let get_events = || {
        let mut usdc_acc = U256::ZERO;
        let mut eth_acc = U256::ZERO;
        let mut events = Vec::new();
        loop {
            let amount = u(random::<u64>() % upper);
            let token = if random::<u64>() & 1 == 1 { *USDC } else { *ETH };
            if token == *USDC {
                usdc_acc += amount;
                if usdc_acc > target {
                    break;
                }
            } else {
                eth_acc += amount;
                if eth_acc > target {
                    break;
                }
            }
            events.push((token, amount));
        }
        events
    };
    let imported_bridge_events = get_events();
    let bridge_events = get_events();
    let balances_proofs = forest.balances_proofs(&imported_bridge_events, &bridge_events);
    let imported_bridge_exits = forest.imported_bridge_exits(imported_bridge_events);
    let bridge_exits = forest.bridge_exits(bridge_events);
    let batch_header = MultiBatchHeader {
        origin_network: *NETWORK_B,
        prev_local_exit_root,
        new_local_exit_root: forest.local_exit_tree_data.get_root(),
        bridge_exits,
        imported_bridge_exits,
        imported_exits_root: None,
        imported_local_exit_roots: [(*NETWORK_A, forest.local_exit_tree_data_a.get_root())].into(),
        balances_proofs,
        prev_balance_root,
        new_balance_root: forest.local_balance_tree.root,
        prev_nullifier_root,
        new_nullifier_root: forest.nullifier_set.root,
    };

    local_state.apply_batch_header(&batch_header).unwrap()
}

// Same as `e2e_local_pp_simple` with an SP1 proof on top
#[test]
#[ignore]
fn test_sp1_simple() {
    // Setup logging.
    utils::setup_logger();

    let mut forest = Forest::new(vec![(*USDC, u(100)), (*ETH, u(200))]);
    let prev_local_exit_root = forest.local_exit_tree_data.get_root();
    let prev_balance_root = forest.local_balance_tree.root;
    let prev_nullifier_root = forest.nullifier_set.root;
    let local_state = LocalNetworkState {
        exit_tree: (&forest.local_exit_tree_data).into(),
        balance_tree: LocalBalanceTree::new_with_root(prev_balance_root),
        nullifier_set: NullifierTree::new_with_root(prev_nullifier_root),
    };
    let imported_bridge_events = vec![(*USDC, u(50)), (*ETH, u(100)), (*USDC, u(10))];
    let bridge_events = vec![(*USDC, u(20)), (*ETH, u(50)), (*USDC, u(130))];
    let balances_proofs = forest.balances_proofs(&imported_bridge_events, &bridge_events);
    let imported_bridge_exits = forest.imported_bridge_exits(imported_bridge_events);
    let bridge_exits = forest.bridge_exits(bridge_events);
    let batch_header = MultiBatchHeader {
        origin_network: *NETWORK_B,
        prev_local_exit_root,
        new_local_exit_root: forest.local_exit_tree_data.get_root(),
        bridge_exits,
        imported_bridge_exits,
        imported_exits_root: None,
        imported_local_exit_roots: [(*NETWORK_A, forest.local_exit_tree_data_a.get_root())].into(),
        balances_proofs,
        prev_balance_root,
        new_balance_root: forest.local_balance_tree.root,
        prev_nullifier_root,
        new_nullifier_root: forest.nullifier_set.root,
    };

    let mut stdin = SP1Stdin::new();
    stdin.write(&local_state);
    stdin.write(&batch_header);

    // Generate the proof for the given program and input.
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let proof = client.prove(&pk, stdin).unwrap();

    // Verify proof and public values
    client.verify(&proof, &vk).expect("verification failed");
}
