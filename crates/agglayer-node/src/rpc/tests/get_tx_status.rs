use std::sync::Arc;
use std::time::Duration;

use agglayer_config::Config;
use agglayer_storage::storage::{pending_db_cf_definitions, state_db_cf_definitions, DB};
use agglayer_storage::stores::debug::DebugStore;
use agglayer_storage::stores::pending::PendingStore;
use agglayer_storage::stores::state::StateStore;
use agglayer_storage::tests::TempDBDir;
use ethers::providers::{Http, Middleware, Provider, ProviderExt as _};
use ethers::types::{TransactionRequest, H256};
use ethers::utils::Anvil;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::ClientError;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;

use super::next_available_addr;
use crate::rpc::tests::DummyStore;
use crate::rpc::{self, TxStatus};
use crate::{kernel::Kernel, rpc::AgglayerImpl, service::AgglayerService};

#[test_log::test(tokio::test)]
async fn check_tx_status() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let client = Provider::<Http>::connect(&anvil.endpoint()).await;
    let accounts = client.get_accounts().await.unwrap();
    let from = accounts[0];
    let to = accounts[1];

    let tx = TransactionRequest::new().to(to).value(1000).from(from);

    let hash = client
        .send_transaction(tx, None)
        .await
        .unwrap()
        .await
        .unwrap()
        .unwrap()
        .transaction_hash;

    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let std::net::IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let kernel = Kernel::new(Arc::new(client), config.clone());

    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);
    let service = AgglayerService::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        config.clone(),
    );

    let _server_handle = AgglayerImpl::new(Arc::new(service)).start().await.unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    let res: TxStatus = client
        .request("interop_getTxStatus", rpc_params![hash])
        .await
        .unwrap();

    // The transaction is not yet mined, so we should get a pending status
    assert_eq!(res, "pending");

    tokio::time::sleep(Duration::from_secs(1)).await;

    let res: TxStatus = client
        .request("interop_getTxStatus", rpc_params![hash])
        .await
        .unwrap();

    assert_eq!(res, "done");
}

#[test_log::test(tokio::test)]
async fn check_tx_status_fail() {
    let anvil = Anvil::new().block_time(1u64).spawn();
    let client = Provider::<Http>::connect(&anvil.endpoint()).await;
    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let std::net::IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    let config = Arc::new(config);

    let kernel = Kernel::new(Arc::new(client), config.clone());

    let tmp = TempDBDir::new();
    let db = Arc::new(DB::open_cf(tmp.path.as_path(), pending_db_cf_definitions()).unwrap());
    let tmp = TempDBDir::new();
    let store_db = Arc::new(DB::open_cf(tmp.path.as_path(), state_db_cf_definitions()).unwrap());
    let store = Arc::new(PendingStore::new(db));
    let state = Arc::new(StateStore::new(store_db));
    let debug = Arc::new(DebugStore::new_with_path(&tmp.path.join("debug")).unwrap());

    let service = AgglayerService::new(
        kernel,
        certificate_sender,
        store,
        state,
        debug,
        config.clone(),
    );

    let _server_handle = AgglayerImpl::new(Arc::new(service)).start().await.unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    // Try to get status using a non-existent address
    let fake_tx_hash = H256([0x27; 32]);
    let result: Result<TxStatus, ClientError> = client
        .request("interop_getTxStatus", rpc_params![fake_tx_hash])
        .await;

    match result.unwrap_err() {
        ClientError::Call(err) => {
            assert_eq!(err.code(), rpc::error::code::STATUS_ERROR);

            let data_expected = serde_json::json! {
                { "status": { "tx-not-found": { "hash":  fake_tx_hash} } }
            };
            let data = serde_json::to_value(err.data().expect("data should not be empty")).unwrap();
            assert_eq!(data_expected, data);
        }
        _ => panic!("Unexpected error returned"),
    }
}
