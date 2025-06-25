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

// Note: We use generics instead of a concrete type alias for HTTP providers
// This allows the method to work with any provider that implements the
// necessary traits

pub struct RawRpcContext {
    pub rpc: crate::AgglayerImpl<
        MockProvider,
        L1RpcClient<MockProvider>,
        PendingStore,
        StateStore,
        DebugStore,
    >,
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

    /// Create a TestContext with a real HTTP provider instead of mocks
    /// This is useful for testing against real blockchain instances like Anvil
    pub async fn new_with_real_provider<P>(config: Config, provider: P) -> Self
    where
        P: alloy::providers::Provider + Clone + 'static,
    {
        Self::new_with_provider(config, provider).await
    }

    /// Common implementation for creating TestContext with any provider
    async fn new_with_provider<P>(config: Config, provider: P) -> Self
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

        // Create AgglayerImpl
        let agglayer_impl = crate::AgglayerImpl::new(v0_service, rpc_service);

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

        let api_url = format!("http://{}/", api_addr);
        let admin_url = format!("http://{}/", admin_addr);

        let client = HttpClientBuilder::default().build(api_url).unwrap();
        let admin_client = HttpClientBuilder::default().build(admin_url).unwrap();

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
            api_client: client,
            admin_client,
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
        use alloy::primitives::Address;

        let inner = PolygonRollupManager::PolygonRollupManagerInstance::new(
            Address::ZERO, // Use real contract address in production
            (*provider).clone(),
        );

        L1RpcClient::new(
            provider,
            inner,
            Address::ZERO,     // Use real L1 info tree address in production
            (0u32, [0u8; 32]), // Use real default L1 info tree entry in production
        )
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

    pub async fn new_raw_rpc() -> RawRpcContext {
        let config = Self::get_default_config();
        Self::new_raw_rpc_with_config(config).await
    }

    pub async fn new_raw_rpc_with_config(config: Config) -> RawRpcContext {
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

        // Create a mock transport with an asserter
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

        // Create AgglayerImpl
        let agglayer_impl = crate::AgglayerImpl::new(v0_service, rpc_service);

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
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}
