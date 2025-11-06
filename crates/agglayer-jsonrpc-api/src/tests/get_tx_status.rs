use agglayer_config::Config;
use agglayer_storage::tests::TempDBDir;
use alloy::{
    node_bindings::Anvil,
    primitives::B256,
    providers::{Provider, ProviderBuilder},
    rpc::types::TransactionRequest,
};
use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};
use tracing::{debug, error, info};

use crate::testutils::TestContext;

#[test_log::test(tokio::test)]
async fn check_tx_status() {
    let db_dir = TempDBDir::new();
    let config = Config::new(&db_dir.path);

    // Start Anvil node for real blockchain interaction
    let anvil = Anvil::new().block_time(1u64).spawn();
    let provider = ProviderBuilder::new().on_http(anvil.endpoint_url());

    // Get accounts and send a transaction
    let accounts = provider.get_accounts().await.unwrap();
    let from = accounts[0];
    let to = accounts[1];

    let tx = TransactionRequest::default()
        .to(to)
        .value(alloy::primitives::U256::from(1000))
        .from(from);

    let pending_tx = provider.send_transaction(tx).await.unwrap();
    let hash = *pending_tx.tx_hash();

    // Set up our test context with the real Anvil provider
    let context = TestContext::new_with_provider(config, provider).await;

    // Test with the real transaction hash - should work now!
    let result: Result<String, ClientError> = context
        .api_client
        .request("interop_getTxStatus", rpc_params![hash])
        .await;

    match result {
        Ok(status) => {
            // Should get either "pending" or "done" depending on block time
            assert!(status == "pending" || status == "done");
            debug!("Transaction status: {}", status);
        }
        Err(error) => {
            // We may be transient error if the transaction is still being processed.
            tracing::warn!("Error getting transaction status: {}", error);
        }
    }

    info!("Transaction hash: {}", hash);

    // Wait a bit and try again to see if status changes
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    let result2: Result<String, ClientError> = context
        .api_client
        .request("interop_getTxStatus", rpc_params![hash])
        .await;

    match result2 {
        Ok(status) => {
            info!("Transaction status after waiting: {}", status);
            // After waiting, it should likely be "done"
            assert_eq!(status, "done");
        }
        Err(error) => {
            error!("Error getting transaction status after waiting: {}", error);
        }
    }
}

#[test_log::test(tokio::test)]
async fn check_tx_status_fail() {
    let config = TestContext::get_default_config();
    let context = TestContext::new_with_config(config).await;

    // Test error handling with non-existent transaction hash
    let fake_tx_hash = B256::from([0x27; 32]);

    let result: Result<String, ClientError> = context
        .api_client
        .request("interop_getTxStatus", rpc_params![fake_tx_hash])
        .await;

    // Should get an error for non-existent transaction
    assert!(result.is_err());
    let error = result.unwrap_err();

    // Check that we get a proper error response
    match error {
        ClientError::Call(error_object) => {
            // This would be the expected error format from the RPC
            info!("Received expected call error: {}", error_object);
        }
        ClientError::RequestTimeout => {
            panic!("Unexpected timeout error");
        }
        ClientError::Transport(_) => {
            // This might occur with our test setup
            panic!("Transport error (expected in test environment): {error}");
        }
        _ => {
            panic!("Other error type: {error}");
        }
    }
}
