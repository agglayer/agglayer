use std::time::Duration;

use agglayer_config::{epoch::TimeClockConfig, Epoch};
use agglayer_types::EpochConfiguration;
use insta::assert_snapshot;
use jsonrpsee::{
    core::{client::ClientT, ClientError},
    rpc_params,
};
use rstest::*;
use serde_json::json;

use super::context;
use super::raw_rpc;
use super::TestContext;
use crate::rpc::{tests::RawRpcContext, AgglayerServer};

#[test_log::test(tokio::test)]
async fn fetch_timeclock_config() {
    let mut config = TestContext::get_default_config();
    config.epoch = Epoch::TimeClock(TimeClockConfig {
        epoch_duration: Duration::from_secs(1),
    });

    let context = TestContext::new_with_config(config).await;

    let payload: Result<EpochConfiguration, ClientError> = context
        .client
        .request("interop_getEpochConfiguration", rpc_params![])
        .await;

    let error = payload.unwrap_err();

    let expected_message = "Internal error: AggLayer isn't configured with a BlockClock \
                            configuration, thus no EpochConfiguration is available";
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn fetch_block_clock_config(#[future] context: TestContext) {
    let payload: EpochConfiguration = context
        .client
        .request("interop_getEpochConfiguration", rpc_params![])
        .await
        .unwrap();

    assert_eq!(payload.epoch_duration, 6);
    assert_eq!(payload.genesis_block, 0);
}

#[rstest]
#[awt]
#[test_log::test(tokio::test)]
async fn block_clock_configuration(#[future] raw_rpc: RawRpcContext) {
    let rpc = raw_rpc.rpc.into_rpc();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getEpochConfiguration",
        "params": Vec::<()>::new(),
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!("get_clock_configuration::block", json);
}

#[test_log::test(tokio::test)]
async fn time_clock_configuration() {
    let mut config = TestContext::get_default_config();
    config.epoch = Epoch::TimeClock(TimeClockConfig {
        epoch_duration: Duration::from_secs(1),
    });

    let raw_rpc = TestContext::new_raw_rpc_with_config(config).await;
    let rpc = raw_rpc.rpc.into_rpc();
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "interop_getEpochConfiguration",
        "params": Vec::<()>::new(),
        "id": 0
    });
    let (response, _) = rpc
        .raw_json_request(&serde_json::to_string(&payload).unwrap(), 1)
        .await
        .unwrap();
    let json = serde_json::from_str::<serde_json::Value>(&response).unwrap();
    let json = serde_json::to_string_pretty(&json).unwrap();

    assert_snapshot!("get_clock_configuration::time", json);
}
