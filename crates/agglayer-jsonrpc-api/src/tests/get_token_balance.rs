use agglayer_storage::stores::StateWriter;
use agglayer_types::{Digest, LocalNetworkStateData, U256};
use jsonrpsee::{core::client::ClientT, rpc_params};
use unified_bridge::{NetworkId, TokenBalanceEntry, TokenInfo, L1_ETH};

use crate::testutils::TestContext;

#[test_log::test(tokio::test)]
async fn admin_get_token_balance() {
    let mut config = TestContext::get_default_config();
    config.debug_mode = true;

    let origin_network = NetworkId::new(72);
    let context = TestContext::new_with_config(config).await;
    let weth = TokenInfo {
        origin_network,
        ..L1_ETH
    };

    let l1_eth_balance = {
        let mut b = [0u8; 32];
        b[31] = 255u8;
        b
    };

    let weth_balance = {
        let mut b = [0u8; 32];
        b[31] = 78u8;
        b
    };

    let new_state = {
        let mut s = LocalNetworkStateData::default();
        s.balance_tree
            .insert(L1_ETH, Digest(l1_eth_balance))
            .unwrap();
        s.balance_tree.insert(weth, Digest(weth_balance)).unwrap();
        s
    };

    context
        .state_store
        .write_local_network_state(&origin_network, &new_state, &[])
        .unwrap();

    // returns only the requested token balance
    let payload: Vec<unified_bridge::TokenBalanceEntry> = context
        .admin_client
        .request(
            "admin_getTokenBalance",
            rpc_params![origin_network, Some(L1_ETH)],
        )
        .await
        .unwrap();

    let expected_entry = TokenBalanceEntry {
        origin_network: L1_ETH.origin_network,
        origin_token_address: L1_ETH.origin_token_address,
        amount: U256::from_be_bytes(l1_eth_balance),
    };

    assert_eq!(payload.len(), 1); // return only 1 token balance
    let single_entry = &payload[0];
    assert_eq!(single_entry.origin_network, expected_entry.origin_network);
    assert_eq!(
        single_entry.origin_token_address,
        expected_entry.origin_token_address
    );
    assert_eq!(single_entry.amount, expected_entry.amount);

    // returns all the balances because None TokenInfo provided
    let payload: Vec<unified_bridge::TokenBalanceEntry> = context
        .admin_client
        .request(
            "admin_getTokenBalance",
            rpc_params![origin_network, None::<TokenInfo>],
        )
        .await
        .unwrap();
    assert_eq!(payload.len(), 2);

    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
}
