use std::future::IntoFuture;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use agglayer_config::Config;
use agglayer_storage::storage::{pending_db_cf_definitions, state_db_cf_definitions, DB};
use agglayer_storage::stores::debug::DebugStore;
use agglayer_storage::stores::pending::PendingStore;
use agglayer_storage::stores::state::StateStore;
use agglayer_storage::tests::TempDBDir;
use agglayer_types::{Certificate, CertificateId, Height, NetworkId};
use ethers::providers::{self, MockProvider, Provider};
use ethers::signers::Signer;
use jsonrpsee::http_client::HttpClientBuilder;
use rstest::*;
use tokio_util::sync::CancellationToken;

use crate::{kernel::Kernel, service::AgglayerService, AgglayerImpl};

#[cfg(test)]
mod errors;
#[cfg(test)]
mod get_certificate_header;
#[cfg(test)]
mod get_epoch_configuration;
#[cfg(test)]
mod get_latest_known_certificate_header;
#[cfg(test)]
mod get_tx_status;
#[cfg(test)]
mod send_certificate;

pub struct RawRpcContext {
    pub rpc: AgglayerImpl<Provider<MockProvider>, PendingStore, StateStore, DebugStore>,
    config: Arc<Config>,
    pub certificate_receiver: tokio::sync::mpsc::Receiver<(NetworkId, Height, CertificateId)>,
}

pub struct TestContext {
    pub client: jsonrpsee::http_client::HttpClient,
    pub config: Arc<Config>,
    pub cancellation_token: CancellationToken,
    pub certificate_receiver: tokio::sync::mpsc::Receiver<(NetworkId, Height, CertificateId)>,
}

impl TestContext {
    pub fn next_available_address() -> SocketAddr {
        next_available_addr()
    }

    async fn new() -> Self {
        let config = Self::get_default_config();
        Self::new_with_config(config).await
    }

    pub async fn new_with_config(config: Config) -> Self {
        let cancellation_token = CancellationToken::new();
        let raw_rpc = Self::new_raw_rpc_with_config(config).await;
        let router = raw_rpc.rpc.start().await.unwrap();

        let url = format!("http://{}/", raw_rpc.config.rpc_addr());
        let client = HttpClientBuilder::default().build(url).unwrap();

        let listener = tokio::net::TcpListener::bind(raw_rpc.config.rpc_addr())
            .await
            .unwrap();
        let api_graceful_shutdown = cancellation_token.clone();
        let api_server = axum::serve(listener, router)
            .with_graceful_shutdown(api_graceful_shutdown.cancelled_owned());

        tokio::spawn(api_server.into_future());

        Self {
            cancellation_token,
            client,
            config: raw_rpc.config,
            certificate_receiver: raw_rpc.certificate_receiver,
        }
    }

    pub fn get_default_config() -> Config {
        let tmp = TempDBDir::new();
        let mut cfg = Config::new(&tmp.path);
        for network_id in 0..10 {
            cfg.proof_signers.insert(
                network_id,
                Certificate::wallet_for_test(NetworkId::new(network_id)).address(),
            );
        }
        cfg
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
        let debug_store = if config.debug_mode {
            Arc::new(DebugStore::new_with_path(&config.storage.debug_db_path).unwrap())
        } else {
            Arc::new(DebugStore::Disabled)
        };
        let (provider, _mock) = providers::Provider::mocked();
        let (certificate_sender, certificate_receiver) = tokio::sync::mpsc::channel(1);

        let kernel = Kernel::new(Arc::new(provider), config.clone());

        let service = AgglayerService::new(kernel);

        let rpc_service = Arc::new(agglayer_rpc::AgglayerService::new(
            certificate_sender,
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
        ));

        let rpc = AgglayerImpl::new(Arc::new(service), rpc_service);

        RawRpcContext {
            rpc,
            config,
            certificate_receiver,
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
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

    // assert!(
    //     std::env::var("NEXTEST").is_ok(),
    //     "Due to concurrency issues, the rpc tests have to be run under `cargo
    // nextest`", );

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
