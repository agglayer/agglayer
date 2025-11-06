use std::{num::NonZeroU64, sync::Arc, time::Duration};

use agglayer_config::{
    epoch::{BlockClockConfig, TimeClockConfig},
    Config, Epoch,
};
use agglayer_grpc_client::node::v1::configuration_service_client::ConfigurationServiceClient;
use agglayer_grpc_server::node::v1::configuration_service_server::ConfigurationServiceServer;
use agglayer_grpc_types::node::{types::v1, v1::GetEpochConfigurationRequest};
use agglayer_rpc::AgglayerService;
use agglayer_storage::{
    storage::backup::BackupClient,
    stores::{debug::DebugStore, epochs::EpochsStore, pending::PendingStore, state::StateStore},
    tests::TempDBDir,
};
use agglayer_types::EpochNumber;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};
use tonic::{transport::Channel, Code};
use tonic_types::StatusExt as _;
use tower::ServiceExt as _;

use crate::configuration_service::ConfigurationServer;

struct L1Rpc {}

#[tokio::test]
async fn timeclock_configuration() {
    let tmp = TempDBDir::new();
    let mut config = Config::new(&tmp.path);
    config.epoch = Epoch::TimeClock(TimeClockConfig {
        epoch_duration: Duration::from_secs(100),
    });

    let (mut client, tx, jh) = start_server_with_configuration_service(Arc::new(config)).await;

    let response = client
        .get_epoch_configuration(GetEpochConfigurationRequest {})
        .await;

    assert!(response.is_err());

    let error = response.unwrap_err();
    assert_eq!(error.code(), Code::InvalidArgument);
    let err_details = error.get_error_details();

    let error_info = err_details.error_info().unwrap();
    assert_eq!(
        error_info.reason,
        agglayer_grpc_types::node::v1::GetEpochConfigurationErrorKind::UnexpectedClockConfiguration
            .as_str_name()
    );
    assert_eq!(
        error_info.domain,
        crate::configuration_service::GET_EPOCH_CONFIGURATION_METHOD_PATH
    );

    tx.send(()).unwrap();
    jh.await.unwrap();
}

#[tokio::test]
async fn blockclock_configuration() {
    let tmp = TempDBDir::new();
    let mut config = Config::new(&tmp.path);
    config.epoch = Epoch::BlockClock(BlockClockConfig {
        epoch_duration: NonZeroU64::new(5).unwrap(),
        genesis_block: 0,
    });

    let (mut client, tx, jh) = start_server_with_configuration_service(Arc::new(config)).await;

    let response = client
        .get_epoch_configuration(GetEpochConfigurationRequest {})
        .await
        .expect("Failed to get epoch configuration");

    let response = response.into_inner();
    assert!(matches!(
        response.epoch_configuration,
        Some(v1::EpochConfiguration {
            genesis_block: 0,
            epoch_duration: 5
        })
    ));

    tx.send(()).unwrap();
    jh.await.unwrap();
}

async fn start_server_with_configuration_service(
    config: Arc<Config>,
) -> (
    ConfigurationServiceClient<Channel>,
    oneshot::Sender<()>,
    JoinHandle<()>,
) {
    let (sender, _receiver) = tokio::sync::mpsc::channel(10);
    let pending_store =
        Arc::new(PendingStore::new_with_path(&config.storage.pending_db_path).unwrap());
    let state_store = Arc::new(
        StateStore::new_with_path(&config.storage.state_db_path, BackupClient::noop()).unwrap(),
    );
    let service = Arc::new(AgglayerService::new(
        sender,
        pending_store.clone(),
        state_store.clone(),
        Arc::new(DebugStore::new_with_path(&config.storage.debug_db_path).unwrap()),
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
    let svc = ConfigurationServiceServer::new(ConfigurationServer { service });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let app = axum::Router::new().route_service(
        "/agglayer.node.v1.ConfigurationService/{*rest}",
        svc.map_request(|r: http::Request<axum::body::Body>| r.map(tonic::body::Body::new)),
    );

    let jh = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async { drop(rx.await) })
            .await
            .unwrap();
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = agglayer_grpc_client::node::v1::configuration_service_client::ConfigurationServiceClient::connect(format!("http://{addr}"))
            .await
            .unwrap();

    (client, tx, jh)
}
