use std::net::IpAddr;
use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::columns::latest_settled_certificate_per_network::SettledCertificate;
use agglayer_storage::storage::{pending_db_cf_definitions, state_db_cf_definitions, DB};
use agglayer_storage::stores::pending::PendingStore;
use agglayer_storage::stores::state::StateStore;
use agglayer_storage::{
    stores::{PendingCertificateWriter, StateReader, StateWriter},
    tests::TempDBDir,
};
use agglayer_types::{Certificate, CertificateId, CertificateStatus, Height, NetworkId};
use ethers::providers::{self, MockProvider, Provider};
use http_body_util::Empty;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use jsonrpsee::http_client::HttpClientBuilder;
use jsonrpsee::server::ServerHandle;
use rstest::*;

use crate::{kernel::Kernel, rpc::AgglayerImpl};

mod errors;
mod get_certificate_header;
mod get_epoch_configuration;
mod get_tx_status;
mod send_certificate;

#[test_log::test(tokio::test)]
async fn healthcheck_method_can_be_called() {
    use hyper::Request;

    let mut config = Config::new_for_test();
    let addr = next_available_addr();
    if let std::net::IpAddr::V4(ip) = addr.ip() {
        config.rpc.host = ip;
    }
    config.rpc.port = addr.port();

    let config = Arc::new(config);
    let (provider, _mock) = providers::Provider::mocked();
    let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

    let kernel = Kernel::new(Arc::new(provider), config.clone());

    let _server_handle = AgglayerImpl::new(
        kernel,
        certificate_sender,
        Arc::new(DummyStore {}),
        Arc::new(DummyStore {}),
        config.clone(),
    )
    .start()
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

pub(crate) struct RawRpcContext {
    pub(crate) rpc: AgglayerImpl<Provider<MockProvider>, PendingStore, StateStore>,
    config: Arc<Config>,
    pub(crate) certificate_receiver:
        tokio::sync::mpsc::Receiver<(NetworkId, Height, CertificateId)>,
}

pub(crate) struct TestContext {
    pub(crate) _server_handle: ServerHandle,
    pub(crate) client: jsonrpsee::http_client::HttpClient,
    pub(crate) certificate_receiver:
        tokio::sync::mpsc::Receiver<(NetworkId, Height, CertificateId)>,
}

impl TestContext {
    async fn new() -> Self {
        let config = Self::get_default_config();
        Self::new_with_config(config).await
    }

    async fn new_with_config(config: Config) -> Self {
        let raw_rpc = Self::new_raw_rpc_with_config(config).await;
        let _server_handle = raw_rpc.rpc.start().await.unwrap();

        let url = format!("http://{}/", raw_rpc.config.rpc_addr());
        let client = HttpClientBuilder::default().build(url).unwrap();

        Self {
            _server_handle,
            client,
            certificate_receiver: raw_rpc.certificate_receiver,
        }
    }

    pub(crate) fn get_default_config() -> Config {
        let tmp = TempDBDir::new();
        Config::new(&tmp.path)
    }

    async fn new_raw_rpc() -> RawRpcContext {
        let config = Self::get_default_config();
        Self::new_raw_rpc_with_config(config).await
    }

    async fn new_raw_rpc_with_config(mut config: Config) -> RawRpcContext {
        let addr = next_available_addr();
        if let IpAddr::V4(ip) = addr.ip() {
            config.rpc.host = ip;
        }
        config.rpc.port = addr.port();

        let config = Arc::new(config);

        let state_db = Arc::new(
            DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions()).unwrap(),
        );
        let pending_db = Arc::new(
            DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions()).unwrap(),
        );

        let state_store = Arc::new(StateStore::new(state_db));
        let pending_store = Arc::new(PendingStore::new(pending_db));
        let (provider, _mock) = providers::Provider::mocked();
        let (certificate_sender, certificate_receiver) = tokio::sync::mpsc::channel(1);

        let kernel = Kernel::new(Arc::new(provider), config.clone());

        let rpc = AgglayerImpl::new(
            kernel,
            certificate_sender,
            pending_store,
            state_store,
            config.clone(),
        );

        RawRpcContext {
            rpc,
            config,
            certificate_receiver,
        }
    }
}

#[fixture]
async fn context() -> TestContext {
    TestContext::new().await
}

#[fixture]
async fn raw_rpc() -> RawRpcContext {
    TestContext::new_raw_rpc().await
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
    fn assign_certificate_to_epoch(
        &self,
        _certificate_id: &CertificateId,
        _epoch_number: &agglayer_types::EpochNumber,
        _certificate_index: &agglayer_types::CertificateIndex,
    ) -> Result<(), agglayer_storage::error::Error> {
        todo!()
    }
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
    fn set_latest_settled_certificate_for_network(
        &self,
        _network_id: &NetworkId,
        _height: &Height,
        _certificate_id: &CertificateId,
        _epoch_number: &agglayer_types::EpochNumber,
        _certificate_index: &agglayer_types::CertificateIndex,
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
    ) -> Result<Vec<(NetworkId, SettledCertificate)>, agglayer_storage::error::Error> {
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
    fn remove_generated_proof(
        &self,
        _certificate_id: &agglayer_types::CertificateId,
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

    fn set_latest_proven_certificate_per_network(
        &self,
        _network_id: &NetworkId,
        _height: &Height,
        _certificate_id: &CertificateId,
    ) -> Result<(), agglayer_storage::error::Error> {
        todo!()
    }
}
