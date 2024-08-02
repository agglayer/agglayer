//! Sample data, either synthetic or taken from real traces.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use hex_literal::hex;
use pessimistic_proof::{
    bridge_exit::{BridgeExit, NetworkId, TokenInfo},
    local_balance_tree, local_exit_tree,
    nullifier_tree::NullifierTree,
    LocalNetworkState,
};
use reth_primitives::{address, U256};

use super::{
    event_data::{parse_json_file, DepositEventData},
    forest::Forest,
};

type TreeHasher = local_exit_tree::hasher::Keccak256Hasher;
type LocalExitTree = local_exit_tree::LocalExitTree<TreeHasher>;
type LocalBalanceTree = local_balance_tree::LocalBalanceTree<TreeHasher>;

lazy_static::lazy_static! {
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

pub fn empty_state() -> LocalNetworkState {
    LocalNetworkState {
        balance_tree: LocalBalanceTree::new(),
        exit_tree: LocalExitTree::new(),
        nullifier_set: NullifierTree::new(),
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

pub fn sample_state_01() -> Forest {
    let balances = [(*ETH, U256::MAX), (*USDC, U256::MAX)];
    Forest::new_with_local_exit_tree(balances, sample_exit_tree_01())
}

fn sample_file_path(filename: impl AsRef<Path>) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data").join(filename)
}

fn load_sample_bridge_exits(filename: impl AsRef<Path>) -> impl Iterator<Item = BridgeExit> {
    parse_json_file::<Vec<DepositEventData>>(sample_file_path(filename).to_str().unwrap())
        .into_iter()
        .map(Into::into)
}

pub fn sample_bridge_exits_01() -> impl Iterator<Item = BridgeExit> {
    load_sample_bridge_exits("withdrawals.json")
}
