//! Tests for the disabled `interop_getTxStatus` method.

use alloy::primitives::B256;
use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};

use crate::testutils::TestContext;

#[test_log::test(tokio::test)]
async fn get_tx_status_returns_method_disabled_error() {
    let config = TestContext::get_default_config();
    let context = TestContext::new_with_config(config).await;

    let result: Result<String, ClientError> = context
        .api_client
        .request("interop_getTxStatus", rpc_params![B256::from([0x27; 32])])
        .await;

    let error = result.unwrap_err();
    let ClientError::Call(err) = error else {
        panic!("expected a call error, got: {error}");
    };

    // Wire contract: `interop_getTxStatus` is disabled (issue #1632).
    assert_eq!(err.code(), -10009);
    assert_eq!(err.message(), "The interop_getTxStatus method is disabled");
}
