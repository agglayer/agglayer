use std::net::IpAddr;
use std::sync::Arc;
use std::time::Duration;

use agglayer_config::Config;
use agglayer_storage::storage::{pending_db_cf_definitions, state_db_cf_definitions, DB};
use agglayer_storage::stores::pending::PendingStore;
use agglayer_storage::stores::state::StateStore;
use agglayer_storage::stores::{PendingCertificateWriter, StateReader, StateWriter};
use agglayer_storage::tests::TempDBDir;
use agglayer_types::{
    Certificate, CertificateHeader, CertificateId, CertificateStatus, Hash, NetworkId,
};
use ethers::providers::{self, Http, Middleware, Provider, ProviderExt as _};
use ethers::types::{TransactionRequest, H256};
use ethers::utils::Anvil;
use http_body_util::Empty;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use jsonrpsee::core::client::ClientT;
use jsonrpsee::core::ClientError;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::rpc_params;

use crate::rpc::{self, TxStatus};
use crate::{kernel::Kernel, rpc::AgglayerImpl};

mod errors;

#[tokio::test]
async fn healthcheck_method_can_be_called() {
    use hyper::Request;

    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let std::net::IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);
    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(provider, config.clone());

    let _server_handle = AgglayerImpl::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
    )
    .start(config.clone())
    .await
    .unwrap();

    let http_client = Client::builder(TokioExecutor::new()).build_http();
    let uri = format!("http://{}/health", config.rpc_addr());

    let req = Request::builder()
        .method("GET")
        .uri(&uri)
        .body(Empty::<hyper::body::Bytes>::new())
        .expect("request builder");
    let res = http_client.request(req).await.unwrap();

    assert!(res.status().is_success());

    let bytes = http_body_util::BodyExt::collect(res.into_body())
        .await
        .unwrap();
    let out = String::from_utf8(bytes.to_bytes().to_vec()).unwrap();
    assert_eq!(out.as_str(), "{\"health\":true}");
}

#[tokio::test]
async fn check_tx_status() {
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

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

    let kernel = Kernel::new(client, config.clone());

    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);
    let _server_handle = AgglayerImpl::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
    )
    .start(config.clone())
    .await
    .unwrap();

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

#[tokio::test]
async fn send_certificate_method_can_be_called() {
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, mut certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(provider, config.clone());

    let _server_handle = AgglayerImpl::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
    )
    .start(config.clone())
    .await
    .unwrap();

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

#[tokio::test]
async fn send_certificate_method_can_be_called_and_fail() {
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(provider, config.clone());

    let _server_handle = AgglayerImpl::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
    )
    .start(config.clone())
    .await
    .unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    drop(certificate_receiver);

    let res: Result<(), _> = client
        .request(
            "interop_sendCertificate",
            rpc_params![Certificate::default()],
        )
        .await;

    assert!(res.is_err());
}

#[tokio::test]
async fn check_tx_status_fail() {
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let anvil = Anvil::new().block_time(1u64).spawn();
    let client = Provider::<Http>::connect(&anvil.endpoint()).await;
    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let std::net::IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    let config = Arc::new(config);

    let kernel = Kernel::new(client, config.clone());

    let tmp = TempDBDir::new();
    let db = Arc::new(DB::open_cf(tmp.path.as_path(), pending_db_cf_definitions()).unwrap());
    let tmp = TempDBDir::new();
    let store_db = Arc::new(DB::open_cf(tmp.path.as_path(), state_db_cf_definitions()).unwrap());
    let store = Arc::new(PendingStore::new(db));
    let state = Arc::new(StateStore::new(store_db));

    let _server_handle = AgglayerImpl::new(kernel, certificate_sender, store, state)
        .start(config.clone())
        .await
        .unwrap();

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

#[tokio::test]
async fn get_certificate_header_after_sending_the_certif() {
    let _ = tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();

    let tmp = TempDBDir::new();
    let mut config = Config::new(&tmp.path);
    let state_db =
        Arc::new(DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions()).unwrap());
    let pending_db = Arc::new(
        DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions()).unwrap(),
    );

    let state_store = Arc::new(StateStore::new(state_db));
    let pending_store = Arc::new(PendingStore::new(pending_db));

    let addr = next_available_addr();
    if let IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);

    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, mut certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(provider, config.clone());

    let _server_handle = AgglayerImpl::new(kernel, certificate_sender, pending_store, state_store)
        .start(config.clone())
        .await
        .unwrap();

    let url = format!("http://{}/", config.rpc_addr());
    let client = HttpClientBuilder::default().build(url).unwrap();

    let certificate = Certificate::new_for_test(1.into(), 0);
    let id = certificate.hash();

    let res: CertificateId = client
        .request("interop_sendCertificate", rpc_params![certificate])
        .await
        .unwrap();

    assert_eq!(id, res);
    assert!(certificate_receiver.try_recv().is_ok());

    let payload: CertificateHeader = client
        .request("interop_getCertificateHeader", rpc_params![id])
        .await
        .unwrap();

    assert_eq!(payload.certificate_id, id);
    assert_eq!(payload.status, CertificateStatus::Pending);

    let payload: Result<CertificateHeader, ClientError> = client
        .request("interop_getCertificateHeader", rpc_params![Hash([0; 32])])
        .await;

    let error = payload.unwrap_err();

    let expected_message = format!("Resource not found: Certificate({:#})", Hash([0; 32]));
    assert!(matches!(error, ClientError::Call(obj) if obj.message() == expected_message));
}

fn next_available_addr() -> std::net::SocketAddr {
    use std::net::{TcpListener, TcpStream};

    assert!(
        std::env::var("NEXTEST").is_ok(),
        "Due to concurrency issues, the rpc tests have to be run under `cargo nextest`",
    );

    let host = "127.0.0.1";
    // Request a random available port from the OS
    let listener = TcpListener::bind((host, 0)).expect("Can't bind to an available port");
    let addr = listener.local_addr().expect("Can't find an available port");

    // Create and accept a connection (which we'll promptly drop) in order to force
    // the port into the TIME_WAIT state, ensuring that the port will be
    // reserved from some limited amount of time (roughly 60s on some Linux
    // systems)
    let _sender = TcpStream::connect(addr).expect("Can't connect to an available port");
    let _incoming = listener.accept().expect("Can't accept an available port");

    addr
}

struct DummyStore {}

impl StateWriter for DummyStore {
    fn insert_certificate_header(
        &self,
        _certificate: &Certificate,
        _status: agglayer_types::CertificateStatus,
    ) -> Result<(), agglayer_storage::error::Error> {
        Ok(())
    }
    fn update_certificate_header_status(
        &self,
        _certificate_id: &agglayer_types::CertificateId,
        _status: &CertificateStatus,
    ) -> Result<(), agglayer_storage::error::Error> {
        Ok(())
    }
}

impl StateReader for DummyStore {
    fn get_active_networks(&self) -> Result<Vec<NetworkId>, agglayer_storage::error::Error> {
        todo!()
    }

    fn get_certificate_header(
        &self,
        _certificate_id: &agglayer_types::CertificateId,
    ) -> Result<Option<agglayer_types::CertificateHeader>, agglayer_storage::error::Error> {
        Ok(None)
    }

    fn get_certificate_header_by_cursor(
        &self,
        _network_id: NetworkId,
        _height: agglayer_types::Height,
    ) -> Result<Option<agglayer_types::CertificateHeader>, agglayer_storage::error::Error> {
        todo!()
    }

    fn get_current_settled_height(
        &self,
    ) -> Result<
        Vec<(
            NetworkId,
            agglayer_types::Height,
            agglayer_types::CertificateId,
        )>,
        agglayer_storage::error::Error,
    > {
        todo!()
    }
}
impl PendingCertificateWriter for DummyStore {
    fn insert_pending_certificate(
        &self,
        _network_id: NetworkId,
        _height: u64,
        _certificate: &Certificate,
    ) -> Result<(), agglayer_storage::error::Error> {
        Ok(())
    }

    fn insert_generated_proof(
        &self,
        _certificate_id: &agglayer_types::CertificateId,
        _proof: &agglayer_types::Proof,
    ) -> Result<(), agglayer_storage::error::Error> {
        Ok(())
    }

    fn remove_pending_certificate(
        &self,
        _network_id: agglayer_types::NetworkId,
        _height: agglayer_types::Height,
    ) -> Result<(), agglayer_storage::error::Error> {
        Ok(())
    }
}
