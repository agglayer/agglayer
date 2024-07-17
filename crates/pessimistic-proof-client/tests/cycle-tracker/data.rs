use std::{collections::BTreeMap, path::Path};

use hex_literal::hex;
use pessimistic_proof::{
    local_balance_tree::{BalanceTree, Deposit},
    local_exit_tree, test_utils, BridgeExit, LocalNetworkState, NetworkId, TokenInfo,
};
use reth_primitives::{address, U256};

type LocalExitTree = local_exit_tree::LocalExitTree<local_exit_tree::hasher::Keccak256Hasher>;

pub fn empty_state() -> LocalNetworkState {
    LocalNetworkState {
        balance_tree: BalanceTree::from(BTreeMap::new()),
        exit_tree: LocalExitTree::new(),
    }
}

fn sample_exit_tree_01() -> LocalExitTree {
    const LEAF_COUNT: u32 = 1853;
    LocalExitTree::from_parts(
        LEAF_COUNT,
        [
            hex!("4a3c0e05a537700590e5cfa29654e7db5b36fbe85b24e7f34bdec7ed2b194aa6"),
            hex!("167e9d479ed70cdd2918875dd368edacc1b900085a2db71832a951ac7df31e10"),
            hex!("480549a7a72ab13cb9dd7a1c48f3b2749be3f3a7dd440f16125a1aa5cbf07991"),
            hex!("81b8a2cf7a80538dee49ae721a87655b080523d37cdad80c6a002a33e91c96cb"),
            hex!("5003a15ab43bbf7e8a86fe84c7af7a515e8086e53308b4321ac83560e44cd17b"),
            hex!("02c16029dec2ad77fb3f45ade9b12be2a191dc5bde71e15c5e873695b06eebb2"),
            hex!("9779f2ddec81f886c42d4813cd3fe44a8e5d077df11dab2d96d8e52e575ad196"),
            hex!("ff709923054a0745097aa2bd8b74f3434c2ef34ba4245af36efbb7792c719012"),
            hex!("47ea61b79f448e3d692755fdd7ea1242148f1736e2ec44910ed34397f093364d"),
            hex!("96f8e65b2aaa2500a40c5f8e72886cbe47248bda77d76d89666e47509649fdba"),
            hex!("50f7e8cc2d5e5e9f6ce5e5d0352fff94f6569449620e6e6a693b3dfb9d44e683"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
            hex!("0000000000000000000000000000000000000000000000000000000000000000"),
        ],
    )
}

fn sample_balance_tree_01(origin_network: NetworkId) -> BalanceTree {
    let eth = TokenInfo {
        origin_network: origin_network.clone(),
        origin_token_address: address!("0000000000000000000000000000000000000000"),
    };

    let token = TokenInfo {
        origin_network: origin_network.clone(),
        origin_token_address: address!("a0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"),
    };

    let infinite_eth = (eth.clone(), Deposit(U256::MAX).into());
    let infinite_token = (token.clone(), Deposit(U256::MAX).into());

    BalanceTree::from(BTreeMap::from_iter([infinite_eth, infinite_token]))
}

pub fn sample_state_01() -> LocalNetworkState {
    let origin_network_id = NetworkId::from(0);
    let balance_tree = sample_balance_tree_01(origin_network_id);
    let exit_tree = sample_exit_tree_01();

    LocalNetworkState {
        balance_tree,
        exit_tree,
    }
}

fn load_sample_bridge_exits(filename: impl AsRef<Path>) -> impl Iterator<Item = BridgeExit> {
    let filepath = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/cycle-tracker")
        .join(filename);
    test_utils::parse_json_file::<Vec<test_utils::DepositEventData>>(filepath.to_str().unwrap())
        .into_iter()
        .map(Into::into)
}

pub fn sample_bridge_exits_01() -> impl Iterator<Item = BridgeExit> {
    load_sample_bridge_exits("withdrawals.json")
}
