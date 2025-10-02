use std::{sync::Arc, time::Duration};

use agglayer_config::Config;
use agglayer_grpc_server::node::v1::node_state_service_server::NodeStateServiceServer;
use agglayer_grpc_types::node::v1::GetCertificateHeaderRequest;
use agglayer_rpc::AgglayerService;
use agglayer_storage::{
    storage::backup::BackupClient,
    stores::{
        debug::DebugStore, epochs::EpochsStore, pending::PendingStore, state::StateStore,
        StateWriter as _,
    },
    tests::TempDBDir,
};
use agglayer_types::{CertificateId, CertificateStatus, Digest, EpochNumber, Height};
use tokio::{net::TcpListener, sync::oneshot};
use tonic::Code;
use tower::ServiceExt as _;

use crate::node_state_service::NodeStateServer;

struct L1Rpc {}

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
        pending_store.clone(),
        state_store.clone(),
        debug_store,
        Arc::new(
            EpochsStore::new(
                config.clone(),
                EpochNumber::ZERO,
                pending_store,
                state_store,
                BackupClient::noop(),
            )
            .unwrap(),
        ),
        config,
        Arc::new(L1Rpc {}),
    ));
    let (tx, rx) = oneshot::channel::<()>();
    let svc = NodeStateServiceServer::new(NodeStateServer { service });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = axum::Router::new().route_service(
        "/agglayer.node.v1.NodeStateService/{*rest}",
        svc.map_request(|r: http::Request<axum::body::Body>| {
            r.map(|b| tonic::body::Body::new(b))
        }),
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
