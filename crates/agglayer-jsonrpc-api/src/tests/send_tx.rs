//! Tests for the disabled `interop_sendTx` method.

use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};
use serde_json::json;

use crate::testutils::TestContext;

/// A wire-valid `SignedTx` JSON payload (zero proof, zero signature with
/// `v = 27`). It must deserialize successfully so the request reaches the
/// handler instead of being rejected with an invalid params error.
fn signed_tx_json() -> serde_json::Value {
    json!({
        "tx": {
            "RollupID": 1,
            "lastVerifiedBatch": "0x0",
            "newVerifiedBatch": "0x1",
            "ZKP": {
                "newStateRoot":
                    "0x0000000000000000000000000000000000000000000000000000000000000001",
                "newLocalExitRoot":
                    "0x0000000000000000000000000000000000000000000000000000000000000002",
                "proof": format!("0x{}", "00".repeat(24 * 32)),
            },
        },
        "signature": format!("0x{}1b", "00".repeat(64)),
    })
}

#[test_log::test(tokio::test)]
async fn send_tx_returns_method_disabled_error() {
    let config = TestContext::get_default_config();
    let context = TestContext::new_with_config(config).await;

    let result: Result<alloy::primitives::B256, ClientError> = context
        .api_client
        .request("interop_sendTx", rpc_params![signed_tx_json()])
        .await;

    let error = result.unwrap_err();
    let ClientError::Call(err) = error else {
        panic!("expected a call error, got: {error}");
    };

    // Wire contract: `interop_sendTx` is disabled (issue #1632).
    assert_eq!(err.code(), -10009);
    assert_eq!(err.message(), "The interop_sendTx method is disabled");
}
