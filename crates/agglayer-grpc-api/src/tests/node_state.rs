use std::{sync::Arc, time::Duration};

use agglayer_config::Config;
use agglayer_grpc_server::node::v1::node_state_service_server::{
    NodeStateService, NodeStateServiceServer,
};
use agglayer_grpc_types::node::v1::{
    GetCertificateHeaderRequest, GetNetworkInfoErrorKind, GetNetworkInfoRequest,
};
use agglayer_rpc::AgglayerService;
use agglayer_storage::{
    backup::BackupClient,
    stores::{
        debug::DebugStore, pending::PendingStore, state::StateStore, PendingCertificateWriter as _,
        StateWriter as _,
    },
    tests::TempDBDir,
};
use agglayer_types::{CertificateId, CertificateStatus, Digest, Height};
use tokio::{net::TcpListener, sync::oneshot};
use tonic::Code;
use tonic_types::StatusExt as _;
use tower::ServiceExt as _;

use crate::node_state_service::NodeStateServer;

struct L1Rpc {}

#[tokio::test]
async fn get_network_info_unknown_network() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let debug_store = Arc::new(DebugStore::new_with_path(&config.storage.debug_db_path).unwrap());
    let service = Arc::new(AgglayerService::new(
        tokio::sync::mpsc::channel(1).0,
        pending_store.clone(),
        state_store.clone(),
        debug_store,
        config,
        Arc::new(L1Rpc {}),
    ));
    let server = NodeStateServer { service };

    let error = server
        .get_network_info(tonic::Request::new(GetNetworkInfoRequest { network_id: 1 }))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::NotFound);
    assert_eq!(error.message(), "Network type could not be determined");
    let details = error.get_error_details();
    let error_info = details.error_info().unwrap();
    assert_eq!(
        error_info.reason,
        GetNetworkInfoErrorKind::UnknownNetworkType.as_str_name()
    );
    assert_eq!(
        error_info.domain,
        "agglayer-node.grpc-api.v1.node-state-service.get_network_info"
    );

    let certificate = agglayer_types::Certificate::new_for_test(1.into(), Height::ZERO);
    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .unwrap();
    pending_store
        .insert_pending_certificate(certificate.network_id, certificate.height, &certificate)
        .unwrap();

    let info = server
        .get_network_info(tonic::Request::new(GetNetworkInfoRequest { network_id: 1 }))
        .await
        .unwrap()
        .into_inner()
        .network_info
        .unwrap();
    assert_eq!(info.network_id, 1);
    assert_eq!(
        info.latest_pending_certificate_id,
        Some(certificate.hash().into())
    );
    assert_eq!(
        info.network_type(),
        agglayer_grpc_types::node::types::v1::NetworkType::Unspecified
    );

    let error = server
        .get_network_info(tonic::Request::new(GetNetworkInfoRequest { network_id: 2 }))
        .await
        .unwrap_err();
    assert_eq!(error.code(), Code::NotFound);
}

#[tokio::test]
async fn get_certificate_header() {
    let tmp = TempDBDir::new();
    let config = Arc::new(Config::new(&tmp.path));

    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let debug_store = Arc::new(DebugStore::new_with_path(&config.storage.debug_db_path).unwrap());

    let certificate = agglayer_types::Certificate::new_for_test(1.into(), Height::ZERO);
    state_store
        .insert_certificate_header(&certificate, CertificateStatus::Pending)
        .expect("Failed to insert certificate header");

    let certificate_id = certificate.hash();

    let (sender, _receiver) = tokio::sync::mpsc::channel(10);
    let service = Arc::new(AgglayerService::new(
        sender,
        pending_store,
        state_store,
        debug_store,
        config,
        Arc::new(L1Rpc {}),
    ));
    let (tx, rx) = oneshot::channel::<()>();
    let svc = NodeStateServiceServer::new(NodeStateServer { service });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = axum::Router::new().route_service(
        "/agglayer.node.v1.NodeStateService/{*rest}",
        svc.map_request(|r: http::Request<axum::body::Body>| r.map(tonic::body::Body::new)),
    );

    let jh = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async { drop(rx.await) })
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client =
        agglayer_grpc_client::node::v1::node_state_service_client::NodeStateServiceClient::connect(
            format!("http://{addr}"),
        )
        .await
        .unwrap();

    let response = client
        .get_certificate_header(GetCertificateHeaderRequest {
            certificate_id: Some(CertificateId::new(Digest([0u8; 32])).into()),
        })
        .await;

    assert!(response.is_err());

    let error = response.unwrap_err();
    assert_eq!(error.code(), Code::NotFound);

    let response = client
        .get_certificate_header(GetCertificateHeaderRequest {
            certificate_id: Some(certificate_id.into()),
        })
        .await;

    assert!(response.is_ok());

    let cert = response.unwrap().into_inner();
    let cert_id = agglayer_types::CertificateId::try_from(
        cert.certificate_header.unwrap().certificate_id.unwrap(),
    )
    .unwrap();

    assert_eq!(cert_id, certificate_id);

    tx.send(()).unwrap();
    jh.await.unwrap();
}
