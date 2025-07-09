use std::{future::IntoFuture as _, net::SocketAddr, sync::Arc};

use agglayer_config::Config;
use agglayer_contracts::L1RpcClient;
use agglayer_storage::{
    storage::{
        backup::BackupClient, debug_db_cf_definitions, pending_db_cf_definitions,
        state_db_cf_definitions, DB,
    },
    stores::{debug::DebugStore, pending::PendingStore, state::StateStore},
    tests::TempDBDir,
};
use agglayer_types::{Certificate, CertificateId, Height, NetworkId};
use alloy::{
    providers::{
        fillers::{BlobGasFiller, ChainIdFiller, FillProvider, GasFiller, JoinFill, NonceFiller},
        mock::Asserter,
        Identity, ProviderBuilder, RootProvider,
    },
    transports::mock::MockTransport,
};
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use rstest::*;
use tokio_util::sync::CancellationToken;

// Import the AgglayerServer trait to get access to into_rpc()
use crate::{admin::AdminAgglayerImpl, AgglayerServer};

// Alloy mock provider type alias - matching pattern from other modules
pub type MockProvider = FillProvider<
    JoinFill<
        Identity,
        JoinFill<GasFiller, JoinFill<BlobGasFiller, JoinFill<NonceFiller, ChainIdFiller>>>,
    >,
    RootProvider,
    alloy::network::Ethereum,
>;

pub type RawRpcClient = crate::AgglayerImpl<
    MockProvider,
    L1RpcClient<MockProvider>,
    PendingStore,
    StateStore,
    DebugStore,
    Box<dyn 'static + Send + Sync + Fn(NetworkId) -> bool>,
>;

pub struct RawRpcContext {
    pub rpc: RawRpcClient,
    pub config: Arc<Config>,
    pub state_store: Arc<StateStore>,
    pub pending_store: Arc<PendingStore>,
    pub debug_store: Arc<DebugStore>,
}

impl RawRpcContext {
    /// Convert the RawRpcContext to an RPC service by delegating to the
    /// underlying rpc field
    pub fn into_rpc(self) -> impl AgglayerServer {
        self.rpc
    }
}

pub struct TestContext {
    pub cancellation_token: CancellationToken,
    pub state_store: Arc<StateStore>,
    pub pending_store: Arc<PendingStore>,
    pub api_client: HttpClient,
    pub admin_client: HttpClient,
    pub proxied_grpc_client: Option<HttpClient>,
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
        // Create a mock provider for the default case
        let asserter = Asserter::new();
        let _transport = MockTransport::new(asserter.clone());
        let mock_provider = ProviderBuilder::new().on_mocked_client(asserter);

        Self::new_with_provider(config, mock_provider).await
    }

    /// Common implementation for creating TestContext with any provider
    pub async fn new_with_provider<P>(config: Config, provider: P) -> Self
    where
        P: alloy::providers::Provider + Clone + 'static,
    {
        let cancellation_token = CancellationToken::new();
        let config = Arc::new(config);

        // Create the databases using the provided paths in the config
        let state_db = Arc::new(
            DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions()).unwrap(),
        );
        let pending_db = Arc::new(
            DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions()).unwrap(),
        );
        let debug_db = Arc::new(
            DB::open_cf(&config.storage.debug_db_path, debug_db_cf_definitions()).unwrap(),
        );

        // Create stores using the provided databases
        let state_store = Arc::new(StateStore::new(state_db, BackupClient::noop()));
        let pending_store = Arc::new(PendingStore::new(pending_db));
        let debug_store = if config.debug_mode {
            Arc::new(DebugStore::new(debug_db))
        } else {
            Arc::new(DebugStore::Disabled)
        };

        // Use the provided provider
        let real_provider = Arc::new(provider);

        // Create L1RpcClient with the provider
        let l1_rpc_client = Self::create_l1_rpc_client(real_provider.clone());

        // Create certificate sender channel
        let (certificate_sender, certificate_receiver) = tokio::sync::mpsc::channel(1);

        // Create AgglayerService (V0Rpc service) with the provider
        let v0_service = Arc::new(crate::service::AgglayerService::new(
            crate::kernel::Kernel::new(real_provider.clone(), config.clone()),
        ));

        // Create agglayer_rpc::AgglayerService with the provider
        let rpc_service = Arc::new(agglayer_rpc::AgglayerService::new(
            certificate_sender,
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
            Arc::new(l1_rpc_client),
        ));

        // Create the allowed_networks function for network filtering
        let proxied_networks = config
            .proxied_networks
            .as_ref()
            .map(|pn| pn.networks.clone());
        let allowed_networks = Box::new(move |incoming| match &proxied_networks {
            None => true,
            Some(pn) => !pn.contains(&incoming),
        }) as Box<dyn Fn(NetworkId) -> bool + Send + Sync + 'static>;

        // Create AgglayerImpl with allowed_networks
        let agglayer_impl = crate::AgglayerImpl::new(v0_service, rpc_service, allowed_networks);

        // Create the routers
        let router = agglayer_impl.start().await.unwrap();
        let admin_router = AdminAgglayerImpl::new(
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
        )
        .start()
        .await
        .unwrap();

        // Create addresses for API and admin servers
        let api_addr = Self::next_available_address();
        let admin_addr = Self::next_available_address();

        let api_url = format!("http://{api_addr}/");
        let admin_url = format!("http://{admin_addr}/");

        let api_client = HttpClientBuilder::default().build(api_url).unwrap();
        let admin_client = HttpClientBuilder::default().build(admin_url).unwrap();

        // Create proxied gRPC client if proxied networks are configured
        let proxied_grpc_client = config.proxied_networks.as_ref().map(|pn| {
            let proxied_url = format!("http://{}:{}", pn.host, pn.grpc_port);
            HttpClientBuilder::default().build(proxied_url).unwrap()
        });

        let listener_api = tokio::net::TcpListener::bind(api_addr).await.unwrap();

        let listener_admin = tokio::net::TcpListener::bind(admin_addr).await.unwrap();

        let api_server = axum::serve(listener_api, router)
            .with_graceful_shutdown(cancellation_token.child_token().cancelled_owned());
        let admin_server = axum::serve(listener_admin, admin_router)
            .with_graceful_shutdown(cancellation_token.child_token().cancelled_owned());

        tokio::spawn(api_server.into_future());
        tokio::spawn(admin_server.into_future());

        Self {
            cancellation_token,
            state_store,
            pending_store,
            api_client,
            admin_client,
            proxied_grpc_client,
            config,
            certificate_receiver,
        }
    }

    /// Helper method to create L1RpcClient with any provider type
    fn create_l1_rpc_client<P>(provider: Arc<P>) -> L1RpcClient<P>
    where
        P: alloy::providers::Provider + Clone + 'static,
    {
        use agglayer_contracts::contracts::PolygonRollupManager;
        use agglayer_types::Address;

        let inner = PolygonRollupManager::PolygonRollupManagerInstance::new(
            Address::ZERO.into(), // Use real contract address in production
            (*provider).clone(),
        );

        L1RpcClient::new(
            provider,
            inner,
            Address::ZERO.into(), // Use real L1 info tree address in non-test environments
            (0u32, [0u8; 32]),    // Use real default L1 info tree entry in non-test environments
            100,                  // Default gas multiplier factor
        )
    }

    pub fn get_default_config() -> Config {
        let tmp = TempDBDir::new();
        let mut cfg = Config::new(&tmp.path);
        for network_id in 0..10 {
            cfg.proof_signers.insert(
                network_id,
                Certificate::wallet_for_test(NetworkId::new(network_id))
                    .address()
                    .into(),
            );
        }
        cfg
    }

    pub async fn new_raw_rpc() -> RawRpcContext {
        let config = Self::get_default_config();
        Self::new_raw_rpc_with_config(config).await
    }

    pub async fn new_raw_rpc_with_config(mut config: Config) -> RawRpcContext {
        // Set up proxied networks configuration with available addresses
        if let Some(proxied_networks) = config.proxied_networks.as_mut() {
            let proxied_addr = Self::next_available_address();
            proxied_networks.host = match proxied_addr.ip() {
                std::net::IpAddr::V4(ip) => ip,
                std::net::IpAddr::V6(_) => std::net::Ipv4Addr::new(127, 0, 0, 1),
            };
            proxied_networks.grpc_port = proxied_addr.port();
        }

        let config = Arc::new(config);

        let state_db = Arc::new(
            DB::open_cf(&config.storage.state_db_path, state_db_cf_definitions()).unwrap(),
        );
        let pending_db = Arc::new(
            DB::open_cf(&config.storage.pending_db_path, pending_db_cf_definitions()).unwrap(),
        );
        let debug_db = Arc::new(
            DB::open_cf(&config.storage.debug_db_path, debug_db_cf_definitions()).unwrap(),
        );

        let state_store = Arc::new(StateStore::new(state_db, BackupClient::noop()));
        let pending_store = Arc::new(PendingStore::new(pending_db));
        let debug_store = Arc::new(DebugStore::new(debug_db));

        // Create mock transport with an asserter
        let asserter = Asserter::new();
        let _transport = MockTransport::new(asserter.clone());

        // Build the provider with the mock transport
        let mock_provider = ProviderBuilder::new().on_mocked_client(asserter);

        // Create L1RpcClient with mock provider
        let l1_rpc_client = Self::create_l1_rpc_client(Arc::new(mock_provider.clone()));

        // Create certificate sender channel
        let (certificate_sender, _certificate_receiver) = tokio::sync::mpsc::channel(1);

        // Create AgglayerService (V0Rpc service)
        let v0_service = Arc::new(crate::service::AgglayerService::new(
            crate::kernel::Kernel::new(Arc::new(mock_provider.clone()), config.clone()),
        ));

        // Create agglayer_rpc::AgglayerService
        let rpc_service = Arc::new(agglayer_rpc::AgglayerService::new(
            certificate_sender,
            pending_store.clone(),
            state_store.clone(),
            debug_store.clone(),
            config.clone(),
            Arc::new(l1_rpc_client),
        ));

        // Create the allowed_networks function for network filtering
        let proxied_networks = config
            .proxied_networks
            .as_ref()
            .map(|pn| pn.networks.clone());
        let allowed_networks = Box::new(move |incoming| match &proxied_networks {
            None => true,
            Some(pn) => !pn.contains(&incoming),
        }) as Box<dyn Fn(NetworkId) -> bool + Send + Sync + 'static>;

        // Create AgglayerImpl with allowed_networks
        let agglayer_impl = crate::AgglayerImpl::new(v0_service, rpc_service, allowed_networks);

        RawRpcContext {
            rpc: agglayer_impl,
            config,
            state_store,
            pending_store,
            debug_store,
        }
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        self.cancellation_token.cancel();
    }
}

#[fixture]
pub async fn context() -> TestContext {
    TestContext::new().await
}

#[fixture]
pub async fn raw_rpc() -> RawRpcContext {
    TestContext::new_raw_rpc().await
}

pub fn next_available_addr() -> std::net::SocketAddr {
    assert!(
        std::env::var("NEXTEST").is_ok(),
        "Due to concurrency issues, the rpc tests have to be run under `cargo nextest`",
    );

    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}
