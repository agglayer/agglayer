use std::{
    future::IntoFuture,
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use agglayer_config::Config;
use agglayer_contracts::{
    polygon_rollup_manager::PolygonRollupManager,
    polygon_zkevm_global_exit_root_v2::PolygonZkEVMGlobalExitRootV2, L1RpcClient,
};
use agglayer_storage::{
    storage::{backup::BackupClient, pending_db_cf_definitions, state_db_cf_definitions, DB},
    stores::{debug::DebugStore, pending::PendingStore, state::StateStore},
    tests::TempDBDir,
};
use agglayer_types::{Certificate, CertificateId, Height, NetworkId};
use ethers::{
    providers::{self, MockProvider, Provider},
    signers::Signer,
};
use jsonrpsee::http_client::HttpClientBuilder;
use rstest::*;
use tokio_util::sync::CancellationToken;

use crate::{admin::AdminAgglayerImpl, kernel::Kernel, service::AgglayerService, AgglayerImpl};

pub(crate) struct RawRpcContext {
    pub(crate) rpc: AgglayerImpl<
        Provider<MockProvider>,
        L1RpcClient<Provider<MockProvider>>,
        PendingStore,
        StateStore,
        DebugStore,
    >,
    pub(crate) admin_rpc: AdminAgglayerImpl<PendingStore, StateStore, DebugStore>,
    pub(crate) config: Arc<Config>,
    pub(crate) state_store: Arc<StateStore>,
    pub(crate) pending_store: Arc<PendingStore>,
    pub(crate) certificate_receiver:
        tokio::sync::mpsc::Receiver<(NetworkId, Height, CertificateId)>,
}

pub struct TestContext {
    pub cancellation_token: CancellationToken,
    pub state_store: Arc<StateStore>,
    pub pending_store: Arc<PendingStore>,
    pub client: jsonrpsee::http_client::HttpClient,
    pub admin_client: jsonrpsee::http_client::HttpClient,
    pub config: Arc<Config>,
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

        let admin_router = raw_rpc.admin_rpc.start().await.unwrap();

        let api_url = format!("http://{}/", raw_rpc.config.readrpc_addr());
        let admin_url = format!("http://{}/", raw_rpc.config.admin_rpc_addr());
        let client = HttpClientBuilder::default().build(api_url).unwrap();
        let admin_client = HttpClientBuilder::default().build(admin_url).unwrap();

        let listener_api = tokio::net::TcpListener::bind(raw_rpc.config.readrpc_addr())
            .await
            .unwrap();

        let listener_admin = tokio::net::TcpListener::bind(raw_rpc.config.admin_rpc_addr())
            .await
            .unwrap();

        let api_server = axum::serve(listener_api, router)
            .with_graceful_shutdown(cancellation_token.child_token().cancelled_owned());
        let admin_server = axum::serve(listener_admin, admin_router)
            .with_graceful_shutdown(cancellation_token.child_token().cancelled_owned());

        tokio::spawn(api_server.into_future());
        tokio::spawn(admin_server.into_future());

        Self {
            cancellation_token,
            state_store: raw_rpc.state_store,
            pending_store: raw_rpc.pending_store,
            client,
            admin_client,
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

    pub(crate) async fn new_raw_rpc() -> RawRpcContext {
        let config = Self::get_default_config();
        Self::new_raw_rpc_with_config(config).await
    }

    pub(crate) async fn new_raw_rpc_with_config(mut config: Config) -> RawRpcContext {
        let addr = next_available_addr();
        let admin_addr = next_available_addr();
        if let IpAddr::V4(ip) = addr.ip() {
            config.rpc.host = ip;
        }
        config.rpc.readrpc_port = addr.port();
        config.rpc.admin_port = admin_addr.port();

        let config = Arc::new(config);

        let state_db = Arc::new(
            DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions()).unwrap(),
        );
        let pending_db = Arc::new(
            DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions()).unwrap(),
        );

        let state_store = Arc::new(StateStore::new(state_db, BackupClient::noop()));
        let pending_store = Arc::new(PendingStore::new(pending_db));
        let debug_store = if config.debug_mode {
            Arc::new(DebugStore::new_with_path(&config.storage.debug_db_path).unwrap())
        } else {
            Arc::new(DebugStore::Disabled)
        };
        let (provider, _mock) = providers::Provider::mocked();
        let (certificate_sender, certificate_receiver) = tokio::sync::mpsc::channel(1);

        let rpc = Arc::new(provider);
        let kernel = Kernel::new(rpc.clone(), config.clone());

        let rollup_manager = Arc::new(L1RpcClient::new(
            rpc.clone(),
            PolygonRollupManager::new(config.l1.rollup_manager_contract, rpc.clone()),
            PolygonZkEVMGlobalExitRootV2::new(
                config.l1.polygon_zkevm_global_exit_root_v2_contract,
                rpc.clone(),
            ),
            (1, [1; 32]),
        ));
        let rpc_service = Arc::new(agglayer_rpc::AgglayerService::new(
            certificate_sender,
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
            rollup_manager,
        ));

        let service = AgglayerService::new(kernel);
        let admin_rpc = AdminAgglayerImpl::new(
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
        );

        let rpc = AgglayerImpl::new(Arc::new(service), rpc_service);

        RawRpcContext {
            rpc,
            admin_rpc,
            config,
            state_store,
            pending_store,
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
pub(crate) async fn context() -> TestContext {
    TestContext::new().await
}

#[fixture]
pub(crate) async fn raw_rpc() -> RawRpcContext {
    TestContext::new_raw_rpc().await
}

pub(crate) fn next_available_addr() -> std::net::SocketAddr {
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
