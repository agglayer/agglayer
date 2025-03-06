use std::sync::Arc;
use std::time::Duration;

use agglayer_config::Config;
use agglayer_contracts::polygon_rollup_manager::PolygonRollupManager;
use agglayer_contracts::polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2;
use agglayer_contracts::L1RpcClient;
use agglayer_storage::storage::backup::BackupClient;
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
use tracing::debug;

use crate::testutils::next_available_addr;
use crate::TxStatus;
use crate::{kernel::Kernel, service::AgglayerService, AgglayerImpl};

#[test_log::test(tokio::test)]
async fn check_tx_status() {
    let db_dir = TempDBDir::new();
    let mut config = Config::new(&db_dir.path);

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

    let addr = next_available_addr();
    if let std::net::IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.readrpc_port = addr.port();

    let config = Arc::new(config);

    let rpc = Arc::new(client);
    let kernel = Kernel::new(rpc.clone(), config.clone());

    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);
    let service = AgglayerService::new(kernel);
    let rollup_manager = Arc::new(L1RpcClient::new(
        rpc.clone(),
        PolygonRollupManager::new(config.l1.rollup_manager_contract, rpc.clone()),
        PolygonZkEVMGlobalExitRootV2::new(
            config.l1.polygon_zkevm_global_exit_root_v2_contract,
            rpc.clone(),
        ),
        (1, [1; 32]),
    ));
    let storage = agglayer_test_suite::StorageContext::new_with_config(config.clone());
    let rpc_service = agglayer_rpc::AgglayerService::new(
        certificate_sender,
        storage.pending.clone(),
        storage.state.clone(),
        storage.debug.clone(),
        config.clone(),
        rollup_manager,
    );

    let router = AgglayerImpl::new(Arc::new(service), Arc::new(rpc_service))
        .start()
        .await
        .unwrap();

    let listener = tokio::net::TcpListener::bind(config.readrpc_addr())
        .await
        .unwrap();
    let api_server = axum::serve(listener, router);

    let _rpc_handle = tokio::spawn(async move {
        _ = api_server.await;
        debug!("Node RPC shutdown requested.");
    });

    let url = format!("http://{}/", config.readrpc_addr());
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
    let rpc = Arc::new(client);
    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let std::net::IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    let config = Arc::new(config);

    let kernel = Kernel::new(rpc.clone(), config.clone());

    let tmp = TempDBDir::new();
    let db = Arc::new(DB::open_cf(tmp.path.as_path(), pending_db_cf_definitions()).unwrap());
    let tmp = TempDBDir::new();
    let store_db = Arc::new(DB::open_cf(tmp.path.as_path(), state_db_cf_definitions()).unwrap());
    let store = Arc::new(PendingStore::new(db));
    let state = Arc::new(StateStore::new(store_db, BackupClient::noop()));
    let debug = Arc::new(DebugStore::new_with_path(&tmp.path.join("debug")).unwrap());

    let service = AgglayerService::new(kernel);
    let rollup_manager = Arc::new(L1RpcClient::new(
        rpc.clone(),
        PolygonRollupManager::new(config.l1.rollup_manager_contract, rpc.clone()),
        PolygonZkEVMGlobalExitRootV2::new(
            config.l1.polygon_zkevm_global_exit_root_v2_contract,
            rpc.clone(),
        ),
        (1, [1; 32]),
    ));
    let rpc_service = agglayer_rpc::AgglayerService::new(
        certificate_sender,
        store,
        state,
        debug,
        config.clone(),
        rollup_manager,
    );

    let router = AgglayerImpl::new(Arc::new(service), Arc::new(rpc_service))
        .start()
        .await
        .unwrap();

    let listener = tokio::net::TcpListener::bind(config.readrpc_addr())
        .await
        .unwrap();
    let api_server = axum::serve(listener, router);

    let _rpc_handle = tokio::spawn(async move {
        _ = api_server.await;
        debug!("Node RPC shutdown requested.");
    });
    let url = format!("http://{}/", config.readrpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    // Try to get status using a non-existent address
    let fake_tx_hash = H256([0x27; 32]);
    let result: Result<TxStatus, ClientError> = client
        .request("interop_getTxStatus", rpc_params![fake_tx_hash])
        .await;

    match result.unwrap_err() {
        ClientError::Call(err) => {
            assert_eq!(err.code(), crate::error::code::STATUS_ERROR);

            let data_expected = serde_json::json! {
                { "status": { "tx-not-found": { "hash":  fake_tx_hash} } }
            };
            let data = serde_json::to_value(err.data().expect("data should not be empty")).unwrap();
            assert_eq!(data_expected, data);
        }
        _ => panic!("Unexpected error returned"),
    }
}
