use std::{net::IpAddr, sync::Arc};

use agglayer_config::Config;
use agglayer_contracts::{
    polygon_rollup_manager::PolygonRollupManager,
    polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2, L1RpcClient,
};
use agglayer_storage::tests::TempDBDir;
use agglayer_types::{Certificate, CertificateId, NetworkId};
use ethers::{providers, signers::Signer as _};
use jsonrpsee::{core::client::ClientT, http_client::HttpClientBuilder, rpc_params};
use tracing::debug;

use super::next_available_addr;
use crate::{kernel::Kernel, service::AgglayerService, AgglayerImpl};

#[test_log::test(tokio::test)]
async fn send_certificate_method_can_be_called_and_succeed() {
    let db_dir = TempDBDir::new();
    let mut config = Config::new(&db_dir.path);
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(NetworkId::new(1)).address());
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let storage = agglayer_test_suite::StorageContext::new_with_config(config.clone());
    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, mut certificate_receiver) = tokio::sync::mpsc::channel(1);

    let rpc = Arc::new(provider);
    let kernel = Kernel::new(rpc.clone(), config.clone());

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

    let listener = tokio::net::TcpListener::bind(config.rpc_addr())
        .await
        .unwrap();
    let api_server = axum::serve(listener, router);

    let _rpc_handle = tokio::spawn(async move {
        _ = api_server.await;
        debug!("Node RPC shutdown requested.");
    });

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    let _: CertificateId = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), 0)],
        )
        .await
        .unwrap();

    assert!(certificate_receiver.try_recv().is_ok());
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_can_be_called_and_fail() {
    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, certificate_receiver) = tokio::sync::mpsc::channel(1);

    let rpc = Arc::new(provider);
    let kernel = Kernel::new(rpc.clone(), config.clone());

    let service = AgglayerService::new(kernel);

    let storage = agglayer_test_suite::StorageContext::new_with_config(config.clone());
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
        storage.pending.clone(),
        storage.state.clone(),
        storage.debug.clone(),
        config.clone(),
        rollup_manager,
    );

    let _server_handle = AgglayerImpl::new(Arc::new(service), Arc::new(rpc_service))
        .start()
        .await
        .unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    drop(certificate_receiver);

    let res: Result<(), _> = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(0.into(), 0)],
        )
        .await;

    assert!(res.is_err());
}

#[test_log::test(tokio::test)]
async fn send_certificate_method_requires_known_signer() {
    let mut config = Config::new_for_test();
    // Willingly insert a signer that is not the one that’ll be used down below
    config
        .proof_signers
        .insert(1, Certificate::wallet_for_test(NetworkId::new(2)).address());
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

    let rpc = Arc::new(provider);
    let kernel = Kernel::new(rpc.clone(), config.clone());

    let service = AgglayerService::new(kernel);

    let storage = agglayer_test_suite::StorageContext::new_with_config(config.clone());
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
        storage.pending.clone(),
        storage.state.clone(),
        storage.debug.clone(),
        config.clone(),
        rollup_manager,
    );

    let _server_handle = AgglayerImpl::new(Arc::new(service), Arc::new(rpc_service))
        .start()
        .await
        .unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    let send_request: Result<CertificateId, _> = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::new_for_test(1.into(), 0)],
        )
        .await;

    assert!(send_request.is_err());
}
