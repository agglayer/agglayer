//! Sample data, either synthetic or taken from real traces.

use pessimistic_proof::bridge_exit::{NetworkId, TokenInfo};
use reth_primitives::address;

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
